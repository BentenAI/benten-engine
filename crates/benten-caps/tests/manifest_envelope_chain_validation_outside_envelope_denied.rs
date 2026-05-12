//! G24-D-FP-2 LOAD-BEARING pin — denies chains that signature-verify
//! cleanly but don't fit source manifest's shares policy.
//!
//! Per T8 defense narrative (sec-4f-r1-3): regression case where
//! cap-backend validates UCAN signatures, forgets to check manifest
//! envelope, admits a delegation outside the envelope.
//!
//! Per pim-2 §3.6b + pim-18 §3.6f: load-bearing SUBSTANTIVE pin —
//! drives production-source arm with actual hostile chain, not just
//! asserts sentinel flag.

#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 wires the hostile-chain rejection; un-ignore at G24-D-FP-2 landing"]
fn ucan_chain_outside_manifest_envelope_denied_load_bearing() {
    // Future surface: SAME validator as the within-envelope test, but
    // with a manifest whose shares policy = SharesPolicyDefault::None
    // for the relevant cap. Hostile chain: plugin A signature-verifies
    // a delegation of cap C to plugin B, but A's manifest.shares does
    // NOT permit delegating C. Validator MUST reject with
    // ErrorCode::PluginDelegationOutsideManifestEnvelope.
    //
    // SUBSTANTIVE per pim-18: the hostile chain is constructed with
    // a real Ed25519 signature (test setup signs locally); the only
    // thing wrong is the manifest envelope mismatch. FAILS-IF-NO-OP
    // because pure UCAN chain validator would accept (signatures
    // valid), and only the envelope layer catches.
    panic!(
        "RED-PHASE: G24-D-FP-2 wave must wire hostile-chain rejection at manifest envelope boundary"
    );
}
