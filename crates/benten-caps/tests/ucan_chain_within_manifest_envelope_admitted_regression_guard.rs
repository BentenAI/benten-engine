//! G24-D-FP-2 regression-guard — false-positive guard.
//!
//! Per T8 defense plan: ensure the envelope check doesn't over-reject.
//! A chain where every delegation step DOES fit the source plugin's
//! manifest `shares` policy MUST be admitted.

#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 wires within-envelope admission path; un-ignore at G24-D-FP-2 landing"]
fn legitimate_chain_within_manifest_envelope_admitted_no_false_positive() {
    // Future surface: validate_chain_with_manifest_envelope returns
    // Ok(()) for a chain where:
    //   step 1: user-DID delegates "store:notes:read" to plugin A
    //   step 2: plugin A delegates "store:notes:read" to plugin B,
    //     where plugin A's manifest.shares includes a rule with
    //     cap_pattern="store:notes:read" + target=PluginDid(B).
    //
    // FAILS-IF-OVER-REJECT (a regression that adds envelope check
    // but breaks valid chains).
    panic!(
        "RED-PHASE: G24-D-FP-2 wave must admit legitimate within-envelope chains (no false positive)"
    );
}
