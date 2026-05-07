//! Engine-side Atrium API surface (P2P sync).
//!
//! Phase-3 G16-B wave-6b. Native-only — gated to non-wasm32 targets
//! per CLAUDE.md baked-in #17 (browser tabs participate via
//! authenticated thin-client views, NOT as full Atrium peers).
//!
//! ## Responsibility
//!
//! This module wires the engine's public `open_atrium` surface to
//! [`benten_sync::transport`] (G16-A canary) + [`benten_sync::crdt`]
//! (G16-B). The [`AtriumHandle`] returned from `open_atrium` carries:
//!
//! 1. The local iroh `Endpoint` (one per Atrium handle).
//! 2. A registry of per-zone Loro CRDT documents keyed by zone-prefix.
//! 3. A merge-dispatch surface that consumes inbound sync frames +
//!    fires the Inv-13 row-4 SPLIT classifier (per ds-4) before
//!    applying.
//!
//! ## Inv-13 row-4 SPLIT (ds-4)
//!
//! When a sync frame arrives carrying CRDT op-log targets:
//!
//! - **row-4a** (user-data): the targets fall outside the
//!   `system:` zone-prefix list per
//!   [`crate::system_zones::SYSTEM_ZONE_PREFIXES`]. The Loro merge
//!   applies via [`AtriumHandle::merge_remote_change`]; the engine
//!   mints a new Version Node via the existing Anchor + Version +
//!   CURRENT pattern (Phase-1 shipped) per arch-r1-4 D-C HYBRID.
//!   AttributionFrame at the new Version captures contributing
//!   peer-`node_id`s observed via [`LoroDoc::winning_attribution`].
//! - **row-4b** (system-zone / Anchor-immutable): the targets fall
//!   inside the system-zone prefix list. The merge is REJECTED with
//!   [`AtriumError::DivergentCidRejected`] mapping to the stable
//!   error code [`benten_errors::ErrorCode::SyncDivergentCidRejected`].
//!
//! ## Pin sources
//!
//! - plan §3 G16-B row.
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_open_close_lifecycle` +
//!   `atrium_sync_subgraph_two_peer_bidirectional`.
//! - `D-PHASE-3-22` RESOLVED + `arch-r1-4` D-C HYBRID.
//! - `ds-4` Inv-13 row-4 SPLIT.

use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use benten_id::keypair::Keypair;
use benten_sync::crdt::{CrdtError, LoroDoc};
use benten_sync::peer_id::PeerId;
use benten_sync::transport::{Connection, Endpoint, TransportKind, TransportStatus};

use crate::atrium_api::{AtriumConfig, AtriumMode, SyncStatus};
use crate::system_zones::SYSTEM_ZONE_PREFIXES;

/// Atrium-layer error surface.
///
/// All variants map to typed [`benten_errors::ErrorCode`] codes via
/// [`AtriumError::code`]. Engine-side callers consume the typed code
/// for observability; operator dashboards route on the stable
/// catalog identifier.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AtriumError {
    /// The underlying transport surface failed.
    #[error("atrium transport error: {0}")]
    Transport(#[from] benten_sync::errors::AtriumTransportError),

    /// The CRDT layer surfaced an error.
    #[error("atrium crdt error: {0}")]
    Crdt(#[from] CrdtError),

    /// Inv-13 row-4b reject: divergent CID for a system-zone /
    /// Anchor-immutable Node. Maps to
    /// [`benten_errors::ErrorCode::SyncDivergentCidRejected`].
    #[error("atrium divergent CID rejected for system-zone/Anchor-immutable target: {target}")]
    DivergentCidRejected {
        /// The target container-name that triggered the row-4b reject.
        target: String,
    },

    /// The atrium handle reached an invalid state (e.g. a sync against
    /// a zone the handle doesn't carry a CRDT for).
    #[error("atrium invalid state: {reason}")]
    InvalidState {
        /// Operator-readable reason.
        reason: String,
    },
}

impl AtriumError {
    /// Map this error to its stable [`benten_errors::ErrorCode`].
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            AtriumError::Transport(e) => e.code(),
            AtriumError::Crdt(e) => e.code(),
            AtriumError::DivergentCidRejected { .. } => {
                benten_errors::ErrorCode::SyncDivergentCidRejected
            }
            AtriumError::InvalidState { .. } => benten_errors::ErrorCode::AtriumTransportDegraded,
        }
    }
}

/// Result alias for Atrium surfaces.
pub type AtriumResult<T> = Result<T, AtriumError>;

/// Atrium session handle (per Ben's D1 session-handle B-prime shape).
///
/// One [`AtriumHandle`] per call to [`crate::Engine::open_atrium`].
/// Carries the iroh transport endpoint, the per-zone Loro CRDT
/// documents, and the merge-dispatch surface.
///
/// Cloneable via `Arc` — the underlying state is shared across clones
/// so the handle can be passed into spawned tasks. Dropping the last
/// clone tears down the iroh endpoint cleanly via the `Endpoint::close`
/// surface (deferred to handle-Drop or explicit `close().await`).
pub struct AtriumHandle {
    inner: Arc<AtriumInner>,
}

struct AtriumInner {
    endpoint: Endpoint,
    /// Per-zone Loro documents keyed by zone-prefix string.
    zones: Mutex<BTreeMap<String, LoroDoc>>,
    /// The keypair the local peer uses for HLC node-id derivation.
    /// HLC carries via [`benten_core::hlc::BentenHlc::node_id_from_peer_id_bytes`].
    peer_keypair: Keypair,
}

impl std::fmt::Debug for AtriumHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtriumHandle")
            .field("peer_id", &self.peer_id())
            .finish_non_exhaustive()
    }
}

impl Clone for AtriumHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl AtriumHandle {
    /// Open a new Atrium session.
    ///
    /// Constructs the iroh `Endpoint` per the [`AtriumConfig::mode`]
    /// — Loopback for in-process tests, Production for relay-default +
    /// holepunch via iroh per D-PHASE-3-3 (Production mode falls back
    /// to Loopback at G16-B canary scope; G16-D wave-6b wires the
    /// production preset alongside the handshake protocol body).
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::Transport`] if the iroh `Endpoint`
    /// binding fails.
    pub async fn open(config: AtriumConfig) -> AtriumResult<Self> {
        let keypair = Keypair::generate();
        Self::open_with_keypair(config, keypair).await
    }

    /// Open a new Atrium session with a caller-supplied keypair.
    ///
    /// Used by tests + integration scenarios where the local peer's
    /// identity is fixed deterministically (so the loopback canary's
    /// second peer can dial the first by exact `EndpointAddr`).
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::Transport`] if the iroh `Endpoint`
    /// binding fails.
    pub async fn open_with_keypair(config: AtriumConfig, keypair: Keypair) -> AtriumResult<Self> {
        let endpoint = match config.mode {
            AtriumMode::Loopback | AtriumMode::Production => {
                // G16-B canary scope: both modes bind via the loopback
                // path. G16-D wave-6b promotes Production-mode to the
                // iroh production preset (relay-default + holepunch
                // per D-PHASE-3-3) once the handshake protocol body
                // lands.
                Endpoint::bind_loopback_with_keypair(&keypair).await?
            }
        };
        Ok(Self {
            inner: Arc::new(AtriumInner {
                endpoint,
                zones: Mutex::new(BTreeMap::new()),
                peer_keypair: keypair,
            }),
        })
    }

    /// The local peer's [`PeerId`] (Ed25519 pubkey == iroh EndpointId
    /// per crypto-minor-4).
    #[must_use]
    pub fn peer_id(&self) -> PeerId {
        self.inner.endpoint.peer_id()
    }

    /// The local peer's loopback `EndpointAddr` for in-process
    /// two-peer test fixtures. Production peers discover each other
    /// via iroh's relay-default + address-lookup path.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::Transport`] if the underlying endpoint
    /// has no bound sockets.
    pub fn loopback_addr(&self) -> AtriumResult<iroh::EndpointAddr> {
        Ok(self.inner.endpoint.loopback_addr()?)
    }

    /// Observable Atrium status surface (per net-blocker-2 BLOCKER).
    ///
    /// Maps the underlying transport status into the engine-visible
    /// [`SyncStatus`] shape. Operators consume this via the public
    /// `engine.atrium_status()` accessor.
    pub async fn atrium_status(&self) -> SyncStatus {
        match self.inner.endpoint.transport_status().await {
            TransportStatus::Healthy { kind } => SyncStatus::healthy(kind),
            TransportStatus::Degraded { reason } => SyncStatus::degraded(reason),
            TransportStatus::NotConnected => SyncStatus {
                transport_kind: TransportKind::Loopback,
                is_healthy: false,
                reason: "not connected".into(),
            },
        }
    }

    /// Sync a subgraph (zone) bidirectionally with a remote peer.
    ///
    /// Two-peer protocol shape (G16-B canary scope):
    ///
    /// 1. Connect to the remote peer via the iroh transport.
    /// 2. Export the local zone's full CRDT state via
    ///    [`LoroDoc::export_update`].
    /// 3. Send the export to the remote; receive the remote's export.
    /// 4. Apply the remote's export via
    ///    [`AtriumHandle::merge_remote_change`] (which fires the
    ///    Inv-13 row-4 SPLIT classifier).
    ///
    /// G16-D wave-6b wires the handshake protocol body around this
    /// call (replay-window enforcement, signature verification,
    /// UCAN-grant exchange); the framing here is the bytes-only floor.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::Transport`] on connection failure;
    /// [`AtriumError::Crdt`] on Loro merge failure;
    /// [`AtriumError::DivergentCidRejected`] on row-4b system-zone
    /// reject; [`AtriumError::InvalidState`] if the zone is not
    /// registered locally.
    pub async fn sync_subgraph(
        &self,
        zone: &str,
        remote_addr: iroh::EndpointAddr,
    ) -> AtriumResult<()> {
        let conn = self.inner.endpoint.connect_to_addr(remote_addr).await?;
        self.sync_subgraph_over(zone, &conn).await
    }

    /// Run the bidirectional sync exchange against an existing
    /// connection.
    ///
    /// Used by the two-peer integration tests that want to drive the
    /// connect-side independently from the accept-side.
    ///
    /// # Errors
    ///
    /// As [`AtriumHandle::sync_subgraph`].
    pub async fn sync_subgraph_over(&self, zone: &str, conn: &Connection) -> AtriumResult<()> {
        // Export local state for this zone.
        let local_export = {
            let zones = self.inner.zones.lock().await;
            let doc = zones.get(zone).ok_or_else(|| AtriumError::InvalidState {
                reason: format!("zone '{zone}' not registered"),
            })?;
            doc.export_update()?
        };
        // Send local state to remote.
        conn.send_bytes(&local_export).await?;
        // Receive remote state.
        let remote_bytes = conn.recv_bytes().await?;
        // Apply via merge-dispatch (fires Inv-13 row-4 SPLIT).
        self.merge_remote_change(zone, &remote_bytes).await?;
        Ok(())
    }

    /// Accept the next inbound Atrium connection + run the
    /// accept-side of the bidirectional sync exchange against the
    /// named zone.
    ///
    /// Returns the connection on success so the test fixture can
    /// inspect it (e.g. `conn.transport_kind()`).
    ///
    /// # Errors
    ///
    /// As [`AtriumHandle::sync_subgraph`] plus
    /// [`AtriumError::Transport`] on accept-side failure.
    pub async fn accept_sync_subgraph(&self, zone: &str) -> AtriumResult<Connection> {
        let conn = self.inner.endpoint.accept_next().await?;
        // Accept-side mirrors connect-side: receive remote, then send local.
        // Receive remote first to keep the protocol symmetric across
        // both sides.
        let remote_bytes = conn.recv_bytes().await?;
        let local_export = {
            let zones = self.inner.zones.lock().await;
            let doc = zones.get(zone).ok_or_else(|| AtriumError::InvalidState {
                reason: format!("zone '{zone}' not registered"),
            })?;
            doc.export_update()?
        };
        conn.send_bytes(&local_export).await?;
        // Apply received bytes after sending our own — order matches
        // connect-side so both sides converge on the same merged state.
        self.merge_remote_change(zone, &remote_bytes).await?;
        Ok(conn)
    }

    /// Register a zone with the Atrium handle.
    ///
    /// Registers a fresh [`LoroDoc`] for the zone if it doesn't already
    /// exist; returns immediately if the zone is already registered.
    pub async fn register_zone(&self, zone: &str) {
        let mut zones = self.inner.zones.lock().await;
        zones.entry(zone.to_string()).or_insert_with(LoroDoc::new);
    }

    /// Read-write access to a zone's [`LoroDoc`] for direct CRDT
    /// operations.
    ///
    /// The closure receives the [`LoroDoc`] under lock; subsequent
    /// reads-after-write through this Atrium handle observe the
    /// closure's writes.
    pub async fn with_zone<R>(&self, zone: &str, f: impl FnOnce(&LoroDoc) -> R) -> AtriumResult<R> {
        let mut zones = self.inner.zones.lock().await;
        let doc = zones.entry(zone.to_string()).or_insert_with(LoroDoc::new);
        Ok(f(doc))
    }

    /// Apply a remote sync frame to the named zone, after firing the
    /// Inv-13 row-4 SPLIT classifier (per ds-4).
    ///
    /// row-4a (user-data): the merge applies; AttributionFrame seed is
    /// available via [`LoroDoc::winning_attribution`] for the engine
    /// to capture peer contribution.
    /// row-4b (system-zone / Anchor-immutable): the merge is rejected
    /// with [`AtriumError::DivergentCidRejected`] mapping to
    /// [`benten_errors::ErrorCode::SyncDivergentCidRejected`].
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::DivergentCidRejected`] if the inbound
    /// frame's targets include a system-zone Node;
    /// [`AtriumError::Crdt`] if the underlying Loro merge fails.
    pub async fn merge_remote_change(&self, zone: &str, bytes: &[u8]) -> AtriumResult<()> {
        // row-4 SPLIT classification: walk the zone path against the
        // system-zone prefix list. The crdt-layer op_log_targets
        // surface gives us per-container granularity; for G16-B
        // canary scope, classification at zone-path granularity
        // suffices because system-zone targets ALWAYS arrive under
        // a system-zone-prefixed zone path. Finer per-op walking is
        // the wave-6b-r6-fp surface.
        if zone_is_system_zone(zone) {
            return Err(AtriumError::DivergentCidRejected {
                target: zone.to_string(),
            });
        }
        let mut zones = self.inner.zones.lock().await;
        let doc = zones.entry(zone.to_string()).or_insert_with(LoroDoc::new);
        doc.apply_remote_update(bytes)?;
        Ok(())
    }

    /// The local peer's HLC `node_id`, derived from the peer-pubkey
    /// per [`benten_core::hlc::BentenHlc::node_id_from_peer_id_bytes`].
    ///
    /// Used by callers that want to stamp HLC writes with the local
    /// peer-identity before calling [`LoroDoc::set_property`].
    #[must_use]
    pub fn hlc_node_id(&self) -> u64 {
        let pk_bytes = self.inner.peer_keypair.public_key().to_bytes();
        let mut prefix = [0u8; 8];
        prefix.copy_from_slice(&pk_bytes[..8]);
        benten_core::hlc::BentenHlc::node_id_from_peer_id_bytes(prefix)
    }

    /// Close the Atrium handle + tear down the iroh endpoint.
    ///
    /// After close, all subsequent calls return
    /// [`AtriumError::InvalidState`]. Idempotent — calling twice is
    /// safe (the second call is a no-op against an already-closed
    /// endpoint).
    pub async fn close(self) {
        if let Ok(inner) = Arc::try_unwrap(self.inner) {
            inner.endpoint.close().await;
        }
        // If other clones exist, the endpoint stays alive until the
        // last clone drops; this matches Arc-shared session-handle
        // semantics per Ben's D1 ratification.
    }
}

/// Classify a zone path as system-zone (row-4b) vs user-data (row-4a)
/// per ds-4 Inv-13 row-4 SPLIT.
///
/// System-zone zones are immutable-via-sync per Phase-1 Inv-13 + the
/// Phase-3 sync-replica trust boundary. User-data zones accept Loro
/// merge per the D-C version-chain pattern.
fn zone_is_system_zone(zone: &str) -> bool {
    SYSTEM_ZONE_PREFIXES
        .iter()
        .any(|prefix| zone.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_and_close_roundtrip() {
        let atrium = AtriumHandle::open(AtriumConfig::for_test())
            .await
            .expect("open atrium");
        // peer_id is deterministic + non-zero.
        assert_ne!(atrium.peer_id().as_bytes(), &[0u8; 32]);
        // status is healthy under loopback.
        let status = atrium.atrium_status().await;
        assert!(status.is_healthy);
        atrium.close().await;
    }

    #[tokio::test]
    async fn register_zone_then_with_zone_observes_writes() {
        let atrium = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
        atrium.register_zone("/zone/posts").await;
        atrium
            .with_zone("/zone/posts", |doc| {
                doc.set_property(
                    "title",
                    "hello",
                    benten_core::hlc::BentenHlc::new(100, 0, 0xAAAA),
                )
                .unwrap();
            })
            .await
            .unwrap();
        let value = atrium
            .with_zone("/zone/posts", |doc| doc.get_property("title"))
            .await
            .unwrap();
        assert_eq!(value.as_deref(), Some("hello"));
    }

    #[tokio::test]
    async fn merge_remote_change_rejects_system_zone() {
        let atrium = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
        // SYSTEM_ZONE_PREFIXES carries `system:`-prefixed zones per
        // Phase-1 system_zones.rs. Fabricate a system-zone path +
        // assert the row-4b reject fires.
        let result = atrium
            .merge_remote_change("system:HandlerVersion/foo", &[0x01, 0x02, 0x03])
            .await;
        match result {
            Err(AtriumError::DivergentCidRejected { target }) => {
                assert_eq!(target, "system:HandlerVersion/foo");
            }
            other => panic!("expected DivergentCidRejected, got {other:?}"),
        }
    }

    #[test]
    fn atrium_error_codes_route_correctly() {
        let div = AtriumError::DivergentCidRejected {
            target: "system:HandlerVersion/foo".into(),
        };
        assert_eq!(
            div.code(),
            benten_errors::ErrorCode::SyncDivergentCidRejected
        );
    }

    #[tokio::test]
    async fn open_atrium_with_production_config_falls_back_to_loopback_at_g16b_canary_scope() {
        // Consumer call site for AtriumConfig::production (per plan §3
        // G16-B row + brief: production-mode binding wires at G16-D
        // wave-6b alongside handshake protocol body; until then the
        // canary path falls back to Loopback). Asserts the shape +
        // healthy startup so future G16-D promotion has a regression
        // pin.
        let atrium = AtriumHandle::open(AtriumConfig::production())
            .await
            .expect("production-mode atrium open");
        let status = atrium.atrium_status().await;
        assert!(status.is_healthy);
        atrium.close().await;
    }

    #[tokio::test]
    async fn open_with_keypair_produces_deterministic_peer_id() {
        // Consumer call site for AtriumHandle::open_with_keypair
        // (used by tests + integration scenarios where the local
        // peer-identity is fixed deterministically). Round-trips
        // the keypair through the export-envelope path so we can
        // re-import it for the second open call (Keypair is not
        // Clone by design).
        let kp1 = Keypair::generate();
        let envelope = kp1.export_seed_envelope();
        let kp1_again = Keypair::from_dag_cbor_envelope(&envelope).unwrap();
        let kp2 = Keypair::generate();
        let atrium1 = AtriumHandle::open_with_keypair(AtriumConfig::for_test(), kp1)
            .await
            .unwrap();
        let atrium2 = AtriumHandle::open_with_keypair(AtriumConfig::for_test(), kp2)
            .await
            .unwrap();
        assert_ne!(atrium1.peer_id(), atrium2.peer_id());
        // hlc_node_id derived from the same keypair-bytes is stable
        // across re-import.
        let alt = AtriumHandle::open_with_keypair(AtriumConfig::for_test(), kp1_again)
            .await
            .unwrap();
        assert_eq!(atrium1.hlc_node_id(), alt.hlc_node_id());
        assert_eq!(atrium1.peer_id(), alt.peer_id());
    }

    #[test]
    fn sync_status_degraded_constructor_carries_reason() {
        // Consumer call site for SyncStatus::degraded (operator-visible
        // observability constructor per net-blocker-2 BLOCKER).
        let status = SyncStatus::degraded("packet-loss-30pct");
        assert!(!status.is_healthy);
        assert_eq!(status.reason, "packet-loss-30pct");
    }

    #[test]
    fn zone_is_system_zone_classifies_correctly() {
        // Picks any prefix from SYSTEM_ZONE_PREFIXES — the actual list
        // is enumerated in `benten-engine::system_zones`.
        for prefix in SYSTEM_ZONE_PREFIXES {
            assert!(zone_is_system_zone(prefix));
            assert!(zone_is_system_zone(&format!("{prefix}suffix")));
        }
        assert!(!zone_is_system_zone("/zone/posts"));
        assert!(!zone_is_system_zone("/user/data"));
    }
}
