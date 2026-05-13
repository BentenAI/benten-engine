//! G24-E wave-7 RED-PHASE pin (T3 + br-r1-2; LOAD-BEARING substantive).
//!
//! Asserts the Tauri renderer's IPC layer rejects any method invocation
//! whose name is NOT in the allowlist derived from the admin UI v0
//! manifest. Defense against XSS-amplified IPC abuse (T3 path in
//! `admin-ui-v0-threat-model.md`).
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7. The `IpcAllowlist` /
//! `TauriRenderer::dispatch_ipc` surfaces this test references do NOT
//! exist in the R3 stub — the test is shape-pinned against the
//! to-be-implemented production surface and would-FAIL if the allowlist
//! check were no-op'd (per `dispatch-conventions.md` §3.6b substantive).
//!
//! ## Compliance
//!
//! - §3.6b LOAD-BEARING substantive: production IPC dispatch arm
//!   exercised (not a unit test of allowlist data structure alone).
//! - §3.6e RED-PHASE staged-pin: `#[ignore]` rationale names the
//!   un-ignore wave explicitly.
//! - §3.6f SHAPE-not-SUBSTANCE pre-flight: would-FAIL-if-no-op'd
//!   because the assertion targets the `Err(...)` return on unknown
//!   method, not the mere absence of side-effects.
//!
//! ## Closes
//!
//! T3 + br-r1-2 (`r2-test-landscape.md` §2.10 row 1)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

// At R3 stub, the renderer surface doesn't exist; the import below is
// intentionally aspirational — un-ignoring at G24-E wave-7 lands the
// real types alongside this test.
use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (IPC allowlist landing)"]
fn ipc_allowlist_rejects_unknown_method_returns_typed_error() {
    // Production arm (G24-E wave-7):
    //
    //   let renderer = TauriRenderer::new_with_manifest(admin_ui_v0_manifest());
    //   let result = renderer.dispatch_ipc(IpcRequest {
    //       method: "engine.read_arbitrary_node".to_string(),
    //       payload: serde_json::json!({}),
    //   });
    //   assert!(matches!(result, Err(IpcError::MethodNotInAllowlist { .. })));
    //
    // Would-FAIL-if-no-op'd: if dispatch_ipc returned Ok for an unknown
    // method, the match would not bind to MethodNotInAllowlist. The
    // assertion's substance is the typed-error variant, not the mere
    // return-of-Err.
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
