//! **LOAD-BEARING** per R2 §5 Gap fix #4 — defense-in-depth pin at
//! `apply_atrium_merge` boundary.
//!
//! Per T8 defense narrative (sec-3.5-r1-8 + sec-4f-r1-3): even if a
//! chain leaks past the synchronous write boundary, the merge-boundary
//! recheck catches it.
//!
//! Per Phase-3 PR #161 G16-B-F sec-r4r1-2 closure (structural-always-
//! on per-row cap-recheck inside `apply_atrium_merge`): that recheck
//! path EXTENDS at Phase 4-Foundation to also call the manifest-
//! envelope check.
//!
//! Promoted to LOAD-BEARING per R2 §5 Gap fix #4 in the F3 commit
//! message + this #[doc] comment.

#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 wave extends apply_atrium_merge with manifest-envelope recheck; un-ignore at G24-D-FP-2 landing"]
fn apply_atrium_merge_per_row_recheck_extends_to_manifest_envelope_check() {
    // Future surface: apply_atrium_merge's existing per-row cap-
    // recheck (PR #161 G16-B-F) extends to ALSO call
    // manifest_envelope_chain_validation::validate_chain_with_manifest_
    // envelope on each row whose write was authorized by a plugin-
    // delegated chain.
    //
    // Hostile scenario: chain leaks past sync sender's cap-policy
    // (signature-verifies but breaks manifest envelope); on receive,
    // apply_atrium_merge's recheck catches the envelope violation +
    // rejects row + surfaces ErrorCode::
    // PluginDelegationOutsideManifestEnvelope.
    //
    // FAILS-IF-NO-OP because the structural-always-on recheck must
    // include the envelope check, not just signature re-verification.
    //
    // LOAD-BEARING (per R2 §5 Gap fix #4): this is the sole pin
    // covering the defense-in-depth seam at the sync merge boundary;
    // failure here means hostile chains that leak past sync are not
    // caught.
    panic!(
        "RED-PHASE: G24-D-FP-2 wave must extend apply_atrium_merge with manifest-envelope recheck — LOAD-BEARING per R2 §5 Gap fix #4"
    );
}

#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 wave wires the regression-guard for legitimate-chain admit at merge boundary"]
fn apply_atrium_merge_legitimate_chain_admitted_no_regression() {
    // False-positive regression guard at merge boundary: ensure
    // legitimate within-envelope chains continue to be admitted
    // through the merge-boundary recheck.
    panic!("RED-PHASE: G24-D-FP-2 wave must admit legitimate chains at apply_atrium_merge");
}
