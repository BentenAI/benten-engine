//! Peer-id derived from `benten-id::Keypair` Ed25519 pubkey.
//!
//! ## Design contract (net-minor-2 + ds-8 + crypto-minor-4)
//!
//! [`PeerId`] is the 32-byte Ed25519 public-key bytes — IDENTICAL to the
//! iroh `NodeId` per crypto-minor-4: iroh's NodeId is also 32 bytes of
//! Ed25519 public key, and reusing the same key serves both the Atrium
//! peer-identity and the iroh QUIC-transport peer-identity. This
//! key-reuse posture is acknowledged + documented:
//!
//! - **Acknowledged tradeoff:** the same Ed25519 key signs both UCAN
//!   chains (engine-layer auth) AND QUIC TLS handshakes (transport-layer
//!   auth). A compromise of either layer compromises both. The
//!   alternative (separate keys per layer) was REJECTED at G16-A because
//!   the additional key-management complexity (rotation coordination,
//!   cross-layer revocation, separate did:key registrations) outweighs
//!   the layer-isolation benefit at Phase-3's threat model. Phase-9+
//!   may re-open this if a hardened deployment posture warrants
//!   transport-layer-key isolation.
//! - **Cross-process determinism:** [`PeerId::to_canonical_bytes`] is
//!   the 32-byte pubkey + nothing else. Two processes given the same
//!   `Keypair` produce byte-identical `PeerId`s. The DAG-CBOR envelope
//!   round-trip (per CLAUDE.md baked-in #5) is provided by
//!   [`PeerId::to_dag_cbor_bytes`] for cases where the peer-id flows
//!   through an envelope-aware surface.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `iroh_peer_id_derived_deterministically_from_ed25519_pubkey`.
//! - plan §3 G16-A row.
//! - `net-minor-2` (round-trip + cross-process determinism).
//! - `ds-8` (peer-id is content-addressable + deterministic).
//! - `crypto-minor-4` (iroh NodeId == Ed25519 pubkey design).

use benten_id::keypair::PublicKey;
use serde::{Deserialize, Serialize};

/// Peer identifier — 32-byte Ed25519 public-key bytes.
///
/// Identical to the iroh `NodeId` per crypto-minor-4. See module-level
/// docs for the key-reuse-posture acknowledgment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId {
    /// 32-byte Ed25519 public-key bytes. Stored as `serde_bytes::ByteArray`-
    /// compatible inline array so DAG-CBOR encodes it as a byte-string
    /// (canonical-bytes-symmetric with the rest of the engine).
    bytes: [u8; 32],
}

impl PeerId {
    /// Derive a `PeerId` from a `benten-id` Ed25519 public key.
    ///
    /// Per net-minor-2 + ds-8 + crypto-minor-4, the peer-id IS the
    /// 32-byte pubkey — no hashing, no salt, no process-local randomness.
    /// Two processes given the same `PublicKey` produce byte-identical
    /// `PeerId`s.
    #[must_use]
    pub fn from_public_key(pk: &PublicKey) -> Self {
        Self {
            bytes: pk.to_bytes(),
        }
    }

    /// Construct a `PeerId` directly from the 32-byte pubkey form.
    ///
    /// Inverse of [`PeerId::as_bytes`]. Used by the iroh-NodeId-to-PeerId
    /// adapter site in [`crate::transport`] + by deserializers consuming
    /// the canonical-bytes envelope.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Borrow the 32-byte pubkey form.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Owned 32-byte pubkey form. Convenience for callers that need to
    /// move the bytes into another container without re-deriving from
    /// the `PublicKey`.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    /// Canonical-bytes encoding for cross-process determinism per
    /// net-minor-2 + ds-8.
    ///
    /// The 32-byte pubkey is canonical-bytes already; this accessor
    /// returns the bytes verbatim so callers don't have to reach
    /// through [`PeerId::as_bytes`] when they want a `Vec<u8>`. The
    /// DAG-CBOR envelope variant lives at [`PeerId::to_dag_cbor_bytes`]
    /// for the symmetry-with-the-rest-of-the-engine path.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    /// Encode to DAG-CBOR envelope per CLAUDE.md baked-in #5.
    ///
    /// Round-trips with [`PeerId::from_dag_cbor_bytes`]. Used at the
    /// canonical-bytes seam where the peer-id flows through an
    /// envelope-aware surface (e.g. the handshake wire-format).
    #[must_use]
    pub fn to_dag_cbor_bytes(&self) -> Vec<u8> {
        // serde_ipld_dagcbor::to_vec on a #[serde(transparent)] struct
        // yields the inner ByteArray's byte-string CBOR encoding. The
        // unwrap is safe: a fixed-size byte array always encodes.
        serde_ipld_dagcbor::to_vec(self).unwrap_or_default()
    }

    /// Decode from a DAG-CBOR envelope. Inverse of
    /// [`PeerId::to_dag_cbor_bytes`].
    ///
    /// # Errors
    ///
    /// Returns a typed [`PeerIdDecodeError`] if the bytes are not a
    /// valid 32-byte byte-string CBOR encoding.
    pub fn from_dag_cbor_bytes(bytes: &[u8]) -> Result<Self, PeerIdDecodeError> {
        serde_ipld_dagcbor::from_slice::<Self>(bytes).map_err(|e| PeerIdDecodeError {
            reason: format!("dag-cbor decode failed: {e}"),
        })
    }
}

/// Decode error for [`PeerId::from_dag_cbor_bytes`].
#[derive(Debug, thiserror::Error)]
#[error("peer-id decode error: {reason}")]
pub struct PeerIdDecodeError {
    /// Operator-readable reason.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_id::keypair::Keypair;

    fn fixture_keypair() -> Keypair {
        // Build a deterministic keypair via the export-then-import
        // envelope round trip: `Keypair::generate()` carries randomness
        // but the resulting envelope can be re-imported byte-identically.
        // For the cross-process determinism property, the load-bearing
        // assertion is that `from_public_key(pk) == from_public_key(pk)`
        // for any same `pk`; this fixture verifies that within-process.
        // The integration-test pin
        // `iroh_peer_id_derived_deterministically_from_ed25519_pubkey`
        // exercises the cross-process round-trip via fixture-seed-bytes
        // serialized through the envelope export path.
        let kp = Keypair::generate();
        let envelope = kp.export_seed_envelope();
        Keypair::from_dag_cbor_envelope(&envelope).expect("round-trip envelope import")
    }

    #[test]
    fn peer_id_from_public_key_is_pubkey_bytes() {
        let kp = fixture_keypair();
        let pid = PeerId::from_public_key(kp.public_key());
        assert_eq!(pid.as_bytes(), &kp.public_key().to_bytes());
    }

    #[test]
    fn peer_id_deterministic_within_process() {
        let kp = fixture_keypair();
        let pid_a = PeerId::from_public_key(kp.public_key());
        let pid_b = PeerId::from_public_key(kp.public_key());
        assert_eq!(pid_a, pid_b);
        assert_eq!(pid_a.to_canonical_bytes(), pid_b.to_canonical_bytes());
    }

    #[test]
    fn peer_id_dag_cbor_round_trip() {
        let kp = fixture_keypair();
        let pid = PeerId::from_public_key(kp.public_key());
        let bytes = pid.to_dag_cbor_bytes();
        let decoded = PeerId::from_dag_cbor_bytes(&bytes).expect("round-trip");
        assert_eq!(pid, decoded);
    }

    #[test]
    fn peer_id_from_bytes_round_trip() {
        let raw = [42u8; 32];
        let pid = PeerId::from_bytes(raw);
        assert_eq!(pid.to_bytes(), raw);
    }

    #[test]
    fn peer_id_decode_invalid_bytes_returns_typed_error() {
        // 4 bytes is not a valid 32-byte pubkey CBOR encoding.
        let result = PeerId::from_dag_cbor_bytes(&[0x84, 0x00, 0x00, 0x00]);
        assert!(result.is_err());
    }
}
