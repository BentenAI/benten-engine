//! G24-D row pin — pull-model CID verification on receive.
//!
//! Per docs/PLUGIN-MANIFEST.md §4.1 step 2(a): receiver verifies bytes
//! hash to declared content-CID. Mismatch surfaces
//! `E_PLUGIN_CONTENT_CID_MISMATCH`.
//!
//! Defends against T6a substitution-at-transit attacks.

mod common;

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_bytes_with_mismatched_announced_cid_rejected_with_typed_error() {
    let claimed_cid = common::manifest_fixtures::stub_cid_one();
    let actual_bytes = b"these bytes do not hash to claimed_cid".to_vec();

    // Future surface:
    //   plugin_lifecycle::receive_plugin(claimed_cid, bytes) ->
    //     Result<PluginManifest>
    //   Hashes bytes -> computed_cid; compares to claimed_cid;
    //   returns ErrorCode::PluginContentCidMismatch on mismatch.
    //
    // FAILS-IF-NO-OP because Phase-3 R5 wave-9 W9-T6 ships content-
    // address verification at benten-sync; this pin verifies the
    // plugin-share path uses it, surfacing the typed error.
    let _ = (claimed_cid, actual_bytes);
    panic!("RED-PHASE: G24-D wave must wire E_PLUGIN_CONTENT_CID_MISMATCH at plugin receive");
}
