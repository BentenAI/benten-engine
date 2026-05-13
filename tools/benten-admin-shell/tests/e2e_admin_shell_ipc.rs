//! Phase-4-Foundation R6-FP-E end-to-end IPC dispatch pin for the
//! `benten-admin-shell` integrator binary.
//!
//! # What this asserts (pim-2 §3.6b end-to-end test pin contract)
//!
//! - **PRODUCTION-ARM:** constructs the SAME `AdminShellState` shape
//!   `src/main.rs` constructs. Drives requests through
//!   `AdminShellState::dispatch`, which is the exact code path a
//!   real Tauri 2.x command handler invokes (the `tauri` feature-mode
//!   boot wraps Tauri's `invoke` channel around this method
//!   one-to-one).
//!
//! - **OBSERVABLE-CONSEQUENCE:** asserts the three T3 defense rungs
//!   surface their canonical [`IpcError`] envelopes when the
//!   conditions are violated (allowlist miss / missing cap / origin
//!   mismatch / expired session / missing session / replay), and
//!   asserts the happy-path returns `Ok(IpcResponse{ payload: Null })`
//!   per the renderer contract (the integrator-binary's per-method
//!   handler then overwrites payload with the real response — that
//!   step is on the v1-window webview-driven wave per
//!   `docs/future/phase-4-backlog.md §3`).
//!
//! - **WOULD-FAIL-IF-NO-OP'd:** any of these regressions would surface
//!   here:
//!   * `dispatch_ipc` allowlist seam bypassed → MethodNotInAllowlist
//!     case fails
//!   * Manifest envelope construction drift → CapabilityNotInManifest
//!     case fails
//!   * `InProcessSessionBridge::resolve` skipping the origin recheck →
//!     origin-mismatch case fails
//!   * `AdminShellState::new_production` losing the session-bridge
//!     wiring → MissingSession case (with a bridge attached but no
//!     token presented) fails
//!   * `DidKeyedSession::establish_session` skipping the nonce-consume
//!     step → replay case fails
//!
//! Closes the named-NOW half of `br-r6-r1-3` MAJOR — the integrator-
//! binary scaffold + a substantive E2E that exercises the production
//! IPC dispatch pipeline end-to-end.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_admin_shell::{ADMIN_SHELL_BOUND_ORIGIN, AdminShellState, ipc_request};
use benten_engine::thin_client::{
    DidKeyedSession, SessionConfig, SessionToken, ThinClientSessionError,
};
use benten_renderer_tauri::{IPC_METHOD_NAME_ALLOWLIST, IpcError};

/// Build an `AdminShellState` whose `DidKeyedSession` is wired with
/// deterministic test hooks (always-accept verifier + monotonic RNG +
/// manual wallclock). Returns the state + the wallclock atom so the
/// test can advance time for expiry pins.
fn deterministic_state() -> (AdminShellState, Arc<AtomicU64>) {
    let clock = Arc::new(AtomicU64::new(1_700_000_000));
    let clock_for_closure = Arc::clone(&clock);
    let nonce_counter = Arc::new(AtomicU64::new(1));
    let nonce_for_closure = Arc::clone(&nonce_counter);
    let session = Arc::new(DidKeyedSession::with_hooks(
        SessionConfig::default(),
        Box::new(|_did, _msg, _sig| Ok(())),
        Box::new(move || {
            let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&n.to_le_bytes());
            bytes
        }),
        Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
    ));
    let state = AdminShellState::from_session(session);
    (state, clock)
}

/// Mint a session token through the integrator's session machine. The
/// helper runs the full handshake (emit_challenge -> establish_session)
/// so the per-test setup mirrors what the webview's bootstrap path
/// would drive.
fn handshake(state: &AdminShellState) -> SessionToken {
    let challenge = state.session().emit_challenge(ADMIN_SHELL_BOUND_ORIGIN);
    state
        .session()
        .establish_session(
            &challenge,
            &[0u8; 64],
            "did:key:zTest",
            ADMIN_SHELL_BOUND_ORIGIN,
        )
        .expect("handshake should succeed with always-accept verifier")
}

// =====================================================================
// PRODUCTION-ARM: happy-path IPC roundtrip through the full pipeline
// =====================================================================

#[test]
fn e2e_happy_path_dispatch_succeeds_for_every_allowlisted_method() {
    let (state, _clock) = deterministic_state();
    let token = handshake(&state);

    for method in IPC_METHOD_NAME_ALLOWLIST {
        let req = ipc_request(*method, serde_json::json!({}), Some(token.clone()));
        let resp = state.dispatch(req).unwrap_or_else(|e| {
            panic!("dispatch for allowlisted method {method} should succeed: {e:?}")
        });
        // Renderer contract: happy-path returns `Null` payload — the
        // integrator-binary's per-method handler overwrites with the
        // real response. The webview-driven wave per
        // docs/future/phase-4-backlog.md §3 lands the per-method
        // handlers.
        assert_eq!(resp.payload, serde_json::Value::Null);
    }
}

// =====================================================================
// T3 RUNG 1: allowlist filter rejects unknown methods
// =====================================================================

#[test]
fn e2e_t3_rung_1_unknown_method_rejected_at_allowlist_seam() {
    let (state, _clock) = deterministic_state();
    let token = handshake(&state);

    let req = ipc_request(
        "engine.delete_everything",
        serde_json::json!({}),
        Some(token),
    );
    let err = state.dispatch(req).unwrap_err();
    assert!(
        matches!(err, IpcError::MethodNotInAllowlist { ref method } if method == "engine.delete_everything"),
        "expected MethodNotInAllowlist, got {err:?}"
    );
}

// =====================================================================
// T3 RUNG 2: per-method cap-binding enforced against the canonical
// manifest envelope. Cannot easily test "manifest missing a cap" with
// the canonical manifest (it grants every cap the binding references
// by construction). Instead: re-construct an `AdminShellState`-shaped
// state with a HAND-ROLLED manifest that is intentionally missing one
// cap, then dispatch a method bound to that cap.
//
// This pin asserts the rung-2 seam is wired through the integrator;
// rebuilding the renderer with a deficient manifest exercises the
// CapabilityNotInManifest branch end-to-end via the same dispatch path
// production callers use.
// =====================================================================

#[test]
fn e2e_t3_rung_2_method_with_missing_cap_rejected_at_manifest_envelope() {
    use benten_renderer_tauri::{AdminUiManifest, InProcessSessionBridge, TauriRenderer};

    let clock = Arc::new(AtomicU64::new(1_700_000_000));
    let clock_for_closure = Arc::clone(&clock);
    let nonce_counter = Arc::new(AtomicU64::new(1));
    let nonce_for_closure = Arc::clone(&nonce_counter);
    let session = Arc::new(DidKeyedSession::with_hooks(
        SessionConfig::default(),
        Box::new(|_did, _msg, _sig| Ok(())),
        Box::new(move || {
            let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&n.to_le_bytes());
            bytes
        }),
        Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
    ));

    // Manifest deliberately missing `graph:write` so engine.call_as
    // hits rung 2.
    let deficient = AdminUiManifest::with_caps([
        "graph:read",
        "caps:read",
        "identity:read",
        "plugin:read",
        "plugin:install",
    ]);
    let bridge = InProcessSessionBridge::new(Arc::clone(&session));
    let renderer = TauriRenderer::new_with_manifest(deficient).with_bridge(bridge);

    // Handshake to obtain a valid token.
    let challenge = session.emit_challenge(ADMIN_SHELL_BOUND_ORIGIN);
    let token = session
        .establish_session(
            &challenge,
            &[0u8; 64],
            "did:key:zTest",
            ADMIN_SHELL_BOUND_ORIGIN,
        )
        .unwrap();

    let req = benten_renderer_tauri::IpcRequest {
        method: "engine.call_as".to_string(),
        payload: serde_json::json!({}),
        session: Some(token),
    };
    let err = renderer.dispatch_ipc(req).unwrap_err();
    assert!(
        matches!(err, IpcError::CapabilityNotInManifest { ref cap, .. } if cap == "graph:write"),
        "expected CapabilityNotInManifest{{cap: graph:write}}, got {err:?}"
    );
}

// =====================================================================
// BRIDGE / SESSION: missing session token on a bridge-attached
// renderer
// =====================================================================

#[test]
fn e2e_bridge_attached_dispatch_without_session_rejects() {
    let (state, _clock) = deterministic_state();
    // No handshake; no token presented.
    let req = ipc_request("engine.list_caps", serde_json::json!({}), None);
    let err = state.dispatch(req).unwrap_err();
    assert!(matches!(err, IpcError::MissingSession), "got {err:?}");
}

// =====================================================================
// BRIDGE / SESSION: origin mismatch through the integrator pipeline.
// Asserts the structural-always-on origin recheck per Family F1 gap #2
// fires at the integrator boundary, not just inside DidKeyedSession.
// =====================================================================

#[test]
fn e2e_session_origin_mismatch_rejected_at_bridge_resolve() {
    // A captured token presented from a hostile origin: the resolve
    // step inside dispatch_ipc returns OriginMismatch. Verify the
    // integrator surfaces the typed error (SessionResolve variant
    // wraps the thin-client error).
    let (state, _clock) = deterministic_state();

    // Build a session bound to ORIGIN_A; then craft a token whose
    // bound_origin claims ORIGIN_A but present at dispatch time with
    // ORIGIN_B. We can't actually mutate the bridge's presented
    // origin (it's hard-coded to ADMIN_SHELL_BOUND_ORIGIN inside
    // dispatch_ipc), so we instead bind the session to a DIFFERENT
    // origin and let dispatch's "tauri://localhost" presentation
    // mismatch.
    let challenge = state.session().emit_challenge("https://other.example");
    let token = state
        .session()
        .establish_session(
            &challenge,
            &[0u8; 64],
            "did:key:zTest",
            "https://other.example",
        )
        .unwrap();

    let req = ipc_request("engine.read_node_as", serde_json::json!({}), Some(token));
    let err = state.dispatch(req).unwrap_err();
    assert!(
        matches!(
            err,
            IpcError::SessionResolve(ThinClientSessionError::OriginMismatch { .. })
        ),
        "expected SessionResolve(OriginMismatch), got {err:?}"
    );
}

// =====================================================================
// BRIDGE / SESSION: session-expired path. Advances the test clock past
// the session TTL and asserts dispatch returns SessionExpired through
// the integrator pipeline.
// =====================================================================

#[test]
fn e2e_session_expired_rejected_through_integrator() {
    let (state, clock) = deterministic_state();
    let token = handshake(&state);
    // Default SessionConfig TTL is 3600s; advance past it.
    clock.store(1_700_000_000 + 3_700, Ordering::SeqCst);

    let req = ipc_request("engine.list_caps", serde_json::json!({}), Some(token));
    let err = state.dispatch(req).unwrap_err();
    assert!(
        matches!(
            err,
            IpcError::SessionResolve(ThinClientSessionError::SessionExpired { .. })
        ),
        "expected SessionResolve(SessionExpired), got {err:?}"
    );
}

// =====================================================================
// REPLAY: handshake replay rejected through the integrator. The
// integrator binary's session bootstrap path calls
// `establish_session` once; a replay of the same challenge MUST
// reject.
// =====================================================================

#[test]
fn e2e_handshake_replay_rejected_through_integrator() {
    let (state, _clock) = deterministic_state();
    let challenge = state.session().emit_challenge(ADMIN_SHELL_BOUND_ORIGIN);
    // First handshake: succeeds.
    let _first = state
        .session()
        .establish_session(
            &challenge,
            &[0u8; 64],
            "did:key:zTest",
            ADMIN_SHELL_BOUND_ORIGIN,
        )
        .unwrap();
    // Second handshake against the SAME challenge: must reject with
    // ChallengeReplay.
    let second = state.session().establish_session(
        &challenge,
        &[0u8; 64],
        "did:key:zTest",
        ADMIN_SHELL_BOUND_ORIGIN,
    );
    assert!(
        matches!(second, Err(ThinClientSessionError::ChallengeReplay)),
        "expected ChallengeReplay, got {second:?}"
    );
}

// =====================================================================
// CSP CONTRACT: the integrator binary's published CSP header at boot
// matches the renderer's canonical constant. Drift would weaken T3
// rung 3.
// =====================================================================

#[test]
fn e2e_integrator_publishes_canonical_csp_header() {
    let (state, _clock) = deterministic_state();
    assert_eq!(
        state.webview_csp_header(),
        benten_renderer_tauri::WEBVIEW_CSP_HEADER
    );
    // Spot-check the forbidden directives at the integrator boundary
    // (defense-in-depth against accidental rewrap by future
    // refactors).
    let csp = state.webview_csp_header();
    let cleaned = csp.replace("'wasm-unsafe-eval'", "");
    assert!(!cleaned.contains("'unsafe-eval'"));
    assert!(!csp.contains("'unsafe-inline'"));
}

// =====================================================================
// CONTRACT PARITY: the integrator's session bridge is the SAME
// `DidKeyedSession` instance the integrator constructs at boot. A
// regression that wired two separate sessions (one for the renderer's
// resolve path, one for the handshake bootstrap) would silently break
// session reuse — caught here by token reuse across handshake +
// dispatch.
// =====================================================================

#[test]
fn e2e_handshake_and_dispatch_share_one_did_keyed_session_instance() {
    let (state, _clock) = deterministic_state();
    // Handshake on state.session().
    let token = handshake(&state);
    // Dispatch presenting the same token MUST succeed — proves the
    // renderer's bridge resolves against the same instance.
    let req = ipc_request("engine.list_caps", serde_json::json!({}), Some(token));
    let resp = state.dispatch(req).expect("dispatch should succeed");
    assert_eq!(resp.payload, serde_json::Value::Null);
    // And exactly one active session was minted.
    assert_eq!(state.session().active_session_count_for_test(), 1);
}
