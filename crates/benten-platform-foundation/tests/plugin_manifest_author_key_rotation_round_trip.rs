//! Phase-4-Foundation R4-FP-1 — T5c pin: plugin manifest author
//! peer-DID key rotation round-trip (RotationLog flow).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T5c + defense step 4 (peer-DID rotation via RotationLog;
//! Phase-3 G16-D wave-6b precedent) + sec-4f-r1-15 replay defense.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T5c: "Peer-DID signing-key compromise. A plugin
//! author's peer-DID signing key is leaked. Attacker signs a malicious
//! content revision. Defended by content-addressing (the malicious
//! content has a different CID) + `benten-id` RotationLog (the leaked-
//! key is rotated to a new key; old key marked rotated; admin UI
//! surfaces 'plugin came from rotated key' warning per D-4F-12)."
//!
//! Round-trip flow:
//!   1. Alice installs key K1; signs content C1.
//!   2. K1 is leaked.
//!   3. Alice rotates to K2 via signed RotationLog event.
//!   4. User receives admin-UI warning "plugin came from rotated key"
//!      at next access (NOT auto-reject; user decides per D-4F-12).
//!   5. Subsequent content signed by K2 is silent (rotation accepted).
//!
//! Per pim-2 §3.6b sub-rule 4: T5c is per-finding-granular (distinct
//! from T5a/T5b).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires content-CID check but doesn't consult RotationLog
//! at install/load. User loads plugin signed by K1 AFTER rotation;
//! no "rotated key" warning surfaces; user has no signal the signing
//! key was compromised. Defense-in-depth gap.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_peer_did_alice, stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-2 wave un-ignores) — \
    T5c peer-DID key-rotation round-trip via RotationLog integration: K1 leaked, \
    Alice rotates to K2 via signed RotationLog event, post-rotation content signed \
    by K1 surfaces 'plugin came from rotated key' warning per D-4F-12 (NOT hard- \
    reject by default). Requires PluginManifest::validate_with_rotation_log (per \
    phase-4-backlog §4.9) shipping at G24-D-FP-2. Named destination: plan §3 \
    G24-D-FP-2 (manifest envelope chain validator + RotationLog integration). \
    HARD RULE 12 clause-(b) BELONGS-NAMED-NOW."]
#[test]
fn plugin_manifest_peer_did_key_rotation_surfaces_warning_round_trip() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_manifest::ManifestStore;
    //   use benten_id::rotation_log::RotationLog;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let alice = stub_peer_did_alice();
    //   let user_did = stub_user_did();
    //   let plugin_did = stub_plugin_did();
    //
    //   // Step 1: Alice installs plugin signed by K1.
    //   let k1 = common::manifest_fixtures::alice_key_v1();
    //   common::manifest_fixtures::install_plugin_signed_by_key(
    //       &mut engine, plugin_did.clone(),
    //       /* peer_did */ alice.clone(),
    //       /* key */ k1.clone(),
    //       /* content */ b"original plugin content",
    //   ).unwrap();
    //
    //   // Step 2-3: K1 leaked; Alice rotates to K2 via signed
    //   // RotationLog event (signed by K1 since rotation event MUST be
    //   // signed by the old key per RotationLog discipline).
    //   let k2 = common::manifest_fixtures::alice_key_v2();
    //   let rotation_event = common::manifest_fixtures::
    //       sign_rotation_event(
    //           alice.clone(),
    //           /* old_key */ k1.clone(),
    //           /* new_key */ k2.clone(),
    //           /* hlc */ 100,
    //           /* nonce */ vec![0u8; 16],
    //       );
    //   engine.rotation_log_mut()
    //       .accept_rotation_event(&rotation_event)
    //       .unwrap();
    //
    //   // Step 4: User loads plugin (still signed by old K1). Admin UI
    //   // surfaces "plugin came from rotated key" warning per D-4F-12.
    //   let load_result = engine.manifest_store()
    //       .load_verified(&plugin_did);
    //   match load_result {
    //       Ok(record) => {
    //           // Load succeeds (NOT auto-reject) but surface warning:
    //           assert!(record.has_key_rotation_warning(),
    //               "T5c: load must SURFACE rotation warning; \
    //                D-4F-12 specifies non-rejecting flow");
    //       },
    //       Err(e) if matches!(e.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_KEY_ROTATED) => {
    //           // Acceptable: typed error surfaces rotation; user
    //           // decides whether to trust.
    //       },
    //       Err(other) => panic!("T5c: must EITHER warn-surface OR \
    //               surface typed E_PLUGIN_CONTENT_PEER_KEY_ROTATED; \
    //               got {:?}", other),
    //   }
    //
    //   // Defense-in-depth: warning surfaces to user via admin UI:
    //   let warnings = engine.captured_user_warnings_for_plugin(&plugin_did);
    //   assert!(
    //       warnings.iter().any(|w| w.is_rotated_key_warning(&alice)),
    //       "T5c: user-facing warning MUST surface; silent rotation \
    //        defeats the user-decides-trust posture per D-4F-12"
    //   );
    //
    //   // Step 5: New content signed by K2 — silent, no warning.
    //   common::manifest_fixtures::upgrade_plugin_signed_by_key(
    //       &mut engine, plugin_did.clone(),
    //       /* peer_did */ alice.clone(),
    //       /* key */ k2.clone(),
    //       /* new_content */ b"rotated-key plugin content",
    //   ).unwrap();
    //   let post_rotation_warnings = engine
    //       .captured_user_warnings_for_plugin(&plugin_did);
    //   assert!(
    //       post_rotation_warnings.iter()
    //           .filter(|w| w.is_rotated_key_warning(&alice))
    //           .count() == warnings.iter()
    //           .filter(|w| w.is_rotated_key_warning(&alice))
    //           .count(),
    //       "T5c: post-rotation content signed by new key K2 MUST be \
    //        silent — no additional rotation warning"
    //   );
    //
    // OBSERVABLE consequence: full round-trip (install K1 → rotate to
    // K2 → load-with-warning → silent post-rotation install).
    panic!(
        "RED-PHASE: G24-D must wire peer-DID key rotation round-trip \
         (T5c). Substantive: install K1 + rotate-via-RotationLog + \
         load-with-warning + silent K2-signed upgrade."
    );
}
