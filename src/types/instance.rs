//! Dart object instances.

use serde::{Deserialize, Serialize};

use super::{
    ClassRef, ContextRef, FieldRef, FuncRef, InstanceKind, ObjectRef, TypeArgumentsRef, ValueRef,
};
use base64::Engine as _;

/// `@Instance`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    pub kind: InstanceKind,
    pub identity_hash_code: Option<i64>,
    /// Absent on a few synthetic instances (e.g. `Null` in some VM versions).
    pub class: Option<ClassRef>,
    pub value_as_string: Option<String>,
    pub value_as_string_is_truncated: Option<bool>,
    /// Length of a String, List, Map, Set, Record or typed-data instance.
    pub length: Option<i64>,
    pub name: Option<String>,
    pub type_class: Option<ClassRef>,
    pub parameterized_class: Option<ClassRef>,
    pub return_type: Option<Box<InstanceRef>>,
    pub parameters: Option<Vec<Parameter>>,
    pub type_parameters: Option<Vec<InstanceRef>>,
    pub pattern: Option<Box<InstanceRef>>,
    pub closure_function: Option<FuncRef>,
    pub closure_context: Option<ContextRef>,
    pub closure_receiver: Option<Box<InstanceRef>>,
    pub port_id: Option<i64>,
    pub allocation_location: Option<Box<InstanceRef>>,
    pub debug_name: Option<String>,
    pub label: Option<String>,
}

impl InstanceRef {
    /// Human readable value for primitives (`valueAsString`), if the VM supplied it.
    pub fn as_display(&self) -> Option<&str> {
        self.value_as_string.as_deref()
    }
}

/// Function-type parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub parameter_type: InstanceRef,
    #[serde(default)]
    pub fixed: bool,
    pub name: Option<String>,
    pub required: Option<bool>,
}

/// Field of an instance (or record position).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundField {
    pub decl: Option<FieldRef>,
    /// Field name, or position (integer) for record fields.
    pub name: serde_json::Value,
    pub value: ValueRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapAssociation {
    pub key: ValueRef,
    pub value: ValueRef,
}

/// `Instance` (`getObject` on an instance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    #[serde(flatten)]
    pub base: InstanceRef,
    pub offset: Option<i64>,
    pub count: Option<i64>,
    pub fields: Option<Vec<BoundField>>,
    pub elements: Option<Vec<ValueRef>>,
    pub associations: Option<Vec<MapAssociation>>,
    /// Base64 encoded bytes of a typed data instance.
    pub bytes: Option<String>,
    pub mirror_referent: Option<ObjectRef>,
    pub is_case_sensitive: Option<bool>,
    pub is_multi_line: Option<bool>,
    pub property_key: Option<ObjectRef>,
    pub property_value: Option<ObjectRef>,
    pub target: Option<ObjectRef>,
    pub type_arguments: Option<TypeArgumentsRef>,
    pub parameter_index: Option<i64>,
    pub target_type: Option<Box<InstanceRef>>,
    pub bound: Option<Box<InstanceRef>>,
    pub callback: Option<Box<InstanceRef>>,
    pub callback_address: Option<Box<InstanceRef>>,
    pub all_entries: Option<Box<InstanceRef>>,
    pub value: Option<Box<InstanceRef>>,
    pub token: Option<Box<InstanceRef>>,
    pub detach: Option<Box<InstanceRef>>,
}

impl Instance {
    /// Decodes [`Instance::bytes`] (typed data) from base64.
    pub fn decoded_bytes(&self) -> Option<Result<Vec<u8>, base64::DecodeError>> {
        self.bytes
            .as_ref()
            .map(|b| base64::engine::general_purpose::STANDARD.decode(b))
    }
}
