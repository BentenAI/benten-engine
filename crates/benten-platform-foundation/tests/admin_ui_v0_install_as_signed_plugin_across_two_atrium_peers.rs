//! G24-D + T6 — admin UI v0 install across two atrium peers; manifest
//! signature verified.

mod common;

use common::manifest_fixtures::admin_ui_v0_manifest;

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn admin_ui_v0_installs_across_two_atrium_peers_when_peer_did_signature_valid() {
    let _manifest = admin_ui_v0_manifest();

    // Future surface: end-to-end test where peer A publishes admin UI
    // bundle to atrium; peer B receives + verifies peer-DID signature
    // + verifies content-CID match + installs. Admin UI's first
    // install is via this path (Phase 4-Foundation v0 ratification #3
    // — direct content-addressed-share over Atriums).
    panic!("RED-PHASE: G24-D wave must wire atrium-share install path for admin UI v0");
}
