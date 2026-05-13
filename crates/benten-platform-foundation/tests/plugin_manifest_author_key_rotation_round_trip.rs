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

use benten_id::did_rotation::RotationLog;
use benten_platform_foundation::ValidationOutcome;
use common::manifest_fixtures::{fresh_keypair, manifest_signed_by, signed_rotation_event};

#[test]
fn plugin_manifest_peer_did_key_rotation_surfaces_warning_round_trip() {
    // G24-D-FP-2 SUBSTANTIVE arm — full T5c round-trip.
    //
    // Step 1: Alice installs plugin signed by K1.
    let k1 = fresh_keypair();
    let manifest_v1 = manifest_signed_by(&k1);

    // Step 2-3: K1 leaked; Alice rotates to K2 via signed RotationLog
    // event (signed by K1 since rotation event MUST be signed by the
    // old key per RotationLog discipline).
    let k2 = fresh_keypair();
    let mut log = RotationLog::new();
    let rotation_event = signed_rotation_event(&k1, &k2, 100);
    log.accept_rotation_event(&rotation_event)
        .expect("first rotation accepts cleanly under HLC-strict");

    // Step 4: User loads plugin (still signed by old K1). Admin UI
    // would surface "plugin came from rotated key" warning per D-4F-12.
    let outcome_for_old_signed = manifest_v1
        .validate_with_rotation_log(&log)
        .expect("D-4F-12: rotation is a warning, not Err");
    assert!(
        outcome_for_old_signed.has_rotated_key_warning(),
        "T5c LOAD-BEARING: load of K1-signed manifest after K1→K2 rotation MUST \
         surface warning; silent rotation defeats user-decides-trust posture per D-4F-12"
    );

    // Step 5: New content signed by K2 — silent, no warning (the
    // signer is K2 which is NOT superseded in the log).
    let manifest_v2 = manifest_signed_by(&k2);
    let outcome_for_new_signed = manifest_v2
        .validate_with_rotation_log(&log)
        .expect("post-rotation content signed by new key validates clean");
    assert_eq!(
        outcome_for_new_signed,
        ValidationOutcome::Valid,
        "T5c: post-rotation content signed by new key K2 MUST be silent — no rotation warning"
    );
}
