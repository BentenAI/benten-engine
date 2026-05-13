//! T6b pin — substitution-with-content-addressing-only attack.
//!
//! Per admin-ui-v0-threat-model.md §T6b: attacker constructs malicious
//! admin UI subgraph; computes its CID `Z`; publishes `Z` to the
//! atrium under same human-readable name as legitimate `X`. Content-
//! addressing PASSES (`Z` matches its own bytes). Defense: peer-DID
//! signature on original content rejects since attacker's signature
//! doesn't match the trusted peer-DID.

#[test]
#[ignore = "RED-PHASE: G24-D wave wires sync-side peer-DID signature check at plugin-share boundary; un-ignore at G24-D landing"]
fn atrium_share_with_different_peer_did_signature_rejected_via_peer_did_signature_check() {
    // Future surface: atrium-share receive path consults manifest's
    // peer-DID signature; if signature was made by Attacker's key and
    // doesn't match Alice's peer-DID (which user trusts), reject with
    // ErrorCode::PluginContentPeerSignatureInvalid OR
    // ErrorCode::PluginAuthorNotTrusted depending on whether the
    // attacker's DID is in trust-list.
    panic!("RED-PHASE: G24-D wave must wire peer-DID signature check at atrium-share receive");
}
