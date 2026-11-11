//! Tiny helper for building JSON-RPC named parameters.

use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Default)]
pub(crate) struct Params(Map<String, Value>);

impl Params {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a required parameter.
    pub fn with<T: Serialize>(mut self, key: &str, value: T) -> Self {
        // Every type passed here is a string, number, bool, enum or string map:
        // serialization to JSON cannot fail.
        let value = serde_json::to_value(value).expect("parameter is always JSON-serializable");
        self.0.insert(key.to_owned(), value);
        self
    }

    /// Adds an optional parameter; `None` is omitted from the request.
    pub fn opt<T: Serialize>(self, key: &str, value: Option<T>) -> Self {
        match value {
            Some(v) => self.with(key, v),
            None => self,
        }
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }
}
