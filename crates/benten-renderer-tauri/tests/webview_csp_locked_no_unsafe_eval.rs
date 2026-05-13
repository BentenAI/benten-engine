//! G24-E wave-7 RED-PHASE pin (T3 + br-r1-11; substantive).
//!
//! Asserts the Tauri webview is loaded with a locked Content-Security-
//! Policy header that:
//!
//! - Allows `script-src 'self' 'wasm-unsafe-eval'` (wasm32 admin UI bundle).
//! - Allows `connect-src 'self' tauri://*` (in-process IPC origin).
//! - Restricts `style-src` and `font-src` to `'self'`.
//! - Sets `default-src 'none'`.
//! - Does NOT include `'unsafe-eval'` or `'unsafe-inline'`.
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7 lands the webview-construction surface.
//!
//! ## Closes
//!
//! T3 + br-r1-11 (`r2-test-landscape.md` §2.10 row 3 + threat-model §T3
//! line ~108)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (CSP enforcement landing)"]
fn webview_csp_is_locked_no_unsafe_eval_or_unsafe_inline() {
    // Production arm (G24-E wave-7):
    //
    //   let renderer = TauriRenderer::new_with_manifest(admin_ui_v0_manifest());
    //   let csp = renderer.webview_csp_header();
    //   assert!(csp.contains("default-src 'none'"));
    //   assert!(csp.contains("script-src 'self' 'wasm-unsafe-eval'"));
    //   assert!(csp.contains("connect-src 'self' tauri://*"));
    //   assert!(!csp.contains("'unsafe-eval'") || csp.contains("'wasm-unsafe-eval'"));
    //   assert!(!csp.contains("'unsafe-inline'"));
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
