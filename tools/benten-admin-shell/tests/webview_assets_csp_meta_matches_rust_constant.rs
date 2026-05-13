//! Phase-4-Foundation R6-FP-E drift-detector for the
//! `webview-assets/index.html` meta-CSP defense-in-depth duplicate.
//!
//! The Rust constant `benten_renderer_tauri::WEBVIEW_CSP_HEADER` is the
//! source of truth for the locked CSP at webview load (T3 defense rung
//! 3). The static `webview-assets/index.html` carries a `<meta
//! http-equiv="Content-Security-Policy" content="...">` duplicate so
//! the same directives apply even if the embedding context strips the
//! server-set header. This test asserts the two stay byte-equivalent.
//!
//! Without this drift detector a refactor of the Rust constant could
//! land + silently leave the HTML defense weaker (or vice versa,
//! HTML edited without updating the Rust constant). Either direction
//! is a T3 rung 3 weakening.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

#[test]
fn webview_assets_index_html_csp_meta_matches_rust_constant() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("webview-assets");
    path.push("index.html");
    let html = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let canonical = benten_renderer_tauri::WEBVIEW_CSP_HEADER;
    assert!(
        html.contains(canonical),
        "webview-assets/index.html meta CSP must contain the canonical \
         `WEBVIEW_CSP_HEADER` byte-for-byte. Drift detected. \
         Expected directive string: {canonical:?}"
    );
}
