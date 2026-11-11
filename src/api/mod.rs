//! Typed wrappers for every public RPC, grouped by area. All of them are inherent
//! methods on [`VmService`](crate::VmService).
//!
//! Conventions:
//! * ids (`isolate_id`, `script_id`, …) are `&str` — the protocol treats them as opaque;
//! * RPCs documented as returning `X|Sentinel` return `Err(Error::Sentinel(..))` for the
//!   sentinel case;
//! * RPCs returning `Success` return `Result<()>`;
//! * fixed value sets are enums, RPCs with many optional parameters take an options struct.

mod debug;
mod io;
mod isolate;
mod objects;
mod profiler;
mod streams;
mod vm;

pub use io::HttpWatchOptions;
