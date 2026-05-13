//! G24-D + T6b defense (sec-3.5-r1-6 reframed).
//!
//! Hostile peer constructs a malicious admin UI subgraph; computes its
//! CID; publishes to atrium under same human-readable name. Without
//! peer-DID signature verification, content-addressing alone passes
//! (hostile bytes hash to their own CID). With peer-DID signature
//! verification, hostile peer's signature doesn't match Alice's
//! peer-DID and install rejects.

mod common;

use common::manifest_fixtures::{admin_ui_v0_manifest, stub_peer_did_attacker};

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    Admin UI v0 substitution-defense end-to-end with user-DID trust-list path: \
    install_plugin must consult user's requires_plugin_authors trust-list and \
    distinguish forged-claim (E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID) from \
    unknown-author (E_PLUGIN_AUTHOR_NOT_TRUSTED). Sibling \
    plugin_manifest_substitution_at_install_rejected.rs wires the forged-claim arm \
    NOW; trust-list arm lands at G24-D-FP-1. Named destination: plan §3 G24-D-FP-1 \
    (plugin_lifecycle hardening — trust-list integration). HARD RULE 12 clause-(b) \
    BELONGS-NAMED-NOW."]
#[test]
fn substituted_bundle_with_different_peer_did_signature_rejected_at_install() {
    let mut hostile = admin_ui_v0_manifest();
    hostile.peer_did = stub_peer_did_attacker();
    // Note: hostile.peer_signature is still 0-bytes; even if the
    // attacker fabricates a signature with their own key, it won't
    // verify against the EXPECTED peer-DID (which user's
    // requires_plugin_authors trust-list would name).

    // Future surface: install_plugin verifies signature against
    // declared peer_did + checks against user's trust-list. Rejects
    // with ErrorCode::PluginAuthorNotTrusted (when DID not in trust
    // list) OR ErrorCode::PluginContentPeerSignatureInvalid (when
    // signature itself is bad).
    panic!("RED-PHASE: G24-D wave must wire substitution-attack rejection via peer-DID signature");
}
