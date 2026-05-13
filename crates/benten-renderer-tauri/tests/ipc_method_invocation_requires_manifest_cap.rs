//! G24-E wave-7 LANDED pin (T3; LOAD-BEARING substantive).
//!
//! Asserts that even an allowlisted IPC method dispatch is denied
//! unless the admin UI v0 manifest's `requires` envelope grants the
//! capability bound to that method (T3 in
//! `admin-ui-v0-threat-model.md`). The per-method cap-binding is the
//! second leg of the defense: allowlist filters the method NAME;
//! manifest-cap check filters by whether the principal currently holds
//! the bound cap.
//!
//! ## Compliance
//!
//! - §3.6b LOAD-BEARING substantive: typed-error variant asserted +
//!   regression-guard companion (would-have-succeeded-with-cap) below.
//!
//! ## Closes
//!
//! T3 (`r2-test-landscape.md` §2.10 row 2)

#![allow(clippy::unwrap_used)]

use benten_renderer_tauri::{AdminUiManifest, IpcError, IpcRequest, TauriRenderer};

#[test]
fn ipc_method_invocation_without_manifest_cap_is_denied() {
    // Manifest envelope grants NO graph:write — admin UI installed
    // read-only.
    let manifest = AdminUiManifest::with_caps(["graph:read"]);
    let renderer = TauriRenderer::new_with_manifest(manifest);

    let result = renderer.dispatch_ipc(IpcRequest {
        method: "engine.call_as".to_string(), // bound to "graph:write"
        payload: serde_json::json!({ "labels": ["note"] }),
        session: None,
    });

    // Would-FAIL-if-cap-check-no-op'd: the typed error variant only
    // emits when the manifest envelope check rejects.
    match result {
        Err(IpcError::CapabilityNotInManifest { method, cap }) => {
            assert_eq!(method, "engine.call_as");
            assert_eq!(cap, "graph:write");
        }
        other => panic!("expected CapabilityNotInManifest, got {other:?}"),
    }
}

#[test]
fn ipc_method_invocation_with_manifest_cap_succeeds_regression_guard() {
    // Regression-guard arm: same shape but manifest GRANTS the cap;
    // assert Ok. This is the companion-pin per §3.6b (a test of the
    // negative path alone permits accidental over-restrictive future
    // change that breaks the happy path).
    let manifest = AdminUiManifest::with_caps(["graph:write"]);
    let renderer = TauriRenderer::new_with_manifest(manifest);

    let result = renderer.dispatch_ipc(IpcRequest {
        method: "engine.call_as".to_string(),
        payload: serde_json::json!({}),
        session: None,
    });
    assert!(result.is_ok(), "expected Ok past rungs 1+2, got {result:?}");
}

#[test]
fn ipc_method_with_no_required_cap_admits_without_manifest_grant() {
    // `ui.notify` has cap binding `""` (empty / no cap required). The
    // manifest MAY be empty + dispatch still admits. Defends against
    // accidentally requiring a UI-only notification method to ride
    // graph caps.
    let manifest = AdminUiManifest::default();
    let renderer = TauriRenderer::new_with_manifest(manifest);
    let result = renderer.dispatch_ipc(IpcRequest {
        method: "ui.notify".to_string(),
        payload: serde_json::json!({ "text": "hi" }),
        session: None,
    });
    assert!(result.is_ok(), "ui.notify should admit, got {result:?}");
}
