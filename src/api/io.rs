//! `dart:io` service extensions: HTTP request monitoring and socket profiling.
//!
//! The VM has no push event for HTTP traffic; tools (DevTools' Network tab
//! included) poll `ext.dart.io.getHttpProfile` with `updatedSince`.
//! [`VmService::watch_http_requests`] wraps that loop into a `Stream`.
//!
//! Only requests made through `dart:io`'s `HttpClient` are visible — that includes
//! `package:http` and `dio` on the VM, but not native stacks such as
//! `cupertino_http`/`cronet_http`. Nothing is recorded before profiling is enabled.

use std::collections::VecDeque;
use std::time::Duration;

use futures_util::Stream;
use futures_util::stream;
use serde::de::DeserializeOwned;

use crate::client::VmService;
use crate::error::Result;
use crate::types::*;

const EXT_PREFIX: &str = "ext.dart.io.";

/// Settings of [`VmService::watch_http_requests`].
#[derive(Debug, Clone)]
pub struct HttpWatchOptions {
    /// Pause between polls (default 500 ms).
    pub interval: Duration,
    /// Turn HTTP profiling on before watching (default `true`).
    pub enable_logging: bool,
    /// Also emit requests recorded before the watch started (default `false`).
    pub include_existing: bool,
}

impl Default for HttpWatchOptions {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(500),
            enable_logging: true,
            include_existing: false,
        }
    }
}

impl HttpWatchOptions {
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    pub fn include_existing(mut self, include: bool) -> Self {
        self.include_existing = include;
        self
    }
}

impl VmService {
    async fn io_ext<T: DeserializeOwned>(
        &self,
        isolate_id: &str,
        name: &str,
        args: Vec<(&str, String)>,
    ) -> Result<T> {
        let value = self
            .call_extension(isolate_id, &format!("{EXT_PREFIX}{name}"), args)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Version of the `dart:io` extension protocol (`ext.dart.io.getVersion`).
    pub async fn io_get_version(&self, isolate_id: &str) -> Result<Version> {
        self.io_ext(isolate_id, "getVersion", vec![]).await
    }

    // ---- HTTP ------------------------------------------------------------

    /// Whether HTTP profiling is on for this isolate.
    pub async fn http_logging_enabled(&self, isolate_id: &str) -> Result<bool> {
        let s: HttpTimelineLoggingState = self
            .io_ext(isolate_id, "httpEnableTimelineLogging", vec![])
            .await?;
        Ok(s.enabled)
    }

    /// Turns HTTP profiling on/off for this isolate. Requests made while it was
    /// off are never recorded.
    pub async fn set_http_logging_enabled(&self, isolate_id: &str, enabled: bool) -> Result<bool> {
        let s: HttpTimelineLoggingState = self
            .io_ext(
                isolate_id,
                "httpEnableTimelineLogging",
                vec![("enabled", enabled.to_string())],
            )
            .await?;
        Ok(s.enabled)
    }

    /// Recorded HTTP requests. With `updated_since` (a previous
    /// [`HttpProfile::timestamp`]) only requests that changed afterwards are returned.
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService, iso: &str) -> dart_vm_service::Result<()> {
    /// vm.set_http_logging_enabled(iso, true).await?;
    /// let profile = vm.get_http_profile(iso, None).await?;
    /// for r in &profile.requests {
    ///     println!("{} {} -> {:?}", r.method, r.uri, r.status_code());
    /// }
    /// let later = vm.get_http_profile(iso, Some(profile.timestamp)).await?; // only new/changed
    /// # let _ = later; Ok(()) }
    /// ```
    pub async fn get_http_profile(
        &self,
        isolate_id: &str,
        updated_since: Option<i64>,
    ) -> Result<HttpProfile> {
        let args = updated_since
            .map(|t| vec![("updatedSince", t.to_string())])
            .unwrap_or_default();
        self.io_ext(isolate_id, "getHttpProfile", args).await
    }

    /// One request including captured request/response bodies.
    pub async fn get_http_profile_request(
        &self,
        isolate_id: &str,
        id: &str,
    ) -> Result<HttpProfileRequest> {
        self.io_ext(
            isolate_id,
            "getHttpProfileRequest",
            vec![("id", id.to_owned())],
        )
        .await
    }

    /// Forgets all recorded HTTP requests.
    pub async fn clear_http_profile(&self, isolate_id: &str) -> Result<()> {
        self.call_extension(
            isolate_id,
            &format!("{EXT_PREFIX}clearHttpProfile"),
            Vec::<(&str, &str)>::new(),
        )
        .await
        .map(|_| ())
    }

    /// Streams HTTP requests of an isolate as they happen.
    ///
    /// A request is emitted every time it changes (when it starts, when the response
    /// begins, when it finishes), so use [`HttpProfileRequestSummary::is_finished`]
    /// to wait for completion. The stream yields an `Err` and ends if polling fails
    /// (connection closed, isolate gone, …).
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService, iso: &str) -> dart_vm_service::Result<()> {
    /// use dart_vm_service::HttpWatchOptions;
    /// use futures_util::StreamExt;
    /// let mut requests = Box::pin(vm.watch_http_requests(iso, HttpWatchOptions::default()).await?);
    /// while let Some(req) = requests.next().await {
    ///     let req = req?;
    ///     if req.is_finished() {
    ///         println!("{} {} -> {:?} in {:?}us", req.method, req.uri, req.status_code(), req.duration_micros());
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    pub async fn watch_http_requests(
        &self,
        isolate_id: &str,
        options: HttpWatchOptions,
    ) -> Result<impl Stream<Item = Result<HttpProfileRequestSummary>> + Send + use<>> {
        if options.enable_logging {
            self.set_http_logging_enabled(isolate_id, true).await?;
        }
        // Establish the starting point so old requests are skipped unless asked for.
        let since = if options.include_existing {
            None
        } else {
            Some(self.get_http_profile(isolate_id, None).await?.timestamp)
        };

        struct State {
            vm: VmService,
            isolate_id: String,
            since: Option<i64>,
            queue: VecDeque<HttpProfileRequestSummary>,
            interval: Duration,
            first: bool,
            done: bool,
        }

        let state = State {
            vm: self.clone(),
            isolate_id: isolate_id.to_owned(),
            since,
            queue: VecDeque::new(),
            interval: options.interval,
            first: true,
            done: false,
        };

        Ok(stream::unfold(state, |mut st| async move {
            loop {
                if let Some(req) = st.queue.pop_front() {
                    return Some((Ok(req), st));
                }
                if st.done {
                    return None;
                }
                if !std::mem::take(&mut st.first) {
                    tokio::time::sleep(st.interval).await;
                }
                match st.vm.get_http_profile(&st.isolate_id, st.since).await {
                    Ok(profile) => {
                        st.since = Some(profile.timestamp);
                        st.queue.extend(profile.requests);
                    }
                    Err(e) => {
                        st.done = true;
                        return Some((Err(e), st));
                    }
                }
            }
        }))
    }

    // ---- sockets ---------------------------------------------------------

    /// Whether socket profiling is on.
    pub async fn socket_profiling_enabled(&self, isolate_id: &str) -> Result<bool> {
        let s: SocketProfilingState = self
            .io_ext(isolate_id, "socketProfilingEnabled", vec![])
            .await?;
        Ok(s.enabled)
    }

    /// Turns socket profiling on/off.
    pub async fn set_socket_profiling_enabled(
        &self,
        isolate_id: &str,
        enabled: bool,
    ) -> Result<bool> {
        let s: SocketProfilingState = self
            .io_ext(
                isolate_id,
                "socketProfilingEnabled",
                vec![("enabled", enabled.to_string())],
            )
            .await?;
        Ok(s.enabled)
    }

    /// Recorded socket statistics.
    pub async fn get_socket_profile(&self, isolate_id: &str) -> Result<SocketProfile> {
        self.io_ext(isolate_id, "getSocketProfile", vec![]).await
    }

    /// Forgets recorded socket statistics.
    pub async fn clear_socket_profile(&self, isolate_id: &str) -> Result<()> {
        self.call_extension(
            isolate_id,
            &format!("{EXT_PREFIX}clearSocketProfile"),
            Vec::<(&str, &str)>::new(),
        )
        .await
        .map(|_| ())
    }
}
