//! Background task that owns the WebSocket.
//!
//! One task reads frames and writes requests; callers talk to it through an
//! mpsc channel and receive their answer on a oneshot. Stream events fan out via
//! a `broadcast` channel. When the socket ends, every pending request fails with
//! [`Error::Closed`] and all [`EventStream`](crate::EventStream)s finish.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_tungstenite::tungstenite::{self, Message, protocol::WebSocketConfig};
use tracing::{debug, trace, warn};
use url::Url;

use crate::error::{Error, Result, RpcError};
use crate::types::{Event, StreamEvent, StreamId};
use crate::url::to_websocket_url;

const MAX_REDIRECTS: usize = 5;

pub(crate) type Reply = oneshot::Sender<Result<Value>>;

pub(crate) enum Command {
    Request {
        id: String,
        payload: String,
        reply: Reply,
    },
    /// Forget a request whose caller gave up (timeout / cancellation).
    Cancel(String),
    Close,
}

pub(crate) struct Connection {
    pub commands: mpsc::UnboundedSender<Command>,
    pub events: broadcast::WeakSender<StreamEvent>,
    pub closed: watch::Receiver<bool>,
    pub url: Url,
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Connects (following the redirect a DDS issues in "single client mode") and
/// spawns the I/O task.
pub(crate) async fn open(url: &str, event_capacity: usize) -> Result<Connection> {
    let mut url = to_websocket_url(url)?;
    let config = WebSocketConfig::default()
        .max_message_size(None)
        .max_frame_size(None);

    let mut redirects = 0;
    let socket: Socket = loop {
        match tokio_tungstenite::connect_async_with_config(url.as_str(), Some(config), true).await {
            Ok((socket, _response)) => break socket,
            Err(tungstenite::Error::Http(resp)) if resp.status().is_redirection() => {
                redirects += 1;
                let location = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| Error::Protocol("redirect without Location header".into()))?;
                if redirects > MAX_REDIRECTS {
                    return Err(Error::Protocol("too many redirects".into()));
                }
                let next = url
                    .join(location)
                    .map_err(|e| Error::InvalidUrl(location.to_owned(), e.to_string()))?;
                debug!(from = %url, to = %next, "following VM service redirect");
                url = to_websocket_url(next.as_str())?;
            }
            Err(e) => return Err(e.into()),
        }
    };

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, _) = broadcast::channel(event_capacity.max(1));
    let weak_events = event_tx.downgrade();
    let (closed_tx, closed_rx) = watch::channel(false);

    tokio::spawn(run(socket, cmd_rx, event_tx, closed_tx));

    Ok(Connection {
        commands: cmd_tx,
        events: weak_events,
        closed: closed_rx,
        url,
    })
}

async fn run(
    socket: Socket,
    mut commands: mpsc::UnboundedReceiver<Command>,
    events: broadcast::Sender<StreamEvent>,
    closed: watch::Sender<bool>,
) {
    let (mut sink, mut stream) = socket.split();
    let mut pending: HashMap<String, Reply> = HashMap::new();

    loop {
        tokio::select! {
            cmd = commands.recv() => match cmd {
                Some(Command::Request { id, payload, reply }) => {
                    trace!(%id, %payload, "-> request");
                    if let Err(e) = sink.send(Message::text(payload)).await {
                        let _ = reply.send(Err(e.into()));
                        break;
                    }
                    pending.insert(id, reply);
                }
                Some(Command::Cancel(id)) => { pending.remove(&id); }
                Some(Command::Close) | None => {
                    let _ = sink.close().await;
                    break;
                }
            },
            frame = stream.next() => match frame {
                Some(Ok(Message::Text(text))) => {
                    if let Some(answer) = handle_text(text.as_str(), &mut pending, &events, None) {
                        if sink.send(Message::text(answer)).await.is_err() {
                            break;
                        }
                    }
                }
                Some(Ok(Message::Binary(bytes))) => handle_binary(&bytes, &mut pending, &events),
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {} // ping / pong are answered by tungstenite
                Some(Err(e)) => {
                    warn!(error = %e, "websocket read failed");
                    break;
                }
            },
        }
    }

    commands.close();
    for (_, reply) in pending.drain() {
        let _ = reply.send(Err(Error::Closed));
    }
    while let Ok(cmd) = commands.try_recv() {
        if let Command::Request { reply, .. } = cmd {
            let _ = reply.send(Err(Error::Closed));
        }
    }
    let _ = closed.send(true);
    // `events` (the only strong sender) is dropped here, ending every EventStream.
}

/// Processes one JSON message; returns an answer to send back, if any.
fn handle_text(
    text: &str,
    pending: &mut HashMap<String, Reply>,
    events: &broadcast::Sender<StreamEvent>,
    binary: Option<Arc<[u8]>>,
) -> Option<String> {
    trace!(%text, "<- message");
    let msg: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "dropping unparsable message");
            return None;
        }
    };

    if let Some(method) = msg.get("method").and_then(Value::as_str) {
        let id = msg.get("id").filter(|v| !v.is_null());
        return match (method, id) {
            ("streamNotify", None) => {
                dispatch_event(&msg, events, binary);
                None
            }
            // The VM (or DDS) is calling a service *we* would have registered.
            // This client does not host services, so say so.
            (_, Some(id)) => Some(
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Method not found: {method}") },
                })
                .to_string(),
            ),
            _ => None,
        };
    }

    let Some(id) = msg.get("id").and_then(id_key) else {
        warn!("dropping message without id or method");
        return None;
    };
    let Some(reply) = pending.remove(&id) else {
        debug!(%id, "response for unknown (cancelled?) request");
        return None;
    };

    let outcome = match msg.get("error") {
        Some(err) => Err(match serde_json::from_value::<RpcError>(err.clone()) {
            Ok(e) => Error::Rpc(e),
            Err(e) => Error::Json(e),
        }),
        None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
    };
    let _ = reply.send(outcome);
    None
}

/// Binary event layout: `u32 dataOffset | metadata (dataOffset-4 bytes JSON) | data`.
///
/// The spec does not state the byte order of `dataOffset`; the VM writes it
/// little-endian.
fn handle_binary(
    bytes: &[u8],
    pending: &mut HashMap<String, Reply>,
    events: &broadcast::Sender<StreamEvent>,
) {
    let parsed = bytes
        .get(..4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
        .filter(|&off| off >= 4 && off <= bytes.len())
        .and_then(|off| Some((std::str::from_utf8(&bytes[4..off]).ok()?, &bytes[off..])));
    match parsed {
        Some((metadata, data)) => {
            handle_text(metadata, pending, events, Some(Arc::from(data)));
        }
        None => warn!(len = bytes.len(), "dropping malformed binary event"),
    }
}

fn dispatch_event(
    msg: &Value,
    events: &broadcast::Sender<StreamEvent>,
    binary_data: Option<Arc<[u8]>>,
) {
    let Some(params) = msg.get("params") else {
        return;
    };
    let stream_id = params
        .get("streamId")
        .and_then(Value::as_str)
        .map(StreamId::from);
    let event = params
        .get("event")
        .map(|e| serde_json::from_value::<Event>(e.clone()));
    match (stream_id, event) {
        (Some(stream_id), Some(Ok(event))) => {
            // No receivers is fine: nobody subscribed.
            let _ = events.send(StreamEvent {
                stream_id,
                event,
                binary_data,
            });
        }
        (_, Some(Err(e))) => warn!(error = %e, "could not decode event"),
        _ => warn!("malformed streamNotify"),
    }
}

/// JSON-RPC ids may be strings or numbers; we key on the string form.
fn id_key(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
