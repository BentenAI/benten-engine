//! G24-E wave-7 LANDED pin (T3 + br-r1-11; substantive).
//!
//! Asserts the Tauri webview is loaded with a locked Content-Security-
//! Policy header that:
//!
//! - Allows `script-src 'self' 'wasm-unsafe-eval'` (wasm32 admin UI bundle).
//! - Allows `connect-src 'self' tauri://*` (in-process IPC origin).
//! - Restricts `style-src` and `font-src` to `'self'`.
//! - Sets `default-src 'none'`.
//! - Does NOT include `'unsafe-eval'` (classic eval) or
//!   `'unsafe-inline'`.
//!
//! ## Closes
//!
//! T3 + br-r1-11 (`r2-test-landscape.md` §2.10 row 3 + threat-model §T3
//! line ~108)

#![allow(clippy::unwrap_used)]

use benten_renderer_tauri::{AdminUiManifest, TauriRenderer, WEBVIEW_CSP_HEADER};

#[test]
fn webview_csp_is_locked_no_unsafe_eval_or_unsafe_inline() {
    let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::default());
    let csp = renderer.webview_csp_header();

    assert!(
        csp.contains("default-src 'none'"),
        "missing default-src: {csp}"
    );
    assert!(
        csp.contains("script-src 'self' 'wasm-unsafe-eval'"),
        "missing wasm-relaxed script-src: {csp}"
    );
    assert!(
        csp.contains("connect-src 'self' tauri://*"),
        "missing tauri connect-src: {csp}"
    );
    assert!(csp.contains("style-src 'self'"));
    assert!(csp.contains("font-src 'self'"));

    // `'wasm-unsafe-eval'` is the wasm-only relaxation; classic
    // `'unsafe-eval'` MUST NOT appear. Strip the wasm-relaxed token
    // before scanning so the substring check is precise.
    let stripped = csp.replace("'wasm-unsafe-eval'", "");
    assert!(
        !stripped.contains("'unsafe-eval'"),
        "classic unsafe-eval present after wasm-token strip: {stripped}"
    );
    assert!(
        !csp.contains("'unsafe-inline'"),
        "unsafe-inline present: {csp}"
    );
}

#[test]
fn webview_csp_constant_matches_renderer_method() {
    // Drift-defense: the const + method must agree byte-for-byte so
    // future agents can't accidentally route one consumer through a
    // weaker header.
    let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::default());
    assert_eq!(renderer.webview_csp_header(), WEBVIEW_CSP_HEADER);
}
