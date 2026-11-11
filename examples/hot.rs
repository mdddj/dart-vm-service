//! Hot reload / hot restart of a running Flutter (or Dart) app.
//!
//! ```text
//! cargo run --example hot -- <vm-service-url> reload|restart
//! ```

use dart_vm_service::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .expect("usage: hot <vm-service-url> reload|restart");
    let action = args.next().unwrap_or_else(|| "reload".into());

    let vm = VmService::connect(&url).await?;
    println!("registered services: {:?}", vm.registered_services().await?);
    let isolate = vm.main_isolate().await?;
    println!("main isolate: {}", isolate.id);

    match action.as_str() {
        "reload" => {
            let result = vm
                .hot_reload(&isolate.id, HotReloadOptions::default())
                .await?;
            println!(
                "hot reload -> {result:?} (success: {})",
                result.is_success()
            );
        }
        "restart" => {
            let new_isolate = vm.hot_restart(HotRestartOptions::default()).await?;
            println!(
                "hot restart -> new main isolate: {:?}",
                new_isolate.map(|i| i.id)
            );
        }
        other => panic!("unknown action `{other}`"),
    }
    Ok(())
}
