//! Phase 2b R3 (R3-E) — wasm32-unknown-unknown bundle-size cap.
//!
//! TDD red-phase. Pin source: wasm-r1-7 (≤500KB gzipped browser-bundle
//! cap) + plan §3 G10-A-browser must-pass + plan §3.1 NEW
//! `wasm-browser.yml` workflow.
//!
//! Browser bundle size is a hard product constraint: above ~500KB
//! gzipped, page-load latency spikes break the "personal AI assistant
//! ships in <1s cold start" Phase-3 commitment. The cap is enforced
//! by both:
//!   - this Rust-side integration test (drift-detector against the
//!     committed `bundle-size-budget.toml`),
//!   - the `bundle-size.yml` workflow (regression guard on PR).
//!
//! Two cross-axis assertions:
//!   1. The gzipped browser-bundle is ≤ 500KB.
//!   2. The browser-bundle does NOT contain the napi node-binary
//!      (separate `wasm-r1-6` finding — bundle conflation would silently
//!      bloat the browser path).
//!
//! **Status:** RED-PHASE (Phase 2b G10-A-browser pending). The browser
//! bundle (`bindings/napi/src/wasm_browser.rs` + `wasm-browser.yml`) does
//! not yet exist.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// Plan-pinned hard cap from wasm-r1-7. Tighter caps belong in
/// `bundle-size-budget.toml` (drift-detected separately); this is the
/// last-line defense.
const BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 500 * 1024;

/// `wasm32_unknown_unknown_bundle_size_under_threshold` — plan §3
/// G10-A-browser must-pass + R2 §2.3.
#[test]
#[ignore = "Phase 2b G10-A-browser pending — wasm-browser bundle build unimplemented"]
fn wasm32_unknown_unknown_bundle_size_under_threshold() {
    // The bundle is produced by `wasm-browser.yml` and committed to a
    // canonical artifact path under `bindings/napi/dist/browser/`. Until
    // G10-A-browser lands, the path does not exist; the test fails loudly
    // with a pointer to the implementation surface.
    let bundle_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bindings/napi/dist/browser/benten_engine.wasm.gz");

    let bytes = std::fs::read(&bundle_path).unwrap_or_else(|e| {
        panic!(
            "wasm32-unknown-unknown gzipped bundle not found at {} ({}). \
             G10-A-browser must build it via wasm-browser.yml + commit a \
             stable artifact under bindings/napi/dist/browser/",
            bundle_path.display(),
            e
        );
    });

    assert!(
        bytes.len() <= BROWSER_BUNDLE_MAX_BYTES_GZIPPED,
        "browser bundle is {} bytes gzipped, exceeds wasm-r1-7 cap of \
         {} bytes — investigate dep bloat / dead-code-elimination",
        bytes.len(),
        BROWSER_BUNDLE_MAX_BYTES_GZIPPED
    );
}

/// `browser_bundle_excludes_napi_node_binary` — plan §3 G10-A-browser
/// must-pass + R2 §2.3 + wasm-r1-6 (don't ship the node-binary in the
/// browser bundle).
///
/// Asserts the dist artifact contains no `*.node` filenames (heuristic
/// drift detector — if napi-rs's two-target build pipeline ever
/// inadvertently bundles the node binary into the browser dist, this
/// fires).
#[test]
#[ignore = "Phase 2b G10-A-browser pending"]
fn browser_bundle_excludes_napi_node_binary() {
    let dist_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bindings/napi/dist/browser");

    let dir = std::fs::read_dir(&dist_dir).unwrap_or_else(|e| {
        panic!(
            "browser dist directory not found at {} ({}) — \
             G10-A-browser owns this artifact root",
            dist_dir.display(),
            e
        );
    });

    for entry in dir {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        assert!(
            !name_str.ends_with(".node"),
            "browser bundle dir must NOT contain a napi node binary \
             ({}); wasm-r1-6 forbids bundling the node target into \
             the browser distribution",
            name_str
        );
    }
}
