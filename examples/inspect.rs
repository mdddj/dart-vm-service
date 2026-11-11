//! Connects to a running Dart/Flutter VM and exercises a cross-section of the API.
//!
//! ```text
//! dart --observe=0 app.dart        # prints http://127.0.0.1:PORT/TOKEN=/
//! cargo run --example inspect -- http://127.0.0.1:PORT/TOKEN=/
//! ```

use std::time::Duration;

use dart_vm_service::*;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::args()
        .nth(1)
        .expect("usage: inspect <vm-service-url>");
    let vm = VmService::connect(&url).await?;
    println!("connected to {}", vm.url());

    println!("version: {:?}", vm.get_version().await?);
    println!(
        "protocols: {:?}",
        vm.get_supported_protocols().await?.protocols
    );
    let info = vm.get_vm().await?;
    println!(
        "vm: {} {} pid={} isolates={}",
        info.name,
        info.version,
        info.pid,
        info.isolates.len()
    );

    let main = vm.main_isolate().await?;
    println!("main isolate: {} ({})", main.name, main.id);
    let isolate = vm.get_isolate(&main.id).await?;
    println!(
        "  runnable={} rootLib={:?} extensions={:?}",
        isolate.runnable,
        isolate.root_lib.as_ref().map(|l| &l.uri),
        isolate.extension_rpcs
    );

    // Subscribe to several streams at once.
    let mut events = vm.events();
    for s in [
        StreamId::Isolate,
        StreamId::Debug,
        StreamId::Extension,
        StreamId::Logging,
        StreamId::Stdout,
    ] {
        vm.subscribe(s).await?;
    }
    // Subscribing a second time must not fail.
    vm.subscribe(StreamId::Debug).await?;

    // Evaluate in the root library.
    let root = isolate.root_lib.as_ref().expect("root lib");
    match vm
        .evaluate(
            &main.id,
            &root.id,
            "counter.value + 1000",
            EvalOptions::default(),
        )
        .await?
    {
        EvalResult::Instance(i) => println!("evaluate -> {:?} {:?}", i.kind, i.value_as_string),
        EvalResult::Error(e) => println!("evaluate error: {}", e.message),
    }
    match vm
        .evaluate(&main.id, &root.id, "nope(", EvalOptions::default())
        .await
    {
        Err(e) => println!("bad expression -> {:?} / {e}", e.rpc_code()),
        Ok(r) => println!("unexpected: {r:?}"),
    }

    // Library -> class -> fields.
    let lib = vm.get_library(&main.id, &root.id).await?;
    println!(
        "library {} has {} classes, {} vars",
        lib.uri,
        lib.classes.len(),
        lib.variables.len()
    );
    for c in &lib.classes {
        let class = vm.get_class(&main.id, &c.id).await?;
        println!(
            "  class {} fields={:?}",
            class.name,
            class.fields.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
    }

    // Memory / profiler.
    println!("memory: {:?}", vm.get_memory_usage(&main.id).await?);
    let prof = vm
        .get_allocation_profile(&main.id, AllocationProfileOptions::default().gc(true))
        .await?;
    println!("allocation profile: {} classes", prof.members.len());
    println!(
        "process memory root: {}",
        vm.get_process_memory_usage().await?.root.name
    );
    println!("flags: {} entries", vm.get_flag_list().await?.flags.len());

    vm.set_flag(VmFlag::Profiler, true).await?;
    let now = vm.get_vm_timeline_micros().await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let samples = vm
        .get_cpu_samples(
            &main.id,
            TimeRange::new(now.timestamp - 5_000_000, 10_000_000),
        )
        .await?;
    println!(
        "cpu samples: {} (period {}us)",
        samples.sample_count, samples.sample_period
    );

    // Timeline.
    let flags = vm.get_vm_timeline_flags().await?;
    println!("timeline streams available: {:?}", flags.available_streams);

    // Scripts and source report.
    let scripts = vm.get_scripts(&main.id).await?;
    println!("scripts: {}", scripts.scripts.len());
    let report = vm
        .get_source_report(
            &main.id,
            &[SourceReportKind::PossibleBreakpoints],
            SourceReportOptions::default().script_id(lib.scripts[0].id.clone()),
        )
        .await?;
    println!("source report ranges: {}", report.ranges.len());

    // Service extension.
    let echo = vm
        .call_extension(&main.id, "ext.demo.echo", [("msg", "hi")])
        .await?;
    println!("extension -> {echo}");

    // Hot reload.
    let reload = vm
        .reload_sources(&main.id, ReloadOptions::default())
        .await?;
    println!("reload success={}", reload.success);

    // Pause / stack / resume.
    vm.pause(&main.id).await?;
    let stack = vm.get_stack(&main.id, None, None).await?;
    println!("stack frames while paused: {}", stack.frames.len());
    vm.resume(&main.id).await?;

    // Errors map to typed codes.
    let err = vm.get_isolate("isolates/does-not-exist").await.unwrap_err();
    println!("bad isolate -> {err}");
    let err = vm
        .call_raw("noSuchMethod", serde_json::json!({}))
        .await
        .unwrap_err();
    println!("bad method -> {:?}", err.rpc_code());

    // Collect events for a moment.
    let deadline = tokio::time::sleep(Duration::from_millis(1600));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            Some(ev) = events.next() => {
                let detail = match ev.event.kind {
                    EventKind::WriteEvent => ev.event.text().unwrap_or_default().trim().to_owned(),
                    EventKind::Extension => format!("{:?} {:?}", ev.event.extension_kind, ev.event.extension_data),
                    EventKind::Logging => format!("{:?}", ev.event.log_record.as_ref().map(|l| l.sequence_number)),
                    _ => String::new(),
                };
                println!("event [{}] {:?} {detail}", ev.stream_id, ev.event.kind);
            }
            _ = &mut deadline => break,
        }
    }

    vm.close();
    vm.closed().await;
    println!("closed");
    Ok(())
}
