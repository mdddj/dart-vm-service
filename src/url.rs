//! Turning the URL a Dart/Flutter process prints into a WebSocket URL.

use url::Url;

use crate::error::{Error, Result};

/// Normalises any of the usual VM service addresses to a `ws://…/ws` URL.
///
/// * `http://127.0.0.1:8181/abc=/`  → `ws://127.0.0.1:8181/abc=/ws`
/// * `https://host:1/abc=/`         → `wss://host:1/abc=/ws`
/// * `ws://127.0.0.1:8181/abc=/ws`  → unchanged
/// * `ws://127.0.0.1:8181/abc=/`    → `ws://127.0.0.1:8181/abc=/ws`
pub fn to_websocket_url(input: &str) -> Result<Url> {
    let bad = |why: &str| Error::InvalidUrl(input.to_owned(), why.to_owned());

    let mut url = Url::parse(input).map_err(|e| bad(&e.to_string()))?;

    let scheme = match url.scheme() {
        "http" | "ws" => "ws",
        "https" | "wss" => "wss",
        other => return Err(bad(&format!("unsupported scheme `{other}`"))),
    };
    url.set_scheme(scheme)
        .map_err(|()| bad("cannot set scheme"))?;

    let mut segments: Vec<String> = url
        .path_segments()
        .map(|s| s.filter(|p| !p.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();
    if segments.last().map(String::as_str) != Some("ws") {
        segments.push("ws".to_owned());
    }
    url.set_path(&segments.join("/"));
    Ok(url)
}
