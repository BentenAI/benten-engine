//! G24-E wave-7 RED-PHASE pin (br-r1-14; cross-protocol-contract).
//!
//! Asserts the in-process Tauri IPC session token contract is byte-shape
//! identical to the over-the-wire DID-keyed session-token contract used
//! by the G24-F thin-client browser-tab deployment shape. This pin
//! protects the dual-deployment-shape invariant: a single auth shape
//! across both `(b)` browser-tab and `(c)` embedded-webview (per CLAUDE.md
//! #17 three-shape commitment).
//!
//! ## RED-PHASE status
//!
//! `#[ignore]` until G24-E wave-7 + G24-F wave-7 both land.
//!
//! ## Closes
//!
//! br-r1-14 (`r2-test-landscape.md` §2.10 row 5)

#![allow(clippy::unwrap_used, dead_code, unused_imports)]

use benten_renderer_tauri as _renderer;

#[test]
#[ignore = "RED-PHASE: closes at R5 G24-E wave-7 (cross-protocol contract; co-lands with G24-F)"]
fn in_process_ipc_session_token_byte_shape_matches_thin_client_did_keyed_session() {
    // Production arm (G24-E + G24-F wave-7):
    //
    //   // Generate a DidKeyedSession for the in-process Tauri IPC layer.
    //   let tauri_session = TauriRenderer::new_with_manifest(manifest())
    //       .establish_in_process_session(user_did());
    //
    //   // Generate a DidKeyedSession for the browser-tab thin-client.
    //   let browser_session = ThinClient::new(server_url)
    //       .establish_session(user_did());
    //
    //   // Token byte-shape (length, header layout, claims envelope)
    //   // must match. Implementation detail: the same DidKeyedSession
    //   // type underlies both — this test pins that no divergence is
    //   // introduced under refactor.
    //   assert_eq!(tauri_session.token_shape(), browser_session.token_shape());
    //
    // Would-FAIL-if-no-op'd: if one shape diverged from the other, the
    // assertion would fail. The pin defends against future "specialize
    // for performance" pressure that would introduce a Tauri-specific
    // session-token shape.
    panic!("RED-PHASE: production surface lands at G24-E + G24-F wave-7");
}
