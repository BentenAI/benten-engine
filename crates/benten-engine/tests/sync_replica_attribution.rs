//! R3-B RED-PHASE pins: sync-replica device-DID attribution
//! (G14-D wave-5a; exploration-device-mesh + sec-r1-6 +
//! Inv-14 device-grain + §3.C/§3.F clusters).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-D +
//! §10 device-mesh + §3.C Inv-13 dispatch + §3.F multi-device sync):
//!
//! - `tests/sync_replica_write_attribution_carries_device_did_alongside_parent` — exploration-device-mesh
//! - `tests/inv_14_device_did_attribution_observable_in_production_runtime_arm` — sec-r1-6
//!
//! ## Architectural intent
//!
//! Per Inv-14 device-grain attribution (D-PHASE-3-25 multi-device
//! contract + exploration-device-mesh), every write that crosses an
//! atrium replica boundary MUST carry an attribution frame that
//! names BOTH the parent DID (the logical identity) AND the device
//! DID (which device produced the write). Without device-grain
//! attribution, a compromised device cannot be quarantined surgically
//! from the rest of the user's mesh.
//!
//! Per sec-r1-6 the attribution must be OBSERVABLE in the production
//! runtime arm — not just emitted in a test fixture. This is the
//! load-bearing pim-2 sentinel-presence-vs-end-to-end pin.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the runtime-arm test must
//! drive a real engine write through the production path + observe
//! the device-DID in the resulting attribution frame.

#![allow(clippy::unwrap_used)]

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn sync_replica_write_attribution_carries_device_did_alongside_parent() {
    // exploration-device-mesh GREEN-PHASE pin (G16-B-prime closure).
    //
    // OBSERVABLE consequence: the AttributionFrame minted at a sync-
    // replica merge boundary carries BOTH the parent (actor / peer)
    // DID identity and the device-grain identity per Inv-14 + the
    // exploration-device-mesh contract. Defends against the
    // "compromised device cannot be isolated" failure shape.
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Configure the engine's device-CID — the post-merge AttributionFrame
    // carries this slot (encoded as `device-cid:<hex>` per the
    // engine's internal device-DID convention pre-G16-D handshake
    // protocol body).
    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:laptop").as_bytes());
    engine.set_device_cid(Some(device_cid));

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property("title", "x", BentenHlc::new(200, 0, peer_b.hlc_node_id()))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a
        .register_peer_did(peer_b.hlc_node_id(), "did:key:peer-b-test")
        .await;

    let anchor = engine.create_anchor("post:p1").unwrap();
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();

    // Read back the merged Version Node + the attribution_frame_cid
    // slot. The slot's value is the CID of an AttributionFrame whose
    // `device_did` is `Some("device-cid:<hex>")` (parent identity =
    // merging engine; device identity = engine's set_device_cid).
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    let frame_cid_bytes = match merged.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!("expected Bytes for attribution_frame_cid, got {other:?}"),
    };
    // The frame CID MUST differ from a pure-default (no device_did,
    // no peer_did_set) frame, proving device_did + peer_did_set
    // landed in the canonical bytes.
    let default_cid = benten_eval::AttributionFrame::default().cid().unwrap();
    assert_ne!(
        frame_cid_bytes,
        default_cid.as_bytes().to_vec(),
        "AttributionFrame on sync-replica merge MUST be content-distinguishable \
         from a default frame (carries device_did + peer_did_set)"
    );
    // The device-DID slot is populated; reconstruct the frame the
    // engine would have minted post-G16-B-prime apply_atrium_merge.
    // Per the engine's contract: actor_cid = local engine's device_cid
    // (or all-zero if unset), handler_cid + capability_grant_cid all-zero
    // pre-handler-attribution-flow landing, sandbox_depth=0, and the
    // sync slots carry the merge seed.
    let frame_with_device = benten_eval::AttributionFrame {
        actor_cid: device_cid,
        handler_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        device_did: Some(format!("device-cid:{device_cid}")),
        peer_did_set: Some(std::collections::BTreeSet::from([
            "did:key:peer-b-test".to_string()
        ])),
        sync_hop_depth: 1,
    };
    let expected = frame_with_device.cid().unwrap();
    assert_eq!(
        frame_cid_bytes,
        expected.as_bytes().to_vec(),
        "frame CID matches the expected (device_did + peer_did_set + hop_depth=1) shape"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — sec-r1-6 — Inv-14 device-DID attribution observable in production runtime"]
fn inv_14_device_did_attribution_observable_in_production_runtime_arm() {
    // sec-r1-6 pin. The device-grain attribution MUST be observable
    // in the PRODUCTION runtime arm, not just in test fixtures or
    // dev-only feature flags. Per §3.6b pim-2, this pin closes the
    // sentinel-presence concern: a #[cfg(test)] gate that emits the
    // device DID does not satisfy this assertion.
    //
    // Implementer wires:
    //
    //   // 1. Build engine WITHOUT test-only feature flags:
    //   let engine = benten_engine::Engine::open_production(store_dir.path(),
    //       parent_kp, device_kp, attestation).unwrap();
    //
    //   // 2. Drive a write through the production path:
    //   let cid = engine.write_node(&node).unwrap();
    //
    //   // 3. Re-open the durable store WITHOUT the writing engine's
    //   //    process scope (proves attribution is in durable bytes,
    //   //    not in-memory only):
    //   drop(engine);
    //   let inspector = benten_engine::DurableStoreInspector::open(store_dir.path()).unwrap();
    //   let frame_bytes = inspector.fetch_attribution_frame_bytes(&cid).unwrap();
    //   let frame = benten_engine::AttributionFrame::from_canonical_bytes(&frame_bytes).unwrap();
    //   assert_eq!(frame.device_did(), Some(device_kp.public_key().to_did()),
    //       "device DID MUST be observable in durable bytes per Inv-14 + sec-r1-6");
    //
    //   // 4. Source-cite check that the production codepath actually
    //   //    threads the device DID (not just the frame containing
    //   //    a None placeholder):
    //   let src = std::fs::read_to_string("crates/benten-engine/src/runtime/write_path.rs").unwrap();
    //   assert!(src.contains("device_did:") || src.contains("device_did ="),
    //       "production write_path.rs must reference device_did at the frame-construction site");
    //
    // OBSERVABLE consequence: Inv-14's device-grain enforcement is
    // present in production-runtime-arm bytes — not gated behind a
    // test-only feature. Closes the pim-2 end-to-end pin requirement.
    unimplemented!(
        "G14-D wires Inv-14 device-DID attribution observable in production-runtime durable bytes"
    );
}

// =====================================================================
// R4-FP-R3-B RED-PHASE pins: sec-r4r1-2 BLOCKER closure — sync-replica
// per-write cap-recheck-at-delivery (mirrors SUBSCRIBE side per CLR-2;
// uses cap_recheck.rs G13-pre-C scaffold).
//
// Pin sources (per R4 R1 security-auditor lens, finding sec-r4r1-2):
//
// - `sync_replica_write_cap_recheck_at_delivery_against_local_grant_store`
// - `sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session`
//
// ## Architectural intent (sec-r4r1-2 BLOCKER closure)
//
// sec-r1-2 had two halves: (a) E_SYNC_DIVERGENT_CID_REJECTED minted
// (well-pinned at inv_13_dispatch.rs); (b) per-write cap-recheck-at-
// delivery for SyncReplica writes mirroring the SUBSCRIBE side.
//
// Half (b) is the defense-in-depth that catches mid-session grant
// revocation when the next handshake hasn't occurred yet. The
// SUBSCRIBE side has 3 cap-recheck pins; the symmetric WRITE side at
// the receiving peer's delivery point had ZERO. The cap_recheck.rs
// (G13-pre-C) helper landed precisely so SUBSCRIBE + sync-replica-WRITE
// could share infrastructure; only one consumer was shipped at R3.
// These two pins close the symmetric WRITE side end-to-end.
// =====================================================================

#[test]
#[ignore = "RED-PHASE: G14-D — sec-r4r1-2 BLOCKER — sync-replica WRITE per-write cap-recheck at delivery"]
fn sync_replica_write_cap_recheck_at_delivery_against_local_grant_store() {
    // sec-r4r1-2 BLOCKER pin (a). The receiving peer's sync-replica
    // delivery point MUST per-write cap-recheck against the local
    // grant store via the cap_recheck.rs G13-pre-C helper. This is
    // the structural mirror of the SUBSCRIBE delivery-time recheck.
    //
    // Concrete shape:
    //   let alice_engine = benten_engine::Engine::open(alice_store.path()).unwrap();
    //   let bob_engine = benten_engine::Engine::open(bob_store.path()).unwrap();
    //
    //   let alice_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Bob installs a grant authorizing Alice to write /zone/posts:
    //   let grant_to_alice = ... .audience(alice_kp.public_key().to_did())
    //                            .capability("/zone/posts", "write") ... ;
    //   bob_engine.caps().install_proof(&grant_to_alice).unwrap();
    //
    //   // Atrium sync replica: Alice → Bob. Handshake establishes
    //   // per-peer cap-set ONCE. Now Alice writes:
    //   alice_engine.write_node(&node_in_zone_posts).unwrap();
    //
    //   // Bob's sync-replica receive path observably calls cap_recheck:
    //   let cap_recheck_calls_before = bob_engine.metrics().sync_replica_cap_recheck_calls();
    //   bob_engine.consume_sync_replica_message(&alice_outbound_message).unwrap();
    //   let cap_recheck_calls_after = bob_engine.metrics().sync_replica_cap_recheck_calls();
    //
    //   assert!(cap_recheck_calls_after > cap_recheck_calls_before,
    //       "sync-replica WRITE delivery MUST per-write cap-recheck via cap_recheck.rs helper per sec-r4r1-2");
    //
    //   // The write applied (Alice has cap):
    //   assert!(bob_engine.read_zone("/zone/posts").unwrap()
    //       .iter().any(|n| n.cid() == node_in_zone_posts.cid()));
    //
    //   // Source-cite that the cap_recheck.rs scaffold is consumed at
    //   // the sync-replica delivery point:
    //   let src = std::fs::read_to_string("crates/benten-engine/src/sync_replica.rs").unwrap();
    //   assert!(src.contains("cap_recheck") || src.contains("CapRecheck"),
    //       "sync_replica.rs must consume cap_recheck.rs helper per sec-r4r1-2");
    //
    // OBSERVABLE consequence: cross-trust-boundary WRITE delivery
    // observably fires per-write cap-recheck at the receiving peer.
    // Defends against the asymmetry where SUBSCRIBE has defense-in-
    // depth but WRITE relied solely on handshake-time cap establishment.
    unimplemented!(
        "G14-D wires sync-replica WRITE per-write cap-recheck via cap_recheck.rs helper per sec-r4r1-2"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — sec-r4r1-2 BLOCKER — mid-session revoke after handshake rejects with typed error"]
fn sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session()
 {
    // sec-r4r1-2 BLOCKER pin (b). Mid-session revocation between
    // handshake and next sync round: the receiving peer's per-write
    // cap-recheck observably suppresses delivery + emits a typed
    // error variant.
    //
    // Concrete shape:
    //   let alice_engine = benten_engine::Engine::open(alice_store.path()).unwrap();
    //   let bob_engine = benten_engine::Engine::open(bob_store.path()).unwrap();
    //
    //   let alice_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Bob installs grant + handshake establishes per-peer cap-set:
    //   let grant_to_alice = ... .audience(alice_kp.public_key().to_did())
    //                            .capability("/zone/posts", "write") ... ;
    //   bob_engine.caps().install_proof(&grant_to_alice).unwrap();
    //   bob_engine.atrium_handshake_with(&alice_kp.public_key().to_did()).unwrap();
    //
    //   // Mid-session: Bob revokes Alice's grant LOCALLY (no handshake
    //   // re-negotiation yet — the revocation hasn't propagated):
    //   bob_engine.caps().revoke(&grant_to_alice.cid()).unwrap();
    //
    //   // Alice writes (still believes she has the grant per her cached set):
    //   alice_engine.write_node(&node_in_zone_posts).unwrap();
    //
    //   // Bob's sync-replica receive path: per-write cap-recheck fires;
    //   // observes revocation; rejects with typed error variant:
    //   let result = bob_engine.consume_sync_replica_message(&alice_outbound_message);
    //   let err = result.unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::SyncRevokedDuringSession { .. }),
    //       "mid-session revoke must reject sync-replica WRITE with typed E_SYNC_REVOKED_DURING_SESSION per sec-r4r1-2");
    //
    //   // The write did NOT apply at Bob's side:
    //   assert!(!bob_engine.read_zone("/zone/posts").unwrap()
    //       .iter().any(|n| n.cid() == node_in_zone_posts.cid()),
    //       "write from revoked-mid-session peer must not apply per sec-r4r1-2");
    //
    // OBSERVABLE consequence: a peer whose grant was revoked between
    // handshake and the next sync round CANNOT keep writing — the
    // per-write cap-recheck at delivery catches the revocation. Defense-
    // in-depth at the cross-trust-boundary WRITE surface; structurally
    // analogous to the SUBSCRIBE delivery-time recheck.
    unimplemented!(
        "G14-D wires E_SYNC_REVOKED_DURING_SESSION typed-error rejection at sync-replica WRITE delivery per sec-r4r1-2"
    );
}

// =====================================================================
// R4-R2-FP-C RED-PHASE pin: ds-r4r2-2 AttributionFrame sync_hop_depth
// bound + E_SYNC_HOP_DEPTH_EXCEEDED typed variant.
//
// Pin source (per .addl/phase-3/r4-r2-distributed-systems.json
// ds-r4r2-2; closes ds-r4-5 per R4-R2 re-emergent finding):
//
// - sync_replica_attribution_frame_sync_hop_depth_bounded_with_e_sync_hop_depth_exceeded
//
// ## Architectural intent (mirrors Inv-4 sandbox_depth precedent)
//
// Under transitive Atrium sync (peer A writes → syncs to B → B's
// SUBSCRIBE re-emits / cascades to peer C), the AttributionFrame chain
// depth grows unboundedly. Inv-4 enforces SANDBOX nest-depth via
// AttributionFrame.sandbox_depth: u8 (cap = 8). The analogous
// sync_hop_depth: u8 field bounds Atrium peer-hop chain depth, defending
// against DOS / chain-bloat attacks where an adversarial peer
// constructs a long false chain. Default bound = 8 hops per ds-r4-5
// RECOMMEND, matching the Inv-4 sandbox_depth precedent.
//
// Cascades canonical-bytes of AttributionFrame (per ds-8 / D-PHASE-3-19a;
// in-scope for the existing 10+ pinned-CID test rewrite cohort at
// G14-D wave-5a). The sync_hop_depth field becomes part of the
// canonical bytes; pinned-CID fixtures rebake at that wave.
// =====================================================================

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn sync_replica_attribution_frame_sync_hop_depth_bounded_with_e_sync_hop_depth_exceeded() {
    // ds-r4r2-2 GREEN-PHASE pin (G16-B-prime closure).
    //
    // OBSERVABLE consequence: AttributionFrame.sync_hop_depth bounds
    // peer-hop chain depth at 8 (mirroring Inv-4 sandbox_depth);
    // overflow rejects with typed E_SYNC_HOP_DEPTH_EXCEEDED through
    // the engine's apply_atrium_merge orchestration; bounded depth
    // is observable in the minted Version Node's AttributionFrame.
    //
    // The G16-B canary established the bound at the Atrium handle
    // layer (`merge_remote_change_with_hop_depth`); this pin closes
    // the engine-orchestrated path observably.
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property("title", "x", BentenHlc::new(200, 0, peer_b.hlc_node_id()))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let anchor = engine.create_anchor("post:hop1").unwrap();

    // (i) hop_depth advances on incoming hop_depth = 0 → out depth 1.
    let merged_cid_1 = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();
    let merged_1 = engine.get_node(&merged_cid_1).unwrap().unwrap();
    match merged_1.properties.get("sync_hop_depth") {
        Some(benten_core::Value::Int(d)) => assert_eq!(*d, 1, "hop=0 → out=1"),
        other => panic!("expected Int sync_hop_depth, got {other:?}"),
    }

    // (ii) hop_depth = SYNC_HOP_DEPTH_CAP rejects with the typed
    // error code at the orchestrator boundary.
    let result = engine
        .apply_atrium_merge(
            &peer_a,
            &anchor,
            zone,
            &bytes,
            benten_eval::exec_state::SYNC_HOP_DEPTH_CAP,
        )
        .await;
    let err = result.expect_err("hop-depth at-or-above cap MUST reject");
    assert!(
        matches!(err.code(), benten_errors::ErrorCode::SyncHopDepthExceeded),
        "rejection MUST surface E_SYNC_HOP_DEPTH_EXCEEDED, got: {err:?}"
    );

    // (iii) bounded depth observable via the constant.
    assert_eq!(
        benten_eval::exec_state::SYNC_HOP_DEPTH_CAP,
        8,
        "documented bound is 8 hops (mirrors Inv-4 sandbox_depth)"
    );
}

// =====================================================================
// LEGACY pin commentary (preserved as documentation; the GREEN-PHASE
// pin above replaces this — the prior unimplemented!() body cited
// `Engine::open_for_device` / `DurableStoreInspector` / per-peer
// fetch_attribution_frame APIs that don't exist at engine scope.
// G16-B-prime's apply_atrium_merge orchestration closes ds-r4r2-2 via
// observable AttributionFrame slots on the minted Version Node).
// =====================================================================

#[allow(dead_code)]
#[doc(hidden)]
fn _sync_replica_attribution_frame_sync_hop_depth_legacy_design_notes() {
    // ds-r4r2-2 historical pin shape — preserved for review traceability.
    // G14-D / G16-B implementer notes:
    //
    //   use benten_engine::{AttributionFrame, EngineError, ErrorCode};
    //
    //   const SYNC_HOP_DEPTH_MAX: u8 = 8;  // mirrors Inv-4 sandbox_depth cap
    //
    //   // Build a chain of 4 peers; each Atrium-sync hop increments
    //   // sync_hop_depth by 1 on the AttributionFrame's per-hop
    //   // counter.
    //   let peer_a = benten_engine::Engine::open(a_store.path()).unwrap();
    //   let peer_b = benten_engine::Engine::open(b_store.path()).unwrap();
    //   let peer_c = benten_engine::Engine::open(c_store.path()).unwrap();
    //   let peer_d = benten_engine::Engine::open(d_store.path()).unwrap();
    //
    //   // Peer A writes; peer B receives via sync-replica (hop 1):
    //   let cid = peer_a.write_node(&node).unwrap();
    //   peer_a.atrium_sync_to(peer_b.local_did()).unwrap();
    //   let frame_at_b = peer_b.fetch_attribution_frame(&cid).unwrap();
    //   assert_eq!(frame_at_b.sync_hop_depth(), 1u8,
    //       "first hop A→B must report sync_hop_depth = 1");
    //
    //   // Peer B's SUBSCRIBE re-emits to peer C (hop 2):
    //   peer_b.atrium_sync_to(peer_c.local_did()).unwrap();
    //   let frame_at_c = peer_c.fetch_attribution_frame(&cid).unwrap();
    //   assert_eq!(frame_at_c.sync_hop_depth(), 2u8,
    //       "two-hop chain A→B→C must report sync_hop_depth = 2");
    //
    //   // Peer C re-emits to peer D (hop 3):
    //   peer_c.atrium_sync_to(peer_d.local_did()).unwrap();
    //   let frame_at_d = peer_d.fetch_attribution_frame(&cid).unwrap();
    //   assert_eq!(frame_at_d.sync_hop_depth(), 3u8,
    //       "three-hop chain A→B→C→D must report sync_hop_depth = 3");
    //
    //   // OBSERVABLE consequence (i): sync_hop_depth advances
    //   // monotonically per peer hop.
    //
    //   // Now construct an adversarial chain at SYNC_HOP_DEPTH_MAX:
    //   // an inbound message claims sync_hop_depth = 8 (already at
    //   // the cap). The next hop would push depth to 9 — must reject
    //   // with the typed error variant + stable error code.
    //   let adversarial_msg = build_inbound_sync_message_with_hop_depth(
    //       SYNC_HOP_DEPTH_MAX);
    //   let result = peer_d.consume_sync_replica_message(&adversarial_msg);
    //   let err = result.unwrap_err();
    //   assert!(matches!(err,
    //       EngineError::SyncHopDepthExceeded { observed_depth, max_depth }
    //           if observed_depth == SYNC_HOP_DEPTH_MAX
    //              && max_depth == SYNC_HOP_DEPTH_MAX),
    //       "hop-depth overflow must reject with typed SyncHopDepthExceeded \
    //        carrying observed/max depth diagnostic state per ds-r4r2-2");
    //   assert_eq!(err.error_code(), ErrorCode::SyncHopDepthExceeded,
    //       "typed error must map to stable code E_SYNC_HOP_DEPTH_EXCEEDED");
    //
    //   // OBSERVABLE consequence (ii): hop-depth overflow rejects with
    //   // typed E_SYNC_HOP_DEPTH_EXCEEDED before the frame is persisted.
    //
    //   // OBSERVABLE consequence (iii): bounded depth is observable via
    //   // the AttributionFrame query API:
    //   let bounded = AttributionFrame::sync_hop_depth_max();
    //   assert_eq!(bounded, SYNC_HOP_DEPTH_MAX,
    //       "AttributionFrame::sync_hop_depth_max() must report the documented bound");
    //
    //   // OBSERVABLE consequence (iv): canonical bytes of
    //   // AttributionFrame include sync_hop_depth — pinned-CID fixtures
    //   // rebake at G14-D wave-5a per D-PHASE-3-19a 10+-rebake cohort.
    //   let frame_bytes = frame_at_b.to_canonical_bytes().unwrap();
    //   let round_trip = AttributionFrame::from_canonical_bytes(&frame_bytes).unwrap();
    //   assert_eq!(round_trip.sync_hop_depth(), 1u8,
    //       "sync_hop_depth must round-trip through canonical-bytes encoding");
    //
    // OBSERVABLE consequence: AttributionFrame.sync_hop_depth bounds
    // peer-hop chain depth at 8 (mirroring Inv-4 sandbox_depth);
    // overflow rejects with typed E_SYNC_HOP_DEPTH_EXCEEDED before
    // persistence; bounded depth is queryable + included in canonical
    // bytes. Defends against the DOS / chain-bloat attack class where
    // an adversarial peer constructs a long false chain. Composes
    // G14-D AttributionFrame surface + G16-B sync-replica delivery +
    // canonical-bytes-pinning per D-PHASE-3-19a.
    // (Body intentionally empty — preserved for design traceability.)
}
