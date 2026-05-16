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

use benten_core::hlc::Hlc;
use benten_id::keypair::Keypair;
use benten_sync::crdt::{CrdtError, LoroDoc};
use benten_sync::peer_id::PeerId;
use benten_sync::transport::{Connection, Endpoint, TransportKind, TransportStatus};

/// Phase-3 R6-FP Wave-C1 (ds-r6-1 closure): system-time-backed
/// physical-clock callback for `Hlc::update` at the Atrium row-4a
/// inbound-sync-frame skew classifier. Returns wall-clock milliseconds
/// since UNIX_EPOCH. Saturating-on-error so a clock that briefly
/// returns `Err` (e.g. cross-boundary into pre-1970 epochs) does not
/// poison the merge boundary; the saturating-zero floor pairs with
/// the asymmetric `Hlc::update` skew check (rejects future-skew but
/// never rejects past-skew per Kulkarni-Demirbas algorithm).
fn system_time_ms_for_atrium_hlc() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

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

    /// G16-D wave-6b fix-pass (cryptographic-attestation closure for
    /// criterion 16 per Ben ratification 2026-05-09): an inbound
    /// on-the-wire [`DeviceAttestationEnvelope`] failed cryptographic
    /// verification. Three failure modes surface this single variant:
    /// (a) device-DID forgery (envelope signature does not verify
    /// against the public key resolved from the declared
    /// `attestation.device_did`); (b) parent-attestation chain
    /// rejection via `benten_id::Acceptor::accept_at`; (c) frame-pair
    /// payload-hash binding violation (MITM swap defense).
    /// Maps to [`benten_errors::ErrorCode::DeviceAttestationForged`].
    #[error("atrium device-attestation envelope verification failed: {reason}")]
    DeviceAttestationForged {
        /// Operator-readable cause naming which of the three failure
        /// modes fired.
        reason: String,
    },

    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 / sec-r4r2-1 attack-vector closure):
    /// an inbound sync-replica row carried an HLC stamp whose
    /// `physical_ms` exceeded the local clock by more than the
    /// configured skew-tolerance window. Defends against an adversarial
    /// peer manipulating its local HLC to inject future-timestamped
    /// writes that would bias LWW resolution + forge revocation-vs-data
    /// ordering. Construction site at
    /// `crates/benten-engine/src/engine.rs::apply_atrium_merge`'s
    /// per-row [`Hlc::update`] verification loop. Carries observable
    /// diagnostic state so operators can distinguish skew detection
    /// from transport-level rejection. Maps to
    /// [`benten_errors::ErrorCode::SyncHlcDrift`]; routes to
    /// `ON_DENIED`.
    #[error(
        "atrium inbound HLC skew exceeded: remote_physical_ms={remote_physical_ms} \
         local_physical_ms={local_physical_ms} tolerance_ms={tolerance_ms}"
    )]
    HlcSkewExceeded {
        /// Local physical clock reading at the merge boundary.
        local_physical_ms: u64,
        /// Remote HLC `physical_ms` carried on the offending row.
        remote_physical_ms: u64,
        /// Configured skew-tolerance window in milliseconds.
        tolerance_ms: u64,
    },

    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 / sec-r4r2-1 attack-vector closure):
    /// an inbound MST-diff entry's payload bytes hashed to a CID
    /// different from the declared CID on the entry. Defends against
    /// MITM-crafted MST entries that pass transport-level structural
    /// checks but carry forged content under a legitimate CID. Maps to
    /// [`benten_errors::ErrorCode::SyncHashMismatch`]; routes to
    /// `ON_DENIED`.
    #[error(
        "atrium MST-diff entry CID-byte mismatch: declared={declared_cid} computed={computed_cid}"
    )]
    MstEntryCidByteMismatch {
        /// Declared CID rendered as hex.
        declared_cid: String,
        /// Computed CID (BLAKE3 of payload bytes) rendered as hex.
        computed_cid: String,
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
    /// Phase-3 G16-D wave-6b: device-DID-attestation observed on the
    /// wire envelope from the remote peer (the device that ORIGINATED
    /// the writes carried in this merge). `None` when the remote peer
    /// did not declare a device-DID at sync time (legacy / pre-G16-D
    /// peers). Consumed by [`crate::Engine::apply_atrium_merge`] to
    /// populate [`benten_eval::AttributionFrame::device_did`] with the
    /// originating-device identity rather than the local merging-engine's
    /// own device-CID. Closes plan §1 exit-criterion 16 (multi-device
    /// support for a single identity) by giving cross-device merges
    /// the device-grain provenance Inv-14 names.
    pub remote_device_did: Option<String>,
}

/// Phase-3 G16-D wave-6b: on-the-wire device-DID-attestation envelope.
///
/// Emitted by [`AtriumHandle::sync_subgraph`] /
/// [`AtriumHandle::accept_sync_subgraph`] BEFORE the Loro CRDT export
/// payload so the receiver can populate
/// [`benten_eval::AttributionFrame::device_did`] from the originating
/// device's declared identity (rather than the receiver's own
/// `device_cid`). The envelope is DAG-CBOR encoded for canonical-bytes
/// determinism + cross-platform parity with the rest of the on-wire
/// envelope shapes (e.g. [`benten_sync::handshake_wire::HandshakeFrame`]).
///
/// ## Cryptographic-attestation closure (V2; Phase-3 G16-D wave-6b fix-pass)
///
/// Per Ben ratification 2026-05-09, the wire envelope is signed +
/// replay-resistant + frame-pair-bound. Three composing surfaces:
///
/// 1. **DID forgery defense**: the embedded
///    `benten_id::DeviceAttestation` is signed by the parent-DID's
///    keypair (the user-identity issuing the device's capability
///    envelope). The envelope itself carries an additional
///    `envelope_signature` produced by the originating device's
///    keypair over the canonical bytes of `(version, attestation,
///    payload_hash, session_nonce)`. The receiver:
///    - Verifies `envelope_signature` against the public key resolved
///      from `attestation.device_did` (links the wire frame to the
///      keypair the attestation names — a peer cannot impersonate
///      another device's DID without holding that device's secret key).
///    - Verifies the embedded attestation via
///      `benten_id::Acceptor::accept_at` (parent signature, freshness
///      window, nonce-store replay defense, revocation list).
/// 2. **Replay defense**: each envelope carries a fresh 32-byte
///    `session_nonce` (independent of the attestation's parent-issued
///    nonce). The signed `envelope_signature` covers `session_nonce`,
///    so a captured envelope cannot be replayed verbatim against a
///    different sync session — the receiver-side `Acceptor::accept_at`
///    additionally rejects replay of the same parent-issued
///    attestation nonce.
/// 3. **Frame-pair binding defense**: the envelope's signed
///    `payload_hash` is `BLAKE3(loro_export_bytes)` for the Loro
///    payload that follows on the wire. The receiver computes the
///    BLAKE3 of the inbound Loro payload and rejects with
///    [`benten_errors::ErrorCode::DeviceAttestationForged`] if the
///    hashes differ — defends against a MITM swapping the payload
///    while preserving the envelope (or vice versa).
///
/// All three failure modes reject with the single typed code
/// [`benten_errors::ErrorCode::DeviceAttestationForged`] so audit
/// pipelines route on the wire-attestation boundary uniformly.
///
/// ## Backward-compat with `attestation = None`
///
/// Pre-G16-D-fp peers / handles with no declared device-attestation
/// emit `attestation = None` + an empty `envelope_signature`. Receivers
/// tolerate this for backward-compat with the G16-B-E / G16-D-pre-fp
/// shipped wire shape — the receiver-side
/// [`crate::Engine::apply_atrium_merge`] falls back to the local
/// engine's `device_cid` for the [`benten_eval::AttributionFrame::device_did`]
/// slot exactly as in V1. Production Atrium peers that have completed
/// device-DID enrollment SHOULD always emit `attestation = Some(_)`;
/// the heterogeneous-cap-envelope filter (phase-3-backlog §6.12 item 8)
/// keys per-zone write rejection on `attestation.is_some()` + the
/// envelope's declared capability scopes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceAttestationEnvelope {
    /// Wire-format version. `2` for G16-D wave-6b-fp-era envelopes
    /// (carries signed attestation + payload-hash + session-nonce +
    /// envelope-signature). `1` is the legacy shape carrying only
    /// `device_did: Option<String>`; a V2 receiver decodes V1 by
    /// treating the embedded `attestation` as `None`.
    pub version: u8,
    /// Signed parent → device-DID attestation, or `None` when the
    /// local handle has not been bound to a device-attestation envelope
    /// (pre-G16-D-fp legacy / test fixture path). Composes the existing
    /// hardened `benten_id::DeviceAttestation` + `Acceptor::accept_at`
    /// surface; not a parallel transport.
    pub attestation: Option<benten_id::device_attestation::DeviceAttestation>,
    /// `BLAKE3(loro_export_bytes)` for the Loro payload that follows
    /// this envelope on the wire. The signed `envelope_signature`
    /// covers this hash so a MITM cannot swap the payload while
    /// preserving the envelope. All-zero when `attestation` is `None`
    /// (the legacy fallback path is purely advisory; binding is moot
    /// when no attestation is asserted).
    pub payload_hash: [u8; 32],
    /// Fresh 32-byte session nonce (independent of the parent-issued
    /// attestation nonce). Defends against verbatim envelope replay.
    /// All-zero when `attestation` is `None`.
    pub session_nonce: [u8; 32],
    /// Ed25519 signature by the originating device's keypair over the
    /// canonical bytes of `(version, attestation, payload_hash,
    /// session_nonce)`. Verified at receive against the public key
    /// resolved from `attestation.device_did`. Empty `Vec` when
    /// `attestation` is `None`.
    pub envelope_signature: Vec<u8>,
}

impl DeviceAttestationEnvelope {
    /// Current wire-format version for the device-attestation envelope.
    /// V2 carries signed attestation + payload-hash + session-nonce +
    /// envelope-signature; V1 (legacy) carried only `device_did:
    /// Option<String>`.
    pub const WIRE_VERSION: u8 = 2;

    /// Maximum supported wire-format version at this build. Receivers
    /// tolerate `version <= MAX_WIRE_VERSION`; newer versions reject
    /// with [`AtriumError::InvalidState`] (caller must upgrade).
    ///
    /// ## V1 → V2 cross-version compat (ds-fp-mr-g16dw6b-MINOR-1)
    ///
    /// V2 introduces signed-attestation + payload-hash + session-nonce
    /// + envelope-signature fields without `#[serde(default)]` shims;
    /// canonical-bytes-decoding a V1-emitted envelope into a V2 struct
    /// would error at field absence. In practice this is harmless: V1
    /// was never deployed (the V1 → V2 promotion lands inside the
    /// SAME Phase-3 fix-pass that introduces the wire envelope; no
    /// V1 peer exists in the wild). The version-rejection is honest:
    /// V1 senders against V2 receivers fail at decode rather than
    /// silently round-tripping unsigned bytes.
    pub const MAX_WIRE_VERSION: u8 = 2;

    /// Construct a legacy `attestation = None` envelope (no signed
    /// attestation; receiver falls back to its own `device_cid`).
    /// Used by handles that have not been bound via
    /// [`AtriumHandle::set_local_device_attestation`] OR by test
    /// fixtures that bypass the wire envelope flow.
    ///
    /// ## Legacy-fallback security semantics (ds-mr-g16dw6b-MINOR-2)
    ///
    /// A peer that emits `attestation = None` (or that omits the
    /// envelope entirely on a backward-compat sync) causes the
    /// receiver's `Engine::apply_atrium_merge` to fall back to its
    /// OWN `device_cid` for the post-merge `AttributionFrame.device_did`.
    /// This breaks Inv-14 device-grain provenance for bytes received
    /// from non-attesting peers — the receiver attributes inbound
    /// writes to ITSELF at the device-grain.
    ///
    /// The fallback is acceptable for backward-compat with pre-G16-D-fp
    /// peers + the two pre-existing pinned-CID fixtures
    /// (`sync_replica_attribution.rs::sync_replica_*`) that bypass the
    /// wire envelope path. Production deployments with adversarial-
    /// peer threat models SHOULD override the receiver-side
    /// `set_acceptor` with a `FreshnessPolicy` that rejects stale
    /// attestations + a parent-DID expected-issuer gate. The
    /// heterogeneous-cap-envelope filter at phase-3-backlog §6.12
    /// item 9 will additionally enforce an envelope-required
    /// per-zone write filter — at that point, `attestation = None`
    /// envelopes will reject at the cap-recheck boundary rather than
    /// falling back silently.
    #[must_use]
    pub fn new_unsigned() -> Self {
        Self {
            version: Self::WIRE_VERSION,
            attestation: None,
            payload_hash: [0u8; 32],
            session_nonce: [0u8; 32],
            envelope_signature: Vec::new(),
        }
    }

    /// Construct a SIGNED envelope binding the supplied
    /// device-attestation to the upcoming Loro payload + a fresh
    /// session nonce.
    ///
    /// The `device_keypair` MUST hold the secret key matching the
    /// public key encoded in `attestation.device_did` (same bytes the
    /// `did:key:<base58>` form resolves to). Mismatch is detected at
    /// receive (`envelope_signature` fails to verify against the DID's
    /// resolved public key) and surfaces as
    /// [`benten_errors::ErrorCode::DeviceAttestationForged`].
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::InvalidState`] if DAG-CBOR encoding of
    /// the signature input fails (impossible under valid fixed-shape
    /// inputs; result-shape preserves explicit-failure semantics).
    pub fn new_signed(
        attestation: benten_id::device_attestation::DeviceAttestation,
        loro_payload: &[u8],
        device_keypair: &benten_id::keypair::Keypair,
    ) -> AtriumResult<Self> {
        let payload_hash: [u8; 32] = *blake3::hash(loro_payload).as_bytes();
        let mut session_nonce = [0u8; 32];
        // Fresh per-envelope OS-CSPRNG nonce — defends against verbatim
        // envelope replay across sessions. Composes with the parent-
        // issued attestation nonce that `Acceptor::accept_at` consumes.
        getrandom::getrandom(&mut session_nonce).map_err(|e| AtriumError::InvalidState {
            reason: format!("DeviceAttestationEnvelope nonce generation failed: {e}"),
        })?;
        let mut env = Self {
            version: Self::WIRE_VERSION,
            attestation: Some(attestation),
            payload_hash,
            session_nonce,
            envelope_signature: Vec::new(),
        };
        let sig_input = env.signature_input_bytes()?;
        let sig = device_keypair.sign(&sig_input);
        env.envelope_signature = sig.to_bytes().to_vec();
        Ok(env)
    }

    /// Canonical bytes covered by `envelope_signature` — `(version,
    /// attestation, payload_hash, session_nonce)` in DAG-CBOR. Excludes
    /// the signature itself (signature self-reference hygiene; mirrors
    /// `benten_id::device_attestation::canonical_bytes` precedent).
    fn signature_input_bytes(&self) -> AtriumResult<Vec<u8>> {
        #[derive(serde::Serialize)]
        struct SigInput<'a> {
            version: u8,
            attestation: &'a Option<benten_id::device_attestation::DeviceAttestation>,
            payload_hash: &'a [u8; 32],
            session_nonce: &'a [u8; 32],
        }
        serde_ipld_dagcbor::to_vec(&SigInput {
            version: self.version,
            attestation: &self.attestation,
            payload_hash: &self.payload_hash,
            session_nonce: &self.session_nonce,
        })
        .map_err(|e| AtriumError::InvalidState {
            reason: format!("DeviceAttestationEnvelope signature input encode failed: {e}"),
        })
    }

    /// Borrow the originating device's declared DID, if any.
    #[must_use]
    pub fn declared_device_did(&self) -> Option<&str> {
        self.attestation.as_ref().map(|a| a.device_did.as_str())
    }

    /// Encode this envelope as DAG-CBOR canonical bytes for on-wire
    /// emission.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::InvalidState`] if DAG-CBOR encoding fails
    /// (impossible for the fixed-shape envelope under valid inputs;
    /// the result-shape preserves explicit-failure semantics).
    pub fn to_canonical_bytes(&self) -> AtriumResult<Vec<u8>> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| AtriumError::InvalidState {
            reason: format!("DeviceAttestationEnvelope encode failed: {e}"),
        })
    }

    /// Decode a [`DeviceAttestationEnvelope`] from on-wire bytes.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::InvalidState`] if the bytes do not decode
    /// as a valid envelope OR if the wire-format version is newer than
    /// this build understands (`version > MAX_WIRE_VERSION`). Per
    /// crypto-minor-5 fix-pass, version validation is mandatory.
    pub fn from_canonical_bytes(bytes: &[u8]) -> AtriumResult<Self> {
        let env: Self =
            serde_ipld_dagcbor::from_slice(bytes).map_err(|e| AtriumError::InvalidState {
                reason: format!("DeviceAttestationEnvelope decode failed: {e}"),
            })?;
        if env.version > Self::MAX_WIRE_VERSION {
            return Err(AtriumError::InvalidState {
                reason: format!(
                    "DeviceAttestationEnvelope version {} exceeds MAX_WIRE_VERSION {} (newer peer; \
                     upgrade local build to consume the wire shape)",
                    env.version,
                    Self::MAX_WIRE_VERSION
                ),
            });
        }
        Ok(env)
    }

    /// Verify the envelope cryptographically + assert frame-pair
    /// binding to the inbound Loro payload.
    ///
    /// Three failure modes all surface
    /// [`AtriumError::DeviceAttestationForged`]:
    ///
    /// 1. `envelope_signature` does not verify against the public key
    ///    resolved from `attestation.device_did`.
    /// 2. `acceptor.accept_at(&attestation, now)` rejects the
    ///    parent-attestation chain (bad parent signature, expired
    ///    freshness window, replayed nonce, revoked device).
    /// 3. `BLAKE3(loro_payload) != self.payload_hash` (frame-pair
    ///    swap defense).
    ///
    /// `attestation = None` envelopes (pre-G16-D-fp legacy / test
    /// fixture path) skip verification — backward-compat with the
    /// V1-shipped wire shape; the receiver falls back to its own
    /// `device_cid` per `Engine::apply_atrium_merge` semantics.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::DeviceAttestationForged`] on any of the
    /// three failure modes above.
    pub fn verify(
        &self,
        loro_payload: &[u8],
        acceptor: &benten_id::device_attestation::Acceptor,
        now_secs: u64,
    ) -> AtriumResult<()> {
        let Some(attestation) = self.attestation.as_ref() else {
            return Ok(());
        };
        // (1) Envelope signature against device-DID's resolved pubkey.
        let device_did = benten_id::did::Did::from_string_unchecked(attestation.device_did.clone());
        let device_pk = device_did
            .resolve()
            .map_err(|e| AtriumError::DeviceAttestationForged {
                reason: format!("device DID resolution failed: {e:?}"),
            })?;
        let sig_input =
            self.signature_input_bytes()
                .map_err(|e| AtriumError::DeviceAttestationForged {
                    reason: format!("envelope signature-input encode failed: {e}"),
                })?;
        let sig_bytes: [u8; 64] = self.envelope_signature.as_slice().try_into().map_err(|_| {
            AtriumError::DeviceAttestationForged {
                reason: format!(
                    "envelope signature has wrong length: got {}, expected 64",
                    self.envelope_signature.len()
                ),
            }
        })?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        ed25519_dalek::Verifier::verify(device_pk.as_verifying_key(), &sig_input, &sig).map_err(
            |_| AtriumError::DeviceAttestationForged {
                reason: "envelope signature does not verify against device DID's pubkey \
                         (DID forgery / wrong-key signing)"
                    .into(),
            },
        )?;
        // (2) Parent-attestation chain via Acceptor (signature +
        //     freshness + nonce-store replay + revocation).
        acceptor.accept_at(attestation, now_secs).map_err(|e| {
            AtriumError::DeviceAttestationForged {
                reason: format!("attestation chain rejected: {e:?}"),
            }
        })?;
        // (3) Frame-pair payload-hash binding.
        let observed: [u8; 32] = *blake3::hash(loro_payload).as_bytes();
        // Constant-time equality on the hash defends against subtle
        // timing-channel inference of the expected hash bytes.
        use subtle::ConstantTimeEq;
        if observed.ct_eq(&self.payload_hash).unwrap_u8() != 1 {
            return Err(AtriumError::DeviceAttestationForged {
                reason: "envelope payload_hash != BLAKE3(received Loro payload) — \
                         frame-pair binding violation (MITM swap defense)"
                    .into(),
            });
        }
        Ok(())
    }
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
            AtriumError::DeviceAttestationForged { .. } => {
                benten_errors::ErrorCode::DeviceAttestationForged
            }
            AtriumError::HlcSkewExceeded { .. } => benten_errors::ErrorCode::SyncHlcDrift,
            AtriumError::MstEntryCidByteMismatch { .. } => {
                benten_errors::ErrorCode::SyncHashMismatch
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
    /// Phase-3 G16-D wave-6b: this handle's local device-DID, emitted
    /// in the on-the-wire [`DeviceAttestationEnvelope`] sent at the
    /// front of every [`AtriumHandle::sync_subgraph`] /
    /// [`AtriumHandle::accept_sync_subgraph`] exchange. Populated
    /// explicitly via [`AtriumHandle::set_local_device_did`] (typically
    /// after the local device-attestation envelope has been verified).
    /// Defaults to `None` (no device declared on the wire); receivers
    /// observe `device_did = None` envelopes and fall back to the
    /// receiver's own `device_cid` per pim-2 end-to-end-pin discipline.
    ///
    /// **G16-D wave-6b fix-pass note:** when
    /// [`Self::local_device_attestation`] is `Some(_)` AND
    /// [`Self::local_device_keypair`] is `Some(_)`, the wire envelope
    /// emitted is signed (V2 shape — see
    /// [`DeviceAttestationEnvelope::new_signed`]). When either is
    /// `None`, the legacy unsigned shape (`new_unsigned`) is emitted +
    /// `local_device_did` is the only carrier of the device-DID for
    /// receiver-side fallback (the receiver's local `device_cid`
    /// continues to win in the `Engine::apply_atrium_merge` fallback
    /// path).
    local_device_did: Mutex<Option<String>>,
    /// Phase-3 G16-D wave-6b fix-pass: the local device's signed
    /// `benten_id::DeviceAttestation` (parent → device-DID binding).
    /// Embedded into the outbound [`DeviceAttestationEnvelope`] when
    /// [`Self::local_device_keypair`] is also bound; `None` falls back
    /// to the legacy unsigned envelope shape.
    local_device_attestation: Mutex<Option<benten_id::device_attestation::DeviceAttestation>>,
    /// Phase-3 G16-D wave-6b fix-pass: the local device's secret keypair,
    /// used to sign outbound [`DeviceAttestationEnvelope`] frames over
    /// `(version, attestation, payload_hash, session_nonce)`. Defaults
    /// to `None`; bound via [`AtriumHandle::set_local_device_keypair`].
    /// Distinct from [`AtriumInner::peer_keypair`] (the iroh-endpoint
    /// keypair derived at handle-open time) — production deployments
    /// typically use the same keypair for both, but the surfaces are
    /// kept separate so test fixtures can drive distinct cases (e.g.
    /// forgery test with mismatched device-DID vs envelope-signing
    /// keypair).
    local_device_keypair: Mutex<Option<benten_id::keypair::Keypair>>,
    /// Phase-3 G16-D wave-6b fix-pass: the local Acceptor used to
    /// verify inbound [`DeviceAttestationEnvelope`] attestation chains
    /// (signature + freshness + nonce-store replay defense + revocation
    /// list). Defaults to a `FreshnessPolicy::seconds(u64::MAX)` accept-
    /// any-age acceptor — production deployments override via
    /// [`AtriumHandle::set_acceptor`] with the trust-store's parent-DID
    /// + a calibrated freshness window. The Acceptor's nonce-store is
    /// replay-defense state for the Atrium handle's lifetime; cross-
    /// handle replay is bounded by per-attestation issuance scope.
    acceptor: Mutex<benten_id::device_attestation::Acceptor>,
    /// Phase-3 G16-D wave-6b: most-recently-received remote
    /// device-DID-envelope keyed by zone. Populated by
    /// [`AtriumHandle::sync_subgraph`] /
    /// [`AtriumHandle::accept_sync_subgraph`] when the inbound envelope
    /// declares a `device_did`. Consumed by
    /// [`crate::Engine::apply_atrium_merge`] to populate
    /// [`benten_eval::AttributionFrame::device_did`] with the
    /// ORIGINATING device's identity (so post-merge attribution
    /// reflects the writer's device, not the receiver's). Cleared
    /// per-zone after a successful merge so a stale envelope does not
    /// leak into a later merge that didn't get a fresh wire envelope.
    last_received_remote_device_did: Mutex<BTreeMap<String, Option<String>>>,
    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 / hlc-r6-r1-1 closure): the
    /// local HLC clock used by the inbound-sync-frame skew classifier
    /// at [`crate::Engine::apply_atrium_merge`]'s per-row
    /// [`Hlc::update`] call. Bound at handle-open time with the
    /// peer-derived `node_id` + the wall-clock-backed
    /// [`system_time_ms_for_atrium_hlc`] physical clock + the default
    /// 5-minute skew tolerance. A row whose remote HLC `physical_ms`
    /// exceeds local clock by more than the tolerance window rejects
    /// with [`AtriumError::HlcSkewExceeded`] mapping to
    /// [`benten_errors::ErrorCode::SyncHlcDrift`] — the merge is
    /// atomic, so a single skew-exceeding row vetoes the whole merge
    /// per the existing per-row cap-recheck precedent (sec-r4r1-2 closure
    /// at G16-B-F PR #161 / `EngineError::SyncRevokedDuringSession`).
    /// Defends against an adversarial peer manipulating its local HLC
    /// to inject future-timestamped writes that bias LWW resolution +
    /// forge revocation-vs-data ordering per sec-r4r2-1 attack-vector
    /// pin `attack_hlc_skew_revocation_ordering.rs`.
    local_hlc: Hlc,
    /// Phase-3 R6-FP Wave-C1: cumulative count of inbound sync-frame
    /// HLC verifications fired by [`crate::Engine::apply_atrium_merge`]'s
    /// per-row `Hlc::update` loop. Mirrors the existing
    /// `sync_replica_cap_recheck_count` shape — observable so test pins
    /// + operator dashboards can assert the classifier observably fires
    /// per row (defends against silent no-op per pim-2 §3.6b
    /// production-flow-drive end-to-end discipline).
    inbound_hlc_skew_classifier_count: std::sync::atomic::AtomicU64,
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
        // Phase-3 R6-FP Wave-C1 (ds-r6-1 closure): derive HLC node-id
        // from the peer-keypair public-key prefix per the existing
        // `BentenHlc::node_id_from_peer_id_bytes` convention so the
        // local Hlc + the per-write LWW HLC stamps both observe the
        // same identity for this peer.
        let pk_bytes = keypair.public_key().to_bytes();
        let mut prefix = [0u8; 8];
        prefix.copy_from_slice(&pk_bytes[..8]);
        let hlc_node_id = benten_core::hlc::BentenHlc::node_id_from_peer_id_bytes(prefix);
        let local_hlc = Hlc::new(hlc_node_id, system_time_ms_for_atrium_hlc);
        Ok(Self {
            inner: Arc::new(AtriumInner {
                endpoint,
                zones: Mutex::new(BTreeMap::new()),
                peer_keypair: keypair,
                declared_device_attestations: Mutex::new(BTreeMap::new()),
                peer_did_registry: Mutex::new(BTreeMap::new()),
                is_active: AtomicBool::new(true),
                local_device_did: Mutex::new(None),
                last_received_remote_device_did: Mutex::new(BTreeMap::new()),
                local_device_attestation: Mutex::new(None),
                local_device_keypair: Mutex::new(None),
                // Default Acceptor: accept-any-age + no revocations + no
                // expected-parent gate (replay defense via nonce-store
                // is always-on regardless). Production deployments
                // override via `set_acceptor`.
                acceptor: Mutex::new(benten_id::device_attestation::Acceptor::new(
                    benten_id::device_attestation::FreshnessPolicy::seconds(u64::MAX),
                )),
                local_hlc,
                inbound_hlc_skew_classifier_count: std::sync::atomic::AtomicU64::new(0),
            }),
        })
    }

    /// Phase-3 G16-D wave-6b: bind a local device-DID for emission in
    /// the on-the-wire [`DeviceAttestationEnvelope`].
    ///
    /// Callers typically invoke this AFTER the local device-attestation
    /// envelope has been verified (production flow) OR with a fixed
    /// test-DID for two-device same-identity selective-zone-sync
    /// integration tests. Setting `None` clears the binding (next
    /// outbound sync emits an envelope with `device_did = None`).
    ///
    /// The binding is idempotent / replaceable — calling twice with
    /// different DIDs replaces the slot. This composes with
    /// [`crate::Engine::set_device_cid`] (which sets the engine-side
    /// `device_cid` slot consumed by `apply_atrium_merge` for the
    /// LOCAL-merging-engine fallback path); this setter governs the
    /// REMOTE-receiver-observed device-DID.
    pub async fn set_local_device_did(&self, did: Option<String>) {
        let mut g = self.inner.local_device_did.lock().await;
        *g = did;
    }

    /// Phase-3 G16-D wave-6b: read the local device-DID currently bound
    /// for on-the-wire emission. Round-trip companion to
    /// [`AtriumHandle::set_local_device_did`].
    pub async fn local_device_did(&self) -> Option<String> {
        self.inner.local_device_did.lock().await.clone()
    }

    /// Phase-3 G16-D wave-6b fix-pass: bind the local device's signed
    /// `benten_id::DeviceAttestation` for embedding in the outbound
    /// [`DeviceAttestationEnvelope`].
    ///
    /// When the attestation is bound AND
    /// [`AtriumHandle::set_local_device_keypair`] has bound the
    /// device's keypair, the wire envelope emitted by `sync_subgraph` /
    /// `accept_sync_subgraph` is SIGNED (V2 shape) — covering
    /// `(version, attestation, payload_hash, session_nonce)` so the
    /// receiver can verify DID binding + replay-resistance + frame-pair
    /// payload-hash.
    ///
    /// Convenience: also updates [`AtriumHandle::local_device_did`]
    /// from `attestation.device_did` so legacy callers reading that
    /// slot observe the same identity.
    pub async fn set_local_device_attestation(
        &self,
        attestation: Option<benten_id::device_attestation::DeviceAttestation>,
    ) {
        let did = attestation.as_ref().map(|a| a.device_did.clone());
        *self.inner.local_device_attestation.lock().await = attestation;
        *self.inner.local_device_did.lock().await = did;
    }

    /// Phase-3 G16-D wave-6b fix-pass: bind the local device's secret
    /// keypair for signing outbound [`DeviceAttestationEnvelope`]
    /// frames. Independent of the iroh-endpoint keypair (held at
    /// `AtriumInner::peer_keypair`); production deployments typically
    /// pass the same keypair to both — but the seam is preserved so
    /// forgery-test fixtures can drive mismatched cases.
    pub async fn set_local_device_keypair(&self, keypair: Option<benten_id::keypair::Keypair>) {
        *self.inner.local_device_keypair.lock().await = keypair;
    }

    /// Phase-3 G16-D wave-6b fix-pass: install a custom
    /// `benten_id::Acceptor` for inbound envelope verification.
    ///
    /// The Acceptor governs (a) freshness policy (`now - issued_at <=
    /// window`); (b) nonce-store replay defense; (c) revocation list;
    /// (d) optional expected-parent gate. Production deployments
    /// configure the Acceptor with the trust-store's parent-DID + a
    /// calibrated freshness window after the local UCAN backend
    /// promotes — pre-promotion the default
    /// `FreshnessPolicy::seconds(u64::MAX)` accept-any-age acceptor
    /// keeps the wire-envelope signature + payload-binding defenses
    /// load-bearing while leaving freshness-window enforcement to the
    /// caller's discretion.
    pub async fn set_acceptor(&self, acceptor: benten_id::device_attestation::Acceptor) {
        *self.inner.acceptor.lock().await = acceptor;
    }

    /// Build the outbound envelope for a given Loro payload — signed
    /// when both [`AtriumInner::local_device_attestation`] +
    /// [`AtriumInner::local_device_keypair`] are bound, unsigned-legacy
    /// otherwise.
    async fn build_outbound_envelope(
        &self,
        loro_payload: &[u8],
    ) -> AtriumResult<DeviceAttestationEnvelope> {
        let attestation = self.inner.local_device_attestation.lock().await.clone();
        match attestation {
            Some(attestation) => {
                // Signed path — requires keypair binding. If not bound,
                // fall back to unsigned (rationale: a handle that has
                // an attestation but no keypair cannot sign; emitting
                // an attestation without a verifiable signature would
                // be misleading — the receiver would treat it as
                // signed-but-broken and reject).
                let kp_guard = self.inner.local_device_keypair.lock().await;
                if let Some(keypair) = kp_guard.as_ref() {
                    DeviceAttestationEnvelope::new_signed(attestation, loro_payload, keypair)
                } else {
                    Ok(DeviceAttestationEnvelope::new_unsigned())
                }
            }
            None => Ok(DeviceAttestationEnvelope::new_unsigned()),
        }
    }

    /// Verify an inbound envelope against the upcoming Loro payload +
    /// the handle's installed Acceptor. Returns the verified
    /// device-DID (if signed) or `None` (legacy unsigned).
    async fn verify_inbound_envelope(
        &self,
        envelope: &DeviceAttestationEnvelope,
        loro_payload: &[u8],
    ) -> AtriumResult<Option<String>> {
        let acceptor = self.inner.acceptor.lock().await;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        envelope.verify(loro_payload, &acceptor, now_secs)?;
        Ok(envelope.declared_device_did().map(|s| s.to_string()))
    }

    /// Phase-3 G16-D wave-6b: read the most-recently-received remote
    /// device-DID for a zone (the `device_did` carried in the inbound
    /// [`DeviceAttestationEnvelope`] of the latest
    /// [`AtriumHandle::sync_subgraph`] /
    /// [`AtriumHandle::accept_sync_subgraph`] for that zone).
    ///
    /// Returns `Some(Some(did))` when a fresh envelope declared a
    /// device-DID; `Some(None)` when a fresh envelope declared
    /// `device_did = None` (legacy / pre-G16-D peer); `None` when no
    /// envelope has been received for the zone yet (or the slot was
    /// cleared post-merge).
    ///
    /// ## Side-channel slot-coupling contract (ds-mr-g16dw6b-MINOR-1)
    ///
    /// This slot is populated by `sync_subgraph` /
    /// `accept_sync_subgraph` and CONSUMED then CLEARED by
    /// `Engine::apply_atrium_merge` per zone. The expected call shape
    /// is **exactly one `sync_subgraph` followed by exactly one
    /// `apply_atrium_merge` per zone** — production paths follow this
    /// shape (the engine's apex orchestrator owns the round-trip).
    ///
    /// Deviations have observable but bounded consequences:
    ///
    /// - **Two `sync_subgraph` calls before any `apply_atrium_merge`:**
    ///   the second envelope's `device_did` overwrites the first's.
    ///   The first envelope's device-DID is silently lost — but the
    ///   Loro merge itself still applies (CRDT replay-safety). The
    ///   AttributionFrame for the first merge inherits the second
    ///   sync's device-DID; correctness depends on operator intent.
    /// - **Two `apply_atrium_merge` calls after one `sync_subgraph`:**
    ///   the second mint observes `None` (slot was cleared by the
    ///   first) and falls back to the local engine's `device_cid`
    ///   per the legacy unsigned-envelope path. The second mint
    ///   carries the LOCAL device's identity, NOT the originator's.
    ///   Correct semantics for "self-driven re-application" but
    ///   subtle if the caller intended a fresh originating-peer
    ///   transmission.
    ///
    /// ## Concurrent same-zone race (ds-mr-g16dw6b-MINOR-3)
    ///
    /// The slot is keyed `BTreeMap<String, Option<String>>`. Two
    /// peers running `accept_sync_subgraph` + `sync_subgraph_over`
    /// against the SAME zone simultaneously race at the
    /// `tbl.insert(zone, ...)` site — the second call's insert
    /// overwrites the first's slot before either's apply consumes
    /// it. Mitigations available (per-zone merge mutex; struct-shaped
    /// envelope + bytes guarded by oneshot consumption) but not
    /// currently in place. Production paths use serial accept-then-
    /// sync per zone; concurrent same-zone two-peer is undefined w.r.t.
    /// AttributionFrame.device_did binding (the Loro merge itself
    /// remains correct + idempotent — only the device-DID-slot is
    /// racy). Cross-zone concurrent merges from a single handle are
    /// safe (distinct keys; no race).
    pub async fn last_received_remote_device_did(&self, zone: &str) -> Option<Option<String>> {
        self.inner
            .last_received_remote_device_did
            .lock()
            .await
            .get(zone)
            .cloned()
    }

    /// Phase-3 G16-D wave-6b: clear the per-zone last-received
    /// remote-device-DID slot. Called by
    /// [`crate::Engine::apply_atrium_merge`] after the post-merge
    /// AttributionFrame has consumed the slot, so a subsequent merge
    /// that did NOT get a fresh wire envelope cannot inherit the prior
    /// envelope's device-DID.
    pub(crate) async fn clear_last_received_remote_device_did(&self, zone: &str) {
        let mut g = self.inner.last_received_remote_device_did.lock().await;
        g.remove(zone);
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

    /// The local peer's loopback transport address for in-process
    /// two-peer test fixtures. Production peers discover each other
    /// via the transport's relay-default + address-lookup path.
    ///
    /// Returns [`benten_sync::TransportAddr`] — the transport-neutral
    /// public alias surfaced through `benten-sync`'s
    /// `TransportEndpoint::Addr` seam (Surf-1 #889 / residual #1232).
    /// The engine's public surface no longer names `iroh::EndpointAddr`
    /// directly; the full `<T: Transport>` engine-generic migration
    /// remains genuinely-post-v1 CLAUDE.md #19 work.
    ///
    /// # Errors
    ///
    /// Returns [`AtriumError::Transport`] if the underlying endpoint
    /// has no bound sockets.
    pub fn loopback_addr(&self) -> AtriumResult<benten_sync::TransportAddr> {
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
        remote_addr: benten_sync::TransportAddr,
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
        // Export local state for this zone FIRST so the outbound
        // envelope can sign over BLAKE3(local_export) — frame-pair
        // binding (G16-D wave-6b fix-pass cryptographic-attestation
        // closure).
        let local_export = {
            let zones = self.inner.zones.lock().await;
            let doc = zones.get(zone).ok_or_else(|| AtriumError::InvalidState {
                reason: format!("zone '{zone}' not registered"),
            })?;
            doc.export_update()?
        };
        // Phase-3 G16-D wave-6b fp: build a SIGNED envelope (when
        // attestation + keypair are bound) carrying the parent-signed
        // attestation + BLAKE3(local_export) payload-hash + fresh
        // session nonce + device-keypair signature. Receiver verifies
        // all three at receive (DID forgery / replay / frame-pair
        // swap defenses).
        let envelope_bytes = self
            .build_outbound_envelope(&local_export)
            .await?
            .to_canonical_bytes()?;
        conn.send_bytes(&envelope_bytes).await?;
        // Send local state to remote.
        conn.send_bytes(&local_export).await?;
        // Receive remote envelope FIRST, then remote Loro export, in
        // the same order the connect-side emitted them.
        let remote_envelope_bytes = conn.recv_bytes().await?;
        let remote_envelope =
            DeviceAttestationEnvelope::from_canonical_bytes(&remote_envelope_bytes)?;
        // Receive remote state.
        let remote_bytes = conn.recv_bytes().await?;
        // G16-D wave-6b fp: cryptographic verification at receive —
        // signature + parent-chain Acceptor + payload-hash binding.
        // Surfaces `AtriumError::DeviceAttestationForged` on any of the
        // three failure modes; legacy unsigned envelopes (attestation
        // = None) skip verification per backward-compat contract.
        let verified_remote_did = self
            .verify_inbound_envelope(&remote_envelope, &remote_bytes)
            .await?;
        // Stash the remote-device-DID (post-verification) before merge
        // so apply_atrium_merge can consume it via
        // `last_received_remote_device_did`.
        {
            let mut tbl = self.inner.last_received_remote_device_did.lock().await;
            tbl.insert(zone.to_string(), verified_remote_did);
        }
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
        // Accept-side mirrors connect-side: receive remote envelope +
        // export FIRST, then send local envelope + export. Connect-side
        // emits envelope-then-export, so accept-side consumes
        // envelope-then-export.
        let remote_envelope_bytes = conn.recv_bytes().await?;
        let remote_envelope =
            DeviceAttestationEnvelope::from_canonical_bytes(&remote_envelope_bytes)?;
        let remote_bytes = conn.recv_bytes().await?;
        // G16-D wave-6b fp: cryptographic verification at receive —
        // signature + parent-chain Acceptor + payload-hash binding.
        let verified_remote_did = self
            .verify_inbound_envelope(&remote_envelope, &remote_bytes)
            .await?;
        // Stash the remote-device-DID (post-verification) before merge
        // so apply_atrium_merge can consume it via
        // `last_received_remote_device_did`.
        {
            let mut tbl = self.inner.last_received_remote_device_did.lock().await;
            tbl.insert(zone.to_string(), verified_remote_did);
        }
        // Phase-3 G16-D wave-6b: emit local envelope BEFORE the Loro
        // export, mirroring connect-side ordering. Build the envelope
        // AFTER export-update so the signed payload-hash binds to the
        // exact bytes we will send (frame-pair binding).
        let local_export = {
            let zones = self.inner.zones.lock().await;
            let doc = zones.get(zone).ok_or_else(|| AtriumError::InvalidState {
                reason: format!("zone '{zone}' not registered"),
            })?;
            doc.export_update()?
        };
        let envelope_bytes = self
            .build_outbound_envelope(&local_export)
            .await?
            .to_canonical_bytes()?;
        conn.send_bytes(&envelope_bytes).await?;
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
        drop(zones);
        // Phase-3 G16-D wave-6b: surface the remote-device-DID stashed
        // by `sync_subgraph` / `accept_sync_subgraph` (or `None` when
        // the merge was driven directly via `apply_atrium_merge`
        // without a wire-side envelope, e.g. test fixtures that bypass
        // the iroh transport). The receiver-side
        // `Engine::apply_atrium_merge` consumes this slot to populate
        // `AttributionFrame.device_did` with the originating-device
        // identity, closing exit-criterion 16.
        let remote_device_did = self
            .inner
            .last_received_remote_device_did
            .lock()
            .await
            .get(zone)
            .cloned()
            .unwrap_or(None);
        Ok(SyncMergeAttribution {
            peer_node_ids,
            sync_hop_depth: next_depth,
            remote_device_did,
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

    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 closure): accessor for the
    /// per-handle HLC clock used by the inbound-sync-frame skew
    /// classifier. Tests can use this to observe the local clock state
    /// (`now()` / `node_id()` / `skew_tolerance_ms()`) but should not
    /// drive `update()` directly — the `apply_atrium_merge` per-row
    /// loop owns the production wireup.
    #[must_use]
    pub fn local_hlc(&self) -> &Hlc {
        &self.inner.local_hlc
    }

    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 closure): cumulative count of
    /// inbound sync-frame HLC verifications fired by
    /// [`crate::Engine::apply_atrium_merge`]'s per-row `Hlc::update`
    /// loop. Mirrors the `sync_replica_cap_recheck_calls` shape per
    /// pim-2 §3.6b end-to-end discipline — observable counter so test
    /// pins assert the classifier observably fires per row.
    #[must_use]
    pub fn inbound_hlc_skew_classifier_calls(&self) -> u64 {
        self.inner
            .inbound_hlc_skew_classifier_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Phase-3 R6-FP Wave-C1 (ds-r6-1 closure) — internal accessor for
    /// the per-row HLC verification counter that
    /// [`crate::Engine::apply_atrium_merge`] increments on every
    /// inbound row. Crate-public so the engine module can bump the
    /// counter without exposing the inner Arc; external callers read
    /// the value through [`Self::inbound_hlc_skew_classifier_calls`].
    #[must_use]
    pub(crate) fn inner_inbound_hlc_skew_classifier_count(&self) -> &std::sync::atomic::AtomicU64 {
        &self.inner.inbound_hlc_skew_classifier_count
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
