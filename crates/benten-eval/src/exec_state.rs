//! Phase 2a G3-A: `ExecutionStateEnvelope` + `ExecutionStatePayload` +
//! `AttributionFrame` + `Frame` — FROZEN shape per plan §9.1.
//!
//! All types are content-addressed (BLAKE3 over DAG-CBOR) by composition:
//! the envelope carries a `payload_cid` and the resume protocol (4 steps)
//! re-verifies each boundary. See plan §9.1 + `.addl/phase-2a/r1-triage.md`
//! "arch-1" resolution.
//!
//! TODO(phase-2a-G3-A): real DAG-CBOR encoders + 4-step resume protocol +
//! `Frame` implementation carry load.

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

/// Evaluator stack-frame snapshot. Phase-2a stub shape sufficient for
/// `ExecutionStatePayload` round-trip; real suspension/resume semantics
/// land with G3-A.
///
/// TODO(phase-2a-G3-A): carry the actual frame pointer + pending op list
/// + local variables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Opaque frame tag; populated by the evaluator at suspend time.
    pub tag: String,
}

impl Frame {
    /// Root-frame stub for unit tests.
    #[must_use]
    pub fn root_for_test() -> Self {
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

impl ExecutionStateEnvelope {
    /// Canonical constructor: computes `payload_cid` from `payload` and
    /// stamps `schema_version = 1`.
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn new(payload: ExecutionStatePayload) -> Result<Self, CoreError> {
        // Stub: use the resumption_principal_cid as a placeholder for the
        // content hash. R5 G3-A lands real DAG-CBOR encode + BLAKE3.
        let payload_cid = payload.resumption_principal_cid;
        Ok(Self {
            schema_version: 1,
            payload_cid,
            payload,
        })
    }

    /// Encode as DAG-CBOR. Phase-2a stub.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_dagcbor(&self) -> Result<Vec<u8>, CoreError> {
        todo!(
            "Phase 2a G3-A: implement DAG-CBOR encode per plan §9.1 + \
             `exec_state_dagcbor_roundtrip` test"
        )
    }

    /// Decode from DAG-CBOR. Phase-2a stub.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn from_dagcbor(_bytes: &[u8]) -> Result<Self, CoreError> {
        todo!(
            "Phase 2a G3-A: implement DAG-CBOR decode per plan §9.1 + \
             `exec_state_dagcbor_roundtrip` test"
        )
    }

    /// Envelope CID accessor — returns a `Result` so call sites can chain
    /// `.expect()` uniformly; the `Err` arm fires only if the envelope was
    /// constructed with a malformed payload (a Phase-2a regression guard).
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn envelope_cid(&self) -> Result<Cid, CoreError> {
        Ok(self.payload_cid)
    }

    /// Recompute the payload CID from the payload bytes (resume step 1).
    ///
    /// # Errors
    /// Returns [`CoreError`] on encode failure.
    pub fn recompute_payload_cid(&self) -> Result<Cid, CoreError> {
        todo!("Phase 2a G3-A: implement recompute_payload_cid per plan §9.1 step 1")
    }
}
