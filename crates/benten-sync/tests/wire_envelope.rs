//! R4-FP/R3-C RED-PHASE pin: Atrium wire-envelope device-DID coverage
//! across ALL message kinds (not just handshake).
//!
//! ## Pin sources
//!
//! - `net-r4-r1-2` (R4 large-council Round 1 networking lens MAJOR —
//!   handshake-frame device-DID is pinned via
//!   `atrium_handshake_wire_format_carries_peer_did_and_device_did` in
//!   `atrium_errors.rs`, but non-handshake message kinds (data
//!   sync chunks, MST diff frames, Loro updates) lack explicit
//!   device-DID coverage at the wire level).
//! - `net-blocker-4` (peer-handshake metadata carries peer-DID AND
//!   device-DID — broader inv-14 device-grain wire-format
//!   commitment).
//! - Inv-14 (device-grain attribution; per CLAUDE.md baked-in #14 +
//!   plan §3 G14-D).
//!
//! ## Why this is distinct from handshake-frame coverage
//!
//! `atrium_handshake_wire_format_carries_peer_did_and_device_did`
//! pins the HANDSHAKE-FRAME wire shape. A G16-A implementer can
//! pass that pin while sending data envelopes carrying ONLY
//! peer-DID (relying on handshake-time binding at receiver to
//! associate device-DID per session). That works for trusted peers
//! but loses device-grain at every cross-trust-boundary delivery and
//! breaks audit replay where each message is verified standalone.
//!
//! Per Inv-14 device-grain attribution depends on the wire format
//! carrying device-DID at EVERY message kind.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A + G16-B + G16-C wave-6b — net-r4-r1-2 — wire envelope device-DID at every message kind"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A + G16-B + G16-C wave-6b — net-r4-r1-2 — Atrium wire envelope carries peer-DID AND device-DID at every message kind"]
fn iroh_wire_envelope_carries_peer_did_and_device_did_distinguishably_at_every_message_kind() {
    // net-r4-r1-2 pin. G16-A + G16-B + G16-C implementers wire this
    // against the production envelope-construction path per
    // message kind (NOT a stub).
    //
    //   use benten_sync::wire_envelope::{WireEnvelope, MessageKind};
    //
    //   let peer_did = test_peer_did();
    //   let device_did = test_device_did();
    //   let ucan_proof_chain = test_ucan_proof_chain();
    //
    //   // 1. Data-sync-chunk envelope:
    //   let data_chunk = synthesize_data_chunk();
    //   let env_data = WireEnvelope::new(
    //       peer_did.clone(),
    //       device_did.clone(),
    //       ucan_proof_chain.clone(),
    //       MessageKind::DataChunk(data_chunk),
    //   );
    //   let bytes_data = env_data.to_canonical_bytes();
    //   let decoded_data = WireEnvelope::from_canonical_bytes(&bytes_data).unwrap();
    //   assert_eq!(decoded_data.peer_did(), &peer_did);
    //   assert_eq!(decoded_data.device_did(), &device_did,
    //       "data-chunk envelope MUST carry device-DID");
    //
    //   // 2. MST-diff-frame envelope:
    //   let mst_diff = synthesize_mst_diff();
    //   let env_mst = WireEnvelope::new(
    //       peer_did.clone(),
    //       device_did.clone(),
    //       ucan_proof_chain.clone(),
    //       MessageKind::MstDiff(mst_diff),
    //   );
    //   let bytes_mst = env_mst.to_canonical_bytes();
    //   let decoded_mst = WireEnvelope::from_canonical_bytes(&bytes_mst).unwrap();
    //   assert_eq!(decoded_mst.device_did(), &device_did,
    //       "MST-diff envelope MUST carry device-DID");
    //
    //   // 3. Loro-update envelope:
    //   let loro_update = synthesize_loro_update();
    //   let env_loro = WireEnvelope::new(
    //       peer_did.clone(),
    //       device_did.clone(),
    //       ucan_proof_chain.clone(),
    //       MessageKind::LoroUpdate(loro_update),
    //   );
    //   let bytes_loro = env_loro.to_canonical_bytes();
    //   let decoded_loro = WireEnvelope::from_canonical_bytes(&bytes_loro).unwrap();
    //   assert_eq!(decoded_loro.device_did(), &device_did,
    //       "Loro-update envelope MUST carry device-DID");
    //
    //   // 4. Revocation-event envelope (load-bearing for net-blocker-3):
    //   let revoke_event = synthesize_revocation_event();
    //   let env_rev = WireEnvelope::new(
    //       peer_did.clone(),
    //       device_did.clone(),
    //       ucan_proof_chain.clone(),
    //       MessageKind::Revocation(revoke_event),
    //   );
    //   let decoded_rev = WireEnvelope::from_canonical_bytes(&env_rev.to_canonical_bytes()).unwrap();
    //   assert_eq!(decoded_rev.device_did(), &device_did);
    //
    //   // ALL message kinds REQUIRE device-DID at construction
    //   // (NOT Optional; building without device-DID is a typed error):
    //   for kind_factory in &all_message_kind_factories() {
    //       let result = WireEnvelope::builder()
    //           .peer_did(peer_did.clone())
    //           .ucan_proof_chain(ucan_proof_chain.clone())
    //           .kind(kind_factory())
    //           .build();
    //       assert!(result.is_err(),
    //           "envelope construction without device-DID must fail \
    //            (per Inv-14 device-grain at every wire frame)");
    //   }
    //
    // OBSERVABLE consequence: every Atrium wire envelope (handshake,
    // data-chunk, MST-diff, Loro-update, revocation-event) carries
    // both peer-DID AND device-DID at the wire level — NOT relying
    // on session-bound mapping at the receiver side. Atrium-replica
    // audit replay + light-client per-message verification + forensic
    // audit all preserve device-grain. Defends against the failure
    // mode where session-bound device-DID-only-at-handshake mapping
    // loses grain at every cross-trust-boundary delivery (the
    // attack class net-r4-r1-2 named).
    unimplemented!(
        "G16-A + G16-B + G16-C wire WireEnvelope (peer_did, device_did, ucan_proof_chain, payload) at every message kind"
    );
}
