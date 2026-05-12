//! LOAD-BEARING per plan §3 G24-D row + CLAUDE.md #18 retense.
//!
//! Verifies plugin-DID is a UCAN audience handle ONLY — NOT an
//! attested sub-identity of user-DID. Should be NO attestation-chain
//! validation running against plugin-DID.
//!
//! Per R2 §5 substance discipline:
//! - NEGATIVE: grep-assert absence of `attestation_chain_for_plugin_did`
//!   or similar device-DID-attestation patterns in benten-id.
//! - POSITIVE: positive audience-handle-flow test (UCAN with audience =
//!   plugin-DID validates without attestation-chain traversal).

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[test]
#[ignore = "RED-PHASE: G24-D wave provides UCAN audience-handle flow; un-ignore at G24-D landing"]
fn ucan_with_audience_equals_plugin_did_validates_without_attestation_chain() {
    let _user = stub_user_did();
    let _plugin = stub_plugin_did();

    // Future G24-D surface: user_did issues UCAN with
    // audience = plugin_did + cap = "store:notes:read"; chain
    // validator at `benten-caps::ucan_grounded::UcanGroundedPolicy`
    // accepts. NO attestation-chain check runs.
    //
    // FAILS-IF-NO-OP because the validator must consult the audience
    // field of the UCAN payload.
    panic!(
        "RED-PHASE: G24-D wave must wire UCAN audience-handle flow at user_did -> plugin_did delegation"
    );
}

#[test]
#[ignore = "RED-PHASE: G24-D wave provides the negative grep-assert; un-ignore at G24-D landing"]
fn no_attestation_chain_for_plugin_did_function_in_benten_id_grep_assert() {
    // Negative substance test (per R2 §5 Gap fix #5 paired discipline).
    // Future surface: grep over crates/benten-id/src/ asserting NO
    // symbol matches `attestation_chain_for_plugin_did|
    // PluginDidAttestationEnvelope|verify_plugin_did_attestation`.
    // Count == 0.
    //
    // Plugin-DIDs are UCAN audiences, NOT attested sub-identities like
    // device-DIDs (which DO have signed DeviceAttestationEnvelope V2
    // per Phase-3 G16-D wave-6b). The categories must remain distinct.
    panic!(
        "RED-PHASE: G24-D wave must implement grep-assertion over crates/benten-id/src/ counting 0 matches for attestation_chain_for_plugin_did|PluginDidAttestationEnvelope patterns"
    );
}
