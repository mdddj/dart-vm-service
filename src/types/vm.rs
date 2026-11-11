use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Breakpoint, ErrorObject, Event, ExceptionPauseMode, IdAssignmentPolicy,
    IdZoneBackingBufferKind, InstanceRef, LibraryRef, TimelineStream,
};

/// `@VM`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmRef {
    pub name: String,
}

/// The VM itself (`getVM`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vm {
    pub name: String,
    pub architecture_bits: i64,
    #[serde(rename = "hostCPU")]
    pub host_cpu: String,
    pub operating_system: String,
    #[serde(rename = "targetCPU")]
    pub target_cpu: String,
    pub version: String,
    pub pid: i64,
    pub start_time: i64,
    pub isolates: Vec<IsolateRef>,
    #[serde(default)]
    pub isolate_groups: Vec<IsolateGroupRef>,
    #[serde(default)]
    pub system_isolates: Vec<IsolateRef>,
    #[serde(default)]
    pub system_isolate_groups: Vec<IsolateGroupRef>,
}

/// `@Isolate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolateRef {
    pub id: String,
    pub number: String,
    pub name: String,
    #[serde(default)]
    pub is_system_isolate: bool,
    #[serde(default)]
    pub isolate_group_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsolateFlag {
    pub name: String,
    #[serde(rename = "valueAsString")]
    pub value_as_string: String,
}

/// `Isolate` (`getIsolate`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Isolate {
    pub id: String,
    pub number: String,
    pub name: String,
    #[serde(default)]
    pub is_system_isolate: bool,
    #[serde(default)]
    pub isolate_group_id: String,
    #[serde(default)]
    pub isolate_flags: Vec<IsolateFlag>,
    pub start_time: i64,
    pub runnable: bool,
    #[serde(default)]
    pub live_ports: i64,
    #[serde(default)]
    pub pause_on_exit: bool,
    pub pause_event: Option<Event>,
    pub root_lib: Option<LibraryRef>,
    #[serde(default)]
    pub libraries: Vec<LibraryRef>,
    #[serde(default)]
    pub breakpoints: Vec<Breakpoint>,
    pub error: Option<ErrorObject>,
    pub exception_pause_mode: Option<ExceptionPauseMode>,
    #[serde(default, rename = "extensionRPCs")]
    pub extension_rpcs: Vec<String>,
}

impl Isolate {
    /// Whether `ext.xxx` has been registered as a service extension on this isolate.
    pub fn has_extension(&self, name: &str) -> bool {
        self.extension_rpcs.iter().any(|e| e == name)
    }
}

/// `@IsolateGroup`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolateGroupRef {
    pub id: String,
    pub number: String,
    pub name: String,
    #[serde(default)]
    pub is_system_isolate_group: bool,
}

/// `IsolateGroup` (`getIsolateGroup`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolateGroup {
    pub id: String,
    pub number: String,
    pub name: String,
    #[serde(default)]
    pub is_system_isolate_group: bool,
    #[serde(default)]
    pub isolates: Vec<IsolateRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
    pub external_usage: i64,
    pub heap_capacity: i64,
    pub heap_usage: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMemoryItem {
    pub name: String,
    pub description: String,
    pub size: i64,
    #[serde(default)]
    pub children: Vec<ProcessMemoryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMemoryUsage {
    pub root: ProcessMemoryItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    pub name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub modified: bool,
    pub value_as_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagList {
    pub flags: Vec<Flag>,
}

/// Result of `reloadSources`.
///
/// The VM may attach extra details (e.g. `notices`); they are kept in `details`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloadReport {
    pub success: bool,
    #[serde(flatten)]
    pub details: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UriList {
    pub uris: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortList {
    pub ports: Vec<InstanceRef>,
}

/// `getVMTimelineFlags`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineFlags {
    pub recorder_name: String,
    pub available_streams: Vec<String>,
    pub recorded_streams: Vec<String>,
}

/// `getVMTimeline`: Chrome trace events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub trace_events: Vec<Value>,
    pub time_origin_micros: i64,
    pub time_extent_micros: i64,
}

impl TimelineFlags {
    /// [`available_streams`](Self::available_streams) as typed values.
    pub fn available(&self) -> Vec<TimelineStream> {
        self.available_streams
            .iter()
            .map(|s| TimelineStream::from(s.as_str()))
            .collect()
    }

    /// [`recorded_streams`](Self::recorded_streams) as typed values.
    pub fn recorded(&self) -> Vec<TimelineStream> {
        self.recorded_streams
            .iter()
            .map(|s| TimelineStream::from(s.as_str()))
            .collect()
    }
}

/// `getPerfettoVMTimeline`; `trace` is base64-encoded Perfetto protobuf data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfettoTimeline {
    pub trace: String,
    pub time_origin_micros: i64,
    pub time_extent_micros: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Microtask {
    pub id: i64,
    #[serde(rename = "stackTrace")]
    pub stack_trace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedMicrotasks {
    pub timestamp: i64,
    pub microtasks: Vec<Microtask>,
}

/// Result of `createIdZone`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdZone {
    pub id: String,
    pub backing_buffer_kind: IdZoneBackingBufferKind,
    pub id_assignment_policy: IdAssignmentPolicy,
}
