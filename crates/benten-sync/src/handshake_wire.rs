//! Handshake wire-format struct per net-blocker-4 BLOCKER.
//!
//! ## net-blocker-4 contract (G16-A canary scope: SHAPE only)
//!
//! Per device-mesh exploration brief-edits + sec-r1-6 + Inv-14
//! device-grain attribution: the peer-handshake wire-format MUST carry
//! BOTH the peer-DID (account identity) AND the device-DID (device
//! identity under that account). Phase-3 multi-device support relies on
//! the device-DID being observable end-to-end, which is only possible
//! if the wire-format makes both fields REQUIRED at construction time.
//!
//! G16-A canary lands the SHAPE: [`HandshakeFrame`] +
//! [`HandshakeFrameBuilder`] + [`HandshakeFrame::to_canonical_bytes`] /
//! [`HandshakeFrame::from_canonical_bytes`] round-trip + the
//! both-DIDs-required construction guarantee.
//!
//! G16-D wave-6b lands the PROTOCOL BODY: replay-window enforcement,
//! signature verification, UCAN-grant exchange, revocation-set
//! synchronization (per `tests/handshake.rs` pins +
//! `D-PHASE-3-N` extensions). The G16-D protocol module
//! (`handshake.rs` — does NOT exist in this PR) consumes
//! [`HandshakeFrame`] as its on-the-wire envelope.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `atrium_handshake_wire_format_carries_peer_did_and_device_did`.
//! - plan §3 G16-A row line "G16-A defines the SHAPE; G16-D wires the
//!   actual handshake exchange".
//! - `net-blocker-4` BLOCKER (peer-DID + device-DID required).
//! - `Inv-14` device-grain attribution.

use benten_id::did::Did;
use serde::{Deserialize, Serialize};

use crate::errors::AtriumTransportError;
use crate::peer_id::PeerId;

/// Current wire-format version. G16-D protocol exchanges set this on
/// every outbound frame; receivers reject mismatched versions at the
/// transport layer (degraded → typed error).
pub const HANDSHAKE_WIRE_VERSION: u8 = 1;

/// Wire-format frame for the Phase-3 Atrium peer handshake.
///
/// Carries both the peer-DID (account identity) AND the device-DID
/// (device identity under that account) per net-blocker-4 BLOCKER.
/// Construction enforces both-DIDs-required via the
/// [`HandshakeFrameBuilder`] type-state pattern: a builder missing
/// either DID is unconstructable at the type level.
///
/// G16-A canary scope: this struct ships only the SHAPE. The protocol
/// state machine (initiate / respond / finalise / replay-rejection /
/// UCAN-grant exchange) is G16-D wave-6b scope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HandshakeFrame {
    /// Wire-format version. Always [`HANDSHAKE_WIRE_VERSION`] for
    /// G16-A-era frames.
    pub version: u8,

    /// Peer-DID — the account-level identity of the peer initiating
    /// or responding to the handshake. Per net-blocker-4 BLOCKER, this
    /// field is REQUIRED at the wire-format layer (not Optional).
    pub peer_did: Did,

    /// Device-DID — the device-level identity under the peer-account.
    /// Per net-blocker-4 BLOCKER + Inv-14 device-grain attribution,
    /// this field is REQUIRED at the wire-format layer (not Optional).
    /// Multi-device support relies on the device-DID being observable
    /// end-to-end through every handshake frame.
    pub device_did: Did,

    /// Sender's [`PeerId`] (Ed25519 pubkey bytes). Per crypto-minor-4,
    /// this is identical to the iroh NodeId — the receiver's transport
    /// layer can verify the QUIC-level peer-identity matches this
    /// declared identity.
    pub peer_id: PeerId,

    /// Opaque payload reserved for G16-D protocol-body content
    /// (initiator-nonce / signature / UCAN-proof-CIDs / revocation-set
    /// snapshot CIDs / etc.). G16-A canary leaves this empty; G16-D
    /// fills it with the protocol exchange content.
    #[serde(with = "serde_bytes")]
    pub protocol_payload: Vec<u8>,
}

impl HandshakeFrame {
    /// Start a builder for a [`HandshakeFrame`].
    ///
    /// Per net-blocker-4 BLOCKER, the builder type-state pattern
    /// enforces both-DIDs-required at construction: only a builder
    /// with both `peer_did` AND `device_did` set can call
    /// [`HandshakeFrameBuilder::build`].
    #[must_use]
    pub fn builder() -> HandshakeFrameBuilder<NoPeerDid, NoDeviceDid, NoPeerId> {
        HandshakeFrameBuilder {
            version: HANDSHAKE_WIRE_VERSION,
            peer_did: NoPeerDid,
            device_did: NoDeviceDid,
            peer_id: NoPeerId,
            protocol_payload: Vec::new(),
        }
    }

    /// Encode to canonical-bytes per CLAUDE.md baked-in #5
    /// (BLAKE3 + DAG-CBOR + CIDv1).
    ///
    /// Round-trips with [`HandshakeFrame::from_canonical_bytes`].
    /// G16-D wires this at the protocol-body send/receive boundaries.
    ///
    /// # Errors
    ///
    /// Returns a typed [`AtriumTransportError::HandshakeWireFormat`]
    /// only if the underlying CBOR encoder fails (which for well-formed
    /// `Did` + `PeerId` + bounded `protocol_payload` should not occur
    /// in practice; the typed error is reserved for forward-compat).
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AtriumTransportError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| AtriumTransportError::HandshakeWireFormat {
            reason: format!("dag-cbor encode failed: {e}"),
        })
    }

    /// Decode from canonical-bytes. Inverse of
    /// [`HandshakeFrame::to_canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if the
    /// bytes are not a valid `HandshakeFrame` CBOR envelope (missing
    /// peer_did / missing device_did / missing peer_id / corrupted
    /// payload). Per net-blocker-4 BLOCKER, a frame missing
    /// `device_did` rejects at this decode boundary — the
    /// `device_did: Did` field is REQUIRED in the struct shape, so a
    /// CBOR object missing the field fails serde's required-field
    /// check.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, AtriumTransportError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|e| {
            AtriumTransportError::HandshakeWireFormat {
                reason: format!("dag-cbor decode failed: {e}"),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Type-state builder per net-blocker-4 BLOCKER
// ---------------------------------------------------------------------------
//
// The type-state pattern guarantees both-DIDs-required at compile time.
// The unit-struct phantom states (`NoPeerDid` / `WithPeerDid` /
// `NoDeviceDid` / `WithDeviceDid` / `NoPeerId` / `WithPeerId`) walk the
// builder through the four possible (peer_did, device_did, peer_id)
// completion states; only `(WithPeerDid, WithDeviceDid, WithPeerId)`
// has a `build()` method. A caller attempting to construct a
// HandshakeFrame missing either DID gets a compile error, not a
// runtime error.

/// Type-state marker: builder has no `peer_did` set.
#[derive(Debug, Default)]
pub struct NoPeerDid;
/// Type-state marker: builder has `peer_did` set.
#[derive(Debug)]
pub struct WithPeerDid(Did);
/// Type-state marker: builder has no `device_did` set.
#[derive(Debug, Default)]
pub struct NoDeviceDid;
/// Type-state marker: builder has `device_did` set.
#[derive(Debug)]
pub struct WithDeviceDid(Did);
/// Type-state marker: builder has no `peer_id` set.
#[derive(Debug, Default)]
pub struct NoPeerId;
/// Type-state marker: builder has `peer_id` set.
#[derive(Debug)]
pub struct WithPeerId(PeerId);

/// Type-state builder for [`HandshakeFrame`].
///
/// Per net-blocker-4 BLOCKER, only a builder in the
/// `(WithPeerDid, WithDeviceDid, WithPeerId)` state has a `build()`
/// method. A caller attempting to build with either DID missing gets
/// a compile-time error, not a runtime error.
#[derive(Debug)]
pub struct HandshakeFrameBuilder<P, D, K> {
    version: u8,
    peer_did: P,
    device_did: D,
    peer_id: K,
    protocol_payload: Vec<u8>,
}

impl<P, D, K> HandshakeFrameBuilder<P, D, K> {
    /// Set the wire-format version. Defaults to
    /// [`HANDSHAKE_WIRE_VERSION`] for G16-A-era frames.
    #[must_use]
    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    /// Set the protocol payload. G16-A canary scope leaves this empty
    /// (zero-length); G16-D wave-6b fills it with the protocol-body
    /// content.
    #[must_use]
    pub fn protocol_payload(mut self, payload: Vec<u8>) -> Self {
        self.protocol_payload = payload;
        self
    }
}

impl<D, K> HandshakeFrameBuilder<NoPeerDid, D, K> {
    /// Set the peer-DID (account identity). Per net-blocker-4 BLOCKER,
    /// this MUST be set before `build()` is reachable.
    #[must_use]
    pub fn peer_did(self, did: Did) -> HandshakeFrameBuilder<WithPeerDid, D, K> {
        HandshakeFrameBuilder {
            version: self.version,
            peer_did: WithPeerDid(did),
            device_did: self.device_did,
            peer_id: self.peer_id,
            protocol_payload: self.protocol_payload,
        }
    }
}

impl<P, K> HandshakeFrameBuilder<P, NoDeviceDid, K> {
    /// Set the device-DID (device identity). Per net-blocker-4 BLOCKER
    /// + Inv-14 device-grain attribution, this MUST be set before
    /// `build()` is reachable.
    #[must_use]
    pub fn device_did(self, did: Did) -> HandshakeFrameBuilder<P, WithDeviceDid, K> {
        HandshakeFrameBuilder {
            version: self.version,
            peer_did: self.peer_did,
            device_did: WithDeviceDid(did),
            peer_id: self.peer_id,
            protocol_payload: self.protocol_payload,
        }
    }
}

impl<P, D> HandshakeFrameBuilder<P, D, NoPeerId> {
    /// Set the peer-id (Ed25519 pubkey bytes). Per crypto-minor-4,
    /// identical to the iroh NodeId.
    #[must_use]
    pub fn peer_id(self, peer_id: PeerId) -> HandshakeFrameBuilder<P, D, WithPeerId> {
        HandshakeFrameBuilder {
            version: self.version,
            peer_did: self.peer_did,
            device_did: self.device_did,
            peer_id: WithPeerId(peer_id),
            protocol_payload: self.protocol_payload,
        }
    }
}

impl HandshakeFrameBuilder<WithPeerDid, WithDeviceDid, WithPeerId> {
    /// Build the [`HandshakeFrame`]. Only reachable when both DIDs +
    /// peer-id are set per net-blocker-4 BLOCKER.
    #[must_use]
    pub fn build(self) -> HandshakeFrame {
        HandshakeFrame {
            version: self.version,
            peer_did: self.peer_did.0,
            device_did: self.device_did.0,
            peer_id: self.peer_id.0,
            protocol_payload: self.protocol_payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_id::keypair::Keypair;

    fn fixture_kp() -> Keypair {
        // Re-import via export envelope to exercise the
        // canonical-bytes round-trip path. Each test invocation
        // produces a fresh random keypair; the assertions below test
        // the wire-format struct's properties, not the keypair's
        // identity, so randomness is fine here.
        let kp = Keypair::generate();
        let envelope = kp.export_seed_envelope();
        Keypair::from_dag_cbor_envelope(&envelope).expect("round-trip envelope import")
    }

    fn fixture_frame() -> HandshakeFrame {
        let peer_kp = fixture_kp();
        let device_kp = fixture_kp();
        HandshakeFrame::builder()
            .peer_did(peer_kp.public_key().to_did())
            .device_did(device_kp.public_key().to_did())
            .peer_id(PeerId::from_public_key(peer_kp.public_key()))
            .build()
    }

    #[test]
    fn builder_round_trip_via_canonical_bytes() {
        let frame = fixture_frame();
        let bytes = frame.to_canonical_bytes().expect("encode");
        let decoded = HandshakeFrame::from_canonical_bytes(&bytes).expect("decode");
        assert_eq!(frame, decoded);
    }

    #[test]
    fn builder_carries_peer_did_and_device_did() {
        let peer_kp = fixture_kp();
        let device_kp = fixture_kp();
        let frame = HandshakeFrame::builder()
            .peer_did(peer_kp.public_key().to_did())
            .device_did(device_kp.public_key().to_did())
            .peer_id(PeerId::from_public_key(peer_kp.public_key()))
            .build();
        assert_eq!(frame.peer_did, peer_kp.public_key().to_did());
        assert_eq!(frame.device_did, device_kp.public_key().to_did());
        assert_ne!(
            frame.peer_did, frame.device_did,
            "peer_did and device_did must be distinct identities at construction time"
        );
    }

    #[test]
    fn version_defaults_to_handshake_wire_version() {
        let frame = fixture_frame();
        assert_eq!(frame.version, HANDSHAKE_WIRE_VERSION);
    }

    #[test]
    fn protocol_payload_round_trips_through_canonical_bytes() {
        let peer_kp = fixture_kp();
        let device_kp = fixture_kp();
        let payload = vec![0xAA; 64];
        let frame = HandshakeFrame::builder()
            .peer_did(peer_kp.public_key().to_did())
            .device_did(device_kp.public_key().to_did())
            .peer_id(PeerId::from_public_key(peer_kp.public_key()))
            .protocol_payload(payload.clone())
            .build();
        assert_eq!(frame.protocol_payload, payload);
        let bytes = frame.to_canonical_bytes().expect("encode");
        let decoded = HandshakeFrame::from_canonical_bytes(&bytes).expect("decode");
        assert_eq!(decoded.protocol_payload, payload);
    }

    #[test]
    fn malformed_bytes_decode_returns_typed_error() {
        let result = HandshakeFrame::from_canonical_bytes(&[0xff, 0xff, 0xff, 0xff]);
        assert!(matches!(
            result,
            Err(AtriumTransportError::HandshakeWireFormat { .. })
        ));
    }

    // Compile-only test asserting that a builder missing peer_did
    // cannot call build(). Removing the `peer_did(...)` call from the
    // chain below produces a compile error per the type-state pattern;
    // we encode that as a doc-comment expectation rather than a
    // negative-compile harness because the latter requires `trybuild`.
    //
    //   let _frame = HandshakeFrame::builder()
    //       .device_did(...)        // missing peer_did
    //       .peer_id(...)
    //       .build();               // compile error: no build() on this type-state
}
