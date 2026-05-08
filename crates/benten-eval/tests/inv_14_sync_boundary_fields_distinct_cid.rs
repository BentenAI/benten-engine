//! G16-B canary GREEN-PHASE: AttributionFrame Phase-3 sync-boundary
//! fields produce content-distinguishable CIDs (ds-r4b-1 BLOCKER closure).
//!
//! Per Inv-14 device-grain attribution + the additive-extension
//! discipline (D20-RESOLVED precedent for sandbox_depth):
//!
//! 1. A frame with all 3 new fields default (peer_did_set=None,
//!    device_did=None, sync_hop_depth=0) canonicalises IDENTICALLY to
//!    a Phase-2a 3-key Node (pinned by invariant_14_fixture_cid.rs).
//! 2. Setting EACH new field to a non-default value produces a
//!    DISTINCT CID — sync-attributed frames are content-
//!    distinguishable from purely-local frames.
//!
//! These pins close ds-r4b-1's R5-implementation gap (R4-R3 PIN-layer
//! convergence existed; R4b surfaced that the fields weren't shipped).

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use benten_core::Cid;
use benten_eval::AttributionFrame;

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

fn frame_default() -> AttributionFrame {
    AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
        peer_did_set: None,
        device_did: None,
        sync_hop_depth: 0,
    }
}

#[test]
fn peer_did_set_non_default_produces_distinct_cid() {
    let cid_default = frame_default().cid().unwrap();
    let mut frame = frame_default();
    let mut set = BTreeSet::new();
    set.insert("did:key:z6MkPeerA".to_string());
    set.insert("did:key:z6MkPeerB".to_string());
    frame.peer_did_set = Some(set);
    let cid_with_peers = frame.cid().unwrap();
    assert_ne!(
        cid_default, cid_with_peers,
        "non-empty peer_did_set MUST produce a distinct CID per Inv-14 \
         (sync-attributed frames are content-distinguishable from local frames)"
    );
}

#[test]
fn device_did_non_default_produces_distinct_cid() {
    let cid_default = frame_default().cid().unwrap();
    let mut frame = frame_default();
    frame.device_did = Some("did:key:z6MkDevice".to_string());
    let cid_with_device = frame.cid().unwrap();
    assert_ne!(
        cid_default, cid_with_device,
        "non-None device_did MUST produce a distinct CID per Inv-14 + D-PHASE-3-25"
    );
}

#[test]
fn sync_hop_depth_non_zero_produces_distinct_cid() {
    let cid_default = frame_default().cid().unwrap();
    let mut frame = frame_default();
    frame.sync_hop_depth = 1;
    let cid_with_depth = frame.cid().unwrap();
    assert_ne!(
        cid_default, cid_with_depth,
        "non-zero sync_hop_depth MUST produce a distinct CID per ds-r4b-1 \
         (mirrors sandbox_depth precedent at Inv-4)"
    );
}

#[test]
fn each_new_field_dimension_is_orthogonal_to_others() {
    // Setting each field independently produces a distinct CID from
    // setting any other field independently — no two dimensions collide.
    let mut peer_only = frame_default();
    let mut peers = BTreeSet::new();
    peers.insert("did:key:z6Mk".to_string());
    peer_only.peer_did_set = Some(peers);
    let cid_peer = peer_only.cid().unwrap();

    let mut device_only = frame_default();
    device_only.device_did = Some("did:key:z6Mk".to_string());
    let cid_device = device_only.cid().unwrap();

    let mut depth_only = frame_default();
    depth_only.sync_hop_depth = 3;
    let cid_depth = depth_only.cid().unwrap();

    assert_ne!(cid_peer, cid_device);
    assert_ne!(cid_device, cid_depth);
    assert_ne!(cid_peer, cid_depth);
}

#[test]
fn additive_extension_preserves_phase_2a_fixture_cid() {
    // The default-all-three-new-fields shape canonicalises to the
    // exact Phase-2a 3-key Node — pinned by invariant_14_fixture_cid.rs
    // FIXTURE_CID. This sentinel re-asserts the discipline at the
    // type-level so a future field whose serialiser doesn't honor the
    // skip-on-default discipline fires loudly here too.
    const PHASE_2A_FIXTURE_CID: &str =
        "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";
    let cid = frame_default().cid().unwrap();
    assert_eq!(
        cid.to_string(),
        PHASE_2A_FIXTURE_CID,
        "Phase-3 G16-B additive sync-boundary fields MUST canonicalise to the \
         Phase-2a fixture CID when all defaulted (additive-extension discipline)"
    );
}
