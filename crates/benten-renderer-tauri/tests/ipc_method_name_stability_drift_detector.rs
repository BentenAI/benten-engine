//! G24-E wave-7 RED-PHASE pin — gap #1a closure (`r2-test-landscape.md` §5).
//!
//! Drift-detector for the Tauri IPC method-name allowlist. Couples to
//! the cargo-public-api baseline file at
//! `docs/public-api/benten-renderer-tauri.json` — the
//! `_ipc_method_name_allowlist_baseline._anticipated_method_set` array
//! is the authoritative method-name allowlist. The production IPC
//! dispatch (G24-E wave-7) MUST expose exactly that method-name set,
//! and any divergence requires an explicit baseline update +
//! manifest-review (T3 defense: silent IPC-surface expansion is a
//! manifest-bypass risk).
//!
//! ## Why this pin matters
//!
//! Gap #1a in `r2-test-landscape.md` §5: the original G24-E pin set
//! enumerated IPC allowlist behavior pins (unknown-method rejection,
//! per-method cap-binding) but NO drift-detector against the canonical
//! method-name list. Without this pin, a future change could quietly
//! add an IPC method that bypasses the manifest review surface —
//! defeating the per-method cap-binding defense at the policy layer
//! (the cap-binding works if the method exists, but the method-name
//! itself was never disclosed to the user at install).
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7. The baseline file
//! [`docs/public-api/benten-renderer-tauri.json`] exists at R3 with the
//! anticipated method set; production dispatch surface lands at G24-E.
//!
//! ## Closes
//!
//! Gap #1a (`r2-test-landscape.md` §5 gap-list)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (IPC method-name stability + baseline coupling)"]
fn ipc_method_name_set_at_head_matches_public_api_baseline() {
    // Production arm (G24-E wave-7):
    //
    //   // Read the baseline from docs/public-api/benten-renderer-tauri.json
    //   let baseline_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs/public-api/benten-renderer-tauri.json");
    //   let baseline: serde_json::Value =
    //       serde_json::from_str(&std::fs::read_to_string(baseline_path).unwrap()).unwrap();
    //   let baseline_methods: std::collections::BTreeSet<String> = baseline
    //       ["_ipc_method_name_allowlist_baseline"]
    //       ["_anticipated_method_set"]
    //       .as_array().unwrap().iter()
    //       .map(|v| v.as_str().unwrap().to_string())
    //       .collect();
    //
    //   // Read the live allowlist from the renderer surface.
    //   let live_methods: std::collections::BTreeSet<String> =
    //       TauriRenderer::ipc_method_allowlist().iter().cloned().collect();
    //
    //   assert_eq!(live_methods, baseline_methods,
    //       "Tauri IPC method-name allowlist drifted from baseline. \
    //        Update docs/public-api/benten-renderer-tauri.json AND re-review \
    //        admin UI v0 manifest cap-bindings before merging.");
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (baseline file existence + shape)"]
fn ipc_method_name_baseline_file_exists_and_has_anticipated_method_set_key() {
    // Grep-assert against the baseline file shape. Lands at G24-E
    // wave-7; un-ignored alongside the drift-detector test above.
    //
    //   let baseline_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs/public-api/benten-renderer-tauri.json");
    //   let raw = std::fs::read_to_string(&baseline_path).unwrap();
    //   assert!(raw.contains("_anticipated_method_set"));
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
