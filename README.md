# dart-vm-service

**English** | [简体中文](https://github.com/mdddj/dart-vm-service/blob/main/README.zh-CN.md)

An async ([Tokio](https://tokio.rs)) client for the **Dart VM Service Protocol** — the
JSON-RPC 2.0 over WebSocket interface that Dart and Flutter debuggers, DevTools and
`flutter attach` talk to. Use it to build tooling in Rust that can:

- inspect isolates, libraries, classes and objects
- set breakpoints, pause, step, evaluate expressions
- hot reload
- profile CPU, memory and allocations, fetch timelines and heap snapshots
- subscribe to VM events (logs, stdout/stderr, GC, debug, isolate lifecycle)
- call service extensions such as `ext.flutter.*`
- **hot reload and hot restart** (through the Flutter tool, or the plain VM where possible)
- **monitor the HTTP requests a Dart program makes**

The protocol is implemented from the official
[`service.md`](https://github.com/dart-lang/sdk/blob/main/runtime/vm/service/service.md)
specification.

## Install

```toml
[dependencies]
dart-vm-service = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
futures-util = "0.3"
```

## Quick start

Start a Dart program with the VM service enabled; it prints an address:

```text
$ dart --observe=0 app.dart
The Dart VM service is listening on http://127.0.0.1:59260/6jEJRXniY7s=/
```

```rust
use dart_vm_service::{EvalOptions, ReloadOptions, StreamId, VmService};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> dart_vm_service::Result<()> {
    // `http://…/token=/` and `ws://…/ws` addresses are both accepted.
    let vm = VmService::connect("http://127.0.0.1:59260/6jEJRXniY7s=/").await?;
    let isolate = vm.main_isolate().await?;

    // Evaluate an expression in the app's root library.
    let info = vm.get_isolate(&isolate.id).await?;
    let root = info.root_lib.expect("root library");
    let result = vm.evaluate(&isolate.id, &root.id, "1 + 2", EvalOptions::default()).await?;
    println!("{result:?}");

    // Hot reload.
    let report = vm.reload_sources(&isolate.id, ReloadOptions::default()).await?;
    println!("reload ok: {}", report.success);

    // Typed event streams.
    let mut logs = vm.subscribe(StreamId::Logging).await?;
    while let Some(event) = logs.next().await {
        println!("{:?}", event.event.kind);
    }
    Ok(())
}
```

## Hot reload and hot restart

These are not single VM RPCs. The VM's own `reloadSources` only re-reads files already on the
device; for a Flutter app started with `flutter run`, the Flutter tool registers `reloadSources`
and `hotRestart` services that recompile, sync, reload and reassemble. The helpers use whichever
is available:

```rust
use dart_vm_service::{HotReloadOptions, HotRestartOptions};

// Flutter tool attached -> full recompile + reload + reassemble.
// Plain Dart VM -> VM reload (plus `ext.flutter.reassemble` on Flutter isolates).
let result = vm.hot_reload(&isolate.id, HotReloadOptions::default()).await?;
assert!(result.is_success());

// Needs the Flutter tool; returns the new main isolate (a restart creates a new one).
let new_isolate = vm.hot_restart(HotRestartOptions::default()).await?;
```

`hot_restart` on a plain `dart run` VM returns `Error::Unsupported`: a bare Dart VM cannot
restart itself. See [`examples/hot.rs`](examples/hot.rs).

## Monitoring HTTP requests

Requests made through `dart:io`'s `HttpClient` (and therefore `package:http` and `dio` on the
Dart VM) are recorded by the VM and exposed through the `ext.dart.io.*` service extensions.

```rust
use dart_vm_service::HttpWatchOptions;
use futures_util::StreamExt;

let mut requests = Box::pin(vm.watch_http_requests(&isolate.id, HttpWatchOptions::default()).await?);
while let Some(req) = requests.next().await {
    let req = req?;
    if req.is_finished() {
        println!("{} {} -> {:?} ({:?} µs)", req.method, req.uri, req.status_code(), req.duration_micros());
    }
}
```

`get_http_profile_request` additionally returns the request and response bodies. See
[`examples/http_monitor.rs`](examples/http_monitor.rs).

## Design

- `VmService` is a cheap `Clone` handle; one background task owns the socket.
- Every public RPC of the spec has a typed method. RPC names are available as the `RpcMethod`
  enum.
- Fixed argument sets are enums: `StreamId`, `StepOption`, `ExceptionPauseMode`,
  `SourceReportKind`, `VmFlag`, `TimelineStream`, `IdAssignmentPolicy`, …
- RPCs with many optional parameters take an options struct (`ReloadOptions`, `EvalOptions`, …).
- Enums the VM sends have an `Unknown` variant, so a newer VM never breaks decoding.
- RPCs documented as `X|Sentinel` return `Err(Error::Sentinel(..))` for the sentinel case.
- VM errors are `Error::Rpc` with a typed `RpcErrorCode` (e.g. `ExpressionCompilationError`).
- Escape hatches: `call_method`, `call_extension` (for `ext.*`) and `call_raw`.

## Examples

```text
cargo run --example inspect      -- <vm-service-url>
cargo run --example http_monitor -- <vm-service-url> [--bodies]
cargo run --example hot          -- <vm-service-url> reload|restart
```

## Compatibility and limitations

Tested against a Dart 3.13.7 VM (service protocol 4.21). The spec at the time of writing is 4.23.

- Only `ws://` is built in; `wss://` would need a TLS feature of `tokio-tungstenite`.
- `registerService` can register a service, but this crate does not host handlers: calls to it
  are answered with "method not found".
- The byte order of the binary-event header is not stated in the spec; little-endian is used
  (heap snapshots decode correctly).
- The `ext.dart.io.*` extensions and the `TimelineStream` names are not part of `service.md`;
  their shapes were checked against a Dart 3.13 VM, and fields are optional where possible.
- Hot reload/restart through the Flutter tool was verified against Flutter 3.47 on macOS
  (`flutter run -d macos`). When a reload fails to compile, the tool answers with an empty
  internal error; the details are only printed in the tool's own console.
- `ext.flutter.*` extensions can be called with `call_extension` but have no typed wrappers.
- HTTP monitoring only sees `dart:io` traffic and only after profiling is enabled; it is not
  available for Flutter Web.
- Minimum supported Rust version: 1.85.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
