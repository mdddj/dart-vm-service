//! [`VmService`]: the cheaply clonable client handle.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::Stream;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tracing::warn;

use crate::connection::{self, Command};
use crate::error::{Error, Result, RpcErrorCode};
use crate::method::RpcMethod;
use crate::params::Params;
use crate::types::{ErrorObject, Sentinel, StreamEvent, StreamId};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_EVENT_BUFFER: usize = 1024;

/// Configures and opens a [`VmService`] connection.
#[derive(Debug, Clone)]
pub struct VmServiceBuilder {
    request_timeout: Option<Duration>,
    event_buffer: usize,
}

impl Default for VmServiceBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Some(DEFAULT_REQUEST_TIMEOUT),
            event_buffer: DEFAULT_EVENT_BUFFER,
        }
    }
}

impl VmServiceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Per-request timeout (default 30 s).
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Wait forever for answers.
    pub fn no_request_timeout(mut self) -> Self {
        self.request_timeout = None;
        self
    }

    /// How many events a slow [`EventStream`] may lag behind before it starts
    /// losing the oldest ones (default 1024).
    pub fn event_buffer(mut self, capacity: usize) -> Self {
        self.event_buffer = capacity;
        self
    }

    /// Connects to `url`: the `http://…/token=/` address printed by `dart run
    /// --observe` / `flutter run`, or the `ws://…/ws` form.
    pub async fn connect(self, url: &str) -> Result<VmService> {
        let conn = connection::open(url, self.event_buffer).await?;
        Ok(VmService {
            inner: Arc::new(Inner {
                commands: conn.commands,
                events: conn.events,
                closed: conn.closed,
                next_id: AtomicU64::new(1),
                request_timeout: self.request_timeout,
                url: conn.url.to_string(),
            }),
        })
    }
}

struct Inner {
    commands: mpsc::UnboundedSender<Command>,
    events: broadcast::WeakSender<StreamEvent>,
    closed: watch::Receiver<bool>,
    next_id: AtomicU64,
    request_timeout: Option<Duration>,
    url: String,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // Last handle gone: shut the socket down.
        let _ = self.commands.send(Command::Close);
    }
}

/// A connection to a Dart VM Service. Clone it freely; all clones share one socket.
///
/// The connection closes when the last clone is dropped or [`VmService::close`]
/// is called.
#[derive(Clone)]
pub struct VmService {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for VmService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmService")
            .field("url", &self.inner.url)
            .finish_non_exhaustive()
    }
}

impl VmService {
    /// Connects with default settings. See [`VmServiceBuilder`] to customise.
    pub async fn connect(url: &str) -> Result<Self> {
        VmServiceBuilder::new().connect(url).await
    }

    pub fn builder() -> VmServiceBuilder {
        VmServiceBuilder::new()
    }

    /// The (normalised) WebSocket URL in use.
    pub fn url(&self) -> &str {
        &self.inner.url
    }

    /// `true` once the socket has ended.
    pub fn is_closed(&self) -> bool {
        *self.inner.closed.borrow()
    }

    /// Resolves when the socket ends (VM exited, network error, [`close`](Self::close)).
    pub async fn closed(&self) {
        let mut rx = self.inner.closed.clone();
        let _ = rx.wait_for(|closed| *closed).await;
    }

    /// Closes the connection. Pending requests fail with [`Error::Closed`].
    pub fn close(&self) {
        let _ = self.inner.commands.send(Command::Close);
    }

    // ---- raw access ------------------------------------------------------

    /// Calls any RPC from the [`RpcMethod`] enum with hand-built params.
    /// `Sentinel` answers become [`Error::Sentinel`].
    pub async fn call_method(&self, method: RpcMethod, params: Value) -> Result<Value> {
        let value = self.send(method.as_str(), params).await?;
        check_sentinel(&value)?;
        Ok(value)
    }

    /// Calls a service extension (`ext.*`, e.g. `ext.flutter.debugPaint`) on an
    /// isolate. Extension parameters are `string -> string`; `isolateId` is added
    /// for you. Returns the raw JSON result.
    pub async fn call_extension<I, K, V>(
        &self,
        isolate_id: &str,
        method: &str,
        args: I,
    ) -> Result<Value>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: ToString,
    {
        let mut params = Params::new().with("isolateId", isolate_id);
        for (k, v) in args {
            params = params.with(k.as_ref(), v.to_string());
        }
        self.send(method, params.into_value()).await
    }

    /// Escape hatch: call any method name (including ones newer than this crate)
    /// and get the raw JSON result without any interpretation.
    pub async fn call_raw(&self, method: &str, params: Value) -> Result<Value> {
        self.send(method, params).await
    }

    // ---- internals shared with the typed API -----------------------------

    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: RpcMethod,
        params: Params,
    ) -> Result<T> {
        let value = self.call_method(method, params.into_value()).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// For RPCs returning `Success` (or `Success|Error`).
    pub(crate) async fn request_unit(&self, method: RpcMethod, params: Params) -> Result<()> {
        let value = self.call_method(method, params.into_value()).await?;
        if value.get("type").and_then(Value::as_str) == Some("Error") {
            let err: ErrorObject = serde_json::from_value(value)?;
            return Err(Error::Vm(Box::new(err)));
        }
        Ok(())
    }

    async fn send(&self, method: &str, params: Value) -> Result<Value> {
        let id = self
            .inner
            .next_id
            .fetch_add(1, Ordering::Relaxed)
            .to_string();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        })
        .to_string();

        let (reply, answer) = oneshot::channel();
        self.inner
            .commands
            .send(Command::Request {
                id: id.clone(),
                payload,
                reply,
            })
            .map_err(|_| Error::Closed)?;

        // If this future is dropped (cancelled / timed out) tell the I/O task to
        // forget the request so the pending map cannot grow.
        let guard = CancelOnDrop {
            commands: &self.inner.commands,
            id: Some(id),
        };
        let result = match self.inner.request_timeout {
            Some(limit) => tokio::time::timeout(limit, answer)
                .await
                .map_err(|_| Error::Timeout(limit))?,
            None => answer.await,
        };
        guard.disarm();
        result.map_err(|_| Error::Closed)?
    }

    // ---- events ----------------------------------------------------------

    /// All events of all streams you have subscribed to on this connection.
    pub fn events(&self) -> EventStream {
        self.make_stream(None)
    }

    /// Events of a single stream. Does **not** call `streamListen`; see
    /// [`subscribe`](Self::subscribe).
    pub fn stream(&self, stream: StreamId) -> EventStream {
        self.make_stream(Some(stream))
    }

    /// `streamListen` + a filtered [`EventStream`] in one step. Subscribing twice
    /// is fine (error 103 is swallowed).
    pub async fn subscribe(&self, stream: StreamId) -> Result<EventStream> {
        // Create the receiver first so no event between listen and stream() is lost.
        let events = self.stream(stream.clone());
        match self.stream_listen(stream).await {
            Ok(()) => {}
            Err(e) if e.rpc_code() == Some(RpcErrorCode::StreamAlreadySubscribed) => {}
            Err(e) => return Err(e),
        }
        Ok(events)
    }

    fn make_stream(&self, filter: Option<StreamId>) -> EventStream {
        match self.inner.events.upgrade() {
            Some(tx) => EventStream {
                inner: Some(BroadcastStream::new(tx.subscribe())),
                filter,
            },
            // Connection already ended: an immediately finished stream.
            None => EventStream {
                inner: None,
                filter,
            },
        }
    }
}

fn check_sentinel(value: &Value) -> Result<()> {
    if value.get("type").and_then(Value::as_str) == Some("Sentinel") {
        let sentinel: Sentinel = serde_json::from_value(value.clone())?;
        return Err(Error::Sentinel(sentinel));
    }
    Ok(())
}

struct CancelOnDrop<'a> {
    commands: &'a mpsc::UnboundedSender<Command>,
    id: Option<String>,
}

impl CancelOnDrop<'_> {
    fn disarm(mut self) {
        self.id = None;
    }
}

impl Drop for CancelOnDrop<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.commands.send(Command::Cancel(id));
        }
    }
}

/// A `futures::Stream` of [`StreamEvent`]s. Ends when the connection closes.
///
/// If the consumer falls more than the configured buffer behind, the oldest
/// events are dropped (a `tracing` warning reports how many).
pub struct EventStream {
    inner: Option<BroadcastStream<StreamEvent>>,
    filter: Option<StreamId>,
}

impl EventStream {
    /// The stream this instance is restricted to, if any.
    pub fn stream_id(&self) -> Option<&StreamId> {
        self.filter.as_ref()
    }
}

impl Stream for EventStream {
    type Item = StreamEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Some(inner) = self.inner.as_mut() else {
                return Poll::Ready(None);
            };
            match Pin::new(inner).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Ok(ev))) => {
                    if self.filter.as_ref().is_none_or(|f| *f == ev.stream_id) {
                        return Poll::Ready(Some(ev));
                    }
                }
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(n)))) => {
                    warn!(skipped = n, "event stream lagged; events were dropped");
                }
            }
        }
    }
}
