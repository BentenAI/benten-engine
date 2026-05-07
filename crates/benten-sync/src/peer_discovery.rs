//! Atrium peer discovery + relay-bootstrap configuration (Phase-3
//! G16-D wave-6b).
//!
//! ## Two bootstrap modes per D-PHASE-3-3 RESOLVED-at-R1
//!
//! - [`BootstrapMode::DefaultRelay`] (the default per D-PHASE-3-3):
//!   peers connect through iroh's public relay infrastructure.
//!   Holepunch upgrades happen in the background; if a direct path
//!   establishes, the transport upgrades transparently. Operators
//!   accept the relay-operator metadata-leakage posture documented
//!   under Compromise #22 in `docs/SECURITY-POSTURE.md`.
//! - [`BootstrapMode::CustomPeerList`]: operator-controlled relay
//!   bootstrap via a caller-supplied list of relay URLs (Phase-7
//!   Garden-relays land as the canonical operator-controlled
//!   alternative). Maps to `iroh::RelayMode::Custom`. Replaces G16-A
//!   canary's `bind_with_relay_url`-returns-`RelayUnreachable`
//!   placeholder per pim-4 §3.10 wave-paired closure.
//! - [`BootstrapMode::Disabled`]: relay infrastructure disabled
//!   entirely. Peers must connect via direct addresses (loopback /
//!   LAN). Used by the loopback canary at G16-A and by tests that
//!   want to assert no-relay-traffic behaviour.
//!
//! ## net-major-1 + sec-r1-12 trust-boundary disclosure
//!
//! [`BootstrapMode::operator_observability_disclosure`] returns the
//! operator-readable text that an Atrium UI surface displays before
//! starting peer discovery. The default-relay mode discloses the
//! metadata observability to relay operators per net-major-1 +
//! sec-r1-12; the custom-peer-list mode discloses the same exposure
//! at the operator's explicit relay choices; the disabled mode
//! discloses no relay traffic flows.
//!
//! ## Pin sources
//!
//! - plan §3 G16-D row line "iroh's relay infrastructure default per
//!   D-PHASE-3-3; opt-in dedicated peer-list bootstrap; trust-boundary
//!   disclosure to relay operators per net-major-1 + sec-r1-12".
//! - `D-PHASE-3-3` RESOLVED-at-R1 (relay-default + holepunch
//!   best-effort).
//! - `Compromise #22` (`docs/SECURITY-POSTURE.md`) — relay-operator
//!   metadata observability.
//! - `pim-4 §3.10` wave-paired closure — promotes G16-A's
//!   `bind_with_relay_url` canary-scope `RelayUnreachable` arm to a
//!   real `RelayMode::Custom` binding.

use iroh::endpoint::presets;
use iroh::{Endpoint as IrohEndpoint, RelayMap, RelayMode, RelayUrl, SecretKey};

use crate::errors::{AtriumTransportError, AtriumTransportResult};
use crate::peer_id::PeerId;
use crate::transport::{ATRIUM_ALPN, Endpoint};

/// Atrium peer-discovery bootstrap mode per D-PHASE-3-3.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootstrapMode {
    /// Use iroh's public relay infrastructure (the production default
    /// per D-PHASE-3-3 RESOLVED-at-R1). Operators accept the
    /// relay-operator metadata-leakage posture per Compromise #22.
    DefaultRelay,
    /// Use a caller-supplied custom peer-list as the relay map.
    /// Phase-7 Garden-relays are the canonical operator-controlled
    /// alternative. Maps to `iroh::RelayMode::Custom`.
    ///
    /// Each entry is a relay URL the operator has chosen to trust
    /// (Garden-deploy / self-hosted / per-org peer registry).
    CustomPeerList(Vec<RelayUrl>),
    /// No relay infrastructure. Peers must connect via direct
    /// addresses (loopback / LAN). The loopback canary (G16-A
    /// `Endpoint::bind_loopback`) consumes this mode implicitly.
    Disabled,
}

impl BootstrapMode {
    /// Construct a `CustomPeerList` mode by parsing string URLs.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::RelayUnreachable`] if any URL
    /// fails to parse as an `iroh::RelayUrl` (per net-blocker-2 typed
    /// error contract).
    pub fn custom_peer_list_from_strs(urls: &[&str]) -> AtriumTransportResult<Self> {
        let mut parsed = Vec::with_capacity(urls.len());
        for url in urls {
            let parsed_url =
                url.parse::<RelayUrl>()
                    .map_err(|e| AtriumTransportError::RelayUnreachable {
                        url: (*url).to_string(),
                        reason: format!("invalid relay url: {e}"),
                    })?;
            parsed.push(parsed_url);
        }
        Ok(BootstrapMode::CustomPeerList(parsed))
    }

    /// Map this mode to the underlying `iroh::RelayMode`.
    ///
    /// G16-A canary's `bind_with_relay_url` returned
    /// `AtriumTransportError::RelayUnreachable` for the canary scope;
    /// G16-D wave-6b promotes that path to a real `RelayMode::Custom`
    /// binding via this accessor (per pim-4 §3.10 wave-paired closure).
    #[must_use]
    pub fn to_iroh_relay_mode(&self) -> RelayMode {
        match self {
            BootstrapMode::DefaultRelay => RelayMode::Default,
            BootstrapMode::CustomPeerList(urls) => {
                if urls.is_empty() {
                    // Empty list — fall through to Disabled rather
                    // than instantiate an empty RelayMap that iroh
                    // treats as a misconfiguration.
                    RelayMode::Disabled
                } else {
                    RelayMode::Custom(urls.iter().cloned().collect::<RelayMap>())
                }
            }
            BootstrapMode::Disabled => RelayMode::Disabled,
        }
    }

    /// Operator-readable trust-boundary disclosure per net-major-1 +
    /// sec-r1-12.
    ///
    /// An Atrium UI surface displays this text before starting peer
    /// discovery so operators understand what metadata flows through
    /// the relay infrastructure under each mode. The string is stable
    /// (asserted by `tests::operator_observability_disclosure_is_stable_text`
    /// in this module) so UI consumers can rely on it for snapshot tests.
    #[must_use]
    pub fn operator_observability_disclosure(&self) -> &'static str {
        match self {
            BootstrapMode::DefaultRelay => {
                "Using iroh's public relay infrastructure. \
                 Relay operators can observe peer-DID + connection \
                 metadata (timing / size / counterparty pairing) per \
                 SECURITY-POSTURE.md Compromise #22. Encrypted payloads \
                 remain confidential. Switch to CustomPeerList for \
                 operator-controlled Garden-relays."
            }
            BootstrapMode::CustomPeerList(_) => {
                "Using a caller-supplied custom relay peer-list. \
                 Each chosen relay operator can observe the same \
                 metadata as iroh's default relays per Compromise #22; \
                 the trust scope is bounded by your relay-list choice."
            }
            BootstrapMode::Disabled => {
                "Relay infrastructure disabled. Peers connect directly \
                 (loopback / LAN). No relay-side metadata observability."
            }
        }
    }
}

/// Configuration for binding an Atrium peer endpoint with a chosen
/// [`BootstrapMode`].
#[derive(Clone, Debug)]
pub struct PeerDiscoveryConfig {
    /// The bootstrap mode for relay discovery.
    pub bootstrap: BootstrapMode,
}

impl PeerDiscoveryConfig {
    /// Default config — production relay-default mode per
    /// D-PHASE-3-3 RESOLVED-at-R1.
    #[must_use]
    pub fn default_relay() -> Self {
        Self {
            bootstrap: BootstrapMode::DefaultRelay,
        }
    }

    /// Custom config — operator-controlled relay peer-list.
    #[must_use]
    pub fn custom_peer_list(urls: Vec<RelayUrl>) -> Self {
        Self {
            bootstrap: BootstrapMode::CustomPeerList(urls),
        }
    }

    /// Disabled config — no relay infrastructure.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            bootstrap: BootstrapMode::Disabled,
        }
    }
}

/// Bind an Atrium peer endpoint with the given keypair + bootstrap
/// config.
///
/// G16-D wave-6b: this is the production-binding entry point that
/// G16-A canary's `Endpoint::bind_with_relay_url` placeholder gates
/// onto per pim-4 §3.10 wave-paired closure. Where G16-A returned
/// `AtriumTransportError::RelayUnreachable` for the canary-scope arm,
/// this function instantiates a real `iroh::RelayMode` per the
/// caller's chosen [`BootstrapMode`].
///
/// # Errors
///
/// Returns [`AtriumTransportError::TransportDegraded`] if iroh's
/// endpoint binding fails (typically a port-exhaustion or
/// network-stack-unavailable scenario). Returns
/// [`AtriumTransportError::RelayUnreachable`] if the chosen relay URL
/// fails resolution at iroh's relay-handshake background phase.
pub async fn bind_atrium_peer(
    keypair: &benten_id::keypair::Keypair,
    config: &PeerDiscoveryConfig,
) -> AtriumTransportResult<Endpoint> {
    // We construct the iroh Endpoint here directly (rather than
    // delegating to G16-A's `Endpoint::bind_with_relay_url`) because
    // G16-A's binding entry-point reflects the canary's pre-D16-D
    // state-machine. Wrapping the constructed iroh::Endpoint into
    // our `Endpoint` type via a constructor that mirrors
    // `bind_with_keypair_inner`'s shape would re-create the same
    // wrapping; instead the wave-6b entry point owns the binding +
    // returns our typed `Endpoint`.

    let secret_bytes = keypair.secret_bytes_for_test();
    let secret = SecretKey::from_bytes(&secret_bytes);
    let peer_id = PeerId::from_public_key(keypair.public_key());

    let inner = match &config.bootstrap {
        BootstrapMode::Disabled => {
            // Disabled mode uses the Minimal preset (relay disabled by
            // default; matches the loopback canary's posture). No
            // relay_mode override needed.
            IrohEndpoint::builder(presets::Minimal)
                .secret_key(secret)
                .alpns(vec![ATRIUM_ALPN.to_vec()])
                .bind()
                .await
        }
        BootstrapMode::DefaultRelay => {
            // Default relay mode uses the N0 preset (relay-default per
            // D-PHASE-3-3) — but with our explicit secret-key + ALPN.
            IrohEndpoint::builder(presets::N0)
                .secret_key(secret)
                .alpns(vec![ATRIUM_ALPN.to_vec()])
                .relay_mode(config.bootstrap.to_iroh_relay_mode())
                .bind()
                .await
        }
        BootstrapMode::CustomPeerList(_) => {
            // Custom relay mode rides on the same N0 preset surface
            // (which configures the address-lookup / pkarr discovery
            // layer) but overrides the relay_mode to point at the
            // operator-controlled relay map per pim-4 §3.10.
            IrohEndpoint::builder(presets::N0)
                .secret_key(secret)
                .alpns(vec![ATRIUM_ALPN.to_vec()])
                .relay_mode(config.bootstrap.to_iroh_relay_mode())
                .bind()
                .await
        }
    }
    .map_err(|e| AtriumTransportError::TransportDegraded {
        reason: format!("iroh endpoint bind failed: {e}"),
    })?;

    Ok(Endpoint::from_iroh_parts(inner, peer_id, &config.bootstrap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_relay_disclosure_mentions_compromise_22() {
        let mode = BootstrapMode::DefaultRelay;
        let disclosure = mode.operator_observability_disclosure();
        assert!(disclosure.contains("Compromise #22"));
        assert!(disclosure.contains("Relay operators"));
    }

    #[test]
    fn custom_peer_list_disclosure_mentions_caller_supplied() {
        let mode = BootstrapMode::CustomPeerList(vec![]);
        let disclosure = mode.operator_observability_disclosure();
        assert!(disclosure.contains("custom relay peer-list"));
        assert!(disclosure.contains("Compromise #22"));
    }

    #[test]
    fn disabled_disclosure_mentions_no_relay_traffic() {
        let mode = BootstrapMode::Disabled;
        let disclosure = mode.operator_observability_disclosure();
        assert!(disclosure.contains("disabled"));
        assert!(disclosure.contains("No relay-side metadata"));
    }

    #[test]
    fn operator_observability_disclosure_is_stable_text() {
        // Snapshot: assert the exact text-prefix so UI consumers can
        // rely on it. If the text changes, this test forces a paired
        // update of UI snapshot tests.
        assert!(
            BootstrapMode::DefaultRelay
                .operator_observability_disclosure()
                .starts_with("Using iroh's public relay infrastructure")
        );
        assert!(
            BootstrapMode::Disabled
                .operator_observability_disclosure()
                .starts_with("Relay infrastructure disabled.")
        );
    }

    #[test]
    fn custom_peer_list_from_strs_rejects_malformed_url() {
        let result = BootstrapMode::custom_peer_list_from_strs(&[""]);
        match result {
            Err(AtriumTransportError::RelayUnreachable { reason, .. }) => {
                assert!(reason.starts_with("invalid relay url:"));
            }
            other => panic!("expected RelayUnreachable, got {other:?}"),
        }
    }

    #[test]
    fn custom_peer_list_from_strs_accepts_well_formed_url() {
        let result =
            BootstrapMode::custom_peer_list_from_strs(&["https://relay.example.test:443/"])
                .unwrap();
        match result {
            BootstrapMode::CustomPeerList(urls) => assert_eq!(urls.len(), 1),
            other => panic!("expected CustomPeerList, got {other:?}"),
        }
    }

    #[test]
    fn empty_custom_peer_list_falls_through_to_disabled_relay_mode() {
        let mode = BootstrapMode::CustomPeerList(vec![]);
        // RelayMode doesn't impl PartialEq exhaustively across variants,
        // so we check the Debug-form instead.
        let relay = mode.to_iroh_relay_mode();
        assert!(matches!(relay, RelayMode::Disabled));
    }

    #[test]
    fn default_relay_mode_maps_to_iroh_default() {
        let mode = BootstrapMode::DefaultRelay;
        let relay = mode.to_iroh_relay_mode();
        assert!(matches!(relay, RelayMode::Default));
    }

    #[test]
    fn config_constructors_round_trip() {
        let cfg = PeerDiscoveryConfig::default_relay();
        assert_eq!(cfg.bootstrap, BootstrapMode::DefaultRelay);
        let cfg = PeerDiscoveryConfig::disabled();
        assert_eq!(cfg.bootstrap, BootstrapMode::Disabled);
    }

    #[tokio::test]
    async fn bind_atrium_peer_disabled_mode_succeeds() {
        // End-to-end binding pin: with Disabled mode, the bind should
        // succeed without any relay infrastructure (matches the
        // loopback canary posture).
        let kp = benten_id::keypair::Keypair::generate();
        let cfg = PeerDiscoveryConfig::disabled();
        let ep = bind_atrium_peer(&kp, &cfg).await.expect("bind disabled");
        assert_eq!(ep.peer_id(), PeerId::from_public_key(kp.public_key()));
        ep.close().await;
    }
}
