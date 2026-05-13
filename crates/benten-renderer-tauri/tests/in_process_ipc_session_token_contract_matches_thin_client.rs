//! G24-E wave-7 LANDED pin (br-r1-14; cross-protocol-contract).
//!
//! Asserts the in-process Tauri IPC session-token contract reuses the
//! SAME [`DidKeyedSession`] + [`SessionToken`] types as the G24-F
//! thin-client browser-tab deployment shape — only the wire transport
//! is swapped (HTTP / fetch for shape (b); in-process IPC for shape
//! (c)). The dual-deployment-shape invariant per CLAUDE.md baked-in #17
//! holds when the BYTE SHAPE of the protocol is identical across
//! shapes.
//!
//! ## Closes
//!
//! br-r1-14 (`r2-test-landscape.md` §2.10 row 5)

#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::thin_client::{DidKeyedSession, SessionConfig, Transport};
use benten_renderer_tauri::InProcessSessionBridge;

/// Construct a [`DidKeyedSession`] wired with deterministic test hooks
/// (always-accept verifier + monotonic RNG + manual wall-clock).
fn test_session() -> (Arc<DidKeyedSession>, Arc<AtomicU64>) {
    let counter = Arc::new(AtomicU64::new(1));
    let rng_counter = Arc::clone(&counter);
    let clock_counter = Arc::clone(&counter);
    let session = DidKeyedSession::with_hooks(
        SessionConfig::default(),
        Box::new(|_did: &str, _msg: &[u8], _sig: &[u8]| Ok(())),
        Box::new(move || {
            let n = rng_counter.fetch_add(1, Ordering::SeqCst);
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&n.to_le_bytes());
            bytes
        }),
        Box::new(move || clock_counter.load(Ordering::SeqCst)),
    );
    (Arc::new(session), counter)
}

#[test]
fn in_process_ipc_session_token_byte_shape_matches_thin_client_did_keyed_session() {
    let (session, _counter) = test_session();
    let bridge = InProcessSessionBridge::new(Arc::clone(&session));

    // Shape (c) bridge wraps the SAME DidKeyedSession the shape (b)
    // thin-client would use. The transport tag is the only difference
    // — confirms br-r1-14 contract identity.
    assert_eq!(bridge.transport(), Transport::Ipc);

    // Establish a session via the same DidKeyedSession used for both
    // shapes; the session bridge's `resolve` method returns the same
    // shape of result the thin-client would.
    let origin = "tauri://localhost";
    let challenge = session.emit_challenge(origin);
    let token = session
        .establish_session(&challenge, &[0u8; 64], "did:key:zTest", origin)
        .expect("establish should succeed with always-accept verifier");

    // SessionToken byte-shape is identical across shapes — the same
    // struct, same fields. Bridge round-trips it.
    assert_eq!(token.principal_did, "did:key:zTest");
    assert_eq!(token.bound_origin, origin);
    let resolved = bridge
        .resolve(&token, origin)
        .expect("resolve should succeed on same origin");
    assert_eq!(resolved, "did:key:zTest");

    // Would-FAIL-if-no-op'd: if shape (c) introduced a Tauri-specific
    // session-token struct, this test would not even type-check
    // against `benten_engine::thin_client::SessionToken` — the
    // assertion's substance is in the type signature compatibility +
    // the cross-shape round-trip.
}

#[test]
fn in_process_ipc_session_origin_recheck_fires_per_request() {
    // Family F1 gap #2 mid-session defense: the origin recheck is
    // structural-always-on, not establishment-only. If a hostile
    // origin captures the token bytes and presents them, the bridge
    // rejects. Mirrors the HTTP thin-client structural recheck.
    let (session, _counter) = test_session();
    let bridge = InProcessSessionBridge::new(Arc::clone(&session));

    let bound = "tauri://localhost";
    let challenge = session.emit_challenge(bound);
    let token = session
        .establish_session(&challenge, &[0u8; 64], "did:key:zTest", bound)
        .expect("establish should succeed");

    // Hostile presentation: different origin string. Must reject.
    let hostile = bridge.resolve(&token, "https://attacker.example");
    assert!(
        matches!(
            hostile,
            Err(benten_engine::thin_client::ThinClientSessionError::OriginMismatch { .. })
        ),
        "expected OriginMismatch, got {hostile:?}"
    );
}
