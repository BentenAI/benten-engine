//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 — §4.22 thin-client bridge
//! principal-resolution + the sealed-`CapabilityPolicy`-trait
//! object-safety compile pin (§1.A.FROZEN item 8).
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! CLAUDE.md baked-in #17/#18: a thin-client (shape b/c — browser tab
//! or embedded webview) is a VIEW into a full peer via the
//! authenticated thin-client protocol; it is NOT a sync participant
//! and it does NOT hold cap tokens. The bridge MUST resolve the
//! acting principal from the authenticated SESSION (the DID-keyed
//! handshake), NOT from anything the client supplies in-band. A
//! client that asserts "I am principal X" must NOT thereby become X —
//! the principal is whatever the session was established as.
//!
//! At SYNCED HEAD `ed03729a`, the G24-F SHIPPED `DidKeyedSession`
//! surface (`crates/benten-engine/src/thin_client.rs:279`) resolves a
//! `SessionToken` → `principal_did` from the server-side session
//! record (the in-band client-asserted principal is structurally
//! impossible — `resolve` takes only `token + presented_origin`, never
//! a client-supplied principal field). What is UNDELIVERED is the
//! Phase-4-Meta plugin/Class-B-β read-path BRIDGE wiring that
//! consumes that resolved principal and threads it as the
//! `Engine::read_node_as` walk-principal — the bridge surface tests
//! exist (`admin_ui_v0_thin_client_bridge_resolves_principal_from_session_not_client.rs`
//! is DESTINATION-REMAPPED RED for §4.22 per its ignore message;
//! `thin_client_did_keyed_handshake_rejects_replay.rs` + the
//! `thin_client_session_origin_mismatch_denied*.rs` cluster exist as
//! the substrate) but the §4.22 bridge wiring is the G-CORE-8 group's
//! C8 thin-client-bridge clause.
//!
//! ## SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 / §3.6f)
//!
//! Each RED arm below **first** exercises the SHIPPED
//! `DidKeyedSession::resolve` surface on a real session round-trip
//! (`with_hooks`-stubbed verifier per the existing `admin_ui_v0_harness`
//! pattern — keeps the test independent of the Ed25519 did:key
//! resolver while still driving the production state-machine end-to-
//! end) with a real assertion + observable would-FAIL consequence,
//! **then** `panic!`-holds the still-undelivered Phase-4-Meta bridge
//! wiring that threads the resolved principal onto the Class-B-β
//! read path.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): exercises the SPECIFIC
//!     session-not-client resolution arm (production bridge call-site,
//!     observable: a client-asserted principal does NOT override the
//!     session principal; would-FAIL if the bridge trusts client input).
//!   - pim-12 (§3.6e): RED-PHASE staged-pin; reviewer verifies landing.
//!   - pim-18 (§3.6f): substantive arm, not "a bridge type exists".
//!     The mostly-undelivered-target-surface hybrid pattern (R4.1
//!     pattern-induction): exercise SHIPPED adjacent primitives
//!     (`DidKeyedSession::{emit_challenge,establish_session,resolve}`)
//!     + `panic!`-hold the missing bridge.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!   - §3.5g: the sealed-trait shape is a §1.A.FROZEN item-8 frozen
//!     surface; the `Arc<dyn CapabilityPolicy>` boxing compile-test
//!     guards object-safety under the private `Sealed` supertrait.
//!
//! Pins: G-CORE-8 · C8 (thin-client bridge) · §1.A.FROZEN item 8
//! (sealed-discipline) + item 13 (engine↔shell runtime-boundary lock).
//! R2 map: TF-8 RED-arm (3) thin-client bridge + (8) sealed object-safety.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::thin_client::{
    DidKeyedSession, SessionConfig, SessionToken, ThinClientSessionError,
};

const ATTACKER_DID: &str = "did:key:zAttacker";
const VICTIM_DID: &str = "did:key:zVictim";
const VALID_SIG: &[u8; 4] = b"sig!";

/// Build a `DidKeyedSession` with a permissive test verifier (per the
/// existing `admin_ui_v0_harness` pattern — verifier accepts the
/// `VALID_SIG` sentinel). This drives the production state-machine
/// end-to-end without standing up the full Ed25519 did:key resolver,
/// which is orthogonal to the §4.22 BRIDGE wiring this pin holds open.
fn test_session() -> DidKeyedSession {
    let clock = Arc::new(AtomicU64::new(1_700_000_000_u64));
    let nonce_counter = Arc::new(AtomicU64::new(1));
    let clock_for_closure = Arc::clone(&clock);
    let nonce_for_closure = Arc::clone(&nonce_counter);
    DidKeyedSession::with_hooks(
        SessionConfig::default(),
        Box::new(|_did, _msg, sig| {
            if sig == VALID_SIG.as_slice() {
                Ok(())
            } else {
                Err(format!("test verifier: bad sig (len={})", sig.len()))
            }
        }),
        Box::new(move || {
            let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
            let mut bytes = [0_u8; 32];
            bytes[..8].copy_from_slice(&n.to_le_bytes());
            bytes
        }),
        Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
    )
}

/// Drive the SHIPPED challenge → sign → establish path, returning the
/// minted token. Models the thin-client → full-peer handshake exactly
/// as the harness does (so the production state-machine is exercised,
/// not a test-double).
fn establish_session_as(
    session: &DidKeyedSession,
    principal_did: &str,
    origin: &str,
) -> SessionToken {
    let challenge = session.emit_challenge(origin.to_string());
    session
        .establish_session(
            &challenge,
            VALID_SIG.as_slice(),
            principal_did.to_string(),
            origin.to_string(),
        )
        .expect("test session establishes against permissive verifier")
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.22 thin-client bridge \
            principal-resolution-from-session-not-client)"]
fn bridge_resolves_principal_from_authenticated_session_not_client_input() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (substantive-arm anchor for pim-18 §3.6f):
    // drive the G24-F SHIPPED `DidKeyedSession::resolve` end-to-end.
    // The session resolves to its server-side bound principal-DID, and
    // there is no client-supplied principal field on the resolve API —
    // a client-asserted "I am X" is structurally impossible to thread
    // through this surface. Would-FAIL signal: if a future regression
    // added an in-band-client-principal field, the assertion below
    // fires (the resolve return MUST equal the originally-established
    // principal-DID, with NO client override path).
    // -----------------------------------------------------------------
    let session = test_session();
    let origin = "https://benten.localhost:8443";

    // Attacker establishes their own session AS the attacker DID.
    let attacker_token = establish_session_as(&session, ATTACKER_DID, origin);

    // Resolve attacker's token AT bound origin. The SHIPPED API surface
    // takes ONLY (token, presented_origin) — there is no client-asserted-
    // principal field by construction. The resolved DID MUST be the
    // attacker's, NOT any victim DID the attacker might attempt to
    // "claim" in-band (the API has no such in-band field).
    let resolved = session
        .resolve(&attacker_token, origin)
        .expect("attacker's own token resolves to attacker DID at bound origin");
    assert_eq!(
        resolved, ATTACKER_DID,
        "shipped surface exercise: the SHIPPED `DidKeyedSession::resolve` \
         returns the server-side bound principal-DID; an attacker cannot \
         elevate to a victim DID because the API has NO client-principal \
         parameter. Would-FAIL if a regression added an in-band-client-\
         principal path that overrode the session-bound DID."
    );
    assert_ne!(
        resolved, VICTIM_DID,
        "shipped surface exercise: attacker's session resolve does NOT \
         elevate to the victim DID — there is no in-band client-principal \
         override path on the SHIPPED API."
    );

    // Origin-mismatch arm (boundary): the resolved principal does NOT
    // surface to a caller presenting a different origin (defense-in-
    // depth against confused-deputy at the bridge entry point).
    let cross_origin_resolve = session.resolve(&attacker_token, "https://other.example");
    assert!(
        matches!(
            cross_origin_resolve,
            Err(ThinClientSessionError::OriginMismatch { .. })
        ),
        "shipped surface exercise: cross-origin resolve is rejected; \
         the bridge cannot be tricked into surfacing a session principal \
         at an attacker-controlled origin. Would-FAIL if cross-origin \
         resolves silently succeeded."
    );

    // -----------------------------------------------------------------
    // RED-arm: the Phase-4-Meta BRIDGE wiring that threads the
    // resolved-from-session principal onto the
    // `Engine::read_node_as`/Class-B-β read path is UNDELIVERED. The
    // shipped `DidKeyedSession::resolve` returns the right principal
    // (exercised above); what is missing is the Phase-4-Meta plugin
    // read-path consumer that calls it + threads the result.
    // -----------------------------------------------------------------
    panic!(
        "§4.22 thin-client bridge principal-resolution undelivered: the \
         SHIPPED `DidKeyedSession::resolve` is exercised above (returns \
         the bound principal-DID, rejects cross-origin), but the \
         Phase-4-Meta BRIDGE wiring that resolves the acting principal \
         from the authenticated DID-keyed session AND threads it as the \
         `Engine::read_node_as` walk-principal on the Class-B-β plugin \
         read path is not yet built. G-CORE-8 must assert a client-\
         asserted principal cannot override the session principal on \
         the Class-B-β read/write path (the DESTINATION-REMAPPED \
         admin_ui_v0_thin_client_bridge_resolves_principal_from_session_not_client.rs \
         pin un-ignores together per §3.6e)."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.22 — no cap tokens \
            cross the thin-client boundary into client storage)"]
fn thin_client_boundary_carries_no_cap_tokens_to_the_client() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the G24-F SHIPPED session API
    // structurally cannot serialize a cap-token across the boundary —
    // its surface is `(challenge, signature, principal_did, origin)
    // → SessionToken` and `SessionToken + origin → principal_did`. No
    // UCAN / grant serialization is in the API. Exercising the surface
    // end-to-end confirms the only thing crossing the boundary is the
    // session token (an opaque correlation handle), NOT capability
    // tokens.
    // -----------------------------------------------------------------
    let session = test_session();
    let origin = "https://benten.localhost:8443";
    let token = establish_session_as(&session, ATTACKER_DID, origin);

    // The `SessionToken` is the ONLY artifact the API hands back
    // across the boundary. The struct carries: token_id, principal_did
    // (client-display only), bound_origin, expires_at_unix_secs — NO
    // capability fields. The resolve path consumes (token, origin)
    // alone and returns the bound principal-DID — again, no cap fields.
    let _principal = session
        .resolve(&token, origin)
        .expect("session round-trip works against the shipped surface");

    // Would-FAIL signal: if the SessionToken struct grew a UCAN /
    // capability-bundle field, this compile-time enumeration of every
    // visible field would no longer fit the destructuring pattern + the
    // build breaks. The destructuring IS the structural assertion.
    let SessionToken {
        token_id: _,
        principal_did: _,
        bound_origin: _,
        expires_at_unix_secs: _,
    } = token;

    // -----------------------------------------------------------------
    // RED-arm: the Phase-4-Meta plugin/Class-B-β read path is the
    // surface where a regression could LEAK a cap-token (a future
    // bridge implementation could erroneously serialize a UCAN onto
    // the client response). That bridge is undelivered; the pin holds
    // open the obligation to test the no-cap-tokens-cross-the-boundary
    // invariant against the Phase-4-Meta read path specifically.
    // -----------------------------------------------------------------
    panic!(
        "§4.22 cap-token-boundary undelivered: the SHIPPED \
         `DidKeyedSession` API does not carry cap-tokens across the \
         boundary (exercised above: only the session token round-trips, \
         and its fields are statically enumerated to contain no cap \
         payload), but the Phase-4-Meta BRIDGE that consumes the \
         session and drives the Class-B-β plugin read path is unbuilt; \
         the no-cap-tokens-cross-the-boundary invariant CANNOT yet be \
         exercised against that production read path. Un-ignore at \
         G-CORE-8."
    );
}

// --- Sealed-trait object-safety compile pin (§1.A.FROZEN item 8) ---
//
// This is a COMPILE-TIME pin, NOT ignored — it asserts that the
// `CapabilityPolicy` trait remains object-safe (`Arc<dyn ...>`
// constructible) under the G-CORE-8 sealed-discipline refactor
// (private `Sealed` supertrait, object-safety preserved). If
// G-CORE-8's sealing breaks object-safety this file fails to compile
// — the structural backstop for §1.A.FROZEN item 8.

#[test]
fn capability_policy_remains_object_safe_under_sealed_supertrait() {
    // Object-safety structural pin. The concrete default
    // `NoAuthBackend` is `Arc<dyn CapabilityPolicy>`-boxable today;
    // post-G-CORE-8 sealing this MUST still hold (object-safety
    // preserved under the private `Sealed` supertrait per §1.A.FROZEN
    // item 8). The boxing line is the compile-time assertion.
    fn assert_boxable(_p: Arc<dyn benten_caps::CapabilityPolicy>) {}

    let policy: Arc<dyn benten_caps::CapabilityPolicy> = Arc::new(benten_caps::NoAuthBackend);
    assert_boxable(policy);
}
