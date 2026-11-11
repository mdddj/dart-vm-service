//! # dart-vm-service
//!
//! **English** | [简体中文](https://github.com/mdddj/dart-vm-service/blob/main/README.zh-CN.md)
//!
//! An async (Tokio) client for the **Dart VM Service Protocol** — the JSON-RPC 2.0
//! over WebSocket interface that Dart/Flutter debuggers, DevTools and
//! `flutter attach` use. Based on the protocol spec in
//! [`dart-lang/sdk/runtime/vm/service/service.md`](https://github.com/dart-lang/sdk/blob/main/runtime/vm/service/service.md).
//!
//! ```no_run
//! use dart_vm_service::{ReloadOptions, StreamId, VmService};
//! use futures_util::StreamExt;
//!
//! # async fn demo() -> dart_vm_service::Result<()> {
//! // The address printed by `dart run --observe` / `flutter run`.
//! let vm = VmService::connect("http://127.0.0.1:8181/AbCdEf=/").await?;
//! println!("protocol {:?}", vm.get_version().await?);
//!
//! let isolate = vm.main_isolate().await?;
//!
//! // Typed streams: pick the stream with an enum, consume with `futures::Stream`.
//! let mut debug = vm.subscribe(StreamId::Debug).await?;
//! tokio::spawn(async move {
//!     while let Some(ev) = debug.next().await {
//!         println!("{:?}", ev.event.kind);
//!     }
//! });
//!
//! // Hot reload.
//! vm.reload_sources(&isolate.id, ReloadOptions::default()).await?;
//! # Ok(()) }
//! ```
//!
//! ## Design
//!
//! * [`VmService`] is a cheap `Clone` handle; one background task owns the socket.
//! * Every public RPC has a typed method (see [`api` docs](VmService)); the RPC
//!   names live in [`RpcMethod`] and fixed argument sets are enums
//!   ([`StreamId`], [`StepOption`], [`ExceptionPauseMode`], [`SourceReportKind`], [`VmFlag`], …).
//! * `X|Sentinel` results map `Sentinel` to [`Error::Sentinel`].
//! * VM-added enum values never break decoding (`Unknown` variants).
//! * [`VmService::hot_reload`] / [`VmService::hot_restart`] pick the Flutter tool's recompiling
//!   services when attached; [`VmService::registered_services`] lists what other clients offer.
//! * [`VmService::watch_http_requests`] streams the `dart:io` HTTP requests of an isolate.
//! * Escape hatches: [`VmService::call_method`], [`VmService::call_extension`]
//!   (for `ext.*` service extensions) and [`VmService::call_raw`].

mod api;
mod client;
mod connection;
mod error;
mod method;
mod options;
mod params;
pub mod types;
mod url;

pub use api::{
    FLUTTER_TOOLS_ALIAS, HotReloadOptions, HotReloadResult, HotReloadStrategy, HotRestartOptions,
    HttpWatchOptions,
};
pub use client::{EventStream, VmService, VmServiceBuilder};
pub use error::{Error, Result, RpcError, RpcErrorCode};
pub use method::RpcMethod;
pub use options::{
    AllocationProfileOptions, EvalOptions, GetObjectOptions, InstancesOptions, InvokeOptions,
    ReloadOptions, SourceReportOptions,
};
pub use types::*;
pub use url::to_websocket_url;
