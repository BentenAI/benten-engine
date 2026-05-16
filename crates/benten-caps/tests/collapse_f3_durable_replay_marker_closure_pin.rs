//! COLLAPSE F3 — MANDATORY closure-pin (pim-2 §3.6b,
//! would-FAIL-if-no-op'd) for the durable replay-marker re-home.
//!
//! # Charter
//!
//! Spec: `DECISION-RECORD-trust-model-reframe.md §4b` (F3 RATIFIED:
//! "the durable replay-marker the P3 collapse deferred to P2/P5;
//! implement its durable re-home here") + `impl-design-COLLAPSE.md
//! §1.1a` (J5 anti-replay is a REWIRE, not a deletion — "flag
//! explicitly so the deletion PR does not silently drop replay
//! defense") + `docs/SECURITY-POSTURE.md` Compromise #23 (owner-
//! ratified): the nonce-replay defense is *"re-homed … not dropped"*.
//!
//! # The property this pins
//!
//! The COLLAPSE P3 rewire DELETED the device-attestation envelope's
//! ephemeral per-handle nonce-store (it collapsed with `Acceptor`);
//! only the freshness window survived inline at
//! `engine_sync::DeviceAttestationEnvelope::verify`. A freshness
//! window alone CANNOT catch a verbatim replay of a captured, still-
//! signature-valid envelope replayed INSIDE its freshness window.
//! Compromise #23's ratified wording asserts that defense is
//! re-homed durably — this pin asserts the durable marker actually
//! works: once a nonce is recorded, a replay of the same nonce is
//! observably detected, and the marker persists (durability contract,
//! same as `revoke`/`is_revoked`).
//!
//! If `record_seen_envelope_nonce` / `is_envelope_nonce_seen` are
//! no-op'd (record does nothing, or is_seen always returns false —
//! i.e. the durable replay defense is silently dropped), the
//! assertions below FAIL. That is the pim-2 §3.6b would-FAIL-if-
//! no-op'd contract: this pin is the durable half of the anti-replay
//! defense the ratified Compromise #23 wording commits to.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use benten_caps::UCANBackend;
use benten_graph::RedbBackend;

fn fresh_backend() -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    UCANBackend::new(Arc::new(inner))
}

/// **MANDATORY closure-pin (would-FAIL-if-no-op'd).**
///
/// A nonce not yet observed is NOT seen; after recording it, it IS
/// seen (the replay is detected). A distinct nonce is unaffected.
#[test]
fn durable_replay_marker_detects_a_replayed_envelope_nonce() {
    let backend = fresh_backend();
    let nonce_a = [7u8; 32];
    let nonce_b = [9u8; 32];

    // First observation: not previously seen (the legitimate first
    // delivery of a signed envelope).
    assert!(
        !backend.is_envelope_nonce_seen(&nonce_a).unwrap(),
        "a never-seen nonce MUST report not-seen on first delivery"
    );

    // The seam records it after a successful first verify.
    backend.record_seen_envelope_nonce(&nonce_a).unwrap();

    // F3 REGRESSION GUARD: a verbatim replay of the SAME signed
    // envelope (same nonce), still inside its freshness window, MUST
    // now be detected. If `record`/`is_seen` are no-op'd this FAILs —
    // that is exactly the durable replay defense Compromise #23's
    // ratified wording commits to ("re-homed … not dropped").
    assert!(
        backend.is_envelope_nonce_seen(&nonce_a).unwrap(),
        "COLLAPSE F3 REGRESSION: a recorded envelope nonce was NOT detected on \
         replay — the durable replay-marker re-home is no-op'd. The freshness \
         window alone cannot catch an in-window verbatim replay; this is the \
         durable anti-replay defense SECURITY-POSTURE Compromise #23 (owner-\
         ratified) states is re-homed, not dropped (DECISION-RECORD §4b F3; \
         pim-2 §3.6b would-FAIL-if-no-op'd)."
    );

    // Negative control: an unrelated nonce is unaffected — the marker
    // is per-nonce, not a blanket "any replay" flag.
    assert!(
        !backend.is_envelope_nonce_seen(&nonce_b).unwrap(),
        "a distinct, never-recorded nonce MUST remain not-seen"
    );
}

/// The durable marker is idempotent under repeated record (a legit
/// re-verify of the same envelope must not error) and stays seen.
#[test]
fn durable_replay_marker_is_idempotent_and_persists() {
    let backend = fresh_backend();
    let nonce = [42u8; 32];

    backend.record_seen_envelope_nonce(&nonce).unwrap();
    // Idempotent re-record must not error.
    backend.record_seen_envelope_nonce(&nonce).unwrap();
    assert!(
        backend.is_envelope_nonce_seen(&nonce).unwrap(),
        "a recorded nonce MUST stay seen across repeated record + probe \
         (durability contract, same as revoke/is_revoked)"
    );
}
