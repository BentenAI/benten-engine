//! G24-D + T6 — admin UI v0 install across two atrium peers; manifest
//! signature verified.

mod common;

use common::manifest_fixtures::admin_ui_v0_manifest;

#[ignore = "R4b-FP-1 redirected — end-to-end cross-Atrium install requires \
    `plugin_lifecycle::accept_atrium_share` (NOT shipped at R4b-FP-1; only the \
    single-process install_plugin seam landed at Seam 1). Single-process install \
    path is covered by `g24d_substantive_pipeline::full_install_pipeline_real_signatures_succeeds`. \
    Named destination: docs/future/phase-4-backlog.md §4.19 (`plugin_lifecycle::accept_atrium_share` \
    cross-peer install seam — Phase-4-Meta). HARD RULE 12 clause-(b) BELONGS-NAMED-NOW \
    satisfied via backlog §4.19 entry."]
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
