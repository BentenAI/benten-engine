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
    // Per the engine's contract POST G16-B-prime fp (cap-g16bp-1 /
    // Ben's RATIFIED Option A 2026-05-08):
    //   - actor_cid = `effective_actor_cid()` which falls back to
    //     device_cid when set_actor_cid has not been called (single-
    //     user single-device case, exercised here).
    //   - handler_cid + capability_grant_cid all-zero pre-handler-
    //     attribution-flow landing.
    //   - sandbox_depth=0; sync slots carry the merge seed.
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

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn sync_replica_explicit_actor_cid_decouples_from_device_cid() {
    // Phase-3 G16-B-prime fp closure (cap-g16bp-1 / Ben's RATIFIED
    // Option A 2026-05-08):
    //   When `set_actor_cid(Some(actor))` is called, the
    //   AttributionFrame.actor_cid minted by apply_atrium_merge MUST
    //   carry the EXPLICIT actor identity, NOT the device identity.
    //   Defends against the failure shape where actor_cid + device_did
    //   are conflated at sync-merge boundaries (would lose Phase-4+
    //   AI-agent / handler-attribution principal identity).
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Distinct actor and device identities — the load-bearing input.
    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:laptop").as_bytes());
    let actor_cid = Cid::from_blake3_digest(*blake3::hash(b"actor:user-parent").as_bytes());
    assert_ne!(
        device_cid, actor_cid,
        "test inputs must differ to assert decoupling"
    );
    engine.set_device_cid(Some(device_cid));
    engine.set_actor_cid(Some(actor_cid));

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

    let anchor = engine.create_anchor("post:p2").unwrap();
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();

    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    let frame_cid_bytes = match merged.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!("expected Bytes for attribution_frame_cid, got {other:?}"),
    };

    // The expected frame uses the EXPLICIT actor_cid + device-cid-string
    // device_did. If the engine wrongly conflated the two we'd get
    // a CID matching the device-as-actor frame instead.
    let expected_correct = benten_eval::AttributionFrame {
        actor_cid,
        handler_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        device_did: Some(format!("device-cid:{device_cid}")),
        peer_did_set: Some(std::collections::BTreeSet::from([
            "did:key:peer-b-test".to_string()
        ])),
        sync_hop_depth: 1,
    }
    .cid()
    .unwrap();
    let conflated_wrong = benten_eval::AttributionFrame {
        actor_cid: device_cid,
        handler_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        device_did: Some(format!("device-cid:{device_cid}")),
        peer_did_set: Some(std::collections::BTreeSet::from([
            "did:key:peer-b-test".to_string()
        ])),
        sync_hop_depth: 1,
    }
    .cid()
    .unwrap();

    assert_eq!(
        frame_cid_bytes,
        expected_correct.as_bytes().to_vec(),
        "AttributionFrame.actor_cid MUST carry the explicitly-set \
         actor identity per cap-g16bp-1 Option A. NOT the device CID."
    );
    assert_ne!(
        frame_cid_bytes,
        conflated_wrong.as_bytes().to_vec(),
        "regression-guard: the conflated (actor_cid == device_cid) \
         frame would be wrong — assertion would fail if the fallback \
         path ignored set_actor_cid()."
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn inv_14_device_did_attribution_observable_in_production_runtime_arm() {
    // sec-r1-6 GREEN-PHASE pin (G16-B-D closure repurposing per the
    // brief). Demonstrates Inv-14's device-grain attribution is
    // observable in PRODUCTION-runtime durable bytes — not gated
    // behind a test-only feature.
    //
    // Pre-G16-B-D the pin assumed a `DurableStoreInspector` API + a
    // `Engine::open_production` constructor that don't exist. The
    // brief redirects to using existing GREEN sibling pin pattern:
    // durable readback via `engine.get_node(...).properties.get(
    // "attribution_frame_cid")` after dropping + re-opening the
    // engine, which proves the attribution lives in durable bytes
    // (not in-memory only).
    //
    // OBSERVABLE consequence (production-arm-without-test-feature):
    //   1. Open engine; set device_cid; drive sync-replica merge
    //      through the production `apply_atrium_merge` path.
    //   2. Drop engine — store contents persisted to redb.
    //   3. Re-open the SAME store_dir in a FRESH Engine (different
    //      process scope; no in-memory carry-over).
    //   4. `get_node(merged_cid)` returns the merged Version Node
    //      with `attribution_frame_cid` slot populated by canonical
    //      bytes whose AttributionFrame contains `device_did =
    //      Some("device-cid:<hex>")` for the originating device.
    //
    // Per §3.6b pim-2: the assertion is would-fail-if-no-op'd
    // because if the production write path stamped only the actor
    // (not device_did), the post-reopen frame CID would equal a
    // device-less-frame CID, which the test asserts does NOT match.
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("benten.redb");

    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:production").as_bytes());

    // Phase 1: open engine, configure device_cid, drive merge.
    let merged_cid = {
        let engine = Engine::open(&store_path).unwrap();
        engine.set_device_cid(Some(device_cid));

        let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
        let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
        let zone = "/zone/posts";
        peer_a.register_zone(zone).await;
        peer_b.register_zone(zone).await;
        peer_b
            .with_zone(zone, |doc| {
                doc.set_property(
                    "title",
                    "production-x",
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
        peer_a
            .register_peer_did(peer_b.hlc_node_id(), "did:key:peer-prod")
            .await;

        let anchor = engine.create_anchor("post:p1").unwrap();
        let cid = engine
            .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
            .await
            .unwrap();

        // Drop everything to flush to the durable store.
        drop(engine);
        cid
    };

    // Phase 2: re-open the SAME store in a FRESH Engine (proves
    // attribution lives in durable bytes, not process-scoped memory).
    let reopened = Engine::open(&store_path).unwrap();
    let merged = reopened
        .get_node(&merged_cid)
        .unwrap()
        .expect("merged Version Node MUST be queryable after engine re-open");

    let frame_cid_bytes = match merged.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!(
            "post-reopen merged Node MUST carry attribution_frame_cid in durable \
             bytes per Inv-14 + sec-r1-6; got {other:?}"
        ),
    };

    // Reconstruct the frame the production-arm path is contracted to
    // mint (per cap-g16bp-1 RATIFIED Option A) and assert match.
    let expected_frame = benten_eval::AttributionFrame {
        actor_cid: device_cid,
        handler_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        device_did: Some(format!("device-cid:{device_cid}")),
        peer_did_set: Some(std::collections::BTreeSet::from([
            "did:key:peer-prod".to_string()
        ])),
        sync_hop_depth: 1,
    };
    let expected_cid = expected_frame.cid().unwrap();
    assert_eq!(
        frame_cid_bytes,
        expected_cid.as_bytes().to_vec(),
        "post-reopen frame CID MUST match the (device_did + peer_did_set + hop_depth=1) \
         shape, proving Inv-14 device-grain attribution lives in PRODUCTION-arm durable \
         bytes (not test-only / in-memory only)"
    );

    // Defense-in-depth: a frame that omits device_did would yield a
    // distinct CID. Assert it doesn't accidentally collide.
    let device_less_frame = benten_eval::AttributionFrame {
        actor_cid: device_cid,
        handler_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        device_did: None, // <- the would-fail-if-no-op'd dimension
        peer_did_set: Some(std::collections::BTreeSet::from([
            "did:key:peer-prod".to_string()
        ])),
        sync_hop_depth: 1,
    };
    let device_less_cid = device_less_frame.cid().unwrap();
    assert_ne!(
        frame_cid_bytes,
        device_less_cid.as_bytes().to_vec(),
        "would-fail-if-no-op'd: if production path failed to stamp device_did, \
         the post-reopen frame CID would equal the device-less variant — \
         this assertion catches that regression"
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

// sec-r4r1-2 BLOCKER closure (Phase-3 G16-B-F): both pins below assert
// the structural-always-on per-row cap-recheck inside
// `Engine::apply_atrium_merge` per Ben's RATIFIED Option (a)
// 2026-05-09 — defense-in-depth mirror of the SUBSCRIBE-side CLR-2
// dual-layer recheck. Pin (a) asserts the metric counter increments
// per-row on a clean merge; pin (b) asserts a mid-session revocation
// observably surfaces the typed `EngineError::SyncRevokedDuringSession`
// shape with the merge rejected end-to-end.
//
// The "concrete shape" pseudo-code in the prior RED-PHASE bodies named
// a `consume_sync_replica_message` API + a `bob_engine.caps()` accessor
// + a `metrics().sync_replica_cap_recheck_calls()` surface — the
// structural API that landed at G16-B-F is:
//   - `Engine::apply_atrium_merge` (production; matches the existing
//     G16-B-prime test fixture pattern at line 36)
//   - `Engine::caps()` returning `EngineCapsHandle` with
//     `install_proof(&mut CapProof)` / `revoke(&CapProof)`
//   - `Engine::sync_replica_cap_recheck_calls()` direct accessor on
//     the engine (no intermediate `metrics()` handle)
// Same observable consequences; landed surface uses real production
// API names.

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn sync_replica_write_cap_recheck_at_delivery_against_local_grant_store() {
    // sec-r4r1-2 BLOCKER pin (a). The receiving peer's sync-replica
    // delivery point MUST per-write cap-recheck against the local
    // grant store. Asserts the metric counter increments per row of
    // the merge so the recheck observably fires.
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let bob_engine = Engine::open(dir.path().join("bob.redb")).unwrap();

    // Alice's actor identity (the originating peer of the merge).
    // Pre-G14-B durable identity backend, the apply_atrium_merge
    // recheck path derives actor_cid from blake3-of-utf8(peer_did);
    // the test installs a grant under the SAME derived shape so the
    // in-memory revocation pair set keys agree. Alice's peer-DID is
    // the canonical resolved-by-trust-store form.
    let alice_did = "did:key:peer-alice-test";
    let alice_actor_cid =
        Cid::from_blake3_digest(*blake3::hash(alice_did.as_bytes()).as_bytes());

    // Bob installs a grant authorizing Alice to write /zone/posts.
    // The scope shape `<zone>:write` matches what the per-row
    // cap-recheck consults (`format!("{zone}:write")`).
    let zone = "/zone/posts";
    let mut grant = benten_engine::CapProof::new(alice_actor_cid, format!("{zone}:write"));
    bob_engine.caps().install_proof(&mut grant).unwrap();
    assert!(grant.proof_cid.is_some(), "install_proof should mint grant CID");

    // Open Bob's atrium + register the trust-store mapping for Alice.
    // Build an Alice-side Loro doc that produces a writes-set the
    // merge will consume.
    let bob_atrium = bob_engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let alice_atrium = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    bob_atrium.register_zone(zone).await;
    alice_atrium.register_zone(zone).await;
    alice_atrium
        .with_zone(zone, |doc| {
            doc.set_property("title", "alice-post", BentenHlc::new(100, 0, alice_atrium.hlc_node_id()))
                .unwrap();
        })
        .await
        .unwrap();
    let alice_bytes = alice_atrium
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    bob_atrium
        .register_peer_did(alice_atrium.hlc_node_id(), alice_did)
        .await;

    let cap_recheck_calls_before = bob_engine.sync_replica_cap_recheck_calls();
    let anchor = bob_engine.create_anchor("alice:p1").unwrap();
    bob_engine
        .apply_atrium_merge(&bob_atrium, &anchor, zone, &alice_bytes, 0)
        .await
        .expect("apply_atrium_merge should succeed for live grant");
    let cap_recheck_calls_after = bob_engine.sync_replica_cap_recheck_calls();

    // OBSERVABLE: per-row cap-recheck observably fired at the
    // receiving peer; metric counter advanced by ≥1 (the merge
    // produced ≥1 row).
    assert!(
        cap_recheck_calls_after > cap_recheck_calls_before,
        "sync-replica WRITE delivery MUST per-write cap-recheck per sec-r4r1-2 \
         (before={cap_recheck_calls_before}, after={cap_recheck_calls_after})"
    );

    // Source-cite the cap_recheck.rs G13-pre-C helper module exists
    // and the engine.rs apply_atrium_merge consumes it via the
    // CapabilityPolicy::check_write hook composition. Pin shifts
    // from the pseudo-code's `sync_replica.rs` reference to the
    // actual consumer site.
    let src = std::fs::read_to_string("src/engine.rs").unwrap();
    assert!(
        src.contains("sec-r4r1-2") && src.contains("sync_replica_cap_recheck_count"),
        "engine.rs apply_atrium_merge must wire sec-r4r1-2 cap-recheck per sec-r4r1-2"
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#[tokio::test]
async fn sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session()
 {
    // sec-r4r1-2 BLOCKER pin (b). Mid-session revocation between
    // handshake and next sync round: the per-row cap-recheck inside
    // apply_atrium_merge observably suppresses delivery + emits the
    // typed `EngineError::SyncRevokedDuringSession` variant.
    use benten_core::Cid;
    use benten_core::hlc::BentenHlc;
    use benten_engine::Engine;
    use benten_engine::EngineError;
    use benten_engine::atrium_api::AtriumConfig;
    use benten_engine::engine_sync::AtriumHandle;

    let dir = tempfile::tempdir().unwrap();
    let bob_engine = Engine::open(dir.path().join("bob.redb")).unwrap();

    let alice_did = "did:key:peer-alice-test";
    let alice_actor_cid =
        Cid::from_blake3_digest(*blake3::hash(alice_did.as_bytes()).as_bytes());

    let zone = "/zone/posts";
    let mut grant = benten_engine::CapProof::new(alice_actor_cid, format!("{zone}:write"));
    bob_engine.caps().install_proof(&mut grant).unwrap();

    let bob_atrium = bob_engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let alice_atrium = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    bob_atrium.register_zone(zone).await;
    alice_atrium.register_zone(zone).await;
    alice_atrium
        .with_zone(zone, |doc| {
            doc.set_property("title", "alice-post-2", BentenHlc::new(200, 0, alice_atrium.hlc_node_id()))
                .unwrap();
        })
        .await
        .unwrap();
    let alice_bytes = alice_atrium
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    bob_atrium
        .register_peer_did(alice_atrium.hlc_node_id(), alice_did)
        .await;

    // Mid-session: Bob revokes Alice's grant LOCALLY before applying
    // the inbound merge — simulates the revocation having landed at
    // Bob's local cap store but not yet propagated through the
    // handshake protocol back to Alice.
    bob_engine.caps().revoke(&grant).unwrap();

    let anchor = bob_engine.create_anchor("alice:p2").unwrap();
    let pre_merge_current = bob_engine
        .read_current_version(&anchor)
        .unwrap()
        .expect("anchor seeds CURRENT at create_anchor time");
    let result = bob_engine
        .apply_atrium_merge(&bob_atrium, &anchor, zone, &alice_bytes, 0)
        .await;
    let err = result.expect_err("revoked-mid-session merge MUST reject");
    assert!(
        matches!(
            err,
            EngineError::SyncRevokedDuringSession { .. }
        ),
        "mid-session revoke must reject with typed E_SYNC_REVOKED_DURING_SESSION per sec-r4r1-2; got {err:?}"
    );

    // OBSERVABLE: the merge did NOT apply at Bob's side — the
    // anchor's CURRENT pointer still points at the pre-merge seed
    // (no Version Node was minted because the per-row recheck fired
    // BEFORE step 4 of apply_atrium_merge built + persisted the
    // merge Version Node).
    let post_merge_current = bob_engine
        .read_current_version(&anchor)
        .unwrap()
        .expect("anchor still exists post-rejected merge");
    assert_eq!(
        post_merge_current, pre_merge_current,
        "write from revoked-mid-session peer must not advance CURRENT per sec-r4r1-2 \
         (CURRENT advanced from {pre_merge_current:?} to {post_merge_current:?})"
    );

    // Catalog code routing pin: the typed variant maps to the stable
    // `E_SYNC_REVOKED_DURING_SESSION` catalog code (ON_DENIED edge).
    assert_eq!(
        err.code().as_static_str(),
        "E_SYNC_REVOKED_DURING_SESSION",
        "EngineError::SyncRevokedDuringSession must map to the stable catalog code"
    );
    assert_eq!(
        err.routed_edge_label(),
        Some("ON_DENIED"),
        "SyncRevokedDuringSession must route via ON_DENIED per CLR-2 cap-denial family"
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
