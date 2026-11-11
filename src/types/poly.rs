//! Polymorphic answers: the spec returns "`A|B|Sentinel`" and distinguishes the
//! variants by the `type` property. `@Foo` (reference) and `Foo` (full object) are
//! both mapped onto one variant per concept.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::{
    Breakpoint, Class, Code, Context, ErrorObject, ErrorRef, Field, Function, Instance,
    InstanceRef, Library, Script, Sentinel, TypeArguments, TypeArgumentsRef,
};

fn type_name(v: &Value) -> &str {
    v.get("type")
        .and_then(Value::as_str)
        .map(|t| t.trim_start_matches('@'))
        .unwrap_or_default()
}

fn parse<T: for<'a> Deserialize<'a>, E: serde::de::Error>(v: &Value) -> Result<T, E> {
    T::deserialize(v).map_err(E::custom)
}

/// `@Instance | @TypeArguments | Sentinel`: the value slot of variables, fields,
/// list elements, map entries …
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)] // InstanceRef is by far the most common payload
pub enum ValueRef {
    Instance(InstanceRef),
    TypeArguments(TypeArgumentsRef),
    Sentinel(Sentinel),
    /// Anything else (kept verbatim).
    Other(Value),
}

impl<'de> Deserialize<'de> for ValueRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        Ok(match type_name(&v) {
            "Instance" | "Null" => ValueRef::Instance(parse(&v)?),
            "TypeArguments" => ValueRef::TypeArguments(parse(&v)?),
            "Sentinel" => ValueRef::Sentinel(parse(&v)?),
            _ => ValueRef::Other(v),
        })
    }
}

impl ValueRef {
    pub fn as_instance(&self) -> Option<&InstanceRef> {
        match self {
            ValueRef::Instance(i) => Some(i),
            _ => None,
        }
    }
}

/// Outcome of `evaluate`, `evaluateInFrame` and `invoke` (`Sentinel` becomes
/// [`Error::Sentinel`](crate::Error::Sentinel)).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)] // success (InstanceRef) is the common case
pub enum EvalResult {
    /// The expression evaluated to this instance.
    Instance(InstanceRef),
    /// A runtime error happened while evaluating.
    Error(ErrorRef),
}

impl EvalResult {
    pub fn into_instance(self) -> Option<InstanceRef> {
        match self {
            EvalResult::Instance(i) => Some(i),
            EvalResult::Error(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for EvalResult {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        match type_name(&v) {
            "Instance" | "Null" => Ok(EvalResult::Instance(parse(&v)?)),
            "Error" => Ok(EvalResult::Error(parse(&v)?)),
            other => Err(D::Error::custom(format!(
                "unexpected eval result type `{other}`"
            ))),
        }
    }
}

/// Result of `getObject`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Obj {
    Breakpoint(Breakpoint),
    Class(Class),
    Code(Code),
    Context(Context),
    Error(ErrorObject),
    Field(Field),
    Function(Function),
    Instance(Instance),
    Library(Library),
    Script(Script),
    TypeArguments(TypeArguments),
    /// Types this crate does not model (kept verbatim).
    Other(Value),
}

impl<'de> Deserialize<'de> for Obj {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        Ok(match type_name(&v) {
            "Breakpoint" => Obj::Breakpoint(parse(&v)?),
            "Class" => Obj::Class(parse(&v)?),
            "Code" => Obj::Code(parse(&v)?),
            "Context" => Obj::Context(parse(&v)?),
            "Error" => Obj::Error(parse(&v)?),
            "Field" => Obj::Field(parse(&v)?),
            "Function" => Obj::Function(parse(&v)?),
            "Instance" | "Null" => Obj::Instance(parse(&v)?),
            "Library" => Obj::Library(parse(&v)?),
            "Script" => Obj::Script(parse(&v)?),
            "TypeArguments" => Obj::TypeArguments(parse(&v)?),
            _ => Obj::Other(v),
        })
    }
}
