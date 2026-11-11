//! Error types for the Dart VM Service client.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{ErrorObject, Sentinel};

/// Convenient result alias used across the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Every error this crate can produce.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The URL given to [`VmService::connect`](crate::VmService::connect) is unusable.
    #[error("invalid VM service url `{0}`: {1}")]
    InvalidUrl(String, String),

    /// WebSocket transport failure.
    #[error("websocket error: {0}")]
    WebSocket(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    /// A payload could not be (de)serialized.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The VM answered with a JSON-RPC `error` object.
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError),

    /// The VM answered with a `Sentinel` (e.g. the isolate/object was collected or expired).
    #[error("sentinel returned: {:?} ({})", .0.kind, .0.value_as_string)]
    Sentinel(Sentinel),

    /// The VM answered a `Success|Error` RPC (such as `setFlag`) with an `Error` object.
    #[error("VM reported an error: {}", .0.message)]
    Vm(Box<ErrorObject>),

    /// A lookup (e.g. "the main isolate") came back empty.
    #[error("not found: {0}")]
    NotFound(String),

    /// The connection is closed (by either side).
    #[error("connection closed")]
    Closed,

    /// No response arrived in time.
    #[error("request timed out after {0:?}")]
    Timeout(Duration),

    /// The peer sent something that violates the protocol.
    #[error("protocol violation: {0}")]
    Protocol(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(Box::new(e))
    }
}

impl Error {
    /// The RPC error code if this is [`Error::Rpc`].
    pub fn rpc_code(&self) -> Option<RpcErrorCode> {
        match self {
            Error::Rpc(e) => Some(e.code()),
            _ => None,
        }
    }

    /// `true` when this is the `Collected`/`Expired`/... sentinel answer.
    pub fn is_sentinel(&self) -> bool {
        matches!(self, Error::Sentinel(_))
    }
}

/// A JSON-RPC `error` object as sent by the VM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[error("[{code}] {message}")]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    /// Typed view of [`RpcError::code`].
    pub fn code(&self) -> RpcErrorCode {
        RpcErrorCode::from(self.code)
    }

    /// The `data.details` string the VM usually attaches, if any.
    pub fn details(&self) -> Option<&str> {
        self.data.as_ref()?.get("details")?.as_str()
    }
}

/// Error codes: the JSON-RPC 2.0 standard ones plus the VM-specific 100..=115
/// range documented in the Service Protocol ("RPCs, Requests, and Responses").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RpcErrorCode {
    // --- JSON-RPC 2.0 standard ---
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    // --- Dart VM specific ---
    /// 100: the operation is unable to complete because a feature is disabled.
    FeatureDisabled,
    /// 101: this operation is only valid when the VM is paused.
    VmMustBePaused,
    /// 102: the VM is unable to add a breakpoint at the specified line or function.
    CannotAddBreakpoint,
    /// 103: the client is already subscribed to the specified stream.
    StreamAlreadySubscribed,
    /// 104: the client is not subscribed to the specified stream.
    StreamNotSubscribed,
    /// 105: this operation cannot happen until the isolate is runnable.
    IsolateMustBeRunnable,
    /// 106: this operation is only valid when the isolate is paused.
    IsolateMustBePaused,
    /// 107: the isolate could not be resumed.
    CannotResume,
    /// 108: the isolate is currently processing another reload request.
    IsolateIsReloading,
    /// 109: the isolate has an unhandled exception and can no longer be reloaded.
    IsolateCannotBeReloaded,
    /// 110: failed to find differences in the last hot reload request.
    IsolateMustHaveReloaded,
    /// 111: a service with this name was already registered by this client.
    ServiceAlreadyRegistered,
    /// 112: failed to fulfil a service request, the handler is probably gone.
    ServiceDisappeared,
    /// 113: request to compile an expression failed.
    ExpressionCompilationError,
    /// 114: the timeline request could not be completed with the current configuration.
    InvalidTimelineRequest,
    /// 115: information about queued microtasks cannot be retrieved.
    CannotGetQueuedMicrotasks,
    /// Any code not listed above (service extensions use their own codes).
    Other(i64),
}

impl RpcErrorCode {
    /// Numeric wire value.
    pub fn as_i64(self) -> i64 {
        use RpcErrorCode::*;
        match self {
            ParseError => -32700,
            InvalidRequest => -32600,
            MethodNotFound => -32601,
            InvalidParams => -32602,
            InternalError => -32603,
            FeatureDisabled => 100,
            VmMustBePaused => 101,
            CannotAddBreakpoint => 102,
            StreamAlreadySubscribed => 103,
            StreamNotSubscribed => 104,
            IsolateMustBeRunnable => 105,
            IsolateMustBePaused => 106,
            CannotResume => 107,
            IsolateIsReloading => 108,
            IsolateCannotBeReloaded => 109,
            IsolateMustHaveReloaded => 110,
            ServiceAlreadyRegistered => 111,
            ServiceDisappeared => 112,
            ExpressionCompilationError => 113,
            InvalidTimelineRequest => 114,
            CannotGetQueuedMicrotasks => 115,
            Other(c) => c,
        }
    }
}

impl From<i64> for RpcErrorCode {
    fn from(code: i64) -> Self {
        use RpcErrorCode::*;
        match code {
            -32700 => ParseError,
            -32600 => InvalidRequest,
            -32601 => MethodNotFound,
            -32602 => InvalidParams,
            -32603 => InternalError,
            100 => FeatureDisabled,
            101 => VmMustBePaused,
            102 => CannotAddBreakpoint,
            103 => StreamAlreadySubscribed,
            104 => StreamNotSubscribed,
            105 => IsolateMustBeRunnable,
            106 => IsolateMustBePaused,
            107 => CannotResume,
            108 => IsolateIsReloading,
            109 => IsolateCannotBeReloaded,
            110 => IsolateMustHaveReloaded,
            111 => ServiceAlreadyRegistered,
            112 => ServiceDisappeared,
            113 => ExpressionCompilationError,
            114 => InvalidTimelineRequest,
            115 => CannotGetQueuedMicrotasks,
            other => Other(other),
        }
    }
}
