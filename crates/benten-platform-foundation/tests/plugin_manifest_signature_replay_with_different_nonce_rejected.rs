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
//! by signed nonce AND HLC monotonicity (Phase-3 G16-D wave-6b
//! precedent + payload-hash binding). This pin asserts that swapping
//! the nonce on a captured rotation event does NOT bypass the defense
//! — replay-with-fresh-nonce is rejected because HLC monotonicity is
//! the primary defense (nonce binds the rotation to a specific
//! transaction context).
//!
//! Companion to `schema_author_rotation_race_replay_rejected.rs` (T9b
//! HLC-monotonic-strict primary). This pin specifically tests the
//! nonce-swap attack variant.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires nonce check but skips HLC monotonicity. Attacker
//! captures a valid rotation event; mutates nonce; replays at a later
//! HLC; signature is recomputed (attacker has compromised K_old) ...
//! OR attacker captures + nonce-swaps; HLC unchanged → MUST be
//! rejected as duplicate. Mutation paths are exhausted by HLC +
//! nonce-binding pair.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_peer_did_alice, stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_manifest_rotation_event_nonce_swap_attack_rejected() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_id::rotation_log::RotationLog;
    //
    //   let mut log = RotationLog::new();
    //   let alice = stub_peer_did_alice();
    //
    //   // Setup: alice has rotated K1 → K2 at HLC 100 with nonce N1.
    //   let k1 = common::manifest_fixtures::alice_key_v1();
    //   let k2 = common::manifest_fixtures::alice_key_v2();
    //   let rotation_v1 = common::manifest_fixtures::sign_rotation_event(
    //       alice.clone(), k1.clone(), k2.clone(),
    //       /* hlc */ 100, /* nonce */ vec![0xAA; 16],
    //   );
    //   log.accept_rotation_event(&rotation_v1).unwrap();
    //
    //   // Attack 1: replay SAME rotation event verbatim — duplicate
    //   // by (peer_did, hlc, nonce) → rejected.
    //   let dup_attempt = log.accept_rotation_event(&rotation_v1);
    //   assert!(dup_attempt.is_err(),
    //       "Verbatim replay must be rejected (nonce-binding)");
    //
    //   // Attack 2: nonce-swap — attacker captures rotation_v1,
    //   // mutates nonce field, re-signs WITH compromised K1. Same HLC
    //   // 100. Since HLC is NOT strictly greater than latest-known
    //   // (which is 100), this is a no-op replay at the HLC ordering
    //   // layer — must be rejected.
    //   let nonce_swapped = common::manifest_fixtures::
    //       rotation_event_with_nonce_swap(
    //           &rotation_v1, /* new_nonce */ vec![0xBB; 16],
    //       );
    //   let nonce_swap_attempt = log.accept_rotation_event(&nonce_swapped);
    //   let err = nonce_swap_attempt.expect_err(
    //       "T5 regression-guard: nonce-swap at same HLC MUST be \
    //        rejected — HLC monotonicity is the primary defense; \
    //        nonce-binding pairs with HLC, not alone"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_KEY_ROTATED
    //           | ErrorCode::E_HLC_NOT_MONOTONIC),
    //       "T5 regression-guard: must surface typed HLC/rotation error; \
    //        got {:?}", err.code()
    //   );
    //
    //   // Attack 3: even with fresh nonce + strictly-greater HLC, the
    //   // event must be signed by the CURRENT KEY (K2 post-rotation),
    //   // not K1. Re-signing with K1 is rejected because K1 is the
    //   // rotated-out key:
    //   let fresh_hlc_replay = common::manifest_fixtures::
    //       sign_rotation_event(
    //           alice.clone(), k1.clone(), k2.clone(),
    //           /* hlc */ 200, /* fresh nonce */ vec![0xCC; 16],
    //       );
    //   let bad_signer = log.accept_rotation_event(&fresh_hlc_replay);
    //   assert!(bad_signer.is_err(),
    //       "T5 regression-guard: rotation event signed by rotated-out \
    //        key MUST be rejected — RotationLog discipline");
    //
    // OBSERVABLE consequence: 3 attack variants (verbatim replay /
    // nonce-swap / rotated-out-key signing) all rejected; HLC +
    // nonce + key-validity form the three-way defense.
    panic!(
        "RED-PHASE: G24-D must wire RotationLog replay defense \
         regression-guard (T5 nonce-swap). Substantive: verbatim-\
         duplicate + nonce-swap + rotated-out-key-signer all rejected."
    );
}
