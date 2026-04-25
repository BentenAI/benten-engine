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

use benten_core::{Cid, CoreError, Value};
use serde::{Deserialize, Serialize};

/// A single attribution frame: `(actor, handler, capability_grant)`.
/// Plan §9.1 + ucca-1 / ucca-4: chain (not 3-tuple) carries this frame as
/// its element type. Phase-2a ships the 3-field shape; Phase-6 additions
/// are provably additive (pinned by `invariant_14_fixture_cid` test).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionFrame {
    /// CID of the actor (principal) that authored the step.
    pub actor_cid: Cid,
    /// CID of the handler subgraph that is executing.
    pub handler_cid: Cid,
    /// CID of the capability grant authorising the step.
    pub capability_grant_cid: Cid,
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
    pub fn to_dagcbor(&self) -> Result<Vec<u8>, CoreError> {
        serde_ipld_dagcbor::to_vec(self)
            .map_err(|e| CoreError::Serialize(format!("exec-state envelope encode: {e}")))
    }

    /// Decode an envelope from canonical DAG-CBOR.
    ///
    /// This is the mirror of [`Self::to_dagcbor`]; `to_dagcbor` →
    /// `from_dagcbor` is a bijection on well-formed bytes. The decoder does
    /// NOT re-verify `payload_cid` against the payload bytes — that is step 1
    /// of the resume protocol (see [`Self::recompute_payload_cid`]).
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn from_dagcbor(bytes: &[u8]) -> Result<Self, CoreError> {
        serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| CoreError::Serialize(format!("exec-state envelope decode: {e}")))
    }

    /// Envelope CID accessor. In Phase 2a this is the `payload_cid` — the
    /// envelope wrapper itself is not separately hashed. `Result` is
    /// preserved so call sites can chain `.expect()` uniformly; future
    /// phases may compute an outer envelope CID lazily here.
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn envelope_cid(&self) -> Result<Cid, CoreError> {
        Ok(self.payload_cid)
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
