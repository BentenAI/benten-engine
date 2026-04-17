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

pub use benten_core::ErrorCode;
use benten_core::{Cid, Value};
use std::collections::BTreeMap;

pub mod context;
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
        }
    }
}

/// Registration-time error surface. Carries per-invariant context so the
/// DX layer can render "your handler has N nodes, max is M".
#[derive(Debug)]
pub struct RegistrationError {
    kind: InvariantViolation,
    depth_actual: Option<usize>,
    fanout_actual: Option<usize>,
    iterate_nest_depth_actual: Option<usize>,
    violated_invariants: Option<Vec<u8>>,
}

impl RegistrationError {
    #[must_use]
    pub fn new(kind: InvariantViolation) -> Self {
        Self {
            kind,
            depth_actual: None,
            fanout_actual: None,
            iterate_nest_depth_actual: None,
            violated_invariants: None,
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.kind.code()
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
    /// **Phase 1 stub** — returns `None` until R5.
    #[must_use]
    pub fn cycle_path(&self) -> Option<Vec<String>> {
        todo!("RegistrationError::cycle_path — E5 (Phase 1)")
    }

    /// Configured max depth when `InvDepthExceeded` fires.
    #[must_use]
    pub fn depth_max(&self) -> Option<usize> {
        todo!("RegistrationError::depth_max — E5 (Phase 1)")
    }

    /// Longest path in the subgraph (diagnostic for `InvDepthExceeded`).
    #[must_use]
    pub fn longest_path(&self) -> Option<Vec<String>> {
        todo!("RegistrationError::longest_path — E5 (Phase 1)")
    }

    /// Configured max iterate nest depth (Invariant 8 stopgap).
    #[must_use]
    pub fn iterate_nest_depth_max(&self) -> Option<usize> {
        todo!("RegistrationError::iterate_nest_depth_max — E5 (Phase 1)")
    }

    /// Reconstructed iterate-nest path.
    #[must_use]
    pub fn iterate_nest_path(&self) -> Option<Vec<String>> {
        todo!("RegistrationError::iterate_nest_path — E5 (Phase 1)")
    }

    /// Declared-by-caller CID for `InvContentHash` failures.
    #[must_use]
    pub fn expected_cid(&self) -> Option<benten_core::Cid> {
        todo!("RegistrationError::expected_cid — E5 (Phase 1)")
    }

    /// Computed-from-bytes CID for `InvContentHash` failures.
    #[must_use]
    pub fn actual_cid(&self) -> Option<benten_core::Cid> {
        todo!("RegistrationError::actual_cid — E5 (Phase 1)")
    }

    /// Configured max nodes (Invariant 5).
    #[must_use]
    pub fn nodes_max(&self) -> Option<usize> {
        todo!("RegistrationError::nodes_max — E5 (Phase 1)")
    }

    /// Actual node count (Invariant 5).
    #[must_use]
    pub fn nodes_actual(&self) -> Option<usize> {
        todo!("RegistrationError::nodes_actual — E5 (Phase 1)")
    }

    /// Configured max edges (Invariant 6).
    #[must_use]
    pub fn edges_max(&self) -> Option<usize> {
        todo!("RegistrationError::edges_max — E5 (Phase 1)")
    }

    /// Actual edge count (Invariant 6).
    #[must_use]
    pub fn edges_actual(&self) -> Option<usize> {
        todo!("RegistrationError::edges_actual — E5 (Phase 1)")
    }

    /// Configured max fan-out (Invariant 3).
    #[must_use]
    pub fn fanout_max(&self) -> Option<usize> {
        todo!("RegistrationError::fanout_max — E5 (Phase 1)")
    }

    /// Node id whose fan-out exceeded the cap (Invariant 3 diagnostic).
    #[must_use]
    pub fn fanout_node_id(&self) -> Option<String> {
        todo!("RegistrationError::fanout_node_id — E5 (Phase 1)")
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
    pub fn validate(&self, _config: &InvariantConfig) -> Result<(), EvalError> {
        todo!("Subgraph::validate — E5 (Phase 1)")
    }

    /// Content-addressed CID.
    pub fn cid(&self) -> Result<Cid, benten_core::CoreError> {
        todo!("Subgraph::cid — E5 (Phase 1)")
    }

    /// Mermaid flowchart serialization (diag feature).
    pub fn to_mermaid(&self) -> String {
        todo!("Subgraph::to_mermaid — E7 (Phase 1)")
    }

    /// Reconstruct a Subgraph from content-addressed bytes + declared CID.
    /// The CID is verified against the bytes; mismatch -> `ErrorCode::InvContentHash`.
    ///
    /// **Phase 1 G6 stub.**
    pub fn load_verified(_cid: &Cid, _bytes: &[u8]) -> Result<Self, RegistrationError> {
        todo!("Subgraph::load_verified — G6 (Phase 1)")
    }

    /// Number of OperationNodes in the subgraph (diagnostic helper).
    #[must_use]
    pub fn primitive_count(&self) -> usize {
        self.nodes.len()
    }

    /// Canonical DAG-CBOR encoding of the subgraph (used for hash checks).
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, benten_core::CoreError> {
        todo!("Subgraph::canonical_bytes — E5 (Phase 1)")
    }
}

/// Ergonomic builder used by the invariant edge-case tests and registration
/// negative-contract tests.
///
/// **Phase 1 G6 stub.**
pub struct SubgraphBuilder {
    handler_id: String,
    nodes: Vec<OperationNode>,
    edges: Vec<(NodeHandle, NodeHandle, String)>,
    deterministic: bool,
}

impl SubgraphBuilder {
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
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
        h
    }

    pub fn read(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Read))
    }

    pub fn write(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Write))
    }

    pub fn transform(&mut self, _prev: NodeHandle, _expr: &str) -> NodeHandle {
        self.push(OperationNode::new("transform", PrimitiveKind::Transform))
    }

    pub fn branch(&mut self, _prev: NodeHandle, _expr: &str) -> NodeHandle {
        self.push(OperationNode::new("branch", PrimitiveKind::Branch))
    }

    pub fn call(&mut self, _prev: NodeHandle, _handler: &str) -> NodeHandle {
        self.push(OperationNode::new("call", PrimitiveKind::Call))
    }

    pub fn iterate(&mut self, _prev: NodeHandle, _body: &str, _max: u64) -> NodeHandle {
        self.push(OperationNode::new("iterate", PrimitiveKind::Iterate))
    }

    pub fn sandbox(&mut self, _prev: NodeHandle, _module: &str) -> NodeHandle {
        self.push(OperationNode::new("sandbox", PrimitiveKind::Sandbox))
    }

    pub fn respond(&mut self, _prev: NodeHandle) -> NodeHandle {
        self.push(OperationNode::new("respond", PrimitiveKind::Respond))
    }

    pub fn emit(&mut self, _prev: NodeHandle, _topic: &str) -> NodeHandle {
        self.push(OperationNode::new("emit", PrimitiveKind::Emit))
    }

    pub fn iterate_parallel(&mut self, _prev: NodeHandle, _body: &str, _max: usize) -> NodeHandle {
        self.push(OperationNode::new("iterate_par", PrimitiveKind::Iterate))
    }

    pub fn add_edge(&mut self, from: NodeHandle, to: NodeHandle) -> &mut Self {
        self.edges.push((from, to, "next".into()));
        self
    }

    /// Build without running validation — used by negative tests that want
    /// to submit invalid subgraphs to the engine's registration path.
    pub fn build_unvalidated_for_test(self) -> Subgraph {
        let id_for = |h: NodeHandle| format!("n{}", h.0);
        let edges = self
            .edges
            .into_iter()
            .map(|(f, t, l)| (id_for(f), id_for(t), l))
            .collect();
        Subgraph {
            nodes: self.nodes,
            edges,
            handler_id: self.handler_id,
        }
    }

    /// Build with structural validation (invariants 1/2/3/5/6/9/10/12).
    ///
    /// **Phase 1 G6 stub.**
    pub fn build_validated(self) -> Result<Subgraph, RegistrationError> {
        todo!("SubgraphBuilder::build_validated — E5 (Phase 1)")
    }

    /// Build with a caller-supplied max-depth cap for the Invariant-2 check.
    /// **Phase 1 G6 stub.**
    pub fn build_validated_with_max_depth(
        self,
        _cap: usize,
    ) -> Result<Subgraph, RegistrationError> {
        todo!("SubgraphBuilder::build_validated_with_max_depth — E5 (Phase 1)")
    }

    /// Aggregate-mode build — returns a single error listing every failed
    /// invariant, instead of stopping at the first.
    pub fn build_validated_aggregate_all(self) -> Result<Subgraph, RegistrationError> {
        todo!("SubgraphBuilder::build_validated_aggregate_all — E5 (Phase 1)")
    }

    /// Test-only escape hatch: forcibly insert N additional cross-edges into
    /// the subgraph so the edge-count invariant trips. Used by
    /// `invariants_5_6_counts.rs`.
    pub fn force_add_cross_edges_for_testing(&mut self, _n: usize) -> &mut Self {
        todo!("SubgraphBuilder::force_add_cross_edges_for_testing — E5 (Phase 1)")
    }
}

impl NodeHandle {
    /// Test-only constructor for the corruption-test path. The test produces
    /// a Subgraph with a deliberately-wrong content hash so registration
    /// rejects with `InvContentHash`.
    ///
    /// **Phase 1 G6 stub** — returns a placeholder Subgraph today.
    pub fn build_validated_for_corruption_test(self) -> Subgraph {
        todo!("NodeHandle::build_validated_for_corruption_test — E5 (Phase 1)")
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
    //! TRANSFORM expression grammar + parser (Phase 1 stub).

    use super::ErrorCode;

    /// Typed parse error surface. Carries the byte offset of the first
    /// rejected token so the DSL source-map can highlight the right character.
    #[derive(Debug, Clone)]
    pub struct TransformParseError {
        pub offset: usize,
        pub message: String,
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

        /// Offending expression text. **Phase 1 stub** — today returns
        /// the stored `message`; R5 threads the original input.
        #[must_use]
        pub fn expression(&self) -> &str {
            &self.message
        }

        /// Pointer to the BNF + denylist documentation file.
        #[must_use]
        pub fn grammar_doc(&self) -> &'static str {
            "docs/TRANSFORM-GRAMMAR.md"
        }
    }

    /// Introspectable AST node — tests call `.uses_only_allowlisted_nodes()`
    /// on the parse result to verify the allowlist invariant.
    #[derive(Debug, Clone)]
    pub struct AstIntrospect {
        _placeholder: (),
    }

    impl AstIntrospect {
        #[must_use]
        pub fn uses_only_allowlisted_nodes(&self) -> bool {
            todo!("AstIntrospect::uses_only_allowlisted_nodes — E4 (Phase 1)")
        }
    }

    /// Parse a TRANSFORM expression string. Returns `Err(E_TRANSFORM_SYNTAX)`
    /// for anything outside the grammar's allowlist.
    pub fn parse_transform(_input: &str) -> Result<AstIntrospect, TransformParseError> {
        todo!("parse_transform — E4 (Phase 1)")
    }
}
