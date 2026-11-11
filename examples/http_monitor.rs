//! Prints every HTTP request a Dart program makes through `dart:io`.
//!
//! ```text
//! dart --observe=0 app.dart        # prints http://127.0.0.1:PORT/TOKEN=/
//! cargo run --example http_monitor -- http://127.0.0.1:PORT/TOKEN=/ [--bodies]
//! ```

use dart_vm_service::*;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .expect("usage: http_monitor <vm-service-url> [--bodies]");
    let show_bodies = args.any(|a| a == "--bodies");

    let vm = VmService::connect(&url).await?;
    let isolate = vm.main_isolate().await?;
    println!(
        "watching HTTP requests of isolate {} ({})",
        isolate.name, isolate.id
    );

    let mut requests = Box::pin(
        vm.watch_http_requests(&isolate.id, HttpWatchOptions::default())
            .await?,
    );
    while let Some(req) = requests.next().await {
        let req = req?;
        if !req.is_finished() {
            continue; // print each request once, when it has completed
        }
        match (req.error(), req.status_code()) {
            (Some(err), _) => println!("{:>6} {}  FAILED: {err}", req.method, req.uri),
            (None, status) => println!(
                "{:>6} {}  -> {}  {:.1} ms  {}",
                req.method,
                req.uri,
                status.map_or("-".into(), |s| s.to_string()),
                req.duration_micros().unwrap_or(0) as f64 / 1000.0,
                req.response_header("content-type").unwrap_or(""),
            ),
        }
        if show_bodies {
            let full = vm.get_http_profile_request(&isolate.id, &req.id).await?;
            if !full.request_body.is_empty() {
                println!("         request body : {}", full.request_text());
            }
            println!("         response body: {}", full.response_text());
        }
    }
    Ok(())
}
