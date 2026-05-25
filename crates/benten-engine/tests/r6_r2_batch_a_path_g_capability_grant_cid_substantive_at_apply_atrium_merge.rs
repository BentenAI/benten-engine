//! **R6 R2 batch-A Item 4 (Path G substantive close)** — verifies
//! that `apply_atrium_merge` populates
//! `AttributionFrame.capability_grant_cid` from the resolved
//! `peer_actor_cid` (chain-anchor at the inbound-sync per-row
//! boundary) rather than the pre-Path-G zero-Cid sentinel.
//!
//! ## Why this test exists
//!
//! Pre-Path-G: `apply_atrium_merge` at `engine.rs:1701` minted the
//! `AttributionFrame` with `capability_grant_cid: Cid::from_blake3_digest([0u8; 32])`
//! — the zero-Cid sentinel — regardless of whether the per-row
//! admit_write_chain call (engine.rs:1569 in FP-B) had presented a
//! substantive grant CID anchor. The chain-anchor flowed THROUGH the
//! admission seam but did NOT land in the AttributionFrame's
//! forensic-recordable bytes; downstream audit pipelines could not
//! reconstruct "what grant authorized this inbound write?" from the
//! durable merge-Node properties.
//!
//! Post-Path-G: the same `peer_actor_cid` (= blake3 hash of the
//! resolved peer-DID, computed at engine.rs:1285) that the
//! per-row admit_write_chain consumes is ALSO threaded into the
//! AttributionFrame's `capability_grant_cid` slot at engine.rs:1701.
//!
//! ## Scope boundary
//!
//! Path G is the **LOCAL-origin (this-hop) substantive population**;
//! multi-hop preservation across `apply_atrium_merge` → outbound-sync
//! → next-peer-merge is OUT of Path G scope (Row D-27 / G-COMP-1
//! deferred; semantic redesign required because `StampedValue`
//! (`crates/benten-sync/src/crdt.rs:174-181`) does not carry an
//! upstream-grant slot).
//!
//! ## Test shape (§3.6f SUBSTANTIVE-arm)
//!
//! 1. Build two atrium peers, register the inbound peer's hlc_node_id
//!    under a known peer-DID.
//! 2. Sync a single property through `apply_atrium_merge(...)` end-to-
//!    end (PRODUCTION ENTRY POINT).
//! 3. Read the durable merge Version Node from the engine + extract
//!    `attribution_frame_cid`.
//! 4. Reconstruct the expected `AttributionFrame` with `capability_grant_cid
//!    = blake3(peer-DID)` (= the Path G post-close shape) + assert the
//!    durable CID matches.
//! 5. Cross-assert that the reconstructed frame would NOT match if
//!    `capability_grant_cid` were zero-Cid (the pre-Path-G shape) —
//!    forensic-discrimination would-FAIL-on-revert proof.
//!
//! ## Would-FAIL-on-revert (§3.6f)
//!
//! Revert `apply_atrium_merge` to use `Cid::from_blake3_digest([0u8; 32])`
//! at the `capability_grant_cid` slot → step-4 assertion fires because
//! the reconstructed CID with `peer_actor_cid` no longer matches the
//! durable AttributionFrame bytes (which would carry the zero-Cid).

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used)]

#[tokio::test]
async fn path_g_capability_grant_cid_populated_from_peer_actor_cid_at_apply_atrium_merge() {
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Configure device-CID + a distinct actor-CID — pins that the
    // capability_grant_cid slot is sourced from `peer_actor_cid` (NOT
    // from actor_cid or device_cid).
    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:laptop-pg").as_bytes());
    engine.set_device_cid(Some(device_cid));

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts-pg";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;

    peer_b
        .with_zone(zone, |doc| {
            doc.set_property(
                "title",
                "from-peer-b",
                BentenHlc::new(300, 0, peer_b.hlc_node_id()),
            )
            .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    // PRODUCTION SETUP: register peer-B's node-id under a known DID
    // so `resolve_peer_dids` returns this DID and `peer_actor_cid`
    // becomes `blake3(this-did)`.
    let peer_b_did = "did:key:zPeerBPathG";
    peer_a
        .register_peer_did(peer_b.hlc_node_id(), peer_b_did)
        .await;

    let anchor = engine.create_anchor("post:pg1").unwrap();

    // PRODUCTION ENTRY POINT: apply_atrium_merge end-to-end.
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();

    // Read back the durable AttributionFrame.
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    let frame_cid_bytes = match merged.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!("expected Bytes for attribution_frame_cid, got {other:?}"),
    };

    // EXPECTED: the post-Path-G shape — capability_grant_cid =
    // blake3(peer_b_did).
    let expected_peer_actor_cid =
        Cid::from_blake3_digest(*blake3::hash(peer_b_did.as_bytes()).as_bytes());
    let expected_frame = benten_eval::AttributionFrame {
        actor_cid: device_cid, // falls back to device_cid (no set_actor_cid)
        handler_cid: Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: expected_peer_actor_cid,
        sandbox_depth: 0,
        peer_did_set: Some(std::collections::BTreeSet::from([peer_b_did.to_string()])),
        device_did: Some(format!("device-cid:{device_cid}")),
        sync_hop_depth: 1,
    };
    let expected_cid = expected_frame.cid().unwrap();
    assert_eq!(
        frame_cid_bytes,
        expected_cid.as_bytes().to_vec(),
        "Path G LOAD-BEARING: AttributionFrame.capability_grant_cid \
         at apply_atrium_merge MUST equal blake3 hash of the resolved \
         peer-DID (= peer_actor_cid threaded through the per-row \
         admit_write_chain seam). Pre-Path-G this slot was the zero-\
         Cid sentinel even when peer_actor_cid was substantively \
         populated."
    );

    // FORENSIC-DISCRIMINATION (would-FAIL-on-revert proof): the
    // pre-Path-G shape with zero-Cid `capability_grant_cid` would
    // mint a DIFFERENT CID. Assert distinct so a revert is observably
    // caught.
    let pre_path_g_frame = benten_eval::AttributionFrame {
        actor_cid: device_cid,
        handler_cid: Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        peer_did_set: Some(std::collections::BTreeSet::from([peer_b_did.to_string()])),
        device_did: Some(format!("device-cid:{device_cid}")),
        sync_hop_depth: 1,
    };
    let pre_path_g_cid = pre_path_g_frame.cid().unwrap();
    assert_ne!(
        frame_cid_bytes,
        pre_path_g_cid.as_bytes().to_vec(),
        "Path G FORENSIC: durable AttributionFrame CID MUST differ \
         from the pre-Path-G zero-Cid `capability_grant_cid` shape; \
         this is the would-FAIL-on-revert dimension proving the slot \
         observably carries the resolved peer-actor-CID"
    );
}
