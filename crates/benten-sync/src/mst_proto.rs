//! MST diff wire-protocol shape per net-blocker-3 BLOCKER.
//!
//! ## net-blocker-3 contract
//!
//! `MessageKind::Revocation` MUST be ordered before
//! `MessageKind::Data` at TWO layers:
//!
//! 1. **Wire-protocol enum** — discriminant + variant ordering encode
//!    the precedence so a peer that decodes a frame's
//!    [`MessageKind`] without consulting application logic can
//!    nonetheless route revocations on the priority path. The
//!    discriminant of [`MessageKind::Revocation`] (`0`) is strictly
//!    less than [`MessageKind::Data`] (`1`); a `BTreeMap` /
//!    sorted-by-discriminant collection drains revocations first by
//!    structural ordering.
//! 2. **Runtime drainer** — [`MstDiffSession::drain`] returns
//!    revocation messages first, in arrival order within the
//!    revocation tier, then data messages in arrival order within
//!    the data tier. This holds even under concurrent arrival of
//!    interleaved revocation + data messages.
//!
//! Companion enforcement at G16-D's handshake protocol body
//! (`handshake.rs`) — both modules import [`MessageKind`] from this
//! file so the enum + ordering live in exactly one place.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `mst_diff_drains_revocation_kind_first_under_concurrent_arrival`.
//! - `net-blocker-3` BLOCKER (revocation-message-kind ordered before
//!   data at handshake + MST diff drain).
//! - plan §3 G16-C row.

use serde::{Deserialize, Serialize};

use crate::errors::AtriumTransportError;
use crate::mst::MstCid;

/// Wire-protocol message-kind discriminant for MST diff session
/// messages.
///
/// Per net-blocker-3 BLOCKER, [`MessageKind::Revocation`] is ordered
/// before [`MessageKind::Data`] at the wire-protocol layer
/// (discriminant `0` vs `1`) AND at the runtime drainer
/// ([`MstDiffSession::drain`]). The two-layer commitment defends
/// against the failure shape where a peer receives interleaved
/// revocation + data messages and applies them in arrival order —
/// a stale data write could land after a fresh revocation if the
/// drainer trusted arrival order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum MessageKind {
    /// Revocation — withdraws a previously-published Node CID,
    /// UCAN delegation, or zone-membership grant. Drains FIRST per
    /// net-blocker-3 BLOCKER.
    ///
    /// Discriminant `0` so structural ordering (e.g. `BTreeMap` keyed
    /// by `MessageKind`) drains revocations before data. G16-D's
    /// handshake protocol body imports this same enum so the
    /// ordering is consistent at handshake + MST diff layers.
    Revocation = 0,

    /// Data — a new or updated Node CID being synchronised between
    /// peers. Drains AFTER revocations per net-blocker-3.
    ///
    /// Discriminant `1` so a stale-data message arriving before a
    /// fresh-revocation message in the same drain cycle still loses
    /// to the revocation at apply time.
    Data = 1,
}

impl MessageKind {
    /// Decode a discriminant byte. Returns
    /// [`AtriumTransportError::HandshakeWireFormat`] if the byte does
    /// not name a known kind — forward-compat policy is to reject
    /// unknown kinds at the wire layer rather than silently drop them
    /// (matches net-blocker-2 typed-error contract).
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if `b`
    /// is not a known [`MessageKind`] discriminant.
    pub fn from_u8(b: u8) -> Result<Self, AtriumTransportError> {
        match b {
            0 => Ok(MessageKind::Revocation),
            1 => Ok(MessageKind::Data),
            other => Err(AtriumTransportError::HandshakeWireFormat {
                reason: format!("unknown MessageKind discriminant: {other}"),
            }),
        }
    }
}

/// MST diff session message — the unit of exchange across the byte
/// stream that [`crate::transport::Connection::send_bytes`] /
/// [`crate::transport::Connection::recv_bytes`] carry.
///
/// Each message names a [`MessageKind`] (Revocation or Data), the
/// target [`MstCid`] (the Node CID being revoked or synchronised),
/// and an opaque payload (the canonical-bytes of the Node for Data
/// messages, or the revocation reason for Revocation messages).
///
/// G16-C scope ships the wire shape + the drain-priority drainer.
/// G16-B's `engine.consume_sync_replica_mst_diff` boundary consumes
/// the drained messages + applies them through the engine's WRITE
/// primitive arm at the receiving peer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MstDiffMessage {
    /// Message kind — Revocation or Data. Per net-blocker-3,
    /// Revocation drains before Data.
    pub kind: MessageKind,

    /// Target Node CID. For Data messages, the CID of the Node whose
    /// canonical bytes appear in `payload`. For Revocation messages,
    /// the CID of the Node being withdrawn.
    pub cid: MstCid,

    /// Opaque payload. For Data messages, the canonical-bytes of the
    /// Node (used for the application-layer rehash check per
    /// sec-r4r2-1 — hash(payload) MUST equal `cid` at receiver). For
    /// Revocation messages, an operator-readable reason string
    /// encoded as bytes (kept opaque at the wire layer so future
    /// revocation-reason schemas extend without wire-format break).
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

impl MstDiffMessage {
    /// Construct a Data-kind message. Helper to avoid construction
    /// errors (the kind discriminant is implicit in the helper name).
    #[must_use]
    pub fn data(cid: MstCid, payload: Vec<u8>) -> Self {
        Self {
            kind: MessageKind::Data,
            cid,
            payload,
        }
    }

    /// Construct a Revocation-kind message.
    #[must_use]
    pub fn revocation(cid: MstCid, reason: impl Into<Vec<u8>>) -> Self {
        Self {
            kind: MessageKind::Revocation,
            cid,
            payload: reason.into(),
        }
    }

    /// Encode to canonical-bytes (DAG-CBOR) for transmission across
    /// the [`crate::transport::Connection`] byte stream.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if the
    /// CBOR encoder fails (which for well-formed `MstCid` + bounded
    /// `payload` should not occur in practice).
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AtriumTransportError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| AtriumTransportError::HandshakeWireFormat {
            reason: format!("dag-cbor encode failed: {e}"),
        })
    }

    /// Decode from canonical-bytes. Inverse of
    /// [`MstDiffMessage::to_canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if the
    /// bytes are not a valid `MstDiffMessage` CBOR envelope.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, AtriumTransportError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|e| {
            AtriumTransportError::HandshakeWireFormat {
                reason: format!("dag-cbor decode failed: {e}"),
            }
        })
    }
}

/// Wire-protocol frame for MST diff exchanges.
///
/// Carries a length-prefixed sequence of [`MstDiffMessage`] entries
/// across the [`crate::transport::Connection::send_bytes`] /
/// [`crate::transport::Connection::recv_bytes`] seam. The G16-A canary's recv path caps
/// per-stream at 4 MiB; large diffs split across multiple frames
/// (the drain-priority invariant holds across frame boundaries via
/// [`MstDiffSession`]).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MstDiffFrame {
    /// Wire-format version. Mirrors
    /// [`crate::handshake_wire::HANDSHAKE_WIRE_VERSION`] discipline:
    /// receivers reject mismatched versions at the transport layer.
    pub version: u8,

    /// Round number within the diff session. Used by the convergence
    /// assertion (`O(log n)` rounds bound) at MST diff observability.
    pub round: u32,

    /// Messages in this frame. Senders SHOULD pre-sort by
    /// [`MessageKind`] so revocations appear first within the
    /// frame; the receiver's [`MstDiffSession`] drainer enforces the
    /// invariant regardless via tier-stratified queues.
    pub messages: Vec<MstDiffMessage>,
}

/// Current MST-diff wire-format version. Receivers reject mismatched
/// versions at the wire layer.
pub const MST_DIFF_WIRE_VERSION: u8 = 1;

impl MstDiffFrame {
    /// Construct an empty diff frame for the given round.
    #[must_use]
    pub fn new(round: u32) -> Self {
        Self {
            version: MST_DIFF_WIRE_VERSION,
            round,
            messages: Vec::new(),
        }
    }

    /// Append a message to this frame. For wire-side determinism,
    /// callers SHOULD push revocations before data; the receiver's
    /// drainer enforces the ordering invariant regardless.
    pub fn push(&mut self, msg: MstDiffMessage) {
        self.messages.push(msg);
    }

    /// Encode to canonical-bytes (DAG-CBOR).
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if the
    /// CBOR encoder fails.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AtriumTransportError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| AtriumTransportError::HandshakeWireFormat {
            reason: format!("dag-cbor encode failed: {e}"),
        })
    }

    /// Decode from canonical-bytes.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::HandshakeWireFormat`] if the
    /// bytes are not a valid `MstDiffFrame` CBOR envelope or if the
    /// version field is not [`MST_DIFF_WIRE_VERSION`].
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, AtriumTransportError> {
        let frame: Self = serde_ipld_dagcbor::from_slice(bytes).map_err(|e| {
            AtriumTransportError::HandshakeWireFormat {
                reason: format!("dag-cbor decode failed: {e}"),
            }
        })?;
        if frame.version != MST_DIFF_WIRE_VERSION {
            return Err(AtriumTransportError::HandshakeWireFormat {
                reason: format!(
                    "unsupported mst-diff wire version: got {} expected {}",
                    frame.version, MST_DIFF_WIRE_VERSION
                ),
            });
        }
        Ok(frame)
    }
}

/// Runtime drainer enforcing the net-blocker-3 invariant: revocation
/// messages drain before data messages, regardless of arrival order.
///
/// Two-tier queue: revocations land in `revocation_queue`, data lands
/// in `data_queue`. [`MstDiffSession::drain`] returns the
/// concatenation in revocation-then-data order, preserving arrival
/// order WITHIN each tier.
///
/// ## Concurrent-arrival invariant
///
/// The drain-priority assertion holds even when revocation + data
/// messages arrive interleaved across multiple frames or multiple
/// rounds — the tier-stratified queue collects across enqueue calls
/// and drains in tier-major order on demand.
#[derive(Debug, Default)]
pub struct MstDiffSession {
    /// Revocation messages queued for drain. FIFO within tier.
    revocation_queue: Vec<MstDiffMessage>,
    /// Data messages queued for drain. FIFO within tier.
    data_queue: Vec<MstDiffMessage>,
}

impl MstDiffSession {
    /// Construct an empty session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a single message into its tier-appropriate queue.
    ///
    /// Per net-blocker-3 the kind is consulted at enqueue time so the
    /// drainer's tier-major drain is O(1) per message rather than
    /// requiring a sort at drain time.
    pub fn enqueue(&mut self, msg: MstDiffMessage) {
        match msg.kind {
            MessageKind::Revocation => self.revocation_queue.push(msg),
            MessageKind::Data => self.data_queue.push(msg),
        }
    }

    /// Enqueue every message in a frame in one shot. Convenience for
    /// the recv path where a whole frame arrives + every message
    /// lands in its tier-appropriate queue.
    pub fn enqueue_frame(&mut self, frame: MstDiffFrame) {
        for msg in frame.messages {
            self.enqueue(msg);
        }
    }

    /// Drain all queued messages in tier-major order: every
    /// [`MessageKind::Revocation`] first (in arrival order within the
    /// tier), then every [`MessageKind::Data`] (in arrival order
    /// within the tier). Empties both queues.
    ///
    /// Per net-blocker-3 BLOCKER. The MST diff pin
    /// `mst_diff_drains_revocation_kind_first_under_concurrent_arrival`
    /// asserts: regardless of interleaved arrival, every Revocation
    /// drains before every Data.
    pub fn drain(&mut self) -> Vec<MstDiffMessage> {
        let mut out = Vec::with_capacity(self.revocation_queue.len() + self.data_queue.len());
        out.append(&mut self.revocation_queue);
        out.append(&mut self.data_queue);
        out
    }

    /// Number of revocation messages currently queued.
    #[must_use]
    pub fn pending_revocations(&self) -> usize {
        self.revocation_queue.len()
    }

    /// Number of data messages currently queued.
    #[must_use]
    pub fn pending_data(&self) -> usize {
        self.data_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mst::MstCid;

    fn cid_seed(seed: u8) -> MstCid {
        MstCid::from_blake3_digest([seed; 32])
    }

    #[test]
    fn message_kind_discriminants_match_priority() {
        // net-blocker-3 wire-layer ordering: Revocation discriminant
        // strictly less than Data so structural sort drains correctly.
        assert_eq!(MessageKind::Revocation as u8, 0);
        assert_eq!(MessageKind::Data as u8, 1);
        assert!(MessageKind::Revocation < MessageKind::Data);
    }

    #[test]
    fn message_kind_round_trips_through_u8() {
        assert_eq!(MessageKind::from_u8(0).unwrap(), MessageKind::Revocation);
        assert_eq!(MessageKind::from_u8(1).unwrap(), MessageKind::Data);
    }

    #[test]
    fn message_kind_unknown_discriminant_rejects_typed() {
        let result = MessageKind::from_u8(255);
        assert!(matches!(
            result,
            Err(AtriumTransportError::HandshakeWireFormat { .. })
        ));
    }

    #[test]
    fn message_round_trip_canonical_bytes() {
        let m = MstDiffMessage::data(cid_seed(0xAA), vec![1, 2, 3]);
        let bytes = m.to_canonical_bytes().unwrap();
        let decoded = MstDiffMessage::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(m, decoded);
    }

    #[test]
    fn frame_round_trip_canonical_bytes() {
        let mut frame = MstDiffFrame::new(7);
        frame.push(MstDiffMessage::revocation(
            cid_seed(0x01),
            b"reason".to_vec(),
        ));
        frame.push(MstDiffMessage::data(cid_seed(0x02), vec![9, 9]));
        let bytes = frame.to_canonical_bytes().unwrap();
        let decoded = MstDiffFrame::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(frame, decoded);
        assert_eq!(decoded.round, 7);
        assert_eq!(decoded.messages.len(), 2);
    }

    #[test]
    fn frame_unsupported_version_rejects_typed() {
        let frame = MstDiffFrame {
            version: 99,
            round: 0,
            messages: vec![],
        };
        let bytes = frame.to_canonical_bytes().unwrap();
        let result = MstDiffFrame::from_canonical_bytes(&bytes);
        assert!(matches!(
            result,
            Err(AtriumTransportError::HandshakeWireFormat { .. })
        ));
    }

    #[test]
    fn session_drains_revocation_before_data_under_interleaved_arrival() {
        // net-blocker-3 BLOCKER pin (companion to
        // tests/mst_revocation_priority.rs at the unit-test layer).
        let mut session = MstDiffSession::new();
        // Interleave arrival order:
        // Hex seeds chosen to be visually distinct: 0xD* for Data
        // tier, 0xA* for revocAtion tier (since "0xR" is not valid
        // hex).
        session.enqueue(MstDiffMessage::data(cid_seed(0xD1), vec![1]));
        session.enqueue(MstDiffMessage::revocation(cid_seed(0xA1), b"r1".to_vec()));
        session.enqueue(MstDiffMessage::data(cid_seed(0xD2), vec![2]));
        session.enqueue(MstDiffMessage::revocation(cid_seed(0xA2), b"r2".to_vec()));
        session.enqueue(MstDiffMessage::data(cid_seed(0xD3), vec![3]));

        assert_eq!(session.pending_revocations(), 2);
        assert_eq!(session.pending_data(), 3);

        let drained = session.drain();
        assert_eq!(drained.len(), 5);

        // First two: revocations in arrival order.
        assert_eq!(drained[0].kind, MessageKind::Revocation);
        assert_eq!(drained[0].cid, cid_seed(0xA1));
        assert_eq!(drained[1].kind, MessageKind::Revocation);
        assert_eq!(drained[1].cid, cid_seed(0xA2));

        // Remaining: data in arrival order.
        assert!(drained[2..].iter().all(|m| m.kind == MessageKind::Data));
        assert_eq!(drained[2].cid, cid_seed(0xD1));
        assert_eq!(drained[3].cid, cid_seed(0xD2));
        assert_eq!(drained[4].cid, cid_seed(0xD3));
    }

    #[test]
    fn session_enqueue_frame_routes_to_tiered_queues() {
        let mut frame = MstDiffFrame::new(0);
        frame.push(MstDiffMessage::data(cid_seed(0x10), vec![]));
        frame.push(MstDiffMessage::revocation(cid_seed(0x11), b"".to_vec()));
        frame.push(MstDiffMessage::data(cid_seed(0x12), vec![]));

        let mut session = MstDiffSession::new();
        session.enqueue_frame(frame);
        assert_eq!(session.pending_revocations(), 1);
        assert_eq!(session.pending_data(), 2);
    }

    #[test]
    fn session_drain_empties_both_queues() {
        let mut session = MstDiffSession::new();
        session.enqueue(MstDiffMessage::data(cid_seed(0), vec![]));
        session.enqueue(MstDiffMessage::revocation(cid_seed(1), b"".to_vec()));
        let _ = session.drain();
        assert_eq!(session.pending_revocations(), 0);
        assert_eq!(session.pending_data(), 0);
    }
}
