//! Event streams.

use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Event;

/// Well-known stream ids accepted by `streamListen` / `streamCancel`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StreamId {
    /// `VMUpdate`, `VMFlagUpdate`.
    Vm,
    /// `IsolateStart`, `IsolateRunnable`, `IsolateExit`, `IsolateUpdate`,
    /// `IsolateReload`, `ServiceExtensionAdded`.
    Isolate,
    /// Pause*, Resume, Breakpoint*, Inspect, None.
    Debug,
    /// `CpuSamples`, `UserTagChanged`.
    Profiler,
    /// `GC`.
    Gc,
    /// `Extension` (events posted with `developer.postEvent`).
    Extension,
    /// `TimelineEvents`, `TimelineStreamSubscriptionsUpdate`.
    Timeline,
    /// `Logging`.
    Logging,
    /// `ServiceRegistered`, `ServiceUnregistered`.
    Service,
    /// `HeapSnapshot`.
    HeapSnapshot,
    /// `TimerSignificantlyOverdue`.
    Timer,
    /// `WriteEvent` for stdout (only some embedders).
    Stdout,
    /// `WriteEvent` for stderr (only some embedders).
    Stderr,
    /// A stream this crate does not know.
    Other(String),
}

impl StreamId {
    pub fn as_str(&self) -> &str {
        match self {
            StreamId::Vm => "VM",
            StreamId::Isolate => "Isolate",
            StreamId::Debug => "Debug",
            StreamId::Profiler => "Profiler",
            StreamId::Gc => "GC",
            StreamId::Extension => "Extension",
            StreamId::Timeline => "Timeline",
            StreamId::Logging => "Logging",
            StreamId::Service => "Service",
            StreamId::HeapSnapshot => "HeapSnapshot",
            StreamId::Timer => "Timer",
            StreamId::Stdout => "Stdout",
            StreamId::Stderr => "Stderr",
            StreamId::Other(s) => s,
        }
    }
}

impl From<&str> for StreamId {
    fn from(s: &str) -> Self {
        match s {
            "VM" => StreamId::Vm,
            "Isolate" => StreamId::Isolate,
            "Debug" => StreamId::Debug,
            "Profiler" => StreamId::Profiler,
            "GC" => StreamId::Gc,
            "Extension" => StreamId::Extension,
            "Timeline" => StreamId::Timeline,
            "Logging" => StreamId::Logging,
            "Service" => StreamId::Service,
            "HeapSnapshot" => StreamId::HeapSnapshot,
            "Timer" => StreamId::Timer,
            "Stdout" => StreamId::Stdout,
            "Stderr" => StreamId::Stderr,
            other => StreamId::Other(other.to_owned()),
        }
    }
}

impl std::fmt::Display for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for StreamId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StreamId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(StreamId::from(s.as_str()))
    }
}

/// One `streamNotify` delivery.
#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub stream_id: StreamId,
    pub event: Event,
    /// Bulk payload of a *binary event* (e.g. heap snapshot chunks); `None` for
    /// ordinary text events.
    pub binary_data: Option<Arc<[u8]>>,
}
