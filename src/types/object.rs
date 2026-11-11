//! Heap/program objects: libraries, classes, functions, fields, scripts, ….

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CodeKind, ErrorKind, InstanceRef, SourceLocation, ValueRef};

/// `@Object`: the lowest common denominator of every reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRef {
    pub id: String,
    pub fixed_id: Option<bool>,
}

// ---- Library ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDependency {
    pub is_import: bool,
    pub is_deferred: bool,
    #[serde(default)]
    pub prefix: String,
    pub target: LibraryRef,
    #[serde(default)]
    pub shows: Vec<String>,
    #[serde(default)]
    pub hides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    pub uri: String,
    #[serde(default)]
    pub debuggable: bool,
    #[serde(default)]
    pub dependencies: Vec<LibraryDependency>,
    #[serde(default)]
    pub scripts: Vec<ScriptRef>,
    #[serde(default)]
    pub variables: Vec<FieldRef>,
    #[serde(default)]
    pub functions: Vec<FuncRef>,
    #[serde(default)]
    pub classes: Vec<ClassRef>,
}

// ---- Script ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub id: String,
    pub fixed_id: Option<bool>,
    pub uri: String,
    pub library: Option<LibraryRef>,
    pub line_offset: Option<i64>,
    pub column_offset: Option<i64>,
    pub source: Option<String>,
    /// `[line, tokenPos, column, tokenPos, column, …]` rows.
    pub token_pos_table: Option<Vec<Vec<i64>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptList {
    pub scripts: Vec<ScriptRef>,
}

// ---- Class -----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    pub location: Option<SourceLocation>,
    pub library: Option<LibraryRef>,
    #[serde(default)]
    pub type_parameters: Vec<InstanceRef>,
    pub error: Option<ErrorRef>,
    #[serde(default, rename = "abstract")]
    pub is_abstract: bool,
    #[serde(default, rename = "const")]
    pub is_const: bool,
    #[serde(default)]
    pub is_sealed: bool,
    #[serde(default)]
    pub is_mixin_class: bool,
    #[serde(default)]
    pub is_base_class: bool,
    #[serde(default)]
    pub is_interface_class: bool,
    #[serde(default)]
    pub is_final: bool,
    #[serde(default)]
    pub trace_allocations: bool,
    #[serde(rename = "super")]
    pub super_class: Option<ClassRef>,
    pub super_type: Option<InstanceRef>,
    #[serde(default)]
    pub interfaces: Vec<InstanceRef>,
    pub mixin: Option<InstanceRef>,
    #[serde(default)]
    pub fields: Vec<FieldRef>,
    #[serde(default)]
    pub functions: Vec<FuncRef>,
    #[serde(default)]
    pub subclasses: Vec<ClassRef>,
    pub ffi_layout: Option<FfiStructLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfiStructField {
    pub name: String,
    pub native_type: String,
    pub offset: i64,
    pub size: i64,
    pub length: Option<i64>,
    pub array_element_type: Option<String>,
    pub kind: Option<String>,
    pub fields: Option<Vec<FfiStructField>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FfiStructLayout {
    pub size: i64,
    pub kind: String,
    pub fields: Vec<FfiStructField>,
}

// ---- Function --------------------------------------------------------------

/// `@Function` (named `FuncRef` to avoid confusion with Rust's `Fn` family).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuncRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    /// `@Library | @Class | @Function`; kept as raw JSON, inspect its `type`.
    pub owner: Option<Value>,
    #[serde(default, rename = "static")]
    pub is_static: bool,
    #[serde(default, rename = "const")]
    pub is_const: bool,
    #[serde(default)]
    pub implicit: bool,
    #[serde(default, rename = "abstract")]
    pub is_abstract: bool,
    #[serde(default)]
    pub is_getter: bool,
    #[serde(default)]
    pub is_setter: bool,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    #[serde(flatten)]
    pub base: FuncRef,
    pub signature: Option<InstanceRef>,
    pub code: Option<CodeRef>,
}

// ---- Field -----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    pub owner: Option<ObjectRef>,
    pub declared_type: Option<InstanceRef>,
    #[serde(default, rename = "const")]
    pub is_const: bool,
    #[serde(default, rename = "final")]
    pub is_final: bool,
    #[serde(default, rename = "static")]
    pub is_static: bool,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    #[serde(flatten)]
    pub base: FieldRef,
    pub static_value: Option<ValueRef>,
}

// ---- Code / Context --------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    pub kind: Option<CodeKind>,
    /// `@Function | NativeFunction`.
    pub function: Option<Value>,
}

pub type Code = CodeRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub length: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextElement {
    pub value: ValueRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub length: i64,
    pub parent: Option<ContextRef>,
    #[serde(default)]
    pub variables: Vec<ContextElement>,
}

// ---- Error -----------------------------------------------------------------

/// `@Error`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRef {
    #[serde(default)]
    pub id: String,
    pub fixed_id: Option<bool>,
    pub kind: ErrorKind,
    #[serde(default)]
    pub message: String,
}

/// `Error` (a VM-side error object, not this crate's [`Error`](crate::Error)).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorObject {
    /// Empty for errors returned inline (e.g. by `setFlag`).
    #[serde(default)]
    pub id: String,
    pub fixed_id: Option<bool>,
    /// Absent for errors returned inline (e.g. by `setFlag`).
    pub kind: Option<ErrorKind>,
    #[serde(default)]
    pub message: String,
    pub exception: Option<InstanceRef>,
    pub stacktrace: Option<InstanceRef>,
}

// ---- TypeArguments ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeArgumentsRef {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeArguments {
    pub id: String,
    pub fixed_id: Option<bool>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub types: Vec<InstanceRef>,
}
