//! Types of the `dart:io` service extensions (`ext.dart.io.*`): HTTP and socket
//! profiling. These are **not** part of `service.md`; the shapes below were
//! checked against a Dart 3.13 VM. Every field not guaranteed by that check is
//! optional so missing data never breaks decoding.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Remote/local endpoint of a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub local_port: i64,
    pub remote_address: String,
    pub remote_port: i64,
}

/// Lifecycle marker of a request ("Connection established", "Request sent",
/// "Waiting (TTFB)", "Content Download", …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpProfileEvent {
    /// Microseconds on the VM's monotonic clock.
    pub timestamp: i64,
    pub event: String,
    pub arguments: Option<Value>,
}

/// What the app sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestData {
    #[serde(default)]
    pub headers: BTreeMap<String, Vec<String>>,
    pub connection_info: Option<ConnectionInfo>,
    pub content_length: Option<i64>,
    #[serde(default)]
    pub cookies: Vec<Value>,
    pub follow_redirects: Option<bool>,
    pub max_redirects: Option<i64>,
    pub method: Option<String>,
    pub persistent_connection: Option<bool>,
    pub uri: Option<String>,
    pub filter_key: Option<String>,
    /// Set when the request failed before a response arrived
    /// (connection refused, DNS failure, …).
    pub error: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

/// What came back. Absent while the response has not started (or for failures).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponseData {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    #[serde(default)]
    pub headers: BTreeMap<String, Vec<String>>,
    pub status_code: Option<i64>,
    pub reason_phrase: Option<String>,
    pub content_length: Option<i64>,
    pub compression_state: Option<String>,
    #[serde(default)]
    pub cookies: Vec<Value>,
    pub is_redirect: Option<bool>,
    pub persistent_connection: Option<bool>,
    #[serde(default)]
    pub redirects: Vec<Value>,
    pub connection_info: Option<ConnectionInfo>,
    pub error: Option<String>,
}

/// One HTTP request as listed by `getHttpProfile` (`@HttpProfileRequest`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProfileRequestSummary {
    /// Pass to [`VmService::get_http_profile_request`](crate::VmService::get_http_profile_request).
    pub id: String,
    pub isolate_id: String,
    pub method: String,
    pub uri: String,
    /// Microseconds on the VM's monotonic clock.
    pub start_time: i64,
    /// Absent while the request is still in flight.
    pub end_time: Option<i64>,
    #[serde(default)]
    pub events: Vec<HttpProfileEvent>,
    pub request: Option<HttpRequestData>,
    pub response: Option<HttpResponseData>,
}

impl HttpProfileRequestSummary {
    /// HTTP status code, once a response has started.
    pub fn status_code(&self) -> Option<i64> {
        self.response.as_ref()?.status_code
    }

    /// Error message if the request failed before getting a response.
    pub fn error(&self) -> Option<&str> {
        self.request.as_ref()?.error.as_deref()
    }

    /// The request is over (completed, failed or cancelled).
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }

    /// Wall time in microseconds, if finished.
    pub fn duration_micros(&self) -> Option<i64> {
        Some(self.end_time? - self.start_time)
    }

    /// Request header (case-insensitive name), first value.
    pub fn request_header(&self, name: &str) -> Option<&str> {
        header(&self.request.as_ref()?.headers, name)
    }

    /// Response header (case-insensitive name), first value.
    pub fn response_header(&self, name: &str) -> Option<&str> {
        header(&self.response.as_ref()?.headers, name)
    }
}

fn header<'a>(h: &'a BTreeMap<String, Vec<String>>, name: &str) -> Option<&'a str> {
    h.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .and_then(|(_, v)| v.first())
        .map(String::as_str)
}

/// `getHttpProfile`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpProfile {
    /// Pass back as `updated_since` to receive only newer changes.
    pub timestamp: i64,
    pub requests: Vec<HttpProfileRequestSummary>,
}

/// `getHttpProfileRequest`: the summary plus the captured bodies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProfileRequest {
    #[serde(flatten)]
    pub summary: HttpProfileRequestSummary,
    #[serde(default)]
    pub request_body: Vec<u8>,
    #[serde(default)]
    pub response_body: Vec<u8>,
}

impl HttpProfileRequest {
    /// Response body as lossy UTF-8.
    pub fn response_text(&self) -> String {
        String::from_utf8_lossy(&self.response_body).into_owned()
    }

    /// Request body as lossy UTF-8.
    pub fn request_text(&self) -> String {
        String::from_utf8_lossy(&self.request_body).into_owned()
    }
}

/// State of `httpEnableTimelineLogging` (which also switches HTTP profiling on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpTimelineLoggingState {
    pub enabled: bool,
}

/// State of `socketProfilingEnabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocketProfilingState {
    pub enabled: bool,
}

/// `getSocketProfile`. The per-socket entries are kept as raw JSON: their shape
/// was not observable in the check (they stayed empty for HTTP traffic).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocketProfile {
    #[serde(default)]
    pub sockets: Vec<Value>,
}
