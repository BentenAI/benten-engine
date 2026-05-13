//! G24-D row test pin — RotationLog integration on plugin provenance.
//!
//! Verifies a manifest whose peer-DID has been rotated surfaces a
//! `E_PLUGIN_CONTENT_PEER_KEY_ROTATED` warning (NOT hard-reject by
//! default per D-4F-12). Couples to `benten-id` RotationLog (Phase-3
//! G14-A2 wave-4a' shipped).

mod common;

use benten_id::did_rotation::RotationLog;
use benten_platform_foundation::ValidationOutcome;
use common::manifest_fixtures::{fresh_keypair, manifest_signed_by, signed_rotation_event};

#[test]
fn manifest_with_rotated_peer_did_surfaces_rotated_key_warning_not_hard_reject() {
    // G24-D-FP-2 SUBSTANTIVE arm.
    //
    // 1. Alice mints keypair K1; manifest is signed by K1.
    let k1 = fresh_keypair();
    let manifest = manifest_signed_by(&k1);

    // 2. Empty rotation log → ValidationOutcome::Valid (no warning).
    let empty_log = RotationLog::new();
    let outcome = manifest
        .validate_with_rotation_log(&empty_log)
        .expect("structurally-valid manifest with no rotation events validates clean");
    assert_eq!(
        outcome,
        ValidationOutcome::Valid,
        "Without RotationLog entries, validate_with_rotation_log returns Valid (no warning)"
    );
    assert!(
        !outcome.has_rotated_key_warning(),
        "Valid outcome carries no rotation warning"
    );

    // 3. K1 leaked; Alice rotates to K2; rotation event signed by K1
    //    accepted into log.
    let k2 = fresh_keypair();
    let mut log = RotationLog::new();
    log.accept_rotation_event(&signed_rotation_event(&k1, &k2, 100))
        .expect("first rotation accepts cleanly");

    // 4. Same manifest (still signed by K1) loaded against the now-
    //    populated log surfaces a RotatedKeyWarning (NOT hard-reject
    //    per D-4F-12 — admin UI surfaces warning, user decides trust).
    let outcome_after_rotation = manifest
        .validate_with_rotation_log(&log)
        .expect("D-4F-12: rotation is WARNING, not Err — caller decides trust");
    assert!(
        outcome_after_rotation.has_rotated_key_warning(),
        "D-4F-12 LOAD-BEARING: rotation observed in log MUST surface warning to caller; \
         silent rotation defeats user-decides-trust posture"
    );
    match outcome_after_rotation {
        ValidationOutcome::ValidWithWarning(w) => {
            assert_eq!(
                w.rotated_peer_did,
                k1.public_key().to_did(),
                "Warning identifies the rotated peer-DID (the OLD key, which is also the manifest's peer_did)"
            );
        }
        ValidationOutcome::Valid => {
            panic!(
                "RotationLog consultation MUST surface rotation warning when peer-DID is superseded"
            );
        }
    }
}
