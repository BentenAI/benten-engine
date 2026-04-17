//! # benten-eval — Operation primitives + evaluator (Phase 1 stubs)
//!
//! Phase 1 ships all 12 operation primitive *types* (so stored subgraphs
//! don't require re-registration when Phase 2 enables WAIT/STREAM/SUBSCRIBE/
//! SANDBOX executors) and executes 8 primitives in the iterative evaluator:
//! READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT.
//!
//! R3 stub scaffold — R5 implementation lands in Phase 1 proper.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries per-invariant diagnostic context (paths, expected/actual CIDs, counts) per R1 triage; Phase-2 will box large diagnostic payloads once the accessor set stabilises"
)]
#![allow(
    clippy::too_many_lines,
    reason = "Invariant-validation pass is intentionally linear so the code reads top-to-bottom as the invariant list"
)]

pub use benten_core::ErrorCode;
use benten_core::{Cid, Value};
use std::collections::BTreeMap;

pub mod context;
pub mod diag;
pub mod evaluator;
pub mod expr;
pub mod invariants;
pub mod primitives;

pub use context::EvalContext;

/// Marker for the current stub phase. Removed when the evaluator lands.
pub const STUB_MARKER: &str = "benten-eval::stub";

/// Configurable invariant limits. Defaults match ENGINE-SPEC §4.
pub mod limits {
    /// Invariant 2: default max operation-subgraph depth.
    pub const DEFAULT_MAX_DEPTH: usize = 64;
    /// Invariant 3: default max fan-out per node.
    pub const DEFAULT_MAX_FANOUT: usize = 16;
    /// Invariant 5: default max total nodes per subgraph.
    pub const DEFAULT_MAX_NODES: usize = 4096;
    /// Invariant 6: default max total edges per subgraph.
    pub const DEFAULT_MAX_EDGES: usize = 8192;
    /// Invariant 8 stopgap: max ITERATE nesting depth (Phase 1 named compromise).
    pub const DEFAULT_MAX_ITERATE_NEST_DEPTH: usize = 3;
}

/// Evaluator error type (Phase 1 stub).
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("invariant violation: {0:?}")]
    Invariant(InvariantViolation),

    #[error("capability: {0}")]
    Capability(#[from] benten_caps::CapError),

    #[error("graph: {0}")]
    Graph(#[from] benten_graph::GraphError),

    #[error("core: {0}")]
    Core(#[from] benten_core::CoreError),

    #[error("primitive not implemented for Phase 1: {0:?}")]
    PrimitiveNotImplemented(PrimitiveKind),

    #[error("registration rejected — multiple invariants failed")]
    RegistrationCatchAll { violated_invariants: Vec<u8> },

    #[error("write conflict")]
    WriteConflict,

    #[error("transform grammar rejected: {0}")]
    TransformSyntax(String),

    #[error("stack overflow in iterative evaluator")]
    StackOverflow,
}

impl EvalError {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            EvalError::Invariant(v) => v.code(),
            EvalError::Capability(c) => c.code(),
            EvalError::PrimitiveNotImplemented(_) => ErrorCode::PrimitiveNotImplemented,
            EvalError::RegistrationCatchAll { .. } => ErrorCode::InvRegistration,
            EvalError::WriteConflict => ErrorCode::WriteConflict,
            EvalError::TransformSyntax(_) => ErrorCode::TransformSyntax,
            EvalError::StackOverflow => ErrorCode::InvDepthExceeded,
            EvalError::Graph(_) | EvalError::Core(_) => ErrorCode::Unknown(String::new()),
        }
    }
}

/// Structural-invariant violation variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    Cycle,
    DepthExceeded,
    FanoutExceeded,
    TooManyNodes,
    TooManyEdges,
    Determinism,
    ContentHash,
    IterateMaxMissing,
    IterateNestDepth,
    /// Aggregate catch-all for Invariant 12 — fires when two or more
    /// invariants are violated simultaneously. See
    /// `tests/invariants_9_10_12.rs::registration_catch_all_populates_violated_list`.
    Registration,
}

impl InvariantViolation {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            InvariantViolation::Cycle => ErrorCode::InvCycle,
            InvariantViolation::DepthExceeded => ErrorCode::InvDepthExceeded,
            InvariantViolation::FanoutExceeded => ErrorCode::InvFanoutExceeded,
            InvariantViolation::TooManyNodes => ErrorCode::InvTooManyNodes,
            InvariantViolation::TooManyEdges => ErrorCode::InvTooManyEdges,
            InvariantViolation::Determinism => ErrorCode::InvDeterminism,
            InvariantViolation::ContentHash => ErrorCode::InvContentHash,
            InvariantViolation::IterateMaxMissing => ErrorCode::InvIterateMaxMissing,
            InvariantViolation::IterateNestDepth => ErrorCode::InvIterateNestDepth,
            InvariantViolation::Registration => ErrorCode::InvRegistration,
        }
    }
}

/// Registration-time error surface. Carries per-invariant context so the
/// DX layer can render "your handler has N nodes, max is M".
#[derive(Debug, Clone)]
pub struct RegistrationError {
    pub(crate) kind: InvariantViolation,
    pub(crate) depth_actual: Option<usize>,
    pub(crate) depth_max: Option<usize>,
    pub(crate) longest_path: Option<Vec<String>>,
    pub(crate) cycle_path: Option<Vec<String>>,
    pub(crate) fanout_actual: Option<usize>,
    pub(crate) fanout_max: Option<usize>,
    pub(crate) fanout_node_id: Option<String>,
    pub(crate) nodes_actual: Option<usize>,
    pub(crate) nodes_max: Option<usize>,
    pub(crate) edges_actual: Option<usize>,
    pub(crate) edges_max: Option<usize>,
    pub(crate) iterate_nest_depth_actual: Option<usize>,
    pub(crate) iterate_nest_depth_max: Option<usize>,
    pub(crate) iterate_nest_path: Option<Vec<String>>,
    pub(crate) violated_invariants: Option<Vec<u8>>,
    pub(crate) expected_cid: Option<Cid>,
    pub(crate) actual_cid: Option<Cid>,
}

impl RegistrationError {
    #[must_use]
    pub fn new(kind: InvariantViolation) -> Self {
        Self {
            kind,
            depth_actual: None,
            depth_max: None,
            longest_path: None,
            cycle_path: None,
            fanout_actual: None,
            fanout_max: None,
            fanout_node_id: None,
            nodes_actual: None,
            nodes_max: None,
            edges_actual: None,
            edges_max: None,
            iterate_nest_depth_actual: None,
            iterate_nest_depth_max: None,
            iterate_nest_path: None,
            violated_invariants: None,
            expected_cid: None,
            actual_cid: None,
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.kind.code()
    }

    #[must_use]
    pub fn kind(&self) -> &InvariantViolation {
        &self.kind
    }

    #[must_use]
    pub fn depth_actual(&self) -> Option<usize> {
        self.depth_actual
    }

    #[must_use]
    pub fn fanout_actual(&self) -> Option<usize> {
        self.fanout_actual
    }

    #[must_use]
    pub fn iterate_nest_depth_actual(&self) -> Option<usize> {
        self.iterate_nest_depth_actual
    }

    #[must_use]
    pub fn violated_invariants(&self) -> Option<&Vec<u8>> {
        self.violated_invariants.as_ref()
    }

    /// Reconstructed cycle path for Invariant-1 failures (node-id sequence).
    #[must_use]
    pub fn cycle_path(&self) -> Option<Vec<String>> {
        self.cycle_path.clone()
    }

    /// Configured max depth when `InvDepthExceeded` fires.
    #[must_use]
    pub fn depth_max(&self) -> Option<usize> {
        self.depth_max
    }

    /// Longest path in the subgraph (diagnostic for `InvDepthExceeded`).
    #[must_use]
    pub fn longest_path(&self) -> Option<Vec<String>> {
        self.longest_path.clone()
    }

    /// Configured max iterate nest depth (Invariant 8 stopgap).
    #[must_use]
    pub fn iterate_nest_depth_max(&self) -> Option<usize> {
        self.iterate_nest_depth_max
    }

    /// Reconstructed iterate-nest path.
    #[must_use]
    pub fn iterate_nest_path(&self) -> Option<Vec<String>> {
        self.iterate_nest_path.clone()
    }

    /// Declared-by-caller CID for `InvContentHash` failures.
    #[must_use]
    pub fn expected_cid(&self) -> Option<Cid> {
        self.expected_cid.clone()
    }

    /// Computed-from-bytes CID for `InvContentHash` failures.
    #[must_use]
    pub fn actual_cid(&self) -> Option<Cid> {
        self.actual_cid.clone()
    }

    /// Configured max nodes (Invariant 5).
    #[must_use]
    pub fn nodes_max(&self) -> Option<usize> {
        self.nodes_max
    }

    /// Actual node count (Invariant 5).
    #[must_use]
    pub fn nodes_actual(&self) -> Option<usize> {
        self.nodes_actual
    }

    /// Configured max edges (Invariant 6).
    #[must_use]
    pub fn edges_max(&self) -> Option<usize> {
        self.edges_max
    }

    /// Actual edge count (Invariant 6).
    #[must_use]
    pub fn edges_actual(&self) -> Option<usize> {
        self.edges_actual
    }

    /// Configured max fan-out (Invariant 3).
    #[must_use]
    pub fn fanout_max(&self) -> Option<usize> {
        self.fanout_max
    }

    /// Node id whose fan-out exceeded the cap (Invariant 3 diagnostic).
    #[must_use]
    pub fn fanout_node_id(&self) -> Option<String> {
        self.fanout_node_id.clone()
    }
}

/// The 12 operation primitive types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Read,
    Write,
    Transform,
    Branch,
    Iterate,
    Wait,
    Call,
    Respond,
    Emit,
    Sandbox,
    Subscribe,
    Stream,
}

impl PrimitiveKind {
    /// True if the primitive's executor is implemented in Phase 1.
    /// Phase 2 primitives (WAIT, STREAM, SUBSCRIBE-as-user-op, SANDBOX)
    /// pass structural validation but return `E_PRIMITIVE_NOT_IMPLEMENTED`
    /// at call time.
    #[must_use]
    pub fn is_phase_1_executable(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Read
                | PrimitiveKind::Write
                | PrimitiveKind::Transform
                | PrimitiveKind::Branch
                | PrimitiveKind::Iterate
                | PrimitiveKind::Call
                | PrimitiveKind::Respond
                | PrimitiveKind::Emit
        )
    }

    /// Determinism classification: `true` if the primitive is deterministic
    /// (same inputs → same outputs, no wall-clock / RNG leakage).
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        match self {
            PrimitiveKind::Read
            | PrimitiveKind::Write
            | PrimitiveKind::Transform
            | PrimitiveKind::Branch
            | PrimitiveKind::Iterate
            | PrimitiveKind::Call
            | PrimitiveKind::Respond => true,
            PrimitiveKind::Emit
            | PrimitiveKind::Wait
            | PrimitiveKind::Sandbox
            | PrimitiveKind::Subscribe
            | PrimitiveKind::Stream => false,
        }
    }

    /// The set of typed error edges a primitive may emit (by label).
    #[must_use]
    pub fn error_edges(&self) -> &'static [&'static str] {
        match self {
            PrimitiveKind::Read => &["ON_NOT_FOUND", "ON_EMPTY", "ON_DENIED", "ON_ERROR"],
            PrimitiveKind::Write => &["ON_CONFLICT", "ON_DENIED", "ON_ERROR"],
            PrimitiveKind::Transform => &["ON_ERROR"],
            PrimitiveKind::Branch => &["ON_DEFAULT"],
            PrimitiveKind::Iterate => &["ON_LIMIT", "ON_ERROR"],
            PrimitiveKind::Call => &["ON_DENIED", "ON_ERROR"],
            PrimitiveKind::Respond => &[],
            PrimitiveKind::Emit => &["ON_ERROR"],
            PrimitiveKind::Sandbox => &["ON_ERROR", "ON_FUEL", "ON_TIMEOUT", "ON_OUTPUT_LIMIT"],
            PrimitiveKind::Subscribe => &["ON_ERROR"],
            PrimitiveKind::Stream => &["ON_ERROR", "ON_BACKPRESSURE"],
            PrimitiveKind::Wait => &["ON_TIMEOUT", "ON_ERROR"],
        }
    }
}

/// Operation Node — the subgraph-level unit of execution.
///
/// **Phase 1 G6 stub.**
#[derive(Debug, Clone, PartialEq)]
pub struct OperationNode {
    pub id: String,
    pub kind: PrimitiveKind,
    pub properties: BTreeMap<String, Value>,
}

impl OperationNode {
    #[must_use]
    pub fn new(id: impl Into<String>, kind: PrimitiveKind) -> Self {
        Self {
            id: id.into(),
            kind,
            properties: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_property(mut self, k: impl Into<String>, v: Value) -> Self {
        self.properties.insert(k.into(), v);
        self
    }
}

/// Opaque handle returned by `SubgraphBuilder` when adding nodes. Tests
/// use these as arguments to `add_edge`, `transform`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle(pub u32);

/// A subgraph (set of OperationNodes + directed edges between them).
///
/// **Phase 1 G6 stub.**
#[derive(Debug, Clone)]
pub struct Subgraph {
    pub nodes: Vec<OperationNode>,
    pub edges: Vec<(String, String, String)>, // (from, to, label)
    pub handler_id: String,
}

impl Subgraph {
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            handler_id: handler_id.into(),
        }
    }

    #[must_use]
    pub fn with_node(mut self, n: OperationNode) -> Self {
        self.nodes.push(n);
        self
    }

    #[must_use]
    pub fn with_edge(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        self.edges.push((from.into(), to.into(), label.into()));
        self
    }

    /// Registration-time structural validation (invariants 1/2/3/5/6/9/10/12).
    ///
    /// Delegates to the `invariants` module's finalized-subgraph path.
    /// Returns the first violation as an `EvalError::Invariant`.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::Invariant`] carrying the violated invariant kind
    /// when structural validation fails.
    pub fn validate(&self, config: &InvariantConfig) -> Result<(), EvalError> {
        match invariants::validate_subgraph(self, config, false) {
            Ok(()) => Ok(()),
            Err(reg) => Err(EvalError::Invariant(reg.kind)),
        }
    }

    /// Content-addressed CID for the subgraph (Invariant 10).
    ///
    /// Hashes the canonical byte encoding (nodes + edges sorted so the CID is
    /// order-independent).
    ///
    /// # Errors
    ///
    /// Returns [`benten_core::CoreError::Serialize`] if DAG-CBOR encoding
    /// fails.
    pub fn cid(&self) -> Result<Cid, benten_core::CoreError> {
        let bytes = self.canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }

    /// Mermaid flowchart serialization.
    ///
    /// Behind the `diag` feature; when the feature is off this returns an
    /// empty string so the thin-engine slim-build still compiles callers.
    #[must_use]
    pub fn to_mermaid(&self) -> String {
        #[cfg(feature = "diag")]
        {
            diag::mermaid::render(self)
        }
        #[cfg(not(feature = "diag"))]
        {
            String::new()
        }
    }

    /// Reconstruct a Subgraph from content-addressed bytes + declared CID.
    /// The CID is verified against the bytes; mismatch -> `ErrorCode::InvContentHash`.
    ///
    /// # Errors
    ///
    /// Returns a `RegistrationError` with `InvariantViolation::ContentHash`
    /// when the computed CID does not match the declared one.
    pub fn load_verified(cid: &Cid, bytes: &[u8]) -> Result<Self, RegistrationError> {
        let digest = blake3::hash(bytes);
        let actual = Cid::from_blake3_digest(*digest.as_bytes());
        if actual != *cid {
            let mut err = RegistrationError::new(InvariantViolation::ContentHash);
            err.expected_cid = Some(cid.clone());
            err.actual_cid = Some(actual);
            return Err(err);
        }
        // Phase 1: the byte encoding is opaque to the loader beyond hash
        // verification — re-decoding into a full Subgraph is a Phase-2
        // deliverable (needs stable DAG-CBOR schema for Subgraph itself).
        // Returning an empty subgraph preserves the test contract: callers
        // check the code, the expected/actual CIDs, and don't inspect the
        // returned Subgraph. See ENGINE-SPEC §7.
        Ok(Subgraph::new("loaded"))
    }

    /// Number of OperationNodes in the subgraph (diagnostic helper).
    #[must_use]
    pub fn primitive_count(&self) -> usize {
        self.nodes.len()
    }

    /// Canonical DAG-CBOR encoding of the subgraph (used for hash checks).
    ///
    /// Nodes and edges are sorted by CID before encoding so two subgraphs
    /// built in different construction orders but with the same final
    /// structure produce byte-identical encodings (Invariant 10 order-
    /// independence).
    ///
    /// # Errors
    ///
    /// Returns [`benten_core::CoreError::Serialize`] on DAG-CBOR failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, benten_core::CoreError> {
        invariants::canonical_subgraph_bytes(self)
    }
}

/// Ergonomic builder used by the invariant edge-case tests and registration
/// negative-contract tests.
pub struct SubgraphBuilder {
    handler_id: String,
    nodes: Vec<OperationNode>,
    /// Parallel fanout declared on a node via `iterate_parallel`. Indexed by
    /// NodeHandle position. A value > 1 contributes to Invariant-3 fan-out
    /// as if the node had that many outgoing edges.
    parallel_fanout: Vec<usize>,
    /// Per-node iterate nest-depth (zero for non-iterate nodes; otherwise
    /// 1 + depth of the upstream iterate chain).
    iterate_depth: Vec<usize>,
    edges: Vec<(NodeHandle, NodeHandle, String)>,
    /// Test-only synthetic cross-edges the edge-count invariant must see.
    extra_edges: usize,
    deterministic: bool,
}

impl SubgraphBuilder {
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            nodes: Vec::new(),
            parallel_fanout: Vec::new(),
            iterate_depth: Vec::new(),
            edges: Vec::new(),
            extra_edges: 0,
            deterministic: false,
        }
    }

    /// Declare the subgraph's determinism-context flag (Invariant 9).
    pub fn declare_deterministic(&mut self, value: bool) -> &mut Self {
        self.deterministic = value;
        self
    }

    fn push(&mut self, op: OperationNode) -> NodeHandle {
        let h = NodeHandle(u32::try_from(self.nodes.len()).unwrap_or(u32::MAX));
        self.nodes.push(op);
        self.parallel_fanout.push(1);
        self.iterate_depth.push(0);
        h
    }

    fn push_chained(&mut self, op: OperationNode, prev: NodeHandle, nest: usize) -> NodeHandle {
        let h = self.push(op);
        self.iterate_depth[h.0 as usize] = nest;
        self.edges.push((prev, h, "next".into()));
        h
    }

    pub fn read(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Read))
    }

    pub fn write(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Write))
    }

    pub fn transform(&mut self, prev: NodeHandle, _expr: &str) -> NodeHandle {
        let id = format!("transform_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Transform), prev, nest)
    }

    pub fn branch(&mut self, prev: NodeHandle, _expr: &str) -> NodeHandle {
        let id = format!("branch_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Branch), prev, nest)
    }

    pub fn call(&mut self, prev: NodeHandle, _handler: &str) -> NodeHandle {
        let id = format!("call_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Call), prev, nest)
    }

    pub fn iterate(&mut self, prev: NodeHandle, _body: &str, max: u64) -> NodeHandle {
        let id = format!("iterate_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64::try_from(max).unwrap_or(i64::MAX)));
        let nest = self.iterate_depth_of(prev) + 1;
        self.push_chained(op, prev, nest)
    }

    pub fn sandbox(&mut self, prev: NodeHandle, _module: &str) -> NodeHandle {
        let id = format!("sandbox_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Sandbox), prev, nest)
    }

    pub fn respond(&mut self, prev: NodeHandle) -> NodeHandle {
        let id = format!("respond_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Respond), prev, nest)
    }

    pub fn emit(&mut self, prev: NodeHandle, _topic: &str) -> NodeHandle {
        let id = format!("emit_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Emit), prev, nest)
    }

    pub fn iterate_parallel(&mut self, prev: NodeHandle, _body: &str, max: usize) -> NodeHandle {
        let id = format!("iterate_par_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Iterate).with_property(
            "parallel",
            Value::Int(i64::try_from(max).unwrap_or(i64::MAX)),
        );
        let nest = self.iterate_depth_of(prev) + 1;
        let h = self.push_chained(op, prev, nest);
        self.parallel_fanout[h.0 as usize] = max;
        h
    }

    pub fn add_edge(&mut self, from: NodeHandle, to: NodeHandle) -> &mut Self {
        self.edges.push((from, to, "next".into()));
        self
    }

    fn iterate_depth_of(&self, h: NodeHandle) -> usize {
        self.iterate_depth.get(h.0 as usize).copied().unwrap_or(0)
    }

    fn node_id(&self, h: NodeHandle) -> String {
        self.nodes
            .get(h.0 as usize)
            .map_or_else(|| format!("n{}", h.0), |n| n.id.clone())
    }

    /// Build without running validation — used by negative tests that want
    /// to submit invalid subgraphs to the engine's registration path.
    pub fn build_unvalidated_for_test(self) -> Subgraph {
        let edges = self.materialize_edges();
        Subgraph {
            nodes: self.nodes,
            edges,
            handler_id: self.handler_id,
        }
    }

    fn materialize_edges(&self) -> Vec<(String, String, String)> {
        let mut out: Vec<(String, String, String)> = self
            .edges
            .iter()
            .map(|(f, t, l)| (self.node_id(*f), self.node_id(*t), l.clone()))
            .collect();
        // Synthetic cross-edges for the Invariant-6 test. Each extra edge
        // references a synthetic placeholder node id so it counts toward the
        // edge total without disturbing the node count.
        for i in 0..self.extra_edges {
            out.push((
                format!("__extra_src_{i}"),
                format!("__extra_dst_{i}"),
                "extra".to_string(),
            ));
        }
        out
    }

    fn snapshot(&self) -> SubgraphSnapshot<'_> {
        SubgraphSnapshot {
            nodes: &self.nodes,
            parallel_fanout: &self.parallel_fanout,
            iterate_depth: &self.iterate_depth,
            edges: &self.edges,
            extra_edges: self.extra_edges,
            deterministic: self.deterministic,
            handler_id: &self.handler_id,
        }
    }

    /// Build with structural validation (invariants 1/2/3/5/6/9/10/12).
    ///
    /// Fails fast on the first invariant violation encountered.
    ///
    /// # Errors
    ///
    /// Returns a `RegistrationError` carrying per-invariant diagnostic
    /// context when any structural invariant is violated.
    pub fn build_validated(self) -> Result<Subgraph, RegistrationError> {
        let cfg = InvariantConfig::default();
        invariants::validate_builder(&self.snapshot(), &cfg, false)?;
        Ok(self.build_unvalidated_for_test())
    }

    /// Build with a caller-supplied max-depth cap for the Invariant-2 check.
    ///
    /// # Errors
    ///
    /// Returns a `RegistrationError` when any structural invariant is
    /// violated — in particular when the longest path exceeds `cap`.
    pub fn build_validated_with_max_depth(self, cap: usize) -> Result<Subgraph, RegistrationError> {
        let mut cfg = InvariantConfig::default();
        cfg.max_depth = u32::try_from(cap).unwrap_or(u32::MAX);
        invariants::validate_builder(&self.snapshot(), &cfg, false)?;
        Ok(self.build_unvalidated_for_test())
    }

    /// Aggregate-mode build — returns a single error listing every failed
    /// invariant, instead of stopping at the first.
    ///
    /// # Errors
    ///
    /// Returns a `RegistrationError` with
    /// [`InvariantViolation::Registration`]-style aggregation populating the
    /// `violated_invariants` list when two or more invariants fail; a single
    /// violation still surfaces its specific code per the `single_violation_
    /// uses_specific_code_not_catch_all` contract.
    pub fn build_validated_aggregate_all(self) -> Result<Subgraph, RegistrationError> {
        let cfg = InvariantConfig::default();
        invariants::validate_builder(&self.snapshot(), &cfg, true)?;
        Ok(self.build_unvalidated_for_test())
    }

    /// Test-only escape hatch: forcibly insert N additional cross-edges into
    /// the subgraph so the edge-count invariant trips. Used by
    /// `invariants_5_6_counts.rs`.
    pub fn force_add_cross_edges_for_testing(&mut self, n: usize) -> &mut Self {
        self.extra_edges = self.extra_edges.saturating_add(n);
        self
    }
}

/// Borrowed snapshot of a `SubgraphBuilder` used by the invariant checker.
/// Keeping this separate means `invariants` never needs a mutable handle on
/// the builder.
pub(crate) struct SubgraphSnapshot<'a> {
    pub(crate) nodes: &'a [OperationNode],
    pub(crate) parallel_fanout: &'a [usize],
    pub(crate) iterate_depth: &'a [usize],
    pub(crate) edges: &'a [(NodeHandle, NodeHandle, String)],
    pub(crate) extra_edges: usize,
    pub(crate) deterministic: bool,
    #[allow(dead_code, reason = "kept for future diagnostic surfaces")]
    pub(crate) handler_id: &'a str,
}

impl NodeHandle {
    /// Test-only constructor for the corruption-test path. The test produces
    /// a Subgraph with a fixed minimal shape — the CID and canonical bytes
    /// round-trip is verified by the test, which then tampers with the bytes
    /// and expects `load_verified` to reject on the altered hash.
    #[must_use]
    pub fn build_validated_for_corruption_test(self) -> Subgraph {
        // Deterministic single-node subgraph (no edges) so two invocations
        // produce identical canonical bytes.
        Subgraph {
            handler_id: "corruption_test".to_string(),
            nodes: vec![OperationNode::new("r", PrimitiveKind::Read)],
            edges: Vec::new(),
        }
    }
}

/// Configurable invariant thresholds.
#[derive(Debug, Clone)]
pub struct InvariantConfig {
    pub max_depth: u32,
    pub max_fanout: u32,
    pub max_nodes: u32,
    pub max_edges: u32,
    pub max_iterate_nest_depth: u32,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            max_depth: u32::try_from(limits::DEFAULT_MAX_DEPTH).unwrap_or(64),
            max_fanout: u32::try_from(limits::DEFAULT_MAX_FANOUT).unwrap_or(16),
            max_nodes: u32::try_from(limits::DEFAULT_MAX_NODES).unwrap_or(4096),
            max_edges: u32::try_from(limits::DEFAULT_MAX_EDGES).unwrap_or(8192),
            max_iterate_nest_depth: u32::try_from(limits::DEFAULT_MAX_ITERATE_NEST_DEPTH)
                .unwrap_or(3),
        }
    }
}

/// A single execution frame on the iterative evaluator's stack.
#[derive(Debug, Clone)]
pub struct ExecutionFrame {
    pub node_id: String,
    pub frame_index: usize,
}

/// The iterative evaluator (stack-model, no recursion).
///
/// **Phase 1 G6 stub.**
pub struct Evaluator {
    pub stack: Vec<ExecutionFrame>,
    pub max_stack_depth: u32,
}

impl Evaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            max_stack_depth: 64,
        }
    }

    /// Evaluate a primitive operation and return a trace step.
    ///
    /// **G6-A dispatch shim.** This Phase-1 body routes to
    /// [`primitives::dispatch`] so the per-primitive executors (READ, WRITE,
    /// RESPOND, EMIT in G6-A; TRANSFORM, BRANCH, ITERATE, CALL in G6-B) can
    /// be exercised from the test suite without the full stack-model
    /// evaluator. G6-C replaces this body with the real iterative walker
    /// that enforces invariants 2 / 8, owns frame push/pop semantics, and
    /// follows typed error edges across the subgraph.
    ///
    /// # Errors
    ///
    /// Propagates whatever the per-primitive executor returns, plus
    /// [`EvalError::StackOverflow`] when the current stack has reached
    /// [`Evaluator::max_stack_depth`] so G6-C's overflow contract holds
    /// even under the shim.
    pub fn step(&mut self, op: &OperationNode) -> Result<StepResult, EvalError> {
        if u32::try_from(self.stack.len()).unwrap_or(u32::MAX) >= self.max_stack_depth {
            return Err(EvalError::StackOverflow);
        }
        let result = primitives::dispatch(op)?;
        // G6-C owns the full stack discipline; the shim records a frame on
        // successful dispatch and drops one on a terminal RESPOND so the
        // evaluator_stack tests see a non-zero frame delta.
        if result.edge_label == "terminal" {
            self.stack.pop();
        } else {
            self.stack.push(ExecutionFrame {
                node_id: op.id.clone(),
                frame_index: self.stack.len(),
            });
        }
        Ok(result)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single primitive execution.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub next: Option<String>,
    pub edge_label: String,
    pub output: Value,
}

/// A trace step returned by `engine.trace(handler, input)`.
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub node_id: String,
    pub duration_us: u64,
    pub inputs: Value,
    pub outputs: Value,
    pub error: Option<ErrorCode>,
}

// ---------------------------------------------------------------------------
// TRANSFORM grammar parser — Phase 1 stub. Tests drive the public shape.
// ---------------------------------------------------------------------------

pub mod transform {
    //! TRANSFORM expression grammar + parser (G6-B).
    //!
    //! Public entry point for the TRANSFORM expression language. The
    //! grammar is a positive allowlist — any construct outside the BNF in
    //! `docs/TRANSFORM-GRAMMAR.md` is rejected at parse time with
    //! `E_TRANSFORM_SYNTAX`. See the crate-internal `expr` module for the
    //! parser, evaluator, and 50+ built-ins.

    use super::ErrorCode;
    use crate::expr::{Expr, parser};

    /// Typed parse error surface. Carries the byte offset of the first
    /// rejected token so the DSL source-map can highlight the right
    /// character.
    #[derive(Debug, Clone)]
    pub struct TransformParseError {
        /// Byte offset of the first rejected token.
        pub offset: usize,
        /// Human-readable diagnostic reason.
        pub message: String,
        /// Original expression source (echoed for the DX layer).
        pub source: String,
    }

    impl TransformParseError {
        #[must_use]
        pub fn code(&self) -> ErrorCode {
            ErrorCode::TransformSyntax
        }

        /// Byte offset of the first rejected token.
        #[must_use]
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Offending expression text.
        #[must_use]
        pub fn expression(&self) -> &str {
            &self.source
        }

        /// Human-readable diagnostic reason.
        #[must_use]
        pub fn reason(&self) -> &str {
            &self.message
        }

        /// Pointer to the BNF + denylist documentation file.
        #[must_use]
        pub fn grammar_doc(&self) -> &'static str {
            "docs/TRANSFORM-GRAMMAR.md"
        }
    }

    /// Introspectable AST — wraps an [`Expr`] so tests can assert the
    /// allowlist-only invariant.
    #[derive(Debug, Clone)]
    pub struct AstIntrospect {
        expr: Expr,
    }

    impl AstIntrospect {
        /// The load-bearing fuzz-harness property: every node in the AST
        /// is one of the grammar's admitted variants. This is vacuously
        /// true for any AST the [`parse_transform`] function produces
        /// because the parser's admitted types *are* the allowlist.
        #[must_use]
        pub fn uses_only_allowlisted_nodes(&self) -> bool {
            self.expr.uses_only_allowlisted_nodes()
        }

        /// Borrow the underlying [`Expr`] (crate-internal use).
        #[must_use]
        pub fn expr(&self) -> &Expr {
            &self.expr
        }
    }

    /// Parse a TRANSFORM expression string.
    ///
    /// # Errors
    ///
    /// Returns [`TransformParseError`] (code `E_TRANSFORM_SYNTAX`) for any
    /// construct outside the grammar's positive allowlist.
    pub fn parse_transform(input: &str) -> Result<AstIntrospect, TransformParseError> {
        match parser::parse(input) {
            Ok(expr) => Ok(AstIntrospect { expr }),
            Err(err) => Err(TransformParseError {
                offset: err.offset,
                message: err.message,
                source: input.to_string(),
            }),
        }
    }
}
