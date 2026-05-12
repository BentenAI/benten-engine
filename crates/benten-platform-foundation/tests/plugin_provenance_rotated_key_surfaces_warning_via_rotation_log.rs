//! G24-D row test pin — RotationLog integration on plugin provenance.
//!
//! Verifies a manifest whose peer-DID has been rotated surfaces a
//! `E_PLUGIN_CONTENT_PEER_KEY_ROTATED` warning (NOT hard-reject by
//! default per D-4F-12). Couples to `benten-id` RotationLog (Phase-3
//! G14-A2 wave-4a' shipped).

mod common;

use common::manifest_fixtures::minimal_manifest;

#[test]
#[ignore = "RED-PHASE: G24-D wave wires RotationLog consultation at manifest-load surface; un-ignore at G24-D landing"]
fn manifest_with_rotated_peer_did_surfaces_rotated_key_warning_not_hard_reject() {
    let _manifest = minimal_manifest();

    // Future G24-D surface:
    //   PluginManifest::validate_with_rotation_log(&rotation_log)
    //     -> Result<ValidationOutcome, ErrorCode>
    // Returns Ok(ValidationOutcome::ValidWithWarning(RotatedKeyWarning))
    // when peer-DID found in RotationLog with a rotation-event.
    // Returns Ok(ValidationOutcome::Valid) when no rotation found.
    // Returns Err(E_PLUGIN_CONTENT_PEER_KEY_ROTATED) ONLY when user has
    // explicitly opted into strict-mode (default = warning).
    //
    // FAILS-IF-NO-OP because rotation_log lookup is a real RotationLog
    // round-trip; stubbed-no-op would return Valid even when rotation
    // is present.
    panic!(
        "RED-PHASE: G24-D wave must wire RotationLog consultation at PluginManifest::validate_with_rotation_log"
    );
}
