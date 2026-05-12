//! G24-E wave-7 RED-PHASE pin (T3; LOAD-BEARING substantive).
//!
//! Asserts that even an allowlisted IPC method dispatch is denied
//! unless the admin UI v0 manifest's `requires` envelope grants the
//! capability bound to that method (T3 in
//! `admin-ui-v0-threat-model.md`). The per-method cap-binding is the
//! second leg of the defense: allowlist filters the method NAME;
//! manifest-cap check filters by whether the principal currently holds
//! the bound cap.
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7 lands `TauriRenderer::dispatch_ipc`
//! with per-method `CapabilityPolicy::check` invocation.
//!
//! ## Compliance
//!
//! - §3.6b LOAD-BEARING substantive: production IPC dispatch arm with
//!   capability check disabled would-FAIL (assertion targets the
//!   typed-error variant + would-have-succeeded-with-cap regression
//!   companion).
//! - §3.6e RED-PHASE staged-pin: un-ignore wave named.
//!
//! ## Closes
//!
//! T3 (`r2-test-landscape.md` §2.10 row 2)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (per-method cap-binding landing)"]
fn ipc_method_invocation_without_manifest_cap_is_denied() {
    // Production arm (G24-E wave-7):
    //
    //   let manifest = admin_ui_v0_manifest_without_cap("graph:write");
    //   let renderer = TauriRenderer::new_with_manifest(manifest);
    //   let result = renderer.dispatch_ipc(IpcRequest {
    //       method: "engine.write_node",  // bound to cap "graph:write"
    //       payload: serde_json::json!({ "labels": ["note"] }),
    //   });
    //   assert!(matches!(result, Err(IpcError::CapabilityNotInManifest { .. })));
    //
    // Would-FAIL-if-cap-check-no-op'd: the typed error variant only
    // emits when CapabilityPolicy::check returns Err.
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (regression-guard companion)"]
fn ipc_method_invocation_with_manifest_cap_succeeds_regression_guard() {
    // Regression-guard arm: same shape but manifest GRANTS the cap;
    // assert Ok. This is the companion-pin per §3.6b (a test of the
    // negative path alone permits accidental over-restrictive future
    // change that breaks the happy path).
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
