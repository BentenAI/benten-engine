//! G16-A LANDED pin for peer-id derivation from Ed25519 pubkey
//! per r2-test-landscape §2.4 G16-A row + plan §3 G16-A row +
//! net-minor-2 + ds-8 + crypto-minor-4.
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
//! - `crypto-minor-4` (iroh EndpointId == Ed25519 pubkey design;
//!   key-reuse posture documented in
//!   `crates/benten-sync/src/peer_id.rs`).

#![allow(clippy::unwrap_used)]

use benten_id::keypair::Keypair;
use benten_sync::peer_id::PeerId;

#[test]
fn iroh_peer_id_derived_deterministically_from_ed25519_pubkey() {
    // net-minor-2 + ds-8 + crypto-minor-4 pin.
    //
    // Cross-process determinism property: the peer-id derivation is a
    // pure function of the Ed25519 public key. Two processes given the
    // SAME `Keypair` (re-imported from the same DAG-CBOR envelope)
    // produce byte-identical `PeerId`s. We exercise this end-to-end
    // by:
    //   1. Generate a fresh keypair + export its envelope.
    //   2. Derive PeerId_A from the in-process keypair.
    //   3. Re-import the envelope (simulates a second process loading
    //      the same persisted seed). Derive PeerId_B.
    //   4. Assert PeerId_A == PeerId_B byte-for-byte.
    //   5. Assert PeerId_A == iroh-NodeId-form (32 pubkey bytes) per
    //      crypto-minor-4 key-reuse posture.

    let kp = Keypair::generate();
    let envelope = kp.export_seed_envelope();
    let pid_a = PeerId::from_public_key(kp.public_key());

    let kp_clone = Keypair::from_dag_cbor_envelope(&envelope).expect("re-import envelope");
    let pid_b = PeerId::from_public_key(kp_clone.public_key());

    assert_eq!(
        pid_a, pid_b,
        "peer-id derivation MUST be deterministic across process \
         boundaries per net-minor-2 + ds-8 + crypto-minor-4"
    );
    assert_eq!(
        pid_a.to_canonical_bytes(),
        pid_b.to_canonical_bytes(),
        "canonical-bytes round-trip MUST be byte-identical"
    );

    // crypto-minor-4: PeerId == iroh EndpointId == Ed25519 pubkey
    // (key-reuse posture acknowledged; documented in peer_id.rs).
    assert_eq!(
        pid_a.as_bytes(),
        &kp.public_key().to_bytes(),
        "PeerId bytes MUST equal Ed25519 pubkey bytes per crypto-minor-4 \
         key-reuse posture"
    );

    // DAG-CBOR envelope round-trip per CLAUDE.md baked-in #5
    // (BLAKE3 + DAG-CBOR + CIDv1 canonical-bytes posture).
    let cbor = pid_a.to_dag_cbor_bytes();
    let decoded = PeerId::from_dag_cbor_bytes(&cbor).expect("dag-cbor round-trip");
    assert_eq!(pid_a, decoded);
}
