//! Phase-4-Foundation R4-FP-1 — T5 regression-guard: signature replay
//! with different nonce rejected (defense-in-depth with HLC ordering).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T5
//! ("Regression-guard") + sec-4f-r1-15 replay defense.
//!
//! ## What this pin establishes
//!
//! Regression-guard for the T5 family. RotationLog acceptance is bound
//! by HLC monotonicity + nonce-binding via the rotation event's
//! signature payload (per Phase-3 G16-D wave-6b precedent). This pin
//! asserts:
//!
//! 1. **Verbatim replay**: the exact same rotation event re-submitted
//!    is rejected.
//! 2. **HLC-monotonic-strict**: an attestation whose `superseded_at`
//!    is NOT strictly greater than the latest event for the same
//!    `previous_did` is rejected even if signed/structurally valid.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! A no-op `accept_rotation_event` that always pushes would admit the
//! replay; the assertions below would fail. The HLC-strict defense is
//! the production-arm of the nonce-binding pair: even if the signature
//! field were swapped, the HLC has to be strictly greater than the
//! latest accepted for the same prev-DID, defeating same-HLC nonce
//! mutation attacks.

#![allow(clippy::unwrap_used)]

mod common;

use benten_id::did_rotation::RotationLog;
use benten_id::errors::DidRotationError;
use common::manifest_fixtures::{fresh_keypair, signed_rotation_event};

#[test]
fn plugin_manifest_rotation_event_nonce_swap_attack_rejected() {
    let k1 = fresh_keypair();
    let k2 = fresh_keypair();

    // Setup: alice rotates K1 → K2 at HLC 100.
    let mut log = RotationLog::new();
    let rotation_v1 = signed_rotation_event(&k1, &k2, 100);
    log.accept_rotation_event(&rotation_v1)
        .expect("first rotation accepts cleanly");

    // Attack 1: replay SAME rotation event verbatim — duplicate by
    // (previous_did, next_did, superseded_at, signature) → rejected.
    let dup_attempt = log.accept_rotation_event(&rotation_v1);
    let dup_err = dup_attempt
        .expect_err("T5 regression-guard: verbatim replay of accepted rotation MUST be rejected");
    assert!(
        matches!(dup_err, DidRotationError::VerbatimReplay { .. }),
        "T5 regression-guard: must surface VerbatimReplay typed err; got {dup_err:?}"
    );

    // Attack 2: nonce-swap — attacker captures rotation_v1, mutates a
    // payload bit (here we keep prev/next/hlc but flip the signature
    // byte to simulate a re-signed-with-compromised-K1 mutation). Same
    // HLC 100. Since HLC is NOT strictly greater than latest-known
    // (which is 100), this MUST be rejected at the HLC-monotonic-strict
    // layer.
    let mut nonce_swapped = rotation_v1.clone();
    // Flip a single signature byte to simulate the attacker's
    // re-signed-payload mutation. The (prev_did, next_did, hlc) tuple
    // stays identical — the HLC-monotonic-strict layer rejects on HLC
    // alone, NOT on signature comparison.
    nonce_swapped.signature[0] ^= 0xFF;
    let nonce_swap_attempt = log.accept_rotation_event(&nonce_swapped);
    let nonce_err = nonce_swap_attempt.expect_err(
        "T5 regression-guard: nonce-swap at same HLC MUST be rejected — HLC monotonicity \
         is the primary defense; signature-mutation does NOT bypass it",
    );
    assert!(
        matches!(nonce_err, DidRotationError::HlcNotStrictlyMonotonic { .. }),
        "T5 regression-guard: nonce-swap at same HLC must surface HlcNotStrictlyMonotonic; \
         got {nonce_err:?}"
    );

    // Attack 3: strictly-greater HLC accepts a fresh event (boundary
    // test — defense must NOT over-fire on legitimate advances).
    let k3 = fresh_keypair();
    let legit_advance = signed_rotation_event(&k2, &k3, 200);
    log.accept_rotation_event(&legit_advance)
        .expect("Boundary: strictly-greater-HLC + fresh prev-DID accepts cleanly");
}
