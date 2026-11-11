//! Data types of the VM Service Protocol.
//!
//! Naming follows the spec: `FooRef` is the spec's `@Foo` reference (a subset of
//! `Foo`). All optional protocol properties are `Option`/`Vec` here.

mod common;
mod debug;
mod enums;
mod event;
mod instance;
mod io;
mod object;
mod poly;
mod profiler;
mod stream;
mod vm;

pub use common::*;
pub use debug::*;
pub use enums::*;
pub use event::*;
pub use instance::*;
pub use io::*;
pub use object::*;
pub use poly::*;
pub use profiler::*;
pub use stream::*;
pub use vm::*;
