//! Profiling, allocation and heap-graph types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ClassRef, MemoryUsage, ObjectRef, ValueRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassHeapStats {
    pub class: ClassRef,
    pub accumulated_size: i64,
    pub bytes_current: i64,
    pub instances_accumulated: i64,
    pub instances_current: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationProfile {
    pub members: Vec<ClassHeapStats>,
    pub memory_usage: MemoryUsage,
    pub date_last_accumulator_reset: Option<i64>,
    pub date_last_service_gc: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassList {
    pub classes: Vec<ClassRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuSample {
    pub tid: i64,
    pub timestamp: i64,
    pub vm_tag: Option<String>,
    pub user_tag: Option<String>,
    pub truncated: Option<bool>,
    /// Indices into the enclosing `functions` table, leaf first.
    pub stack: Vec<i64>,
    pub identity_hash_code: Option<i64>,
    pub class_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileFunction {
    pub kind: String,
    pub inclusive_ticks: i64,
    pub exclusive_ticks: i64,
    pub resolved_url: String,
    /// `@Function | NativeFunction`.
    pub function: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuSamples {
    pub sample_period: i64,
    pub max_stack_depth: i64,
    pub sample_count: i64,
    pub time_origin_micros: i64,
    pub time_extent_micros: i64,
    pub pid: i64,
    pub functions: Vec<ProfileFunction>,
    pub samples: Vec<CpuSample>,
}

/// Payload of the `CpuSamples` event (functions are plain objects here).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuSamplesEvent {
    pub sample_period: i64,
    pub max_stack_depth: i64,
    pub sample_count: i64,
    pub time_origin_micros: i64,
    pub time_extent_micros: i64,
    pub pid: i64,
    pub functions: Vec<Value>,
    pub samples: Vec<CpuSample>,
}

/// `getPerfettoCpuSamples`; `samples` is base64 encoded Perfetto data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfettoCpuSamples {
    pub sample_period: i64,
    pub max_stack_depth: i64,
    pub sample_count: i64,
    pub time_origin_micros: i64,
    pub time_extent_micros: i64,
    pub pid: i64,
    pub samples: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundReference {
    pub source: ObjectRef,
    pub parent_list_index: Option<i64>,
    /// `@Field | string | int`.
    pub parent_field: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboundReferences {
    pub references: Vec<InboundReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainingObject {
    pub value: ValueRef,
    pub parent_list_index: Option<i64>,
    pub parent_map_key: Option<ValueRef>,
    /// `string | int`.
    pub parent_field: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainingPath {
    pub length: i64,
    pub gc_root_type: String,
    pub elements: Vec<RetainingObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSet {
    pub total_count: i64,
    pub instances: Vec<ValueRef>,
}
