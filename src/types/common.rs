use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::SentinelKind;

/// "This object is gone" answer returned instead of the requested type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sentinel {
    pub kind: SentinelKind,
    #[serde(default)]
    pub value_as_string: String,
}

/// Protocol version spoken by the VM (`getVersion`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
}

/// One protocol (core or a middleware extension such as DDS).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Protocol {
    pub protocol_name: String,
    pub major: i64,
    pub minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolList {
    pub protocols: Vec<Protocol>,
}

/// Monotonic microsecond clock (`getVMTimelineMicros`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp {
    pub timestamp: i64,
}

/// A `[origin, origin + extent]` window on the VM's monotonic microsecond clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub origin_micros: i64,
    pub extent_micros: i64,
}

impl TimeRange {
    pub fn new(origin_micros: i64, extent_micros: i64) -> Self {
        Self {
            origin_micros,
            extent_micros,
        }
    }
}

/// Resulting `ExtensionData`/`TimelineEvent`: free-form JSON objects.
pub type JsonObject = serde_json::Map<String, Value>;
