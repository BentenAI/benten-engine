//! `TransportConfig` — the transport-selection codepoint reserve (F-full
//! §3.9 / Compromise #53 / GAP-2c).
//!
//! `GossipPlusBlobs` (iroh-gossip broadcast + iroh-blobs content transfer)
//! is the ONE transport that ships at v1-beta. The
//! [`TransportConfig::Willow`] / [`TransportConfig::IrohRoq`] /
//! [`TransportConfig::IrohLive`] variants are **reserved-and-typed-rejected**
//! at v1-beta: selecting one yields [`TransportError::TransportReservedAtV1Beta`]
//! — NEVER a silent accept, NEVER a silent fallback to gossip (the
//! NQ-A1 conservative-fallback discipline; the reserved transports become
//! LIVE additively at unused codepoints, never via a wire-break).

/// The transport selected for an Atrium's sync session.
///
/// Only [`Self::GossipPlusBlobs`] ships at v1-beta; the rest are reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportConfig {
    /// The v1-beta DEFAULT (and only shipping) transport: iroh-gossip
    /// broadcast + iroh-blobs content transfer.
    GossipPlusBlobs,
    /// Willow protocol transport — RESERVED + typed-rejected at v1-beta (#53).
    Willow,
    /// iroh-roq (RTP-over-QUIC) transport — RESERVED + typed-rejected (#53).
    IrohRoq,
    /// iroh-live real-time media transport — RESERVED + typed-rejected (#53).
    IrohLive,
}

/// Typed transport-selection failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportError {
    /// A reserved transport was selected at v1-beta. Carries the reserved
    /// transport's stable name for the operator-facing diagnostic.
    TransportReservedAtV1Beta(&'static str),
}

impl TransportConfig {
    /// Select the transport. The shipping transport returns its stable wire
    /// name; a reserved transport typed-rejects (NEVER a silent accept, NEVER
    /// a silent fallback to gossip — #53 / NQ-A1).
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::TransportReservedAtV1Beta`] for the
    /// Willow / iroh-roq / iroh-live reserves.
    pub fn select(self) -> Result<&'static str, TransportError> {
        match self {
            Self::GossipPlusBlobs => Ok("gossip+blobs"),
            Self::Willow => Err(TransportError::TransportReservedAtV1Beta("Willow")),
            Self::IrohRoq => Err(TransportError::TransportReservedAtV1Beta("iroh-roq")),
            Self::IrohLive => Err(TransportError::TransportReservedAtV1Beta("iroh-live")),
        }
    }

    /// Whether this transport ships at v1-beta (only `GossipPlusBlobs` does).
    #[must_use]
    pub const fn ships_at_v1_beta(self) -> bool {
        matches!(self, Self::GossipPlusBlobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gossip_ships() {
        assert_eq!(
            TransportConfig::GossipPlusBlobs.select(),
            Ok("gossip+blobs")
        );
        assert!(TransportConfig::GossipPlusBlobs.ships_at_v1_beta());
    }

    #[test]
    fn reserves_typed_reject() {
        for (cfg, name) in [
            (TransportConfig::Willow, "Willow"),
            (TransportConfig::IrohRoq, "iroh-roq"),
            (TransportConfig::IrohLive, "iroh-live"),
        ] {
            assert_eq!(
                cfg.select(),
                Err(TransportError::TransportReservedAtV1Beta(name))
            );
            assert!(!cfg.ships_at_v1_beta());
        }
    }
}
