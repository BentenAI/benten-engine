//! T6a pin — substitution at transit (content-addressing defense).
//!
//! Per admin-ui-v0-threat-model.md §T6a: peer A publishes admin UI
//! bundle CID `X`; peer B receives bundle bytes that hash to a
//! different CID `Y`. Without content-addressing verification on
//! receive, B installs the wrong bundle.
//!
//! Defense: `benten-sync` content-address-verifies all received Node
//! bytes against announced CIDs (Phase-3 R5 wave-9 W9-T6 shipped).
//! This pin verifies the plugin-share path uses it, surfacing the
//! typed error `E_PLUGIN_CONTENT_CID_MISMATCH` specifically.

#[test]
#[ignore = "RED-PHASE: G24-D wave wires plugin-share CID verification at sync layer; un-ignore at G24-D landing"]
fn atrium_share_bytes_dont_match_announced_cid_rejected_with_plugin_content_cid_mismatch() {
    // Future surface: sync receive path verifies bytes hash against
    // declared CID; surfaces ErrorCode::PluginContentCidMismatch for
    // the plugin-share-boundary case (distinct from generic sync CID
    // mismatch which surfaces ErrorCode::SyncDivergentCidRejected).
    //
    // The typed error allows admin UI to present a plugin-specific
    // error UI ("This plugin bundle was tampered with in transit")
    // rather than the generic sync error.
    panic!("RED-PHASE: G24-D wave must wire plugin-share CID mismatch typed error");
}
