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
