//! Typed atrium-transport errors per net-blocker-2 BLOCKER.
//!
//! ## net-blocker-2 contract
//!
//! When the transport surface fails — relay unreachable, transport
//! degraded, peer connect refused — the failure surfaces as a typed
//! [`AtriumTransportError`] variant carrying the stable
//! [`benten_errors::ErrorCode`]. NEVER as a panic. NEVER as an untyped
//! `String` error.
//!
//! ## Stable error codes
//!
//! - [`AtriumTransportError::RelayUnreachable`] →
//!   [`benten_errors::ErrorCode::AtriumRelayUnreachable`]
//!   (`E_ATRIUM_RELAY_UNREACHABLE`).
//! - [`AtriumTransportError::TransportDegraded`] →
//!   [`benten_errors::ErrorCode::AtriumTransportDegraded`]
//!   (`E_ATRIUM_TRANSPORT_DEGRADED`).
//! - [`AtriumTransportError::PeerConnectFailed`] → maps to the
//!   relay-unreachable code when caused by relay-side failure;
//!   otherwise to the transport-degraded code.
//! - [`AtriumTransportError::HandshakeWireFormat`] → maps to the
//!   transport-degraded code (handshake wire-format violation surfaces
//!   the connection to the engine as degraded; G16-D wires the
//!   protocol-level rejection at the handshake state machine).
//!
//! ## Pin sources
//!
//! - plan §3 G16-A row.
//! - r2-test-landscape §2.4 G16-A row.
//! - `net-blocker-2` BLOCKER.

use benten_errors::ErrorCode;
use thiserror::Error;

/// Result alias for atrium-transport surfaces.
pub type AtriumTransportResult<T> = Result<T, AtriumTransportError>;

/// Typed atrium-transport error variants per net-blocker-2 BLOCKER.
///
/// Each variant maps to a stable [`benten_errors::ErrorCode`] via
/// [`AtriumTransportError::code`] so observability pipelines can route
/// on the catalog identifier independent of the variant struct shape.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AtriumTransportError {
    /// The relay is unreachable. Maps to `E_ATRIUM_RELAY_UNREACHABLE`.
    ///
    /// Fires when the configured iroh relay endpoint refuses the
    /// connection (DNS-resolution failure, TLS handshake refused,
    /// transport-level timeout). Distinct from
    /// [`AtriumTransportError::TransportDegraded`] (which signals an
    /// established connection has degraded mid-flight). Per net-blocker-2,
    /// this MUST be a typed variant — never a panic, never an untyped
    /// `String` error.
    #[error("atrium relay unreachable at {url}: {reason}")]
    RelayUnreachable {
        /// The relay URL the transport attempted to reach.
        url: String,
        /// Operator-readable reason (DNS failure / TLS-refused /
        /// transport-timeout / etc.).
        reason: String,
    },

    /// The transport has degraded. Maps to
    /// `E_ATRIUM_TRANSPORT_DEGRADED`.
    ///
    /// Fires when an established connection's quality drops below the
    /// degrade threshold (high packet loss, relay path slow, direct
    /// connection lost). The engine's
    /// `engine.atrium_status()` surface (G16-B/D) propagates this
    /// state observably so operators can react. Per net-blocker-2,
    /// degraded transport is an EXPLICIT state — not a missing value,
    /// not a panic.
    #[error("atrium transport degraded: {reason}")]
    TransportDegraded {
        /// Operator-readable reason (packet-loss-fraction /
        /// relay-fallback-active / direct-connection-lost / etc.).
        reason: String,
    },

    /// A peer-connect attempt failed. Maps to
    /// `E_ATRIUM_RELAY_UNREACHABLE` if the failure was relay-side,
    /// otherwise to `E_ATRIUM_TRANSPORT_DEGRADED`.
    ///
    /// Fires when [`crate::transport::Endpoint::connect`] cannot
    /// establish a session to the named peer. Carries the peer-id +
    /// operator-readable reason; the [`AtriumTransportError::code`]
    /// accessor inspects `relay_side` to choose the catalog code.
    #[error("atrium peer connect failed for peer {peer}: {reason}")]
    PeerConnectFailed {
        /// Peer identifier (Ed25519 pubkey hex).
        peer: String,
        /// Operator-readable reason.
        reason: String,
        /// `true` if the failure was relay-side (DNS / TLS /
        /// relay-refused); `false` otherwise. Used by
        /// [`AtriumTransportError::code`] to choose between the
        /// `RelayUnreachable` and `TransportDegraded` catalog codes.
        relay_side: bool,
    },

    /// Handshake wire-format violation. Maps to
    /// `E_ATRIUM_TRANSPORT_DEGRADED`.
    ///
    /// Fires when the handshake wire-format struct
    /// [`crate::handshake_wire::HandshakeFrame`] fails to decode (per
    /// net-blocker-4 — a handshake without device-DID is rejected at
    /// the wire-format layer before any protocol logic runs). G16-D
    /// wires the protocol-level rejection (replay-window / signature /
    /// UCAN-grant) at the handshake state machine; this variant is the
    /// transport-layer floor.
    #[error("atrium handshake wire-format violation: {reason}")]
    HandshakeWireFormat {
        /// Operator-readable reason (missing-device-did /
        /// missing-peer-did / cbor-decode-error / etc.).
        reason: String,
    },
}

impl AtriumTransportError {
    /// Map this error to its stable [`benten_errors::ErrorCode`] per
    /// net-blocker-2 BLOCKER.
    ///
    /// Observability pipelines route on the catalog code rather than
    /// the variant struct shape so consumers stay forward-compatible
    /// across new transport-error variants.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            AtriumTransportError::RelayUnreachable { .. } => ErrorCode::AtriumRelayUnreachable,
            AtriumTransportError::TransportDegraded { .. }
            | AtriumTransportError::HandshakeWireFormat { .. } => {
                ErrorCode::AtriumTransportDegraded
            }
            AtriumTransportError::PeerConnectFailed { relay_side, .. } => {
                if *relay_side {
                    ErrorCode::AtriumRelayUnreachable
                } else {
                    ErrorCode::AtriumTransportDegraded
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_unreachable_maps_to_e_atrium_relay_unreachable() {
        let err = AtriumTransportError::RelayUnreachable {
            url: "https://nonexistent.invalid".into(),
            reason: "dns-failure".into(),
        };
        assert_eq!(err.code(), ErrorCode::AtriumRelayUnreachable);
        assert_eq!(err.code().as_str(), "E_ATRIUM_RELAY_UNREACHABLE");
    }

    #[test]
    fn transport_degraded_maps_to_e_atrium_transport_degraded() {
        let err = AtriumTransportError::TransportDegraded {
            reason: "packet-loss-30pct".into(),
        };
        assert_eq!(err.code(), ErrorCode::AtriumTransportDegraded);
        assert_eq!(err.code().as_str(), "E_ATRIUM_TRANSPORT_DEGRADED");
    }

    #[test]
    fn handshake_wire_format_maps_to_transport_degraded() {
        let err = AtriumTransportError::HandshakeWireFormat {
            reason: "missing-device-did".into(),
        };
        assert_eq!(err.code(), ErrorCode::AtriumTransportDegraded);
    }

    #[test]
    fn peer_connect_failed_relay_side_maps_to_relay_unreachable() {
        let err = AtriumTransportError::PeerConnectFailed {
            peer: "abcd".into(),
            reason: "relay-refused".into(),
            relay_side: true,
        };
        assert_eq!(err.code(), ErrorCode::AtriumRelayUnreachable);
    }

    #[test]
    fn peer_connect_failed_not_relay_side_maps_to_transport_degraded() {
        let err = AtriumTransportError::PeerConnectFailed {
            peer: "abcd".into(),
            reason: "direct-path-down".into(),
            relay_side: false,
        };
        assert_eq!(err.code(), ErrorCode::AtriumTransportDegraded);
    }
}
