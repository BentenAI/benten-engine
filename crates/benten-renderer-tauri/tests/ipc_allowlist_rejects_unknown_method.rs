//! G24-E wave-7 LANDED pin (T3 + br-r1-2; LOAD-BEARING substantive).
//!
//! Asserts the Tauri renderer's IPC layer rejects any method invocation
//! whose name is NOT in the allowlist derived from the admin UI v0
//! manifest. Defense against XSS-amplified IPC abuse (T3 path in
//! `admin-ui-v0-threat-model.md`).
//!
//! ## Compliance
//!
//! - §3.6b LOAD-BEARING substantive: production IPC dispatch arm
//!   exercised (not a unit test of allowlist data structure alone).
//! - §3.6f SHAPE-not-SUBSTANCE pre-flight: assertion targets the
//!   `Err(MethodNotInAllowlist { .. })` typed-error variant — if the
//!   allowlist check were no-op'd, dispatch_ipc would return `Ok(..)`
//!   and the match would not bind.
//!
//! ## Closes
//!
//! T3 + br-r1-2 (`r2-test-landscape.md` §2.10 row 1)

#![allow(clippy::unwrap_used)]

use benten_renderer_tauri::{AdminUiManifest, IpcError, IpcRequest, TauriRenderer};

#[test]
fn ipc_allowlist_rejects_unknown_method_returns_typed_error() {
    let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::with_caps([
        "graph:read",
        "graph:write",
        "caps:read",
        "identity:read",
        "plugin:read",
        "plugin:install",
    ]));

    let result = renderer.dispatch_ipc(IpcRequest {
        method: "engine.read_arbitrary_node".to_string(),
        payload: serde_json::json!({}),
        session: None,
    });

    // Would-FAIL-if-no-op'd: if dispatch_ipc returned Ok for an unknown
    // method, the match would not bind to MethodNotInAllowlist.
    match result {
        Err(IpcError::MethodNotInAllowlist { method }) => {
            assert_eq!(method, "engine.read_arbitrary_node");
        }
        other => panic!("expected MethodNotInAllowlist, got {other:?}"),
    }
}

#[test]
fn ipc_allowlist_rejects_empty_method() {
    // Companion regression-guard: empty method string is also outside
    // the allowlist. Defends against an attacker submitting a blank
    // method to probe the dispatch surface.
    let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::default());
    let result = renderer.dispatch_ipc(IpcRequest {
        method: String::new(),
        payload: serde_json::Value::Null,
        session: None,
    });
    assert!(matches!(result, Err(IpcError::MethodNotInAllowlist { .. })));
}

#[test]
fn ipc_allowlist_admits_known_method_past_rung_one() {
    // Regression-guard companion (per §3.6b): the negative-path test
    // alone permits an accidental over-restrictive future change that
    // breaks the happy path. Assert that an allowlisted method whose
    // cap IS in the manifest passes rung-1 (and rung-2). Bridge
    // unattached, so rung-3 short-circuits.
    let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::with_caps(["graph:read"]));
    let result = renderer.dispatch_ipc(IpcRequest {
        method: "engine.read_node_as".to_string(),
        payload: serde_json::Value::Null,
        session: None,
    });
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}
