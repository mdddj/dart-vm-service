//! Debugger related types: breakpoints, stack, frames, source reports.

use serde::{Deserialize, Serialize};

use super::{CodeRef, ErrorRef, FrameKind, FuncRef, InstanceRef, ScriptRef, ValueRef};

/// A resolved source position (`SourceLocation`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    pub script: ScriptRef,
    pub token_pos: i64,
    pub end_token_pos: Option<i64>,
    pub line: Option<i64>,
    pub column: Option<i64>,
}

/// A position that could not be resolved yet (deferred libraries etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedSourceLocation {
    pub script: Option<ScriptRef>,
    pub script_uri: Option<String>,
    pub token_pos: Option<i64>,
    pub line: Option<i64>,
    pub column: Option<i64>,
}

/// Either flavour of breakpoint location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum BreakpointLocation {
    Resolved(SourceLocation),
    Unresolved(UnresolvedSourceLocation),
}

impl<'de> Deserialize<'de> for BreakpointLocation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or_default();
        if ty == "UnresolvedSourceLocation" {
            serde_json::from_value(v)
                .map(Self::Unresolved)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(v)
                .map(Self::Resolved)
                .map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub id: String,
    pub breakpoint_number: i64,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub resolved: bool,
    pub is_synthetic_async_continuation: Option<bool>,
    pub location: BreakpointLocation,
}

/// A local variable visible in a frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundVariable {
    pub name: String,
    pub value: ValueRef,
    pub static_type: Option<InstanceRef>,
    pub declaration_token_pos: Option<i64>,
    pub scope_start_token_pos: Option<i64>,
    pub scope_end_token_pos: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub index: i64,
    pub function: Option<FuncRef>,
    pub code: Option<CodeRef>,
    pub location: Option<SourceLocation>,
    pub vars: Option<Vec<BoundVariable>>,
    pub kind: Option<FrameKind>,
}

/// A queued isolate message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub index: i64,
    pub name: String,
    pub message_object_id: String,
    pub size: i64,
    pub handler: Option<FuncRef>,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    pub frames: Vec<Frame>,
    pub async_causal_frames: Option<Vec<Frame>>,
    pub awaiter_frames: Option<Vec<Frame>>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub truncated: bool,
}

// ---- Source report ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReportCoverage {
    #[serde(default)]
    pub hits: Vec<i64>,
    #[serde(default)]
    pub misses: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReportRange {
    pub script_index: i64,
    pub start_pos: i64,
    pub end_pos: i64,
    #[serde(default)]
    pub compiled: bool,
    pub error: Option<ErrorRef>,
    pub coverage: Option<SourceReportCoverage>,
    pub possible_breakpoints: Option<Vec<i64>>,
    pub branch_coverage: Option<SourceReportCoverage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceReport {
    pub ranges: Vec<SourceReportRange>,
    pub scripts: Vec<ScriptRef>,
}
