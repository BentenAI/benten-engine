//! Phase 2a G3-A: `ExecutionStateEnvelope` + `ExecutionStatePayload` +
//! `AttributionFrame` + `Frame` — FROZEN shape per plan §9.1.
//!
//! All types are content-addressed (BLAKE3 over DAG-CBOR) by composition:
//! the envelope carries a `payload_cid` and the resume protocol (4 steps)
//! re-verifies each boundary. See plan §9.1 + `.addl/phase-2a/r1-triage.md`
//! "arch-1" resolution.
//!
//! Encoding: `serde_ipld_dagcbor` produces canonical DAG-CBOR bytes that
//! are bit-stable across re-encodes (BTreeMap-ordered keys, deterministic
//! float handling). The envelope's `payload_cid` is BLAKE3 of the canonical
//! payload bytes (32-byte digest wrapped in a CIDv1 `dag-cbor` / `blake3`
//! envelope — matches `benten_core::Cid` throughout the engine).

use std::collections::BTreeSet;

use benten_core::{Cid, CoreError, Value};
use serde::{Deserialize, Serialize};

/// A `did:key:...` identifier (logical-principal or device-grain).
///
/// Phase-3 G16-B canary uses `String` at the eval layer to avoid pulling
/// `benten-id` into `benten-eval`'s dep graph (layering: eval is identity-
/// ignorant per arch-r1-10). Downstream consumers (engine_sync glue) wrap
/// the resolved-string form of `benten_id::did::Did` and round-trip it
/// through `Did::resolve` at the engine boundary when typed identity is
/// required.
pub type Did = String;

/// A single attribution frame: `(actor, handler, capability_grant)`.
/// Plan §9.1 + ucca-1 / ucca-4: chain (not 3-tuple) carries this frame as
/// its element type. Phase-2a ships the 3-field shape; Phase-3 additions
/// are provably additive (pinned by `invariant_14_fixture_cid` test).
///
/// Phase-2b G7-B (D20-RESOLVED): adds `sandbox_depth: u8` for Inv-4
/// nest-depth tracking. The counter is INHERITED across CALL boundaries
/// (not reset) so that handler A SANDBOXes → CALLs handler B → SANDBOXes
/// is depth-2, not two separate depth-1s. To preserve the Phase-2a
/// schema-fixture CID (`bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`),
/// the field defaults to `0` AND `AttributionFrame::cid()` only includes the
/// `sandbox_depth` slot in the canonical Node when the value is non-zero.
/// A frame with `sandbox_depth = 0` therefore round-trips to the exact
/// Phase-2a CID; a frame with non-zero depth produces a distinct CID
/// (asserted by `invariant_4_overflow.rs::attribution_frame_sandbox_depth_field_present_default_zero`).
///
/// Phase-3 G16-B (ds-r4b-1 BLOCKER closure): adds three sync-boundary
/// fields for Inv-14 device-grain attribution at the Loro merge seam:
///
/// - `peer_did_set`: `None` for purely-local writes; `Some(BTreeSet<Did>)`
///   for frames that originate from a sync merge (carries the set of
///   contributing peer DIDs observed via `LoroDoc::winning_attribution`).
/// - `device_did`: `None` for legacy / actor-only writes; `Some(Did)` for
///   sync-attributed or device-DID-attested writes per D-PHASE-3-25
///   device-heterogeneity contract.
/// - `sync_hop_depth`: `0` for local writes; increments by 1 per CRDT
///   merge hop. Bounded by [`SYNC_HOP_DEPTH_CAP`]; overflow surfaces as
///   `E_SYNC_HOP_DEPTH_EXCEEDED`.
///
/// Per the Phase-2a-CID-stability discipline (D20-RESOLVED precedent),
/// each new slot is included in the canonical Node ONLY when its value is
/// non-default (`None` / `0`). A frame with all three fields default
/// therefore canonicalises to the exact Phase-2a 3-key Node and produces
/// the pinned schema-fixture CID; any non-default value adds the slot
/// and produces a distinct CID — the security claim of Inv-14
/// (a sync-bearing attribution chain is content-distinguishable from a
/// purely-local chain).
///
/// `serde(default)` lets older DAG-CBOR payloads decode cleanly into the
/// extended struct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionFrame {
    /// CID of the actor (principal) that authored the step.
    pub actor_cid: Cid,
    /// CID of the handler subgraph that is executing.
    pub handler_cid: Cid,
    /// CID of the capability grant authorising the step.
    pub capability_grant_cid: Cid,
    /// Inv-4 SANDBOX nest-depth counter (D20-RESOLVED). Incremented at
    /// every SANDBOX entry; INHERITED across CALL boundaries (CALL itself
    /// does NOT increment). Default `0` keeps the Phase-2a schema-fixture
    /// CID stable for non-SANDBOX flows. `serde(default)` lets older
    /// DAG-CBOR payloads decode cleanly into the extended struct.
    #[serde(default)]
    pub sandbox_depth: u8,
    /// Phase-3 G16-B sync-boundary extension. `Some(set)` when this frame
    /// originates from a Loro CRDT merge (row-4a per ds-4); captures the
    /// contributing peer DIDs observed via
    /// `benten_sync::crdt::LoroDoc::winning_attribution`. `None` for
    /// purely-local writes. The exact peer-node-id → DID resolution is
    /// performed by the engine layer (`crates/benten-engine/src/engine_sync.rs`)
    /// against the local trust store; pre-trust-store-wireup the engine
    /// emits raw node-ids serialised as decimal strings (see
    /// G16-B wave-6b residual at `docs/future/phase-3-backlog.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_did_set: Option<BTreeSet<Did>>,
    /// Phase-3 G16-B device-grain attribution per Inv-14 sync-grain
    /// requirement + D-PHASE-3-25 device-heterogeneity contract.
    /// `None` for legacy / local writes; `Some(did)` for sync-attributed
    /// or device-DID-attested writes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_did: Option<Did>,
    /// Phase-3 G16-B sync-hop-depth bound (D-PHASE-3-25
    /// sync-hop-depth-bounded contract). Increments per CRDT merge hop;
    /// `E_SYNC_HOP_DEPTH_EXCEEDED` fires at the configured cap
    /// ([`SYNC_HOP_DEPTH_CAP`] default). Default `0` keeps the Phase-2a
    /// schema-fixture CID stable for purely-local frames; the field is
    /// elided from the serde-encoded map when zero so envelope bytes
    /// for purely-local frames stay byte-identical to Phase-2a (mirrors
    /// the `peer_did_set` / `device_did` skip-on-default discipline).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub sync_hop_depth: u32,
}

/// Phase-3 G16-B sync-hop-depth cap mirroring the Inv-4 sandbox_depth
/// cap precedent. A frame whose `sync_hop_depth` would exceed this cap
/// after a merge MUST surface as
/// [`benten_errors::ErrorCode::SyncHopDepthExceeded`] at the merge seam.
pub const SYNC_HOP_DEPTH_CAP: u32 = 8;

/// `skip_serializing_if` predicate for [`AttributionFrame::sync_hop_depth`].
/// Non-pub helper kept here so the canonical Node encoding stays byte-
/// stable with the Phase-2a 4-key shape when the field is zero.
///
/// `serde`'s `skip_serializing_if` requires a `&u32` signature even
/// though the trivial `Copy` could pass by value — silence the lint
/// here.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires the &T signature"
)]
fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// Default `AttributionFrame` with all-zero CIDs and purely-local
/// (non-sync) Phase-3 sync fields.
///
/// **Architectural intent:** this `Default` exists to support the
/// `..Default::default()` spread pattern at test/bench/legacy callsites
/// that pre-date the Phase-3 G16-B sync-boundary fields (`peer_did_set`,
/// `device_did`, `sync_hop_depth`). Cascading struct-literal additions
/// across ~50 construction sites is the carrying cost we accept in
/// exchange for keeping the canonical Node encoding additive.
///
/// **Production callsites should NOT rely on this `Default`.** The
/// engine's WRITE path constructs frames explicitly (see
/// `crates/benten-engine/src/engine_sync.rs`) and SANDBOX entry points
/// derive `sandbox_depth` from the parent frame; the all-zero CID
/// values here are placeholders that would not pass invariant checks
/// in a live evaluator. The default is intentionally test-/bench-
/// shaped: `actor_cid` / `handler_cid` / `capability_grant_cid` are
/// all-zero, `sandbox_depth = 0`, and the three Phase-3 sync slots
/// default to `None` / `0` so `serde` elides them from the canonical
/// envelope (preserving the Phase-2a schema-fixture CID).
impl Default for AttributionFrame {
    fn default() -> Self {
        Self {
            actor_cid: Cid::from_blake3_digest([0u8; 32]),
            handler_cid: Cid::from_blake3_digest([0u8; 32]),
            capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
            sandbox_depth: 0,
            peer_did_set: None,
            device_did: None,
            sync_hop_depth: 0,
        }
    }
}

impl AttributionFrame {
    /// Content-addressed CID of this frame. Used by `invariant_14_fixture_cid`
    /// to pin the Phase-2a shape so Phase-6 additions are provably additive.
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn cid(&self) -> Result<Cid, CoreError> {
        // Encode the three CIDs as a canonical node and hash it.
        use benten_core::Node;
        use std::collections::BTreeMap;
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert(
            "actor".into(),
            Value::Bytes(self.actor_cid.as_bytes().to_vec()),
        );
        props.insert(
            "handler".into(),
            Value::Bytes(self.handler_cid.as_bytes().to_vec()),
        );
        props.insert(
            "grant".into(),
            Value::Bytes(self.capability_grant_cid.as_bytes().to_vec()),
        );
        // D20-RESOLVED Phase-2a-CID-stability discipline: the
        // `sandbox_depth` slot is included in the canonical Node ONLY
        // when the value is non-zero. A default-zero AttributionFrame
        // therefore canonicalises to the exact Phase-2a 3-key Node and
        // produces the pinned schema-fixture CID
        // (`bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`).
        // Any non-zero value adds the slot and produces a distinct CID,
        // which is the security claim of Inv-4: a SANDBOX-bearing
        // attribution chain is content-distinguishable from a non-SANDBOX
        // chain.
        if self.sandbox_depth != 0 {
            props.insert(
                "sandbox_depth".into(),
                Value::Int(i64::from(self.sandbox_depth)),
            );
        }
        // Phase-3 G16-B Inv-14 sync-boundary extension: each slot is
        // included only when its value is non-default. A purely-local
        // frame (peer_did_set=None, device_did=None, sync_hop_depth=0)
        // canonicalises to the Phase-2a 3-key Node. Any non-default
        // value adds a slot — sync-attributed frames are content-
        // distinguishable from local frames per Inv-14.
        if let Some(peer_set) = &self.peer_did_set {
            // Encode as a sorted list of DID strings (BTreeSet already
            // gives sorted iteration order; canonical-bytes-stable).
            let list: Vec<Value> = peer_set
                .iter()
                .map(|did| Value::text(did.clone()))
                .collect();
            props.insert("peer_did_set".into(), Value::List(list));
        }
        if let Some(device) = &self.device_did {
            props.insert("device_did".into(), Value::text(device.clone()));
        }
        if self.sync_hop_depth != 0 {
            props.insert(
                "sync_hop_depth".into(),
                Value::Int(i64::from(self.sync_hop_depth)),
            );
        }
        let node = Node::new(vec!["AttributionFrame".into()], props);
        node.cid()
    }
}

/// Evaluator stack-frame snapshot. Phase-2a ships the `{ tag: String }`
/// shape; this is sufficient for `ExecutionStatePayload` DAG-CBOR round-trip
/// semantics. The "real frame pointer + pending op list + local variables"
/// elaboration is a Phase 2b concern owned by G3-B / G6-C when the evaluator
/// is fully reshaped around multi-suspend + SANDBOX integration. Per plan §9.1
/// the Frame shape is FROZEN at Phase 2a close as `{ tag: String }`; any
/// field additions are additive (new required field = schema_version bump).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Opaque frame tag; populated by the evaluator at suspend time.
    pub tag: String,
}

impl Frame {
    /// Construct the canonical root frame. Engine entry-points (notably
    /// `Engine::call_with_suspension` via `payload_for_handler`) use this
    /// as the bottom of a fresh frame stack at suspend-time. The previous
    /// `root_for_test` name was misleading — this constructor is
    /// production-reachable, not test-only.
    #[must_use]
    pub fn root() -> Self {
        Self { tag: "root".into() }
    }
}

/// Content-addressed execution-state payload — the frozen shape per plan
/// §9.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionStatePayload {
    /// Chain of `AttributionFrame`s (NOT a 3-tuple). Phase-2a single-frame.
    pub attribution_chain: Vec<AttributionFrame>,
    /// Sorted + deduplicated pinned subgraph CIDs.
    pub pinned_subgraph_cids: Vec<Cid>,
    /// Context bindings snapshotted inline at suspend time (CID-substitution
    /// attack mitigation).
    pub context_binding_snapshots: Vec<(String, Cid, Vec<u8>)>,
    /// Principal identity that owns the resume authority.
    pub resumption_principal_cid: Cid,
    /// Evaluator frame stack.
    pub frame_stack: Vec<Frame>,
    /// Index into `frame_stack` identifying the currently-suspended frame.
    pub frame_index: usize,
}

impl ExecutionStatePayload {
    /// Construct a payload with the `pinned_subgraph_cids` invariant
    /// enforced (sorted + deduped).
    #[must_use]
    pub fn new_with_pinned(mut cids: Vec<Cid>) -> Self {
        cids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        cids.dedup();
        Self {
            attribution_chain: Vec::new(),
            pinned_subgraph_cids: cids,
            context_binding_snapshots: Vec::new(),
            resumption_principal_cid: Cid::from_blake3_digest([0u8; 32]),
            frame_stack: Vec::new(),
            frame_index: 0,
        }
    }
}

/// Envelope wrapping an [`ExecutionStatePayload`] with a `schema_version`
/// and a pre-computed `payload_cid`. Content-addressed by composition.
///
/// # Cross-process safety (Phase 2b G12-E)
///
/// `ExecutionStateEnvelope` bytes are now safe to round-trip across a
/// process boundary against a shared on-disk redb file. The engine
/// persists the envelope into its
/// [`crate::suspension_store::SuspensionStore`] at suspend time
/// (default impl: `benten_engine::RedbSuspensionStore` over the engine's
/// `Arc<RedbBackend>`). A fresh engine opened against the same path
/// hydrates the envelope on `suspend_to_bytes` lookup AND restores the
/// associated [`crate::suspension_store::WaitMetadata`] so deadline +
/// signal-shape checks fire correctly post-restart. This closes the
/// Phase-2a Compromise #10 cross-process metadata gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStateEnvelope {
    /// Envelope schema version; `= 1` in Phase 2a.
    pub schema_version: u8,
    /// CID of the payload bytes.
    pub payload_cid: Cid,
    /// The payload itself.
    pub payload: ExecutionStatePayload,
}

impl ExecutionStatePayload {
    /// Canonical DAG-CBOR encoding of this payload. The bytes are the
    /// hash-input for the envelope's `payload_cid`.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        serde_ipld_dagcbor::to_vec(self)
            .map_err(|e| CoreError::Serialize(format!("exec-state payload encode: {e}")))
    }

    /// BLAKE3 CID of the canonical DAG-CBOR bytes.
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.to_canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

impl ExecutionStateEnvelope {
    /// Canonical constructor: computes `payload_cid` from the canonical
    /// DAG-CBOR bytes of `payload` and stamps `schema_version = 1`.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn new(payload: ExecutionStatePayload) -> Result<Self, CoreError> {
        let payload_cid = payload.cid()?;
        Ok(Self {
            schema_version: 1,
            payload_cid,
            payload,
        })
    }

    /// Encode the envelope as canonical DAG-CBOR.
    ///
    /// Two independent calls with the same payload MUST produce byte-
    /// identical output (proptest `prop_exec_state_dagcbor_roundtrip` and
    /// the `wait_resume_determinism` integration gate depend on this).
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        serde_ipld_dagcbor::to_vec(self)
            .map_err(|e| CoreError::Serialize(format!("exec-state envelope encode: {e}")))
    }

    /// Decode an envelope from canonical DAG-CBOR.
    ///
    /// This is the mirror of [`Self::to_canonical_bytes`]; `to_canonical_bytes` →
    /// `from_canonical_bytes` is a bijection on well-formed bytes. The decoder does
    /// NOT re-verify `payload_cid` against the payload bytes — that is step 1
    /// of the resume protocol (see [`Self::recompute_payload_cid`]).
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CoreError> {
        serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| CoreError::Serialize(format!("exec-state envelope decode: {e}")))
    }

    /// Envelope CID accessor. The envelope wrapper itself is not separately
    /// hashed; the envelope CID **is** the `payload_cid` (computed once at
    /// [`ExecutionStateEnvelope::new`] from the canonical DAG-CBOR bytes of
    /// the payload). This is a pure infallible field accessor — it returns
    /// the precomputed value and never re-encodes.
    ///
    /// v1-API-stabilization (refinement-audit #794): the prior
    /// `Result<Cid, CoreError>` was an aspirational shape — the body was
    /// `Ok(self.payload_cid)` with no fallible path, forcing every caller
    /// through a `.expect()`/`?` that could never fire. Returning `Cid`
    /// directly removes the lying boundary. If a future phase introduces a
    /// separately-hashed outer envelope CID it will be a distinct fallible
    /// constructor, not a silent re-Result of this accessor.
    #[must_use]
    pub fn envelope_cid(&self) -> Cid {
        self.payload_cid
    }

    /// Recompute the payload CID from the current payload bytes (resume
    /// protocol step 1 per plan §9.1). A mismatch vs. `self.payload_cid`
    /// means the persisted bytes were tampered with between suspend and
    /// resume and the resume MUST be rejected.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn recompute_payload_cid(&self) -> Result<Cid, CoreError> {
        self.payload.cid()
    }
}
