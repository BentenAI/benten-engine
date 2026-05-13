//! G24-E wave-7 RED-PHASE pin (T3 LOAD-BEARING; substantive end-to-end).
//!
//! The cornerstone defense pin for T3 (cap-elevation under Tauri 2.x
//! embedded-webview). A compromised webview (simulated via direct IPC
//! invocation against `fs:*` allowlist methods that DO NOT appear in
//! the admin UI v0 manifest's `requires` envelope) MUST be denied at
//! the IPC layer even though the renderer process has native filesystem
//! authority.
//!
//! Defense composition:
//!
//! 1. Tauri allowlist locks fs methods to NONE at config-time.
//! 2. CSP at webview load denies inline-script + non-tauri connect-src.
//! 3. IPC per-method cap-binding requires `fs:read` / `fs:write`
//!    capability granted by the admin UI manifest.
//!
//! The admin UI v0 manifest does NOT request `fs:*` caps; a compromised
//! webview that crafts an IPC request directly cannot escalate.
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7. This is one of the LOAD-BEARING
//! pins per `r2-test-landscape.md` §2.10 row 4.
//!
//! ## Closes
//!
//! T3 LOAD-BEARING (`r2-test-landscape.md` §2.10 row 4 + threat-model
//! T3 line ~118 narrative)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (T3 end-to-end defense composition)"]
fn compromised_webview_cannot_escalate_to_native_filesystem_via_ipc() {
    // Production arm (G24-E wave-7):
    //
    //   // Admin UI v0 manifest grants ONLY graph-scoped caps, never fs:*
    //   let manifest = admin_ui_v0_manifest_without_fs_caps();
    //   let renderer = TauriRenderer::new_with_manifest(manifest);
    //
    //   // Simulate XSS-amplified IPC: attempt to invoke an arbitrary
    //   // method that would write to the user's home directory.
    //   let result = renderer.dispatch_ipc(IpcRequest {
    //       method: "fs:write".to_string(),
    //       payload: serde_json::json!({
    //           "path": "/Users/victim/.ssh/authorized_keys",
    //           "content": "ssh-ed25519 AAAA...",
    //       }),
    //   });
    //
    //   // Must error at IPC layer; the renderer's native fs authority
    //   // is NOT reachable via the webview's IPC surface.
    //   assert!(matches!(
    //       result,
    //       Err(IpcError::MethodNotInAllowlist { .. })
    //           | Err(IpcError::CapabilityNotInManifest { .. })
    //   ));
    //
    // Would-FAIL-if-no-op'd: any of the three defense rungs (allowlist,
    // CSP, cap-binding) being no-op would cause this assertion to fail
    // because either the typed error wouldn't emit OR the side-effect
    // (the file write) would observably occur.
    panic!("RED-PHASE: production surface lands at G24-E wave-7");
}
