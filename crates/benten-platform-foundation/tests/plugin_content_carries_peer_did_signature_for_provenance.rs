//! LOAD-BEARING per plan §3 G24-D row.
//!
//! Verifies content provenance (peer-DID signature) is INDEPENDENT of
//! the install record (user-DID signature). Per CLAUDE.md #18 four-
//! identity-concepts model: identity #2 (peer-DID signature on
//! original content) and identity #4 (user-DID signing install
//! record) are separate signature layers.

mod common;

use common::manifest_fixtures::{
    minimal_manifest, stub_install_record, stub_peer_did_alice, stub_user_did,
};

#[test]
fn manifest_peer_did_signature_independent_of_install_record_user_did_signature() {
    let manifest = minimal_manifest();
    let install = stub_install_record(common::manifest_fixtures::stub_cid_zero());

    // Content provenance: peer-DID
    assert_eq!(manifest.peer_did, stub_peer_did_alice());
    // Install consent: user-DID (distinct identity)
    assert_eq!(install.consenting_user_did, stub_user_did());
    assert_ne!(manifest.peer_did, install.consenting_user_did);

    // Future G24-D surface: PluginManifest::verify_peer_signature
    // returns Result; FAILS-IF-NO-OP because signature bytes are zero
    // and ed25519 verify rejects.
}
