# dart-vm-service

[English](https://github.com/mdddj/dart-vm-service/blob/main/README.md) | **简体中文**

基于 [Tokio](https://tokio.rs) 的 **Dart VM Service 协议**异步客户端。该协议是通过 WebSocket 传输的
JSON-RPC 2.0 接口,Dart / Flutter 的调试器、DevTools、`flutter attach` 都使用它。
你可以用它在 Rust 中开发工具,实现:

- 查看 isolate、library、class 和各种对象
- 设置断点、暂停、单步、执行表达式
- 热重载和热重启(通过 Flutter 工具,或在可行时直接通过 VM)
- 分析 CPU、内存和对象分配,获取时间线和堆快照
- 订阅 VM 事件(日志、stdout/stderr、GC、调试、isolate 生命周期)
- 调用 `ext.flutter.*` 等服务扩展
- **监控 Dart 程序发出的 HTTP 请求**

协议实现依据官方规范
[`service.md`](https://github.com/dart-lang/sdk/blob/main/runtime/vm/service/service.md)。

## 安装

```toml
[dependencies]
dart-vm-service = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
futures-util = "0.3"
```

## 快速开始

启动一个开启了 VM service 的 Dart 程序,它会打印出服务地址:

```text
$ dart --observe=0 app.dart
The Dart VM service is listening on http://127.0.0.1:59260/6jEJRXniY7s=/
```

```rust
use dart_vm_service::{EvalOptions, ReloadOptions, StreamId, VmService};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> dart_vm_service::Result<()> {
    // `http://…/token=/` 和 `ws://…/ws` 两种地址都可以。
    let vm = VmService::connect("http://127.0.0.1:59260/6jEJRXniY7s=/").await?;
    let isolate = vm.main_isolate().await?;

    // 在应用的根 library 中执行表达式。
    let info = vm.get_isolate(&isolate.id).await?;
    let root = info.root_lib.expect("root library");
    let result = vm.evaluate(&isolate.id, &root.id, "1 + 2", EvalOptions::default()).await?;
    println!("{result:?}");

    // 通过 VM 重载源码。
    let report = vm.reload_sources(&isolate.id, ReloadOptions::default()).await?;
    println!("reload ok: {}", report.success);

    // 带类型的事件流。
    let mut logs = vm.subscribe(StreamId::Logging).await?;
    while let Some(event) = logs.next().await {
        println!("{:?}", event.event.kind);
    }
    Ok(())
}
```

## 热重载与热重启

这两个操作并不是单个 VM RPC。VM 自带的 `reloadSources` 只会重新读取设备上**已有**的文件;
对于用 `flutter run` 启动的 Flutter 应用,Flutter 工具会向 VM 注册 `reloadSources` 和
`hotRestart` 两个服务,它们会重新编译、同步文件、重载并重建界面。本库的辅助函数会自动选择可用的方式:

```rust
use dart_vm_service::{HotReloadOptions, HotRestartOptions};

// 有 Flutter 工具 -> 完整的重新编译 + 重载 + 重建界面。
// 普通 Dart VM   -> VM 重载(Flutter isolate 上再调用 `ext.flutter.reassemble`)。
let result = vm.hot_reload(&isolate.id, HotReloadOptions::default()).await?;
assert!(result.is_success());

// 需要 Flutter 工具;返回新的主 isolate(重启会创建新的 isolate)。
let new_isolate = vm.hot_restart(HotRestartOptions::default()).await?;
```

在普通 `dart run` 的 VM 上调用 `hot_restart` 会返回 `Error::Unsupported`:裸的 Dart VM
无法自己重启。示例见 [`examples/hot.rs`](examples/hot.rs)。

## 监控 HTTP 请求

通过 `dart:io` 的 `HttpClient` 发出的请求(因此在 Dart VM 上的 `package:http` 和 `dio` 也包含在内)
会被 VM 记录,并通过 `ext.dart.io.*` 服务扩展暴露出来。

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

`get_http_profile_request` 还能返回请求体和响应体。见
[`examples/http_monitor.rs`](examples/http_monitor.rs)。

## 设计

- `VmService` 是可以随意 `clone` 的轻量句柄,由一个后台任务独占 socket。
- 规范中的每个公开 RPC 都有带类型的方法,RPC 名称也提供为 `RpcMethod` 枚举。
- 固定的参数取值都是枚举:`StreamId`、`StepOption`、`ExceptionPauseMode`、
  `SourceReportKind`、`VmFlag`、`TimelineStream`、`IdAssignmentPolicy` 等。
- 可选参数较多的 RPC 使用选项结构体(`ReloadOptions`、`EvalOptions` 等)。
- VM 返回的枚举带有 `Unknown` 变体,因此 VM 新增取值不会导致解码失败。
- 规范中返回 `X|Sentinel` 的 RPC,遇到 Sentinel 时返回 `Err(Error::Sentinel(..))`。
- VM 错误为 `Error::Rpc`,带有类型化的 `RpcErrorCode`(例如 `ExpressionCompilationError`)。
- 兜底接口:`call_method`、`call_extension`(用于 `ext.*`)和 `call_raw`。

## 示例

```text
cargo run --example inspect      -- <vm-service-url>
cargo run --example http_monitor -- <vm-service-url> [--bodies]
cargo run --example hot          -- <vm-service-url> reload|restart
```

## 兼容性与限制

在 Dart 3.13.7 的 VM(服务协议 4.21)上测试过。撰写本文时规范版本为 4.23。

- 只内置了 `ws://`;`wss://` 需要为 `tokio-tungstenite` 启用 TLS 特性。
- `registerService` 可以注册服务,但本库不托管处理函数:别人调用你注册的服务时,会收到"方法不存在"。
- 规范没有说明二进制事件头的字节序,这里按小端处理(堆快照可以正确解码)。
- `ext.dart.io.*` 扩展和 `TimelineStream` 的名称不属于 `service.md`;
  它们的结构是在 Dart 3.13 的 VM 上实测的,能设为可选的字段都设为了可选。
- 通过 Flutter 工具热重载/热重启,在 macOS 上的 Flutter 3.47(`flutter run -d macos`)中验证过。
  当重载因编译失败时,工具只返回一个空消息的内部错误,详细信息只会打印在工具自己的控制台里。
- `ext.flutter.*` 扩展可以用 `call_extension` 调用,但没有带类型的封装。
- HTTP 监控只能看到 `dart:io` 的流量,且需要先开启记录;Flutter Web 不可用。
- 最低支持的 Rust 版本:1.85。

## 许可证

可任选以下之一:

- Apache License, Version 2.0([LICENSE-APACHE](LICENSE-APACHE))
- MIT license([LICENSE-MIT](LICENSE-MIT))
