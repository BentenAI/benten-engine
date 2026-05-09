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
//!   applies via [`AtriumHandle::merge_remote_change`]; the merge
//!   surfaces an `benten_eval::AttributionFrame` seed populated from
//!   [`benten_sync::crdt::LoroDoc::winning_attribution`] (G16-B canary
//!   landed) — engine-side mint of a new Version Node via the existing
//!   Anchor + Version + CURRENT pattern (per arch-r1-4 D-C HYBRID)
//!   wires through G16-B wave-6b (post-canary; the AttributionFrame
//!   structural fields `peer_did_set` + `device_did` + `sync_hop_depth`
//!   are added at G16-B canary so wave-6b can populate them).
//! - **row-4b** (system-zone / Anchor-immutable): the targets fall
//!   inside the system-zone prefix list. The merge is REJECTED with
//!   [`AtriumError::DivergentCidRejected`] mapping to the stable
//!   error code [`benten_errors::ErrorCode::SyncDivergentCidRejected`].
//!
//! ## State at HEAD (G16-B canary)
//!
//! - Landed: [`AtriumHandle::open`] / [`AtriumHandle::merge_remote_change`]
//!   row-4 SPLIT classifier; [`benten_sync::crdt::LoroDoc::winning_attribution`]
//!   accessor surfacing contributing peer node-ids; sync-hop-depth
//!   bound check at the merge seam (rejects with
//!   [`benten_errors::ErrorCode::SyncHopDepthExceeded`] when the
//!   incoming frame's hop-depth would exceed
//!   [`benten_eval::exec_state::SYNC_HOP_DEPTH_CAP`]).
//! - Pinned-deferred (G16-B wave-6b post-canary): engine-side Version
//!   Node mint via [`crate::Engine::create_anchor`] +
//!   `append_version` (currently Phase-1 stubs returning
//!   `E_NOT_IMPLEMENTED` — wiring lands alongside the broader
//!   anchor-store wave). The structural `benten_eval::AttributionFrame` surface
//!   that wave-6b populates is shipped at G16-B canary so the wiring
//!   has an existing carrier shape.
//!
//! ## Pin sources
//!
//! - plan §3 G16-B row.
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_open_close_lifecycle` +
//!   `atrium_sync_subgraph_two_peer_bidirectional`.
//! - `D-PHASE-3-22` RESOLVED + `arch-r1-4` D-C HYBRID.
//! - `ds-4` Inv-13 row-4 SPLIT.
//! - `ds-r4b-1` BLOCKER (R4b round 1 distributed-systems lens) —
//!   AttributionFrame field-layer extension at sync boundary.
//! - `arch-r4b-1` (R4b round 1 architect lens) — module rustdoc retense
//!   distinguishing landed-state from pinned-deferred-state.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

    /// G16-B canary (ds-r4b-1 BLOCKER closure): an inbound CRDT merge
    /// would push the resulting `AttributionFrame::sync_hop_depth` past
    /// [`benten_eval::exec_state::SYNC_HOP_DEPTH_CAP`]. Maps to
    /// [`benten_errors::ErrorCode::SyncHopDepthExceeded`].
    #[error(
        "atrium sync hop-depth bound exceeded: incoming_hop_depth={incoming_hop_depth} cap={cap}"
    )]
    SyncHopDepthExceeded {
        /// Hop-depth carried by the incoming frame.
        incoming_hop_depth: u32,
        /// The configured cap.
        cap: u32,
    },
}

/// Phase-3 G16-B canary (ds-r4b-1 BLOCKER closure): the
/// AttributionFrame seed surfaced by a successful row-4a Loro merge.
///
/// Composes with the engine-side Version Node mint (G16-B wave-6b
/// post-canary) to populate the new Version's
/// [`benten_eval::AttributionFrame`] with `peer_did_set` (after engine-
/// side trust-store resolution of the `peer_node_ids` to peer-DIDs) +
/// `sync_hop_depth` (carried verbatim).
#[derive(Debug, Clone)]
pub struct SyncMergeAttribution {
    /// Contributing peer node-ids observed via
    /// [`benten_sync::crdt::LoroDoc::winning_attribution`]. The engine
    /// resolves these to `did:key:` DIDs via the local trust-store at
    /// G16-B wave-6b — pre-trust-store the engine emits the raw
    /// node-ids serialised as decimal strings into
    /// [`benten_eval::AttributionFrame::peer_did_set`].
    pub peer_node_ids: std::collections::BTreeSet<u64>,
    /// New `sync_hop_depth` for the AttributionFrame minted at this
    /// merge boundary (= incoming_hop_depth + 1, bounded by
    /// [`benten_eval::exec_state::SYNC_HOP_DEPTH_CAP`]).
    pub sync_hop_depth: u32,
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
            AtriumError::InvalidState { .. } => benten_errors::ErrorCode::AtriumInactive,
            AtriumError::SyncHopDepthExceeded { .. } => {
                benten_errors::ErrorCode::SyncHopDepthExceeded
            }
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
    /// G21-T2 §D audit-6-3 wireup hook — declared device-attestation
    /// envelopes to present at handshake time, keyed by `device_did`.
    /// Populated by [`AtriumHandle::register_device_attestation`].
    /// Frame emission at peer-handshake time wires through the
    /// `crates/benten-sync/src/handshake_wire.rs::HandshakeFrame` builder
    /// once the broader G16-D wave-6b handshake protocol body lands.
    declared_device_attestations: Mutex<BTreeMap<String, DeclaredDeviceAttestation>>,
    /// Phase-3 G16-B-prime (§6.12 deferred surface decision #3): trust-
    /// store mapping from CRDT peer node-id → resolved peer-DID. The
    /// engine populates this map as remote peers handshake (the
    /// handshake protocol body work lands at G16-D wave-6b; until then
    /// callers register entries explicitly via
    /// [`AtriumHandle::register_peer_did`]).
    ///
    /// On a successful row-4a merge, [`AtriumHandle::resolve_peer_dids`]
    /// translates the [`SyncMergeAttribution::peer_node_ids`] set to
    /// peer-DIDs by walking this map; unresolved node-ids fall back to
    /// `node-id:NNN` synthetic strings so the AttributionFrame still
    /// carries provenance per pim-2 end-to-end pin discipline.
    peer_did_registry: Mutex<BTreeMap<u64, String>>,
    /// Phase-3 G16-B-prime (§6.12 item 7): peer-churn lifecycle flag.
    ///
    /// `true` after [`AtriumHandle::open`] / [`AtriumHandle::open_with_keypair`]
    /// + after [`AtriumHandle::rejoin`]; `false` after
    /// [`AtriumHandle::leave`]. While `false`, sync-touching surfaces
    /// (`sync_subgraph` / `accept_sync_subgraph` / `merge_remote_change`)
    /// reject with [`AtriumError::InvalidState`]. The iroh endpoint
    /// stays bound across `leave()`/`rejoin()` cycles so the same handle
    /// can resume without rebinding the underlying socket.
    ///
    /// Idempotency contract: `leave()` on an already-inactive handle is
    /// a no-op; `rejoin()` on an already-active handle is a no-op. The
    /// Loro CRDT state in `zones` survives across the
    /// leave-rejoin window so post-rejoin merges reconcile via
    /// CRDT-natural delta-state replay.
    is_active: AtomicBool,
}

/// Phase-3 G21-T2 §D — declared device-attestation envelope recorded
/// on an [`AtriumHandle`]. Round-trip surface so the
/// [`AtriumHandle::register_device_attestation`] caller can list /
/// inspect declared envelopes; the on-the-wire emission to peer
/// handshakes wires through G16-D wave-6b.
#[derive(Clone, Debug)]
pub struct DeclaredDeviceAttestation {
    /// `did:key:...` identifier of the declaring device.
    pub device_did: String,
    /// Per-claim capabilities the device may exercise.
    pub claims: Vec<DeclaredCapabilityClaim>,
    /// TTL in seconds before the attestation must be re-declared.
    pub freshness_window: u32,
}

/// One capability claim inside a [`DeclaredDeviceAttestation`].
#[derive(Clone, Debug)]
pub struct DeclaredCapabilityClaim {
    /// Path-glob the claim applies to.
    pub path: String,
    /// Ability the claim grants (e.g. `read` / `write` / `emit`).
    pub ability: String,
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
                declared_device_attestations: Mutex::new(BTreeMap::new()),
                peer_did_registry: Mutex::new(BTreeMap::new()),
                is_active: AtomicBool::new(true),
            }),
        })
    }

    /// Phase-3 G21-T2 §D audit-6-3 — record a declared
    /// device-attestation envelope on this handle. The envelope is
    /// presented at peer-handshake time so peers observe the local
    /// device's capability declaration on the wire.
    ///
    /// G21-T2 scope: the recording surface lands here so the napi
    /// `JsAtrium.declareDeviceAttestation` shim can forward to it.
    /// The on-the-wire emission of these envelopes via
    /// `HandshakeFrame::device_did` decoration wires through G16-D
    /// wave-6b's broader handshake protocol body work
    /// (`crates/benten-sync/src/handshake.rs`); pre-G16-D-wave-6b
    /// the recording is observable via
    /// [`AtriumHandle::list_declared_device_attestations`] but does
    /// not yet ride the wire frame.
    pub async fn register_device_attestation(&self, attestation: DeclaredDeviceAttestation) {
        let mut tbl = self.inner.declared_device_attestations.lock().await;
        tbl.insert(attestation.device_did.clone(), attestation);
    }

    /// List declared device-attestation envelopes recorded on this
    /// handle. Round-trip companion to
    /// [`AtriumHandle::register_device_attestation`].
    pub async fn list_declared_device_attestations(&self) -> Vec<DeclaredDeviceAttestation> {
        let tbl = self.inner.declared_device_attestations.lock().await;
        tbl.values().cloned().collect()
    }

    /// Phase-3 G16-B-prime (§6.12 deferred surface decision #3):
    /// register a peer-DID for a CRDT peer node-id.
    ///
    /// G16-B-prime scope: the registration is a manual hook — tests
    /// + integration scenarios populate the trust-store explicitly so
    /// the merge-callback path in
    /// [`crate::Engine::apply_atrium_merge`] resolves
    /// [`SyncMergeAttribution::peer_node_ids`] to real DID strings
    /// (vs the `node-id:NNN` fallback). G16-D wave-6b's handshake
    /// protocol body wires the production path: as remote peers
    /// complete handshake, the handshake body inserts the
    /// `peer_node_id → did:key:...` mapping here without manual
    /// registration.
    pub async fn register_peer_did(&self, peer_node_id: u64, did: impl Into<String>) {
        let mut reg = self.inner.peer_did_registry.lock().await;
        reg.insert(peer_node_id, did.into());
    }

    /// Phase-3 G16-B-prime: resolve a set of CRDT peer node-ids to
    /// peer-DID strings.
    ///
    /// For each node-id: looks up the local trust-store
    /// (`peer_did_registry`); falls back to a synthetic
    /// `node-id:NNN` decimal-string if no resolved DID is registered.
    /// The fallback ensures the
    /// [`benten_eval::AttributionFrame::peer_did_set`] carries SOMETHING
    /// per the pim-2 end-to-end-pin discipline — defending against the
    /// failure mode where the AttributionFrame is left empty when the
    /// trust-store hasn't yet learned the peer.
    ///
    /// Returns a `BTreeSet<String>` so the AttributionFrame's
    /// canonical bytes are deterministic across calls.
    pub async fn resolve_peer_dids(
        &self,
        peer_node_ids: &std::collections::BTreeSet<u64>,
    ) -> std::collections::BTreeSet<String> {
        let reg = self.inner.peer_did_registry.lock().await;
        peer_node_ids
            .iter()
            .map(|nid| {
                reg.get(nid)
                    .cloned()
                    .unwrap_or_else(|| format!("node-id:{nid}"))
            })
            .collect()
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
        self.ensure_active("sync_subgraph")?;
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
        self.ensure_active("sync_subgraph_over")?;
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
        self.ensure_active("accept_sync_subgraph")?;
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
    /// available via [`benten_sync::crdt::LoroDoc::winning_attribution`]
    /// for the engine to capture peer contribution. The G16-B canary
    /// returns the seed CRDT-layer node-id set; the engine-side Version
    /// Node mint that consumes the seed lands at G16-B wave-6b
    /// post-canary (per `arch-r4b-1` rustdoc retense at module level).
    /// row-4b (system-zone / Anchor-immutable): the merge is rejected
    /// with [`AtriumError::DivergentCidRejected`] mapping to
    /// [`benten_errors::ErrorCode::SyncDivergentCidRejected`].
    ///
    /// G16-B canary additionally enforces the sync-hop-depth bound at
    /// the merge seam — see [`AtriumHandle::merge_remote_change_with_hop_depth`]
    /// for the depth-aware variant. The plain entry point preserves
    /// the Phase-2b call shape and is equivalent to passing
    /// `incoming_hop_depth = 0` (i.e. fresh-from-source).
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::DivergentCidRejected`] if the inbound
    /// frame's targets include a system-zone Node;
    /// [`AtriumError::Crdt`] if the underlying Loro merge fails.
    pub async fn merge_remote_change(&self, zone: &str, bytes: &[u8]) -> AtriumResult<()> {
        self.merge_remote_change_with_hop_depth(zone, bytes, 0)
            .await
            .map(|_seed| ())
    }

    /// Apply a remote sync frame with an explicit incoming
    /// `sync_hop_depth` per D-PHASE-3-25 sync-hop-depth-bounded
    /// contract (G16-B canary; ds-r4b-1 BLOCKER closure).
    ///
    /// Returns the [`SyncMergeAttribution`] seed surfacing contributing
    /// peer node-ids + the new hop-depth (incoming + 1) that the engine
    /// consumes to populate `benten_eval::AttributionFrame::peer_did_set`
    /// + `benten_eval::AttributionFrame::sync_hop_depth` when minting a
    /// new Version Node post-merge.
    ///
    /// Bounds: rejects with
    /// [`benten_errors::ErrorCode::SyncHopDepthExceeded`] when the
    /// resulting hop-depth would exceed
    /// [`benten_eval::exec_state::SYNC_HOP_DEPTH_CAP`] (default 8).
    ///
    /// # Errors
    ///
    /// As [`AtriumHandle::merge_remote_change`] plus
    /// [`AtriumError::SyncHopDepthExceeded`] when the bound is hit.
    pub async fn merge_remote_change_with_hop_depth(
        &self,
        zone: &str,
        bytes: &[u8],
        incoming_hop_depth: u32,
    ) -> AtriumResult<SyncMergeAttribution> {
        // §6.12 item 7 peer-churn lifecycle: reject inbound merges
        // when the handle is in the post-`leave()` quiesced state. The
        // check fires BEFORE row-4 SPLIT so a leave-then-merge attempt
        // does not mutate the Loro doc state.
        self.ensure_active("merge_remote_change_with_hop_depth")?;
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
        // G16-B canary: enforce sync-hop-depth bound BEFORE the merge.
        // checked_add(1) = None on u32::MAX overflow; > cap = explicit
        // reject. The pre-merge order ensures a rejected frame leaves
        // the doc state unchanged (ds-r4b-1 closure semantics).
        let next_depth =
            incoming_hop_depth
                .checked_add(1)
                .ok_or(AtriumError::SyncHopDepthExceeded {
                    incoming_hop_depth,
                    cap: benten_eval::exec_state::SYNC_HOP_DEPTH_CAP,
                })?;
        if next_depth > benten_eval::exec_state::SYNC_HOP_DEPTH_CAP {
            return Err(AtriumError::SyncHopDepthExceeded {
                incoming_hop_depth,
                cap: benten_eval::exec_state::SYNC_HOP_DEPTH_CAP,
            });
        }
        let mut zones = self.inner.zones.lock().await;
        let doc = zones.entry(zone.to_string()).or_insert_with(LoroDoc::new);
        doc.apply_remote_update(bytes)?;
        // Capture the AttributionFrame seed: contributing peer node-ids
        // observed via Loro's per-container winning-attribution view +
        // the new sync-hop-depth. Engine-side wave-6b consumes this
        // seed when minting a new Version Node post-merge.
        let peer_node_ids = doc.winning_attribution();
        Ok(SyncMergeAttribution {
            peer_node_ids,
            sync_hop_depth: next_depth,
        })
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

    /// Phase-3 §6.12 item 7: observable accessor for the
    /// `is_active` lifecycle flag.
    ///
    /// Returns `true` when the handle is participating in Atrium sync
    /// (post-`open` / post-`rejoin`); `false` after a `leave()` until the
    /// next `rejoin()`. Operators consume this to gate UI affordances
    /// + observability dashboards on the peer-churn lifecycle state.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.is_active.load(Ordering::SeqCst)
    }

    /// Phase-3 §6.12 item 7: gate sync-touching surfaces on the
    /// `is_active` flag. Returns [`AtriumError::InvalidState`] when the
    /// handle is in the post-`leave()` quiesced state.
    fn ensure_active(&self, op: &'static str) -> AtriumResult<()> {
        if self.inner.is_active.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(AtriumError::InvalidState {
                reason: format!("atrium handle is inactive (post-leave); {op} requires rejoin()"),
            })
        }
    }

    /// Phase-3 §6.12 item 7: non-consuming graceful tear-down.
    ///
    /// Flips the [`AtriumHandle::is_active`] flag to `false`. While
    /// inactive, the sync-touching surfaces
    /// ([`AtriumHandle::sync_subgraph`] / [`AtriumHandle::accept_sync_subgraph`]
    /// / [`AtriumHandle::merge_remote_change`] /
    /// [`AtriumHandle::merge_remote_change_with_hop_depth`]) return
    /// [`AtriumError::InvalidState`] (mapped to `E_ATRIUM_INACTIVE`)
    /// without touching the underlying transport. Inbound merges are
    /// gated at the merge seam by `ensure_active`; outbound publish /
    /// share / close-share paths are likewise gated.
    ///
    /// The iroh endpoint stays bound + the per-zone Loro documents
    /// survive in-memory so [`AtriumHandle::rejoin`] can resume on
    /// the same handle without re-binding the underlying socket. This
    /// is the peer-churn-friendly counterpart to the consuming
    /// [`AtriumHandle::close`] surface.
    ///
    /// Idempotent: a `leave()` on an already-inactive handle is a no-op.
    ///
    /// # Outbound subscription drop scope (current limitation)
    ///
    /// `engine_sync.rs` does NOT today carry a per-handle outbound
    /// subscription registry — the eval-side `ON_CHANGE_REGISTRY`
    /// (`crates/benten-eval/src/primitives/subscribe.rs`) is process-
    /// scoped (`LazyLock<Mutex<HashMap>>`), shared across all engine
    /// instances in the process. `leave()` therefore CANNOT
    /// independently revoke outbound `Engine::subscribe_change_events`
    /// registrations bound through the eval-side registry — the flag-
    /// flip only gates the Atrium-handle-owned sync surfaces.
    ///
    /// In-process subscription cross-talk is mitigated separately by
    /// the F6 `is_actor_active` cap-recheck at SUBSCRIBE delivery time
    /// (per phase-3-backlog §2.2 / §3.2), which auto-cancels deliveries
    /// to revoked actors. That guard fires on cap revocation, NOT on
    /// lifecycle changes — so a `leave()`d handle whose actor is still
    /// active will still observe its eval-side subscriptions firing
    /// (until the per-engine subscription registry refactor lands per
    /// §6.12 item 8 option-(b)).
    ///
    /// # Errors
    ///
    /// Currently infallible (returns `Ok(())` always); the result-shape
    /// is preserved for future versions that may surface drain-failure
    /// reasons (e.g. an outbound subscription that refused to release
    /// its registration cleanly once item 8 option-(b) lands).
    #[allow(clippy::unused_async)]
    pub async fn leave(&self) -> AtriumResult<()> {
        // SeqCst per the §6.12 item 7 contract — the flag transition
        // strictly precedes any subsequent merge-time check across
        // arbitrary task scheduling.
        self.inner.is_active.store(false, Ordering::SeqCst);
        // Per the rustdoc above: the flag-flip alone gates the Atrium-
        // handle-owned sync surfaces (inbound merge + outbound publish/
        // share/close-share). It does NOT touch the eval-side
        // `ON_CHANGE_REGISTRY` (process-scoped); per-handle outbound
        // subscription drop lands when §6.12 item 8 option-(b) lifts the
        // registry to engine-instance scope.
        Ok(())
    }

    /// Phase-3 §6.12 item 7: idempotent re-establishment on the same
    /// handle.
    ///
    /// Flips the [`AtriumHandle::is_active`] flag back to `true`,
    /// re-enabling the sync-touching surfaces. The iroh endpoint stays
    /// bound across `leave()` / `rejoin()` cycles; the per-zone Loro
    /// documents survive in-memory across the leave-rejoin window so
    /// the next inbound merge reconciles state via Loro's natural
    /// delta-state replay (CRDT idempotency under repeated application
    /// of the same bytes).
    ///
    /// Continuity guarantee: the trust-store
    /// (`peer_did_registry`) + the declared device-attestation table
    /// + the per-zone Loro state ALL survive across `leave()` /
    /// `rejoin()`. An `AttributionFrame::peer_did_set` minted on a
    /// post-rejoin merge therefore observes the same peer-DID
    /// resolutions as a pre-leave merge would have — preserving causal
    /// history continuity per the R4b dist-systems lens carry.
    ///
    /// Idempotent: a `rejoin()` on an already-active handle is a no-op.
    ///
    /// # Errors
    ///
    /// Currently infallible (returns `Ok(())` always); the result-shape
    /// is preserved for future versions that may surface re-bind
    /// failure (e.g. when a future `close()`-then-`rejoin()` sequence
    /// has to re-bind the iroh endpoint).
    #[allow(clippy::unused_async)]
    pub async fn rejoin(&self) -> AtriumResult<()> {
        self.inner.is_active.store(true, Ordering::SeqCst);
        Ok(())
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
