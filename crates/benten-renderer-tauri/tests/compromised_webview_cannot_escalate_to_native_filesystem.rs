//! G24-E wave-7 LANDED pin (T3 LOAD-BEARING; substantive end-to-end).
//!
//! The cornerstone defense pin for T3 (cap-elevation under Tauri 2.x
//! embedded-webview). A compromised webview (simulated via direct IPC
//! invocation against `fs:*`-style methods that DO NOT appear in the
//! IPC allowlist + against `engine.call_as` whose `graph:write` cap is
//! NOT in the admin UI v0 manifest's `requires` envelope) MUST be
//! denied at the IPC layer even though the renderer process holds
//! native authority.
//!
//! Defense composition exercised end-to-end:
//!
//! 1. IPC allowlist rejects unknown `fs:*` method (T3 rung 1).
//! 2. Per-method cap-binding rejects `engine.call_as` when manifest
//!    envelope withholds `graph:write` (T3 rung 2).
//! 3. CSP header at webview-load forbids `unsafe-eval` /
//!    `unsafe-inline` (T3 rung 3; composition-side defense).
//!
//! The admin UI v0 manifest does NOT request `fs:*` caps; a compromised
//! webview that crafts an IPC request directly cannot escalate.
//!
//! ## Closes
//!
//! T3 LOAD-BEARING (`r2-test-landscape.md` §2.10 row 4 + threat-model
//! T3 line ~118 narrative)

#![allow(clippy::unwrap_used)]

use benten_renderer_tauri::{AdminUiManifest, IpcError, IpcRequest, TauriRenderer};

/// Admin UI v0 manifest as installed: read-only graph + capability
/// listing + identity read + plugin metadata + UI notify. No `fs:*`,
/// no `graph:write`.
fn admin_ui_v0_manifest_without_fs_caps() -> AdminUiManifest {
    AdminUiManifest::with_caps(["graph:read", "caps:read", "identity:read", "plugin:read"])
}

#[test]
fn compromised_webview_cannot_escalate_to_native_filesystem_via_ipc() {
    let manifest = admin_ui_v0_manifest_without_fs_caps();
    let renderer = TauriRenderer::new_with_manifest(manifest);

    // (1) Simulate XSS-amplified IPC: attempt an arbitrary `fs:write`
    // method that would touch the user's home directory. NOT on the
    // allowlist — denied at rung 1.
    let fs_attempt = renderer.dispatch_ipc(IpcRequest {
        method: "fs:write".to_string(),
        payload: serde_json::json!({
            "path": "/Users/victim/.ssh/authorized_keys",
            "content": "ssh-ed25519 AAAA...",
        }),
        session: None,
    });
    assert!(
        matches!(fs_attempt, Err(IpcError::MethodNotInAllowlist { .. })),
        "fs:write should reject at allowlist: got {fs_attempt:?}"
    );

    // (2) Simulate an allowlisted-method pivot: webview tries to
    // upgrade its read-only manifest by calling `engine.call_as`
    // which DOES exist on the allowlist but its bound `graph:write`
    // cap is NOT in the manifest envelope. Denied at rung 2.
    let pivot_attempt = renderer.dispatch_ipc(IpcRequest {
        method: "engine.call_as".to_string(),
        payload: serde_json::json!({
            "op": "WRITE",
            "input": { "labels": ["malicious_node"] },
        }),
        session: None,
    });
    match pivot_attempt {
        Err(IpcError::CapabilityNotInManifest { method, cap }) => {
            assert_eq!(method, "engine.call_as");
            assert_eq!(cap, "graph:write");
        }
        other => panic!("expected CapabilityNotInManifest, got {other:?}"),
    }

    // (3) CSP at webview-load: forbids classic `unsafe-eval` +
    // `unsafe-inline`. The composition-side defense — even if rung 1
    // + 2 were both bypassed by some future bug, the webview itself
    // could not execute inline-script payloads. Asserted at
    // load-boundary; we test the header content here as the proxy
    // for "the integrator binary wired this into Tauri's CSP at boot".
    let csp = renderer.webview_csp_header();
    assert!(csp.contains("default-src 'none'"));
    let stripped = csp.replace("'wasm-unsafe-eval'", "");
    assert!(!stripped.contains("'unsafe-eval'"));
    assert!(!csp.contains("'unsafe-inline'"));

    // Would-FAIL-if-no-op'd: any of the three defense rungs being
    // no-op'd would cause one of the assertions above to fail. The
    // composition is end-to-end: this pin exercises all three rungs
    // through the production dispatch surface.
}

#[test]
fn compromised_webview_cannot_invoke_arbitrary_method_even_with_full_manifest() {
    // Bolt-on substantive arm: even a manifest that grants EVERY cap
    // in the canonical binding set still can't reach a method that
    // isn't on the allowlist. Defends against an over-permissive
    // manifest somehow becoming a back-door — the allowlist is the
    // load-bearing wall.
    let over_permissive = AdminUiManifest::with_caps([
        "graph:read",
        "graph:write",
        "caps:read",
        "identity:read",
        "plugin:read",
        "plugin:install",
        // Plus speculative future caps an attacker might guess:
        "fs:read",
        "fs:write",
        "process:spawn",
        "net:open",
    ]);
    let renderer = TauriRenderer::new_with_manifest(over_permissive);

    let result = renderer.dispatch_ipc(IpcRequest {
        method: "process.spawn".to_string(),
        payload: serde_json::json!({ "cmd": "/bin/sh", "args": ["-c", "rm -rf /"] }),
        session: None,
    });
    assert!(matches!(result, Err(IpcError::MethodNotInAllowlist { .. })));
}
