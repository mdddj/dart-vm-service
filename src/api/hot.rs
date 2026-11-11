//! Hot reload and hot restart helpers.
//!
//! These two operations are **not** a single VM RPC:
//!
//! * The VM only knows `reloadSources` — it re-reads the sources *already on the
//!   device*. It never compiles anything and does not rebuild Flutter widgets.
//! * When the app was started by `flutter run`, the Flutter tool registers the
//!   services `reloadSources` and `hotRestart` (alias `Flutter Tools`) with the VM
//!   service. Calling those makes the tool recompile, sync the files, reload (or
//!   restart) and reassemble the widget tree, exactly like pressing `r` / `R`.
//! * A bare `dart run` VM has no hot restart at all.
//!
//! [`VmService::hot_reload`] picks the right path for you,
//! [`VmService::hot_restart`] needs the Flutter tool.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::json;

use crate::client::VmService;
use crate::error::{Error, Result};
use crate::options::ReloadOptions;
use crate::types::*;

/// Alias the Flutter tool registers its services under.
pub const FLUTTER_TOOLS_ALIAS: &str = "Flutter Tools";

const RELOAD_SERVICE: &str = "reloadSources";
const RESTART_SERVICE: &str = "hotRestart";
const REASSEMBLE_EXTENSION: &str = "ext.flutter.reassemble";
/// How long without a new `ServiceRegistered` event counts as "replay finished".
const REPLAY_IDLE: Duration = Duration::from_millis(250);

/// Which mechanism [`VmService::hot_reload`] should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HotReloadStrategy {
    /// Flutter Tools if it is attached, otherwise the plain VM reload.
    #[default]
    Auto,
    /// Require the `reloadSources` service of the Flutter tool.
    FlutterTools,
    /// Always use the VM's own `reloadSources` RPC (no recompilation).
    Vm,
}

/// Options for [`VmService::hot_reload`].
#[derive(Debug, Clone)]
pub struct HotReloadOptions {
    pub strategy: HotReloadStrategy,
    /// Reload even unchanged sources (VM path; the Flutter tool ignores it).
    pub force: bool,
    /// Pause the isolate after the reload.
    pub pause: bool,
    /// VM path only.
    pub root_lib_uri: Option<String>,
    /// VM path only.
    pub packages_uri: Option<String>,
    /// VM path only: call `ext.flutter.reassemble` after a successful reload so
    /// Flutter rebuilds the widget tree (default `true`; skipped for non-Flutter
    /// isolates).
    pub reassemble: bool,
}

impl Default for HotReloadOptions {
    fn default() -> Self {
        Self {
            strategy: HotReloadStrategy::Auto,
            force: false,
            pause: false,
            root_lib_uri: None,
            packages_uri: None,
            reassemble: true,
        }
    }
}

impl HotReloadOptions {
    pub fn strategy(mut self, strategy: HotReloadStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn pause(mut self, pause: bool) -> Self {
        self.pause = pause;
        self
    }

    pub fn root_lib_uri(mut self, uri: impl Into<String>) -> Self {
        self.root_lib_uri = Some(uri.into());
        self
    }

    pub fn packages_uri(mut self, uri: impl Into<String>) -> Self {
        self.packages_uri = Some(uri.into());
        self
    }

    pub fn reassemble(mut self, reassemble: bool) -> Self {
        self.reassemble = reassemble;
        self
    }
}

/// Outcome of [`VmService::hot_reload`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum HotReloadResult {
    /// The Flutter tool did the whole job (compile, sync, reload, reassemble). It
    /// reports only success; failures come back as `Err`.
    FlutterTools,
    /// Plain VM reload.
    Vm {
        report: ReloadReport,
        /// `ext.flutter.reassemble` was called afterwards.
        reassembled: bool,
    },
}

impl HotReloadResult {
    /// `false` if the VM rejected the new code (for example a compile error).
    pub fn is_success(&self) -> bool {
        match self {
            HotReloadResult::FlutterTools => true,
            HotReloadResult::Vm { report, .. } => report.success,
        }
    }
}

/// Options for [`VmService::hot_restart`].
#[derive(Debug, Clone)]
pub struct HotRestartOptions {
    /// Pause the isolate at start after the restart.
    pub pause: bool,
    /// How long to wait for the restarted app's isolate to become runnable
    /// (default 15 s).
    pub wait_timeout: Duration,
}

impl Default for HotRestartOptions {
    fn default() -> Self {
        Self {
            pause: false,
            wait_timeout: Duration::from_secs(15),
        }
    }
}

impl HotRestartOptions {
    pub fn pause(mut self, pause: bool) -> Self {
        self.pause = pause;
        self
    }

    pub fn wait_timeout(mut self, timeout: Duration) -> Self {
        self.wait_timeout = timeout;
        self
    }
}

impl VmService {
    /// Services other clients registered with `registerService` (the Flutter tool's
    /// `reloadSources` / `hotRestart`, IDE services, …).
    ///
    /// The first call subscribes to the `Service` stream, so the list is complete after
    /// a short settle time (~250 ms); later calls are instant and stay up to date.
    pub async fn registered_services(&self) -> Result<Vec<RegisteredService>> {
        if !self.service_stream_listening() {
            let mut replay = self.subscribe(StreamId::Service).await?;
            // The VM/DDS replays existing registrations right after subscribing.
            while let Ok(Some(_)) = tokio::time::timeout(REPLAY_IDLE, replay.next()).await {}
        }
        let services = self.service_registry().lock().map_err(|_| Error::Closed)?;
        Ok(services.values().cloned().collect())
    }

    /// Looks up a registered service by its name (`hotRestart`, …). If several
    /// clients registered it, the Flutter tool's one wins.
    pub async fn find_registered_service(
        &self,
        service: &str,
    ) -> Result<Option<RegisteredService>> {
        let all = self.registered_services().await?;
        let mut matches = all.into_iter().filter(|s| s.service == service);
        let first = matches.next();
        Ok(match first {
            Some(f) if f.alias == FLUTTER_TOOLS_ALIAS => Some(f),
            Some(f) => matches.find(|s| s.alias == FLUTTER_TOOLS_ALIAS).or(Some(f)),
            None => None,
        })
    }

    /// `true` when a Flutter tool (`flutter run` / `flutter attach`) is attached and
    /// offers hot reload and hot restart.
    pub async fn has_flutter_tools(&self) -> Result<bool> {
        Ok(self
            .find_registered_service(RESTART_SERVICE)
            .await?
            .is_some())
    }

    /// Hot reload.
    ///
    /// With [`HotReloadStrategy::Auto`] (default): if the Flutter tool is attached it
    /// recompiles your changes and reloads; otherwise the VM re-reads the sources on
    /// disk and, for Flutter isolates, Flutter is told to reassemble.
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService) -> dart_vm_service::Result<()> {
    /// let isolate = vm.main_isolate().await?;
    /// let result = vm.hot_reload(&isolate.id, Default::default()).await?;
    /// assert!(result.is_success());
    /// # Ok(()) }
    /// ```
    pub async fn hot_reload(
        &self,
        isolate_id: &str,
        options: HotReloadOptions,
    ) -> Result<HotReloadResult> {
        let tool = match options.strategy {
            HotReloadStrategy::Vm => None,
            HotReloadStrategy::Auto => self.find_registered_service(RELOAD_SERVICE).await?,
            HotReloadStrategy::FlutterTools => Some(
                self.find_registered_service(RELOAD_SERVICE).await?.ok_or_else(|| {
                    Error::Unsupported(
                        "no Flutter tool is attached (service `reloadSources` is not registered)".into(),
                    )
                })?,
            ),
        };

        if let Some(service) = tool {
            let params =
                json!({ "isolateId": isolate_id, "force": options.force, "pause": options.pause });
            self.call_raw(&service.method, params).await?;
            return Ok(HotReloadResult::FlutterTools);
        }

        let mut vm_options = ReloadOptions::default()
            .force(options.force)
            .pause(options.pause);
        vm_options.root_lib_uri = options.root_lib_uri;
        vm_options.packages_uri = options.packages_uri;
        let report = self.reload_sources(isolate_id, vm_options).await?;

        let mut reassembled = false;
        if report.success && options.reassemble {
            let isolate = self.get_isolate(isolate_id).await?;
            if isolate.has_extension(REASSEMBLE_EXTENSION) {
                self.call_extension(isolate_id, REASSEMBLE_EXTENSION, Vec::<(&str, &str)>::new())
                    .await?;
                reassembled = true;
            }
        }
        Ok(HotReloadResult::Vm {
            report,
            reassembled,
        })
    }

    /// Hot restart: throws away the app state and runs `main()` again.
    ///
    /// Needs the Flutter tool (`flutter run` / `flutter attach`); a plain Dart VM
    /// cannot restart itself and yields [`Error::Unsupported`].
    ///
    /// A restart creates a new isolate, so old isolate ids stop working. This
    /// function waits for the new one and returns it (`None` if it did not show up in
    /// [`HotRestartOptions::wait_timeout`]); subscriptions on the old isolate must be
    /// renewed.
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService) -> dart_vm_service::Result<()> {
    /// let new_isolate = vm.hot_restart(Default::default()).await?;
    /// println!("restarted, main isolate is now {:?}", new_isolate.map(|i| i.id));
    /// # Ok(()) }
    /// ```
    pub async fn hot_restart(&self, options: HotRestartOptions) -> Result<Option<IsolateRef>> {
        let service = self
            .find_registered_service(RESTART_SERVICE)
            .await?
            .ok_or_else(|| {
                Error::Unsupported(
                    "hot restart needs the Flutter tool (`flutter run`/`flutter attach`); \
                 service `hotRestart` is not registered"
                        .into(),
                )
            })?;

        let before: Vec<String> = self
            .get_vm()
            .await?
            .isolates
            .into_iter()
            .map(|i| i.id)
            .collect();
        self.call_raw(&service.method, json!({ "pause": options.pause }))
            .await?;
        Ok(self
            .wait_for_new_isolate(&before, options.wait_timeout)
            .await)
    }

    /// Polls until a runnable, non-system isolate that was not in `before` appears.
    async fn wait_for_new_isolate(
        &self,
        before: &[String],
        timeout: Duration,
    ) -> Option<IsolateRef> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Ok(vm) = self.get_vm().await {
                for candidate in vm.isolates.iter().filter(|i| !i.is_system_isolate) {
                    if before.contains(&candidate.id) {
                        continue;
                    }
                    if matches!(self.get_isolate(&candidate.id).await, Ok(i) if i.runnable) {
                        return Some(candidate.clone());
                    }
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
