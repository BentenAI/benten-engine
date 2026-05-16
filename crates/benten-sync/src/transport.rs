//! iroh QUIC transport core for the Atrium peer mesh.
//!
//! ## D-PHASE-3-3 RESOLVED-at-R1
//!
//! iroh's relay-default + best-effort holepunch posture: peers connect
//! through the iroh public relay infrastructure by default, with
//! holepunch upgrade attempts firing in the background. If a direct
//! holepunch path establishes, the transport upgrades transparently;
//! if it doesn't, the relay path carries traffic for the connection's
//! lifetime. Operators can opt into a dedicated peer-list bootstrap
//! (Phase 7 Garden-relays per Compromise #22) instead of the public
//! relay default.
//!
//! ## scope-real-10 CI-conditional gating
//!
//! - **Loopback round-trip** (`net-minor-1`): two `Endpoint`s in a
//!   single process round-trip bytes via the local-network path. NO
//!   relay infrastructure required. **Required-on-every-PR.**
//! - **Relay-fallback** (plan §3 G16-A): under simulated holepunch
//!   failure, iroh falls back to the relay path. **Required-on-every-PR**
//!   when relay infrastructure is reachable; CI-conditional otherwise
//!   per scope-real-10.
//! - **Holepunch smoke** (`scope-real-10`): two peers behind simulated
//!   NATs negotiate a direct path via the iroh holepunch protocol.
//!   **CI-conditional** — gated to specific runner cells with relay
//!   infrastructure available; informational at G16-A launch.
//!
//! The CI gating shape is documented at the test files
//! `crates/benten-sync/tests/transport_loopback.rs` per pin-source
//! call-out. Relay-fallback + holepunch tests are `#[ignore]`'d at
//! G16-A landing pending iroh test-fixture stabilization on CI; the
//! G16-A canary's load-bearing assertion is the loopback round-trip.
//!
//! ## net-blocker-2 BLOCKER (typed errors + observability)
//!
//! Every transport-layer failure surfaces as a typed
//! [`crate::errors::AtriumTransportError`] variant carrying a stable
//! [`benten_errors::ErrorCode`]. The
//! [`Endpoint::transport_status`] +
//! [`Connection::transport_kind`] accessors expose the connection
//! state so the engine-side `engine.atrium_status()` surface (G16-B/D)
//! can propagate degraded/relay/direct state observably.
//!
//! ## G16-A canary scope
//!
//! G16-A ships:
//! - [`Endpoint::bind_loopback`] — binds an iroh `Endpoint` on the
//!   local-network path for the in-process round-trip canary.
//! - [`Endpoint::bind_with_keypair`] — binds an iroh `Endpoint` with
//!   the caller-supplied [`benten_id::keypair::Keypair`] so the iroh
//!   EndpointId == Ed25519 pubkey design (per crypto-minor-4) holds
//!   end-to-end.
//! - [`Endpoint::connect`] / [`Endpoint::accept_next`] — bidirectional
//!   stream open/accept against a remote [`crate::peer_id::PeerId`].
//! - [`Connection::send_bytes`] / [`Connection::recv_bytes`] — the
//!   minimum-viable bytes round-trip surface that loopback +
//!   relay-fallback tests assert against.
//! - [`Endpoint::transport_status`] +
//!   [`TransportStatus`] + [`TransportKind`] —
//!   net-blocker-2 observable state.
//!
//! G16-D wave-6b consumes this transport surface to wire the handshake
//! protocol body (`handshake.rs` — does NOT exist in this PR); G16-B
//! wires Loro CRDT against the same surface for sync-replica delivery.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A row.
//! - plan §3 G16-A row.
//! - `D-PHASE-3-3` RESOLVED-at-R1.
//! - `net-minor-1`, `net-blocker-2`, `scope-real-10`, `crypto-minor-4`.

use std::sync::Arc;
use std::time::Duration;

use iroh::endpoint::presets;
use iroh::{EndpointAddr, EndpointId, SecretKey};
use tokio::sync::Mutex;

use crate::errors::{AtriumTransportError, AtriumTransportResult};
use crate::peer_id::PeerId;

/// ALPN identifier for the Atrium peer-mesh protocol.
///
/// iroh's `Endpoint::accept` filters incoming connections by ALPN; only
/// connections advertising this ALPN are routed to the Atrium handler.
/// The version-suffixed identifier lets future protocol revisions
/// negotiate alongside the canary (e.g. `benten/atrium/2`).
pub const ATRIUM_ALPN: &[u8] = b"benten/atrium/1";

/// Transport-layer connection-kind discriminator per net-blocker-2
/// observability.
///
/// G16-B/D consumers route `engine.atrium_status()` on this enum so
/// operators can distinguish a happy direct path from a relay-fallback
/// path from a degraded path.
///
/// ## Safe-3 #603 — establishment-time semantics (honest disclosure)
///
/// This discriminator is **captured at connection establishment and is
/// NOT refreshed afterward**. iroh's holepunch may upgrade a
/// Relay→Direct path mid-connection (D-PHASE-3-3 RESOLVED-at-R1's
/// relay-default + best-effort holepunch), and a NAT rebind / holepunch
/// timeout may degrade Direct→Relay; neither transition updates a live
/// [`Connection`]'s kind. Operators reading `engine.atrium_status()`
/// therefore see the *establishment-time* path classification, not the
/// *current* one. The dynamic-refresh wiring (iroh
/// `Connection::watch_conn_type` → background task → `Arc<Mutex<…>>`
/// kind) is a Phase-4-Meta v1-assessment-window backlog row
/// (`docs/future/phase-4-backlog.md`); until it lands, the
/// Compromise #22 metadata-leakage observability story is
/// establishment-time-accurate only. Honest disclosure (per
/// Compromise #19 framing) is preferred over a docstring promise the
/// code does not keep.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    /// Direct holepunched path. iroh has established a peer-to-peer
    /// QUIC tunnel without relay-side relay.
    Direct,
    /// Relay-routed path. iroh is carrying traffic through the
    /// relay-default infrastructure (Compromise #22 metadata-leakage
    /// posture applies).
    Relay,
    /// Loopback path (single-process two-Endpoint canary). Used by the
    /// `iroh_transport_two_peer_loopback_round_trip` net-minor-1 pin.
    Loopback,
}

/// Transport-layer connection-state discriminator per net-blocker-2
/// observability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportStatus {
    /// Connection is healthy. `kind` carries the active path
    /// discriminator.
    Healthy {
        /// The active path discriminator (Direct / Relay / Loopback).
        kind: TransportKind,
    },
    /// Connection has degraded. `reason` is operator-readable; the
    /// engine-side `engine.atrium_status()` surface (G16-B/D) consumes
    /// this state.
    Degraded {
        /// Operator-readable reason for the degradation.
        reason: String,
    },
    /// No connection has been established yet (Endpoint binding
    /// state).
    NotConnected,
}

impl TransportStatus {
    /// Map this status to its stable
    /// [`benten_errors::ErrorCode`], if the status is degraded.
    /// Returns `None` for healthy / not-connected states.
    #[must_use]
    pub fn error_code(&self) -> Option<benten_errors::ErrorCode> {
        match self {
            TransportStatus::Degraded { .. } => {
                Some(benten_errors::ErrorCode::AtriumTransportDegraded)
            }
            TransportStatus::Healthy { .. } | TransportStatus::NotConnected => None,
        }
    }
}

/// iroh-backed Atrium transport endpoint.
///
/// One `Endpoint` per peer-process. Peers connect to other peers'
/// `Endpoint`s by [`PeerId`] (Ed25519 pubkey == iroh EndpointId per
/// crypto-minor-4); the established [`Connection`]s carry bytes
/// between them.
pub struct Endpoint {
    /// Underlying iroh endpoint (driven by the tokio runtime).
    inner: iroh::Endpoint,
    /// Peer identity. Identical to the iroh EndpointId per
    /// crypto-minor-4.
    peer_id: PeerId,
    /// Last-observed transport status. Wrapped in `Mutex` so the
    /// foreground connect/accept calls + the explicit `mark_degraded`
    /// status-pipeline path can update it under shared ownership.
    ///
    /// Safe-3 #603: there is **no background relay-fallback task** at
    /// HEAD — the prior comment promised one that does not exist. The
    /// `Mutex` is load-bearing for the explicit status-update path
    /// only; dynamic iroh-conn-type-driven refresh is the Phase-4-Meta
    /// backlog item (`docs/future/phase-4-backlog.md`).
    status: Arc<Mutex<TransportStatus>>,
}

impl std::fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Endpoint")
            .field("peer_id", &self.peer_id)
            .field("status", &"<async>")
            .finish_non_exhaustive()
    }
}

impl Endpoint {
    /// Bind a loopback endpoint for the in-process two-Endpoint canary
    /// (net-minor-1).
    ///
    /// The endpoint binds with relay disabled — this is the
    /// load-bearing canary that gates G16-A landing per Q7 RESOLVED.
    /// No relay infrastructure required; CI runs this on every PR.
    /// The caller pairs two loopback `Endpoint`s in the same process
    /// + uses [`Endpoint::loopback_addr`] to wire the second peer's
    /// connect target.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if iroh's
    /// endpoint binding fails (typically a port-exhaustion or
    /// network-stack-unavailable scenario).
    pub async fn bind_loopback() -> AtriumTransportResult<Self> {
        let kp = benten_id::keypair::Keypair::generate();
        Self::bind_with_keypair_inner(&kp, /* loopback */ true).await
    }

    /// Bind a loopback endpoint using a caller-supplied keypair so the
    /// iroh EndpointId is deterministic across test runs.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if iroh's
    /// endpoint binding fails.
    pub async fn bind_loopback_with_keypair(
        keypair: &benten_id::keypair::Keypair,
    ) -> AtriumTransportResult<Self> {
        Self::bind_with_keypair_inner(keypair, /* loopback */ true).await
    }

    /// Bind an endpoint with the caller-supplied keypair so the iroh
    /// EndpointId == Ed25519 pubkey design (crypto-minor-4) holds
    /// end-to-end.
    ///
    /// The endpoint advertises [`ATRIUM_ALPN`] so only connections
    /// negotiating the Atrium protocol are accepted. Uses iroh's
    /// production relay default per D-PHASE-3-3 RESOLVED-at-R1.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if iroh's
    /// endpoint binding fails.
    pub async fn bind_with_keypair(
        keypair: &benten_id::keypair::Keypair,
    ) -> AtriumTransportResult<Self> {
        Self::bind_with_keypair_inner(keypair, /* loopback */ false).await
    }

    async fn bind_with_keypair_inner(
        keypair: &benten_id::keypair::Keypair,
        loopback: bool,
    ) -> AtriumTransportResult<Self> {
        // Reuse the Ed25519 secret bytes for iroh's QUIC identity per
        // crypto-minor-4. iroh::SecretKey::from_bytes consumes 32-byte
        // Ed25519 form; we pass the same bytes the benten-id Keypair
        // wraps so the resulting iroh EndpointId == benten-id PublicKey.
        let secret_bytes = keypair.secret_bytes_for_test();
        let secret = SecretKey::from_bytes(&secret_bytes);
        let peer_id = PeerId::from_public_key(keypair.public_key());

        // For the loopback canary we use `Minimal` — sets only the
        // mandatory crypto-provider option; relay disabled by default
        // (the empty-by-default `Minimal` preset doesn't add an
        // address-lookup service, so peers connect via direct
        // EndpointAddrs the caller passes through `Endpoint::connect`).
        // For production endpoints we'd use `presets::N0` (sets the
        // n0-DNS pkarr address-lookup + relay-default). G16-A canary
        // uses Minimal everywhere because the loopback canary is the
        // only required-on-every-PR test; production-relay binding
        // wires when G16-D's handshake protocol body lands.
        let inner = iroh::Endpoint::builder(presets::Minimal)
            .secret_key(secret)
            .alpns(vec![ATRIUM_ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| AtriumTransportError::TransportDegraded {
                reason: format!("iroh endpoint bind failed: {e}"),
            })?;

        let kind = if loopback {
            TransportKind::Loopback
        } else {
            TransportKind::Relay
        };

        Ok(Self {
            inner,
            peer_id,
            status: Arc::new(Mutex::new(TransportStatus::Healthy { kind })),
        })
    }

    /// This endpoint's [`PeerId`] (Ed25519 pubkey == iroh EndpointId
    /// per crypto-minor-4).
    #[must_use]
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// The full iroh [`EndpointAddr`] for this loopback endpoint.
    ///
    /// Constructs the address directly from
    /// [`iroh::Endpoint::bound_sockets`] so the loopback canary
    /// works under the `Minimal` preset (no DNS lookup, no relay,
    /// no pkarr address-publishing).
    /// [`iroh::Endpoint::watch_addr`] and [`iroh::Endpoint::addr`]
    /// rely on the address-lookup-service watcher having settled
    /// at least once, which under `Minimal` never fires; using
    /// `bound_sockets` directly bypasses that watcher requirement.
    ///
    /// Used by the loopback canary's second peer to wire its
    /// `connect()` target without DNS / relay discovery: the caller
    /// asks the first peer for its `loopback_addr()` + passes it
    /// directly to [`Endpoint::connect_to_addr`]. Production peers
    /// (non-loopback) use [`Endpoint::connect`] with just a
    /// [`PeerId`] and rely on iroh's relay-default discovery per
    /// D-PHASE-3-3.
    pub fn loopback_addr(&self) -> AtriumTransportResult<EndpointAddr> {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
        let endpoint_id = self.inner.id();
        let sockets = self.inner.bound_sockets();
        if sockets.is_empty() {
            return Err(AtriumTransportError::TransportDegraded {
                reason: "loopback endpoint has no bound sockets — call after bind".into(),
            });
        }
        // iroh's default bind uses unspecified addresses (0.0.0.0:N /
        // [::]:N). Connecting to those from another endpoint won't
        // resolve to the listening socket; rewrite to loopback
        // (127.0.0.1 / ::1) so the in-process two-Endpoint canary
        // path actually routes. Production endpoints
        // (`bind_with_keypair`) don't use this accessor — they
        // discover peers via iroh's relay-default + address-lookup
        // path per D-PHASE-3-3.
        let rewritten = sockets.into_iter().map(|sa| match sa {
            SocketAddr::V4(v4) if v4.ip().is_unspecified() => {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), v4.port())
            }
            SocketAddr::V6(v6) if v6.ip().is_unspecified() => {
                SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), v6.port())
            }
            other => other,
        });
        Ok(EndpointAddr::from_parts(
            endpoint_id,
            rewritten.map(iroh::TransportAddr::Ip),
        ))
    }

    /// Current transport status per net-blocker-2 observability.
    ///
    /// G16-B/D's `engine.atrium_status()` surface delegates to this
    /// accessor + maps through [`TransportStatus::error_code`] to
    /// expose the stable [`benten_errors::ErrorCode`] to operator
    /// dashboards.
    pub async fn transport_status(&self) -> TransportStatus {
        self.status.lock().await.clone()
    }

    /// Mark the transport as degraded. Used by the
    /// [`Endpoint::simulate_packet_loss`] test fixture per
    /// net-blocker-2 observability assertions.
    pub async fn mark_degraded(&self, reason: impl Into<String>) {
        let mut s = self.status.lock().await;
        *s = TransportStatus::Degraded {
            reason: reason.into(),
        };
    }

    /// Test-only synthetic packet-loss simulator. Flips the transport
    /// status to [`TransportStatus::Degraded`] without actually
    /// dropping packets. Used by the
    /// `atrium_transport_degraded_explicit_state_observable_*` pin
    /// per net-blocker-2.
    ///
    /// G16-B/D wires real degrade-detection (packet-loss-fraction
    /// over a sliding window) once the protocol body lands.
    pub async fn simulate_packet_loss(&self, fraction: f32) {
        if fraction > 0.10 {
            self.mark_degraded(format!("synthetic-packet-loss-{:.0}pct", fraction * 100.0))
                .await;
        }
    }

    /// Connect to a remote peer by [`PeerId`] (relay/holepunch path).
    ///
    /// Per D-PHASE-3-3, iroh's relay-default + best-effort holepunch
    /// posture handles the path negotiation transparently. The
    /// returned [`Connection`] carries bytes between the two
    /// endpoints. For the loopback canary, prefer
    /// [`Endpoint::connect_to_addr`] which skips the relay discovery
    /// step and connects directly to the supplied [`EndpointAddr`].
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::PeerConnectFailed`] if the
    /// connection cannot establish.
    pub async fn connect(&self, remote: PeerId) -> AtriumTransportResult<Connection> {
        let addr = EndpointAddr::new(public_key_from_peer_id(remote)?);
        self.connect_to_addr(addr).await
    }

    /// Connect to a remote peer by full [`EndpointAddr`].
    ///
    /// Used by the loopback canary's second peer: the second peer
    /// calls `first_peer.loopback_addr()` to obtain the full
    /// `EndpointAddr` (with direct socket addresses populated) +
    /// passes it here. iroh resolves the connection through the
    /// direct path without relay infrastructure.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::PeerConnectFailed`] if the
    /// connection cannot establish.
    pub async fn connect_to_addr(
        &self,
        remote_addr: EndpointAddr,
    ) -> AtriumTransportResult<Connection> {
        let remote_peer = PeerId::from_bytes(*remote_addr.id.as_bytes());

        let conn = self
            .inner
            .connect(remote_addr, ATRIUM_ALPN)
            .await
            .map_err(|e| AtriumTransportError::PeerConnectFailed {
                peer: hex_encode(remote_peer.as_bytes()),
                reason: format!("iroh connect failed: {e}"),
                relay_side: false,
            })?;

        let kind = match self.transport_status().await {
            TransportStatus::Healthy { kind } => kind,
            _ => TransportKind::Relay,
        };

        Ok(Connection {
            inner: conn,
            kind,
            remote_peer,
        })
    }

    /// Accept the next inbound connection.
    ///
    /// Returns the established [`Connection`] when a peer connects
    /// negotiating [`ATRIUM_ALPN`]. Non-Atrium connections are filtered
    /// at iroh's ALPN layer.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if the
    /// accept loop fails (typically because the endpoint was closed).
    pub async fn accept_next(&self) -> AtriumTransportResult<Connection> {
        let incoming =
            self.inner
                .accept()
                .await
                .ok_or_else(|| AtriumTransportError::TransportDegraded {
                    reason: "endpoint accept loop closed".into(),
                })?;

        let connecting =
            incoming
                .accept()
                .map_err(|e| AtriumTransportError::TransportDegraded {
                    reason: format!("incoming.accept failed: {e}"),
                })?;

        let conn = connecting
            .await
            .map_err(|e| AtriumTransportError::TransportDegraded {
                reason: format!("connecting.await failed: {e}"),
            })?;

        let remote_endpoint_id = conn.remote_id();
        let remote_peer = PeerId::from_bytes(*remote_endpoint_id.as_bytes());

        let kind = match self.transport_status().await {
            TransportStatus::Healthy { kind } => kind,
            _ => TransportKind::Relay,
        };

        Ok(Connection {
            inner: conn,
            kind,
            remote_peer,
        })
    }

    /// Internal constructor for the wave-6b
    /// [`crate::peer_discovery::bind_atrium_peer`] entry point.
    ///
    /// G16-A canary owns [`Endpoint::bind_with_keypair`] +
    /// [`Endpoint::bind_loopback`]; G16-D wave-6b owns the
    /// `peer_discovery::bind_atrium_peer` flow that constructs the
    /// underlying `iroh::Endpoint` directly with a chosen
    /// [`crate::peer_discovery::BootstrapMode`]. This constructor wraps
    /// the produced iroh Endpoint into our typed `Endpoint` so the
    /// rest of the transport surface (status / connect / accept) lights
    /// up unchanged.
    ///
    /// Status defaults to `TransportKind::Direct` for `Disabled` /
    /// `Relay` for the relay-default + custom-peer-list modes. Safe-3
    /// #603: this is the **establishment-time** classification and is
    /// not subsequently refined — the prior comment promised a
    /// background relay-handshake task that does not exist at HEAD.
    /// Dynamic refinement (iroh `Connection::watch_conn_type`) is the
    /// Phase-4-Meta backlog row (`docs/future/phase-4-backlog.md`).
    pub fn from_iroh_parts(
        inner: iroh::Endpoint,
        peer_id: PeerId,
        bootstrap: &crate::peer_discovery::BootstrapMode,
    ) -> Self {
        use crate::peer_discovery::BootstrapMode;
        let kind = match bootstrap {
            BootstrapMode::Disabled => TransportKind::Direct,
            BootstrapMode::DefaultRelay | BootstrapMode::CustomPeerList(_) => TransportKind::Relay,
        };
        Self {
            inner,
            peer_id,
            status: Arc::new(Mutex::new(TransportStatus::Healthy { kind })),
        }
    }

    /// Bind a relay-using endpoint that fails fast against an
    /// unreachable / malformed relay URL per net-blocker-2 typed-error
    /// contract.
    ///
    /// G16-A canary scope: this fn is the production construction site
    /// for the typed `AtriumTransportError::RelayUnreachable` variant.
    /// It validates the relay URL at parse time + surfaces the typed
    /// error if the URL is malformed; iroh's runtime relay-handshake-
    /// failure path (DNS / TLS / relay-refused) surfaces the same
    /// typed code at connect-time once G16-D wave-6b wires the
    /// CI-conditional relay-fallback fixture per scope-real-10.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::RelayUnreachable`] if:
    /// - the relay URL fails to parse;
    /// - iroh's endpoint binding fails (e.g. crypto-provider not
    ///   configured).
    #[allow(
        clippy::unused_async,
        reason = "G16-A canary scope: signature is async-shaped because G16-D wave-6b body wires async iroh relay-mode bind. Keeping the canary entry-point async-stable so wave-6b is a pure body-fill (no signature change) per pim-4 §3.10 wave-paired closure."
    )]
    pub async fn bind_with_relay_url(
        keypair: &benten_id::keypair::Keypair,
        relay_url: &str,
    ) -> AtriumTransportResult<Self> {
        let _ = keypair;
        // Parse + validate the relay URL up-front; surface
        // `RelayUnreachable` immediately on a malformed URL rather than
        // letting iroh's runtime relay-handshake surface an opaque
        // error later. Any URL parse failure maps to the typed
        // catalog code per net-blocker-2 BLOCKER.
        let _url = relay_url.parse::<iroh::RelayUrl>().map_err(|e| {
            AtriumTransportError::RelayUnreachable {
                url: relay_url.to_string(),
                reason: format!("invalid relay url: {e}"),
            }
        })?;

        // G16-A canary scope: relay-fallback test infrastructure
        // (synthetic NAT harness + relay endpoint) lands at G16-D
        // wave-6b per scope-real-10 + pim-4 §3.10 wave-paired
        // closure. Until then, we surface the typed
        // `RelayUnreachable` error rather than instantiate a real
        // relay-mode binding so the test pin
        // `atrium_relay_unreachable_fires_typed_error_no_panic`
        // exercises the wave-6b promotion seam without the wave-6b
        // fixture in place.
        Err(AtriumTransportError::RelayUnreachable {
            url: relay_url.to_string(),
            reason: "G16-A canary scope: relay-fallback fixture wires at G16-D wave-6b per scope-real-10".to_string(),
        })
    }

    /// Close the endpoint and tear down all connections.
    pub async fn close(self) {
        self.inner.close().await;
    }
}

/// Decode a [`PeerId`] into the iroh [`EndpointId`] / [`iroh::PublicKey`].
fn public_key_from_peer_id(p: PeerId) -> AtriumTransportResult<EndpointId> {
    EndpointId::from_bytes(p.as_bytes()).map_err(|e| AtriumTransportError::PeerConnectFailed {
        peer: hex_encode(p.as_bytes()),
        reason: format!("invalid endpoint-id bytes: {e}"),
        relay_side: false,
    })
}

/// Established connection between two Atrium peers.
///
/// G16-A canary scope ships only [`Connection::send_bytes`] /
/// [`Connection::recv_bytes`] — the minimum-viable bytes round-trip
/// surface that loopback + relay-fallback tests assert against. G16-B
/// wave-6b consumes this surface to layer Loro deltas + MST diffs on
/// top.
pub struct Connection {
    /// Underlying iroh QUIC connection.
    inner: iroh::endpoint::Connection,
    /// Path-discriminator (Direct / Relay / Loopback) at connection-
    /// establishment time.
    kind: TransportKind,
    /// Remote peer's identity (Ed25519 pubkey == iroh EndpointId).
    remote_peer: PeerId,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("kind", &self.kind)
            .field("remote_peer", &self.remote_peer)
            .finish_non_exhaustive()
    }
}

impl Connection {
    /// Path-discriminator **captured at connection-establishment time**
    /// (Safe-3 #603 — honest naming). G16-B/D consumers route
    /// observability on this enum.
    ///
    /// This value is NOT refreshed when iroh's holepunch upgrades a
    /// Relay→Direct path mid-connection or a NAT rebind degrades
    /// Direct→Relay. See [`TransportKind`]'s type-level docs for the
    /// full establishment-time-semantics disclosure + the Phase-4-Meta
    /// backlog row for the dynamic-refresh wiring.
    #[must_use]
    pub fn transport_kind(&self) -> TransportKind {
        self.kind
    }

    /// Remote peer identity.
    #[must_use]
    pub fn remote_peer(&self) -> PeerId {
        self.remote_peer
    }

    /// Send a bytes payload to the remote peer via a fresh
    /// uni-directional stream.
    ///
    /// G16-A canary scope: minimum-viable bytes round-trip. The
    /// receiver consumes via [`Connection::recv_bytes`]. G16-B layers
    /// Loro deltas + MST diffs on top of this primitive.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if the
    /// stream open or write fails.
    pub async fn send_bytes(&self, payload: &[u8]) -> AtriumTransportResult<()> {
        let mut send =
            self.inner
                .open_uni()
                .await
                .map_err(|e| AtriumTransportError::TransportDegraded {
                    reason: format!("open_uni failed: {e}"),
                })?;
        send.write_all(payload)
            .await
            .map_err(|e| AtriumTransportError::TransportDegraded {
                reason: format!("write_all failed: {e}"),
            })?;
        send.finish()
            .map_err(|e| AtriumTransportError::TransportDegraded {
                reason: format!("send.finish failed: {e}"),
            })?;
        // Wait for the receiver to ACK via stopped(). This guarantees
        // the receiver has actually consumed the bytes before
        // send_bytes returns — load-bearing for the loopback
        // round-trip canary's deterministic assertion order.
        let _ = send.stopped().await;
        Ok(())
    }

    /// Receive a bytes payload from the remote peer.
    ///
    /// Accepts the next inbound uni-directional stream + drains it to
    /// completion. The 4-MiB cap defends against an unbounded-allocation
    /// attack from a malicious peer; G16-B/D layer the framed
    /// length-prefix encoding that ships per-message bounds.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumTransportError::TransportDegraded`] if the
    /// stream accept or read fails.
    pub async fn recv_bytes(&self) -> AtriumTransportResult<Vec<u8>> {
        let mut recv =
            self.inner
                .accept_uni()
                .await
                .map_err(|e| AtriumTransportError::TransportDegraded {
                    reason: format!("accept_uni failed: {e}"),
                })?;

        // 4-MiB cap per stream. G16-A canary scope; G16-B/D layers
        // framed encoding with per-message length-prefix.
        const RECV_CAP: usize = 4 * 1024 * 1024;
        let bytes = recv.read_to_end(RECV_CAP).await.map_err(|e| {
            AtriumTransportError::TransportDegraded {
                reason: format!("read_to_end failed: {e}"),
            }
        })?;

        Ok(bytes)
    }

    /// Close the connection.
    pub fn close(self) {
        self.inner.close(0u32.into(), b"closed");
    }
}

/// Wait for an endpoint's iroh-relay machinery to home. iroh's bind
/// succeeds eagerly + the relay handshake happens in the background;
/// for tests asserting `EndpointAddr` availability we wait briefly.
/// Used by the loopback canary so the second endpoint's connect
/// against the first endpoint's address resolves through iroh's
/// local-discovery path.
pub async fn wait_briefly_for_home() {
    tokio::time::sleep(Duration::from_millis(50)).await;
}

/// Hex-encode bytes for the connect-error formatter. Keeping this as a
/// tiny inline impl avoids pulling the whole `hex` crate in as a
/// runtime dep — the formatter is the only construction site.
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_status_error_code_only_for_degraded() {
        assert_eq!(
            TransportStatus::Healthy {
                kind: TransportKind::Loopback
            }
            .error_code(),
            None
        );
        assert_eq!(TransportStatus::NotConnected.error_code(), None);
        assert_eq!(
            TransportStatus::Degraded {
                reason: "test".into()
            }
            .error_code(),
            Some(benten_errors::ErrorCode::AtriumTransportDegraded)
        );
    }

    #[test]
    fn transport_kind_variants_distinct() {
        assert_ne!(TransportKind::Direct, TransportKind::Relay);
        assert_ne!(TransportKind::Relay, TransportKind::Loopback);
        assert_ne!(TransportKind::Direct, TransportKind::Loopback);
    }

    #[tokio::test]
    async fn loopback_endpoint_binds_and_reports_loopback_kind() {
        // net-minor-1 minimum-viable build: bind a loopback endpoint;
        // assert the status reports Loopback kind. Full two-endpoint
        // round-trip lives in tests/transport_loopback.rs.
        let ep = Endpoint::bind_loopback().await.expect("bind loopback");
        match ep.transport_status().await {
            TransportStatus::Healthy {
                kind: TransportKind::Loopback,
            } => {}
            other => panic!("expected loopback kind, got {other:?}"),
        }
        ep.close().await;
    }

    #[tokio::test]
    async fn bind_with_relay_url_rejects_malformed_url_with_typed_error() {
        // net-blocker-2 BLOCKER pin (typed-error construction at the
        // bind boundary; URL-PARSE arm). A malformed relay URL surfaces
        // as `AtriumTransportError::RelayUnreachable` with a parse-
        // failure reason — never a panic, never an untyped String.
        // Distinguishing assertion: the reason MUST cite "invalid
        // relay url" (the parse-rejection arm at line ~503), NOT the
        // canary-scope marker (the wave-6b promotion-seam arm).
        // g16a-mr-minor-2 closure pin: this test must FAIL if the
        // parse arm regresses to bypassing parse + always returning
        // the canary-scope error.
        let kp = benten_id::keypair::Keypair::generate();
        // Empty string is one of the cleanest URL-parse failures —
        // url::Url::parse("") returns Err("relative URL without a
        // base") deterministically across all url-crate versions.
        // Other "obviously-bad" strings like "not-a-valid-url::wat"
        // can pass url::Url::parse (which is lenient) and then
        // surface the canary-scope arm instead — so they don't
        // discriminate the parse arm. Empty-string is the load-
        // bearing parse-failure pin.
        let result = Endpoint::bind_with_relay_url(&kp, "").await;
        match result {
            Err(AtriumTransportError::RelayUnreachable { reason, .. }) => {
                assert!(
                    reason.starts_with("invalid relay url:"),
                    "malformed URL must surface the parse-rejection reason, not the canary-scope marker; got: {reason}"
                );
            }
            other => panic!("expected RelayUnreachable from URL parse, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn bind_with_relay_url_well_formed_returns_canary_scope_typed_error() {
        // g16a-mr-minor-2 companion pin (typed-error construction at
        // the bind boundary; CANARY-SCOPE arm). A well-formed relay
        // URL parses successfully then surfaces the canary-scope
        // marker because G16-A's relay-mode wiring is not yet in
        // place; G16-D wave-6b promotes this path to a real iroh
        // RelayMode::Custom binding per pim-4 §3.10.
        //
        // Distinguishing assertion: the reason MUST cite "G16-A
        // canary scope" (the wave-6b promotion-seam arm, NOT the
        // parse-rejection arm). Together with the malformed-URL
        // companion test, these two pins exercise BOTH error arms
        // discriminatingly.
        let kp = benten_id::keypair::Keypair::generate();
        let result = Endpoint::bind_with_relay_url(&kp, "https://relay.example.test:443/").await;
        match result {
            Err(AtriumTransportError::RelayUnreachable { reason, .. }) => {
                assert!(
                    reason.contains("G16-A canary scope"),
                    "well-formed URL must surface the canary-scope marker (wave-6b promotion seam), not the parse-rejection reason; got: {reason}"
                );
            }
            other => panic!("expected RelayUnreachable from canary-scope arm, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn mark_degraded_flips_status_observable() {
        let ep = Endpoint::bind_loopback().await.expect("bind");
        ep.mark_degraded("test-degrade").await;
        assert!(matches!(
            ep.transport_status().await,
            TransportStatus::Degraded { .. }
        ));
        assert_eq!(
            ep.transport_status().await.error_code(),
            Some(benten_errors::ErrorCode::AtriumTransportDegraded)
        );
        ep.close().await;
    }
}
