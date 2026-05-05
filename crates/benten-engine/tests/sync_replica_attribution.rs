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

#[test]
#[ignore = "RED-PHASE: G14-D — exploration-device-mesh — sync-replica attribution carries device DID"]
fn sync_replica_write_attribution_carries_device_did_alongside_parent() {
    // exploration-device-mesh pin. G14-D implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope::default();
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(), envelope).unwrap();
    //
    //   let engine = benten_engine::Engine::open_for_device(store_dir.path(),
    //       parent_kp.clone(), device_kp, attestation).unwrap();
    //
    //   // Engine writes a node:
    //   let cid = engine.write_node(&node).unwrap();
    //
    //   // Attribution frame attached to the write Node carries BOTH DIDs:
    //   let frame = engine.fetch_attribution_frame(&cid).unwrap();
    //   assert_eq!(frame.parent_did(), parent_kp.public_key().to_did());
    //   assert_eq!(frame.device_did(), Some(device_kp.public_key().to_did()));
    //
    // OBSERVABLE consequence: the attribution frame on a sync-replica
    // write observably carries device-grain identity. Defends against
    // the "compromised device cannot be isolated" failure shape.
    unimplemented!(
        "G14-D wires AttributionFrame carrying both parent_did + device_did at sync replica write"
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
