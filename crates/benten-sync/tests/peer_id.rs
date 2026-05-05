//! R3-C RED-PHASE pin for peer-id derivation from Ed25519 pubkey
//! (G16-A wave-6 canary; per r2-test-landscape §2.4 G16-A row +
//! plan §3 G16-A row + net-minor-2 + ds-8).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `iroh_peer_id_derived_deterministically_from_ed25519_pubkey`.
//! - plan §3 G16-A row line "peer-id derived from `benten-id`
//!   Ed25519 keypair; cross-process determinism per ds-8 +
//!   net-minor-2".
//! - `net-minor-2` (round-trip + cross-process determinism).
//! - `ds-8` (peer-id is content-addressable + deterministic).
//! - `crypto-minor-4` (iroh NodeId == Ed25519 pubkey design
//!   acknowledged; key-reuse posture documented in
//!   `crates/benten-sync/src/peer_id.rs` per plan §3 G16-A row).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 canary lands peer-id derivation"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — net-minor-2 + ds-8 — peer-id deterministic from Ed25519"]
fn iroh_peer_id_derived_deterministically_from_ed25519_pubkey() {
    // net-minor-2 + ds-8 pin. G16-A implementer wires this:
    //
    //   use benten_id::keypair::Keypair;
    //   use benten_sync::peer_id::PeerId;
    //
    //   let kp = Keypair::from_seed_bytes_via_dag_cbor_envelope(&fixture_seed()).unwrap();
    //   let pid_a = PeerId::from_public_key(&kp.public_key());
    //   let pid_b = PeerId::from_public_key(&kp.public_key());
    //
    //   // Determinism: same pubkey → same PeerId (within process).
    //   assert_eq!(pid_a, pid_b);
    //
    //   // Cross-process determinism: serialize PeerId to canonical
    //   // bytes; round-trip through DAG-CBOR; assert byte equality.
    //   let bytes_a = pid_a.to_canonical_bytes();
    //   let bytes_b = pid_b.to_canonical_bytes();
    //   assert_eq!(bytes_a, bytes_b);
    //
    //   // Cross-process: spawn a child process with the same seed
    //   // file; observe the child writes the same PeerId bytes.
    //   let child_bytes = run_child_process_with_seed(&fixture_seed());
    //   assert_eq!(bytes_a, child_bytes);
    //
    //   // crypto-minor-4: PeerId == iroh NodeId == Ed25519 pubkey
    //   // (key-reuse posture acknowledged; documented in peer_id.rs).
    //   assert_eq!(pid_a.as_iroh_node_id().as_bytes(), kp.public_key().as_bytes());
    //
    // OBSERVABLE consequence: peer-id is content-addressed +
    // deterministic across process boundaries. Defends against the
    // failure shape where peer-id includes randomness or
    // process-local state that would break cross-process replay.
    unimplemented!("G16-A wires deterministic peer-id derivation + cross-process round-trip");
}
