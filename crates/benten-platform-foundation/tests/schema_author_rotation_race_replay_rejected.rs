//! Phase-4-Foundation R4-FP-1 — T9b pin: schema author key-rotation
//! race replay rejected (RotationLog HLC-monotonic-strict ordering).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T9
//! defense step 3 (RotationLog with HLC-monotonic-strict acceptance
//! window per sec-4f-r1-10 + sec-4f-r1-15) + ratification #6 RotationLog
//! MVP.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T9b + defense step 3: attacker exploits the window
//! between a peer-DID's key rotation and the user's view of the new key.
//! Defense: RotationLog uses HLC-monotonic-strict ordering — stale
//! rotation events (HLC < latest-known) are REJECTED; replay-with-fresh-
//! RotationLog retry path handles the race window. Replay bounded by
//! signed nonce + payload-hash binding (Phase-3 G16-D wave-6b precedent).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires RotationLog accept path but skips HLC monotonicity
//! check OR omits nonce binding. Attacker replays a stale rotation
//! event (signed by old key, valid signature, but pre-rotation HLC); a
//! no-op verifier admits it; race-window exploitation succeeds.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: T9b is a SEPARATE pin from T9a
//! (different attack class — key-rotation race vs static forgery).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "DESTINATION-REMAPPED post-R1-triage per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW: T9b defense lives at PLUGIN-MANIFEST namespace (G24-D-FP-2), NOT schema-namespace (G23-B). \
RotationLog HLC-monotonic-strict + VerbatimReplay defense shipped at G24-D-FP-2 in `crates/benten-id/src/did_rotation.rs::RotationLog::accept_rotation_event`. \
Substantive coverage for T9b (rotation-race replay) at G24-D-FP-2's `plugin_manifest_rotation_event_nonce_swap_attack_rejected.rs` (3 attack variants) + `plugin_manifest_peer_did_key_rotation_surfaces_warning_round_trip.rs`. \
Per phase-4-foundation-backlog §4.12: this pin is retained as forward-looking documentation should schema-level provenance ever be needed. Un-ignore is N/A — defense lives at the manifest surface."]
fn schema_author_rotation_race_replay_rejected_via_hlc_monotonic_strict() {
    // G23-B wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::schema_provenance_validation::
    //       verify_schema_provenance;
    //   use benten_id::rotation_log::RotationLog;
    //
    //   let alice_did = common::manifest_fixtures::stub_peer_did_alice();
    //
    //   // Setup: Alice has rotated her signing key at HLC time T_n.
    //   // RotationLog records:
    //   //   - rotation event signed by old key, carrying new key + nonce
    //   //   - HLC = T_n (latest-known)
    //   let mut rotation_log = RotationLog::new();
    //   rotation_log.record_rotation(
    //       /* peer_did */ alice_did.clone(),
    //       /* old_key */ common::manifest_fixtures::alice_old_signing_key(),
    //       /* new_key */ common::manifest_fixtures::alice_new_signing_key(),
    //       /* hlc */ 100,
    //       /* nonce */ vec![0u8; 16],
    //   ).unwrap();
    //
    //   // Attack: attacker replays a STALE rotation event signed by old
    //   // key at HLC = 50 (pre-rotation). Valid signature; valid old key;
    //   // but HLC < latest-known.
    //   let stale_event = common::manifest_fixtures::stale_rotation_event(
    //       alice_did.clone(),
    //       /* hlc */ 50, // < 100
    //       /* signed by old key */
    //   );
    //   let replay_result = rotation_log.accept_rotation_event(&stale_event);
    //
    //   let err = replay_result.expect_err(
    //       "T9b: stale rotation event (HLC < latest-known) MUST be \
    //        REJECTED — HLC-monotonic-strict ordering is the defense"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_KEY_ROTATED
    //           | ErrorCode::E_HLC_NOT_MONOTONIC),
    //       "T9b: must surface typed HLC monotonicity violation; got {:?}",
    //       err.code()
    //   );
    //
    //   // Defense-in-depth: replay with FRESH nonce + stale HLC still
    //   // rejected (defense is HLC-based, not nonce-only):
    //   let stale_event_fresh_nonce = common::manifest_fixtures::
    //       stale_rotation_event_with_fresh_nonce(
    //           alice_did.clone(),
    //           /* hlc */ 50,
    //           /* fresh nonce */ vec![1u8; 16],
    //       );
    //   let replay2 = rotation_log.accept_rotation_event(&stale_event_fresh_nonce);
    //   assert!(replay2.is_err(),
    //       "T9b: fresh nonce does NOT bypass HLC-monotonic-strict; \
    //        HLC ordering is the primary defense");
    //
    //   // Schemas signed by Alice's OLD key (post-rotation) surface
    //   // "schema came from rotated key" warning at materializer entry
    //   // (NOT auto-reject per D-4F-12; user decides):
    //   let resolver = common::manifest_fixtures::test_did_resolver_with(&rotation_log);
    //   let schema_signed_by_old_key = common::schema_fixtures::
    //       schema_signed_by_peer_did(
    //           alice_did.clone(),
    //           common::manifest_fixtures::alice_old_signing_key(),
    //       );
    //   let provenance_result = verify_schema_provenance(
    //       &schema_signed_by_old_key, &resolver
    //   );
    //   match provenance_result {
    //       Err(e) if matches!(e.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_KEY_ROTATED) => {},
    //       _ => panic!("T9b: post-rotation schema MUST surface \
    //                    E_PLUGIN_CONTENT_PEER_KEY_ROTATED warning"),
    //   }
    //
    // OBSERVABLE consequence: HLC-monotonic-strict ordering closes the
    // rotation race window; replay defense reuses Phase-3 G16-D shape.
    panic!(
        "RED-PHASE: G23-B must wire RotationLog HLC-monotonic-strict \
         replay defense (T9b). Substantive: stale-HLC rejection + \
         fresh-nonce-doesn't-bypass + post-rotation-key warning surface."
    );
}
