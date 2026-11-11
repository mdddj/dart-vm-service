use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Breakpoint, CpuSamplesEvent, EventKind, Frame, InstanceRef, IsolateGroupRef, IsolateRef, VmRef,
};

/// `LogRecord` carried by `Logging` events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    pub message: InstanceRef,
    pub time: i64,
    pub level: i64,
    pub sequence_number: i64,
    pub logger_name: InstanceRef,
    pub zone: InstanceRef,
    pub error: InstanceRef,
    pub stack_trace: InstanceRef,
}

/// An asynchronous notification from the VM.
///
/// Which optional fields are populated depends on [`Event::kind`]; the field docs
/// follow the spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub kind: EventKind,
    pub isolate_group: Option<IsolateGroupRef>,
    pub isolate: Option<IsolateRef>,
    pub vm: Option<VmRef>,
    #[serde(default)]
    pub timestamp: i64,

    // Pause / breakpoint events.
    pub breakpoint: Option<Breakpoint>,
    pub pause_breakpoints: Option<Vec<Breakpoint>>,
    pub top_frame: Option<Frame>,
    pub exception: Option<InstanceRef>,
    pub at_async_suspension: Option<bool>,
    /// `Inspect` event payload.
    pub inspectee: Option<InstanceRef>,

    /// `WriteEvent`: base64 encoded bytes written to stdout/stderr.
    pub bytes: Option<String>,

    /// `GC` event: type of collection.
    pub gc_type: Option<String>,

    /// `Extension` event (`developer.postEvent`).
    #[serde(rename = "extensionRPC")]
    pub extension_rpc: Option<String>,
    pub extension_kind: Option<String>,
    pub extension_data: Option<Value>,

    /// `TimelineEvents`.
    pub timeline_events: Option<Vec<Value>>,
    /// `TimelineStreamSubscriptionsUpdate`.
    pub updated_streams: Option<Vec<String>>,

    /// `IsolateReload`.
    pub status: Option<String>,
    pub reload_failure_reason: Option<String>,

    /// `Logging`.
    pub log_record: Option<LogRecord>,

    /// `TimerSignificantlyOverdue` and similar.
    pub details: Option<String>,

    /// `ServiceRegistered` / `ServiceUnregistered`.
    pub service: Option<String>,
    pub method: Option<String>,
    pub alias: Option<String>,

    /// `VMFlagUpdate`.
    pub flag: Option<String>,
    pub new_value: Option<String>,

    /// `HeapSnapshot`: whether this is the last chunk.
    pub last: Option<bool>,

    /// `UserTagChanged`.
    pub updated_tag: Option<String>,
    pub previous_tag: Option<String>,

    /// `CpuSamples`.
    pub cpu_samples: Option<CpuSamplesEvent>,
}

impl Event {
    /// Decodes [`Event::bytes`] (stdout/stderr `WriteEvent`).
    pub fn decoded_bytes(&self) -> Option<Result<Vec<u8>, base64::DecodeError>> {
        self.bytes
            .as_ref()
            .map(|b| base64::engine::general_purpose::STANDARD.decode(b))
    }

    /// Decodes [`Event::bytes`] as lossy UTF-8 text.
    pub fn text(&self) -> Option<String> {
        match self.decoded_bytes()? {
            Ok(b) => Some(String::from_utf8_lossy(&b).into_owned()),
            Err(_) => None,
        }
    }
}
