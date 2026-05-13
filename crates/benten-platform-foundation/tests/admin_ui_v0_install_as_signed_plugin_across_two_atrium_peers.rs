//! G24-D + T6 — admin UI v0 install across two atrium peers; manifest
//! signature verified.

mod common;

use common::manifest_fixtures::admin_ui_v0_manifest;

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    Admin UI v0 end-to-end cross-atrium install path: peer A publishes admin UI \
    bundle; peer B receives + verifies peer-DID signature + verifies content-CID \
    + installs. Requires accept_atrium_share seam at plugin_lifecycle (G24-D-FP-1 \
    hardening). The CID + signature verification primitives ship at G24-D primary; \
    end-to-end cross-peer wiring is G24-D-FP-1 scope. Named destination: plan §3 \
    G24-D-FP-1 (plugin_lifecycle accept_atrium_share + install lifecycle). HARD \
    RULE 12 clause-(b) BELONGS-NAMED-NOW."]
#[test]
fn admin_ui_v0_installs_across_two_atrium_peers_when_peer_did_signature_valid() {
    let _manifest = admin_ui_v0_manifest();

    // Phase 4-Foundation R5 G24-D-FP-1 surface (NOT at G24-D primary):
    // end-to-end cross-atrium install path. The umbrella
    // `g24d_substantive_pipeline::full_install_pipeline_real_signatures_succeeds`
    // covers single-process install; cross-peer atrium-share requires
    // plugin_lifecycle::accept_atrium_share which ships at G24-D-FP-1.
    panic!("G24-D-FP-1 wires accept_atrium_share end-to-end cross-peer install path");
}
