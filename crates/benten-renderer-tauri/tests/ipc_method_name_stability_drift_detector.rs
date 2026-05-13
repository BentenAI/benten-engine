//! G24-E wave-7 LANDED pin — gap #1a closure (`r2-test-landscape.md` §5).
//!
//! Drift-detector for the Tauri IPC method-name allowlist. Couples to
//! the cargo-public-api baseline file at
//! `docs/public-api/benten-renderer-tauri.json` — the
//! `_ipc_method_name_allowlist_baseline._anticipated_method_set` array
//! is the authoritative method-name allowlist. The production IPC
//! dispatch (G24-E wave-7) exposes exactly that method-name set, and
//! any divergence requires an explicit baseline update +
//! manifest-review (T3 defense: silent IPC-surface expansion is a
//! manifest-bypass risk).
//!
//! ## Closes
//!
//! Gap #1a (`r2-test-landscape.md` §5 gap-list)

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use benten_renderer_tauri::TauriRenderer;

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs/public-api/benten-renderer-tauri.json")
}

#[test]
fn ipc_method_name_set_at_head_matches_public_api_baseline() {
    let raw = std::fs::read_to_string(baseline_path()).expect("read baseline file");
    let baseline: serde_json::Value = serde_json::from_str(&raw).expect("parse baseline JSON");
    let baseline_methods: BTreeSet<String> =
        baseline["_ipc_method_name_allowlist_baseline"]["_anticipated_method_set"]
            .as_array()
            .expect("_anticipated_method_set is array")
            .iter()
            .map(|v| v.as_str().expect("method is string").to_string())
            .collect();

    let live_methods: BTreeSet<String> =
        TauriRenderer::ipc_method_allowlist().into_iter().collect();

    assert_eq!(
        live_methods, baseline_methods,
        "Tauri IPC method-name allowlist drifted from baseline. \
         Update docs/public-api/benten-renderer-tauri.json AND re-review \
         admin UI v0 manifest cap-bindings before merging. \
         live={live_methods:?} baseline={baseline_methods:?}"
    );
}

#[test]
fn ipc_method_name_baseline_file_exists_and_has_anticipated_method_set_key() {
    let path = baseline_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("baseline file missing at {}: {e}", path.display()));
    assert!(
        raw.contains("_anticipated_method_set"),
        "baseline file at {} missing _anticipated_method_set key",
        path.display()
    );
    assert!(
        raw.contains("_ipc_method_name_allowlist_baseline"),
        "baseline file at {} missing _ipc_method_name_allowlist_baseline key",
        path.display()
    );
}
