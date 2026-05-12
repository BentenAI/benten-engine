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

#[test]
#[ignore = "RED-PHASE: G24-D wave wires peer-DID signature verification at install; un-ignore at G24-D landing"]
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
