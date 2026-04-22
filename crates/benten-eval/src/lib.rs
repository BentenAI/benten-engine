//! # benten-eval — Operation primitives + evaluator (Phase 1 stubs)
//!
//! Phase 1 ships all 12 operation primitive *types* (so stored subgraphs
//! don't require re-registration when Phase 2 enables WAIT/STREAM/SUBSCRIBE/
//! SANDBOX executors) and executes 8 primitives in the iterative evaluator:
//! READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT.
//!
//! R3 stub scaffold — R5 implementation lands in Phase 1 proper.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    missing_docs,
    reason = "TODO(phase-2-docs): benten-eval has ~120 pub items (Subgraph builder, primitives, RegistrationError diagnostic fields, expr parser surface). Crate-root + module-root docs land Phase-1 R6; per-item sweep deferred to Phase-2 when the public surface is re-audited post-evaluator-completion."
)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries per-invariant diagnostic context (paths, expected/actual CIDs, counts) per R1 triage; Phase-2 will box large diagnostic payloads once the accessor set stabilises"
)]
#![allow(
    clippy::too_many_lines,
    reason = "Invariant-validation pass is intentionally linear so the code reads top-to-bottom as the invariant list"
)]

use benten_core::{Cid, Value};
pub use benten_errors::ErrorCode;
use std::collections::BTreeMap;

pub mod context;
pub mod diag;
pub mod evaluator;
pub mod exec_state;
pub mod expr;
pub mod host;
pub mod host_error;
pub mod invariants;
pub mod primitives;
pub mod time_source;

pub use context::EvalContext;
pub use exec_state::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame};
pub use host::{NullHost, PrimitiveHost, ViewQuery};
pub use host_error::HostError;
pub use primitives::wait::{SignalShape, SuspendedHandle, WaitOutcome, WaitResumeSignal};
pub use time_source::{
    HlcTimeSource, InstantMonotonicSource, MockTimeSource, MonotonicSource, TimeSource,
    default_monotonic_source, default_time_source,
};

/// Phase 2a G4-A test harness: register a callee handler with a declared
/// iteration-budget bound. Consumed by `invariant_8_isolated_call`.
///
/// TODO(phase-2a-G4-A): swap in a real handler-registration path.
pub fn register_test_callee(_name: &str, _bound: u64) {
    // Placeholder — R5 G4-A wires a real handler-registration table.
}

/// Phase 2a G3-B: crate-root alias for
/// [`primitives::wait::evaluate`]. Tests call `benten_eval::evaluate(...)`
/// through this re-export.
///
/// # Errors
/// See [`primitives::wait::evaluate`].
pub fn evaluate(sg: &Subgraph, ctx: &mut EvalContext, input: benten_core::Value) -> Outcome {
    match primitives::wait::evaluate(sg, ctx, input) {
        Ok(WaitOutcome::Complete(v)) => Outcome::Complete(v),
        Ok(wo @ WaitOutcome::Suspended(_)) => Outcome::Suspended(wo),
        Err(e) => Outcome::Err(e.code()),
    }
}

/// Phase 2a G3-B: crate-root alias for [`primitives::wait::resume`]. The
/// `handle` arg accepts a [`WaitOutcome`] so test harnesses that pipe
/// `evaluate(...).expect_suspended()` through `resume` compile without
/// mapping the `Outcome::Suspended(h)` arm back to a raw `SuspendedHandle`.
///
/// # Errors
/// See [`primitives::wait::resume`].
pub fn resume(
    _sg: &Subgraph,
    _ctx: &mut EvalContext,
    _handle: WaitOutcome,
    _signal: WaitResumeSignal,
) -> Outcome {
    // Phase-2a stub: run-time tests fail at the nested `todo!()`.
    todo!("Phase 2a G3-B: crate-root resume alias per wait_timeout / wait_signal_shape tests")
}

/// Phase 2a G4-A test harness: expose a budget-probe for the multiplicative
/// benchmark (`benches/multiplicative_budget_overhead.rs`).
pub mod testing {
    /// Phase-2a placeholder probe; Phase 2a G4-A benches call this to
    /// measure cumulative-budget computation overhead.
    ///
    /// TODO(phase-2a-G4-A): real probe implementation.
    #[must_use]
    pub fn multiplicative_budget_probe() -> u64 {
        0
    }
}

/// Phase 2a G3-A: `Outcome` shape mirrored from `benten-engine` so tests
/// can name `benten_eval::Outcome::{Complete, Suspended, Err}` alongside
/// `SuspendedHandle`. Phase-1 owns the real type in `benten-engine`; the
/// re-export is a narrow proxy whose variants match the expected surface.
///
/// `Suspended` carries a [`WaitOutcome`] — not a raw [`SuspendedHandle`] —
/// so test harnesses that pipe `evaluate(...).expect_suspended()` (declared
/// `-> WaitOutcome`) compile without mapping.
///
/// TODO(phase-2a-G3-B): unify the eval-side and engine-side `Outcome`s after
/// the WAIT surface is live.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Handler ran to completion.
    Complete(benten_core::Value),
    /// Handler suspended at a WAIT primitive. Carries the full
    /// [`WaitOutcome`] so multi-variant tests can re-inspect the shape.
    Suspended(WaitOutcome),
    /// Terminal error.
    Err(ErrorCode),
}

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
///
/// `#[non_exhaustive]` (R6b bp-17) — Phase 2 adds STREAM / WAIT / SUBSCRIBE /
/// SANDBOX runtime errors; downstream matchers must include `_ =>` so adding
/// variants is a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EvalError {
    #[error("invariant violation: {0:?}")]
    Invariant(InvariantViolation),

    #[error("capability: {0}")]
    Capability(#[from] benten_caps::CapError),

    /// Host-boundary failure surfaced through a [`PrimitiveHost`] call.
    /// Replaces the Phase-1 `Graph(GraphError)` variant as part of arch-1
    /// dep-break (phil-r1-2 / plan §9.10 + §9.14) — `benten-eval` no longer
    /// depends on `benten-graph`, so storage-layer rejections route through
    /// the opaque [`HostError`] envelope. The wrapped `HostError` carries a
    /// stable catalog code + optional context on the wire plus an opaque
    /// `Box<dyn StdError>` source that never reaches the wire (sec-r1-6 /
    /// atk-6).
    #[error("host: {0}")]
    Host(HostError),

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

    /// Backend / host-side error surfaced through a [`PrimitiveHost`] call.
    /// Used by primitive executors (READ, WRITE, CALL, EMIT, ITERATE) when
    /// the host implementation rejects or fails. The engine's `impl
    /// PrimitiveHost` populates this with a debug rendering of its own
    /// `EngineError`.
    #[error("backend: {0}")]
    Backend(String),

    /// An operation on the `PrimitiveHost` boundary is not yet supported by
    /// any Phase-1 replay path (r6b-ce-2). Distinct from
    /// `PrimitiveNotImplemented` (which names a structural primitive that
    /// the evaluator cannot execute) — `Unsupported` names a host-boundary
    /// method whose backing replay is not yet wired. Maps to
    /// `E_NOT_IMPLEMENTED` at the catalog layer so TS callers get the same
    /// stable code used elsewhere for deferred surfaces.
    #[error("unsupported host operation: {operation}")]
    Unsupported {
        /// Name of the unsupported operation, e.g. `"put_edge"`.
        operation: String,
    },

    /// Typed pass-through of an engine-side "unknown view" rejection
    /// (r6b-err-1). Carried so the origin catalog code
    /// (`ErrorCode::UnknownView`) survives the `PrimitiveHost::read_view`
    /// boundary; previously this collapsed into an opaque
    /// `EvalError::Backend(String)` with a debug-formatted payload.
    #[error("unknown view: {0}")]
    UnknownView(String),

    /// Typed pass-through of an engine-side "IVM view stale" rejection.
    /// Carried so `ErrorCode::IvmViewStale` survives the
    /// `PrimitiveHost::read_view` boundary (r6b-err-1).
    #[error("IVM view stale: {0}")]
    IvmViewStale(String),

    /// Typed pass-through of an engine-side "subsystem disabled"
    /// rejection — the thin-engine honest-no (`without_ivm`,
    /// `without_caps`). Carried so `ErrorCode::SubsystemDisabled` survives
    /// the host-boundary (r6b-err-1).
    #[error("subsystem disabled: {0}")]
    SubsystemDisabled(String),
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
            // Preserve the stable catalog code across the cross-crate
            // boundary. Prior to r6-err-1 these collapsed into `Unknown("")`,
            // which made `EvalError → EngineError → napi → TS` lose the
            // origin error code. Dispatch to inner `.code()` so the catalog
            // identifier survives the round-trip. arch-1 dep-break (G1-B):
            // the former `EvalError::Graph(GraphError)` arm is now
            // `EvalError::Host(HostError)`; HostError's `code` field is the
            // catalog discriminant.
            EvalError::Host(h) => h.code.clone(),
            EvalError::Core(e) => e.code(),
            // r6b-err-3: both `EvalError::Backend` and the engine-side
            // `eval_error_to_engine_error` now spell the stable string the
            // same way — the prior `E_BACKEND` / `E_EVAL_BACKEND` split
            // gave one conceptual state two catalog identifiers.
            EvalError::Backend(_) => ErrorCode::Unknown(String::from("E_EVAL_BACKEND")),
            EvalError::Unsupported { .. } => ErrorCode::NotImplemented,
            EvalError::UnknownView(_) => ErrorCode::UnknownView,
            EvalError::IvmViewStale(_) => ErrorCode::IvmViewStale,
            EvalError::SubsystemDisabled(_) => ErrorCode::SubsystemDisabled,
        }
    }
}

/// Structural-invariant violation variants.
///
/// `#[non_exhaustive]` (R6b bp-17) — invariants 4 (SANDBOX nest depth), 7
/// (SANDBOX output limit), 11 (system-zone reachability), 13 (immutability),
/// 14 (causal attribution) land in Phase 2 and each introduces a variant
/// here; downstream matchers must include `_ =>` so adding variants is a
/// minor version bump.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    /// Runtime cumulative-iteration-budget exhaustion (invariant 8 runtime
    /// leg). Distinct from `IterateNestDepth` (registration-time nesting
    /// stopgap). Fires from the iterative evaluator when the per-run step
    /// counter reaches `DEFAULT_ITERATION_BUDGET`. Maps to
    /// [`ErrorCode::InvIterateBudget`] / `E_INV_ITERATE_BUDGET`. See
    /// mini-review finding `g6-cag-1` / `g6-opl-6` / `g6-cr-2`.
    IterateBudget,
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
            InvariantViolation::IterateBudget => ErrorCode::InvIterateBudget,
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
        self.expected_cid
    }

    /// Computed-from-bytes CID for `InvContentHash` failures.
    #[must_use]
    pub fn actual_cid(&self) -> Option<Cid> {
        self.actual_cid
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
///
/// `#[non_exhaustive]` (R6b bp-17): while the set of 12 primitives is
/// deliberately closed by ENGINE-SPEC §3, `non_exhaustive` guards against
/// a future Phase 2+ decision to introduce a 13th primitive without forcing
/// a major-version bump across downstream matchers. The vocabulary is
/// architectural; the enum representation is a mechanical surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

    /// Determinism classification (Invariant 9).
    ///
    /// Returns `true` if this primitive's **output-to-caller** is a pure
    /// function of its inputs — repeat executions with identical inputs
    /// produce identical return values, with no wall-clock / RNG / network
    /// non-determinism leaking into the returned `Value`.
    ///
    /// Primitives with side effects (WRITE, RESPOND) are still classified
    /// `true` under this semantic: their observable return to the caller
    /// is determined only by inputs, and the side-effect itself (a storage
    /// mutation, a response emit) is separately tracked by the engine.
    /// ENGINE-SPEC §5 groups these as "deterministic-with-side-effects";
    /// for Invariant 9 purposes they fall on the deterministic side of the
    /// fence because a subgraph declared `deterministic: true` is allowed
    /// to mutate storage and return a response as long as the return value
    /// itself is input-determined.
    ///
    /// This flag is **not** "safe to replay without rerunning side effects"
    /// — for that, Phase 2 adds a separate replay-safety classification
    /// (or an idempotency marker). See mini-review finding `g6-opl-2`.
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
            // ON_LIMIT routes on timeout (see `primitives/call.rs`); the
            // structural validator must accept it or registration-time
            // edge-label validation would reject a valid CALL subgraph
            // (mini-review finding `g6-opl-1`).
            PrimitiveKind::Call => &["ON_DENIED", "ON_LIMIT", "ON_ERROR"],
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

    /// Read a property by key.
    #[must_use]
    pub fn property(&self, k: &str) -> Option<&Value> {
        self.properties.get(k)
    }

    /// Alias for [`Self::kind`] — back-compat name used by Phase 2a tests.
    #[must_use]
    pub fn primitive_kind(&self) -> PrimitiveKind {
        self.kind
    }
}

/// Opaque handle returned by `SubgraphBuilder` when adding nodes. Tests
/// use these as arguments to `add_edge`, `transform`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle(pub u32);

/// A subgraph (set of OperationNodes + directed edges between them).
///
/// **Phase 1 G6 stub.**
///
/// Fields are `pub(crate)` so in-place mutation after `build_validated`
/// cannot corrupt the correspondence between the structure and its
/// computed CID (mini-review finding `g6-cag-3`). External access goes
/// through the [`nodes`](Self::nodes) / [`edges`](Self::edges) /
/// [`handler_id`](Self::handler_id) accessor methods. Phase 2 will add
/// the full immutability enforcement required by invariant 13.
#[derive(Debug, Clone)]
pub struct Subgraph {
    pub(crate) nodes: Vec<OperationNode>,
    pub(crate) edges: Vec<(String, String, String)>, // (from, to, label)
    pub(crate) handler_id: String,
    /// Invariant 9 — declared determinism context. Preserved across the
    /// builder-to-finalized projection (5d-J workstream 4) so the
    /// `validate_subgraph` path can re-run the per-primitive
    /// determinism check on a round-tripped Subgraph. Defaults `false`
    /// (unconstrained) so the semantics of legacy builders that never
    /// set the flag are unchanged. Phase-2 threads this through the
    /// DAG-CBOR schema; until then the flag is in-memory only.
    pub(crate) deterministic: bool,
}

impl Subgraph {
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            handler_id: handler_id.into(),
            deterministic: false,
        }
    }

    /// True when the builder declared this handler deterministic via
    /// [`SubgraphBuilder::declare_deterministic`]. Invariant 9 rejects
    /// any non-deterministic primitive inside a deterministic handler
    /// at both builder and finalized-subgraph validation time.
    #[must_use]
    pub fn is_declared_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Declare this finalized Subgraph's determinism context after the
    /// fact. Useful when a Subgraph is materialised outside the builder
    /// path and the caller still wants Invariant 9 to fire at
    /// [`Subgraph::validate`] time. Mirrors
    /// [`SubgraphBuilder::declare_deterministic`].
    pub fn set_deterministic(&mut self, value: bool) {
        self.deterministic = value;
    }

    /// Read-only accessor for the subgraph's [`OperationNode`]s.
    #[must_use]
    pub fn nodes(&self) -> &[OperationNode] {
        &self.nodes
    }

    /// Read-only accessor for the subgraph's `(from, to, label)` edges.
    #[must_use]
    pub fn edges(&self) -> &[(String, String, String)] {
        &self.edges
    }

    /// Read-only accessor for the subgraph's stable handler id.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }

    /// Mutable accessor for the first `OperationNode`. Used by
    /// `benten-engine`'s `dispatch_call` to backfill properties on a
    /// synthesized READ / WRITE node after the builder has finalized the
    /// shape.
    ///
    /// # Safety / invariants
    ///
    /// Mutating the property set after `build_unvalidated_for_test` does NOT
    /// change the Subgraph's structural shape (node count, edge topology),
    /// so invariants 1–6 remain valid. The subgraph's CID will change —
    /// callers that mutate must re-compute `cid()` if they rely on it.
    pub fn first_op_mut(&mut self) -> Option<&mut OperationNode> {
        self.nodes.first_mut()
    }

    /// Mutable accessor for the (only) `Write` primitive node. Used by the
    /// `crud:create` / `crud:delete` dispatch shims in `benten-engine`.
    pub fn write_op_mut(&mut self) -> Option<&mut OperationNode> {
        self.nodes
            .iter_mut()
            .find(|n| matches!(n.kind, PrimitiveKind::Write))
    }

    /// Mutable accessor for the `OperationNode` whose id matches `id`.
    pub fn op_by_id_mut(&mut self, id: &str) -> Option<&mut OperationNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Phase 2a G4-A test helper: return the precomputed cumulative
    /// Inv-8 budget for the root frame. Stub — G4-A lands the real
    /// computation.
    ///
    /// Returns a `u64` directly; Phase-2a default is `0` so tests asserting
    /// non-zero budgets fail at run-time with a clear delta.
    #[must_use]
    pub fn cumulative_budget_for_root_for_test(&self) -> u64 {
        0
    }

    /// Phase 2a G4-A test helper: cumulative budget at an arbitrary handle,
    /// returned as `Option<u64>` so the `invariant_8_isolated_call` tests'
    /// `.expect()` / `.unwrap()` chains compile. Phase-2a default: `None`.
    #[must_use]
    pub fn cumulative_budget_for_handle_for_test(&self, _h: NodeHandle) -> Option<u64> {
        None
    }

    /// Phase 2a G4-A test helper: returns `true` once multiplicative Inv-8
    /// budget tracking is live.
    #[must_use]
    pub fn has_multiplicative_budget_tracked_for_test(&self) -> bool {
        false
    }

    /// Phase 2a G4-A test helper: return the `NodeHandle` for an operation
    /// node id.
    #[must_use]
    pub fn handle_of(&self, id: &str) -> NodeHandle {
        let idx = self.nodes.iter().position(|n| n.id == id).unwrap_or(0);
        NodeHandle(u32::try_from(idx).unwrap_or(u32::MAX))
    }

    /// Phase 2a G3-B test helper: empty Subgraph with the given handler id.
    #[must_use]
    pub fn empty_for_test(handler_id: impl Into<String>) -> Self {
        Self::new(handler_id)
    }

    /// Phase 2a G3-B test helper: look up a node by its handle.
    #[must_use]
    pub fn node_by_handle(&self, h: NodeHandle) -> Option<&OperationNode> {
        self.nodes.get(h.0 as usize)
    }

    /// Phase 2a C5 / G5-A: DAG-CBOR encode (stub — wired in G5-A).
    ///
    /// # Errors
    /// Returns [`benten_core::CoreError::Serialize`] on encode failure.
    pub fn to_dagcbor(&self) -> Result<Vec<u8>, benten_core::CoreError> {
        todo!("Phase 2a C5 / G5-A: Subgraph DAG-CBOR encode (eval side)")
    }

    /// Phase 2a C5 / G5-A: DAG-CBOR decode (stub — wired in G5-A).
    ///
    /// # Errors
    /// Returns [`benten_core::CoreError::Serialize`] on decode failure.
    pub fn from_dagcbor(_bytes: &[u8]) -> Result<Self, benten_core::CoreError> {
        todo!("Phase 2a C5 / G5-A: Subgraph DAG-CBOR decode (eval side)")
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
            err.expected_cid = Some(*cid);
            err.actual_cid = Some(actual);
            return Err(err);
        }
        // Phase 1: the byte encoding is opaque to the loader beyond hash
        // verification — re-decoding into a full Subgraph is a Phase-2
        // deliverable (needs stable DAG-CBOR schema for Subgraph itself).
        // Returning an empty subgraph preserves the test contract: callers
        // check the code, the expected/actual CIDs, and don't inspect the
        // returned Subgraph. See ENGINE-SPEC §7.
        //
        // TODO(R4b / Phase-2): nail down the CanonNode/CanonEdge decoder so
        // `load_verified` can return a fully reconstructed Subgraph; this
        // is the other half of the Invariant-9 round-trip work documented
        // in `invariants.rs::validate_subgraph`. Mini-review `g6-cag-6`.
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

    /// Phase 2a G3-B (dx-r1-8): WAIT signal variant. Sets the `signal`
    /// property on the created node.
    pub fn wait_signal(&mut self, prev: NodeHandle, signal_name: impl Into<String>) -> NodeHandle {
        let id = format!("wait_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Wait)
            .with_property("signal", Value::text(signal_name));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a G3-B: WAIT signal variant with optional static typing (DX
    /// signal-payload typing addendum). Takes a [`SignalShape`].
    pub fn wait_signal_typed(
        &mut self,
        prev: NodeHandle,
        signal_name: impl Into<String>,
        _shape: crate::SignalShape,
    ) -> NodeHandle {
        // TODO(phase-2a-G3-B): encode shape into a property.
        self.wait_signal(prev, signal_name)
    }

    /// Phase 2a G3-B: WAIT signal variant with explicit timeout.
    pub fn wait_signal_with_timeout(
        &mut self,
        prev: NodeHandle,
        signal_name: impl Into<String>,
        timeout: std::time::Duration,
    ) -> NodeHandle {
        let h = self.wait_signal(prev, signal_name);
        let idx = h.0 as usize;
        let ms = i64::try_from(timeout.as_millis()).unwrap_or(i64::MAX);
        if let Some(n) = self.nodes.get_mut(idx) {
            n.properties.insert("timeout_ms".into(), Value::Int(ms));
        }
        h
    }

    /// Phase 2a G3-B: WAIT duration variant (already-shipped in Phase 1 stub;
    /// signature kept stable).
    pub fn wait_duration(&mut self, prev: NodeHandle, duration: std::time::Duration) -> NodeHandle {
        let id = format!("wait_{}", self.nodes.len());
        let ms = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);
        let op = OperationNode::new(id, PrimitiveKind::Wait)
            .with_property("duration_ms", Value::Int(ms));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a G4-A / Code-as-graph Major #2: CALL with an explicit
    /// `isolated` flag. `isolated: true` resets the multiplicative budget
    /// to the callee grant's declared bound.
    pub fn call_with_isolated(
        &mut self,
        prev: NodeHandle,
        handler: &str,
        isolated: bool,
    ) -> NodeHandle {
        let id = format!("call_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Call)
            .with_property("handler", Value::text(handler.to_string()))
            .with_property("isolated", Value::Bool(isolated));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a test-only property setter — used by
    /// `wait_signal_shape_optional_typing` to inject malformed payloads.
    pub fn set_property_for_test(&mut self, h: NodeHandle, key: &str, value: Value) -> &mut Self {
        if let Some(n) = self.nodes.get_mut(h.0 as usize) {
            n.properties.insert(key.to_string(), value);
        }
        self
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
            // Invariant 9 (5d-J workstream 4): propagate the builder's
            // declared determinism flag into the finalized Subgraph so a
            // subsequent `validate_subgraph` call can re-run the Invariant-9
            // check without requiring the caller to re-declare.
            deterministic: self.deterministic,
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
            deterministic: false,
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
    pub fn step(
        &mut self,
        op: &OperationNode,
        host: &dyn PrimitiveHost,
    ) -> Result<StepResult, EvalError> {
        if u32::try_from(self.stack.len()).unwrap_or(u32::MAX) >= self.max_stack_depth {
            return Err(EvalError::StackOverflow);
        }
        let result = primitives::dispatch(op, host)?;
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
///
/// Phase 2a dx-r1 / §9.12: the Phase-1 single-variant shape is promoted to
/// an enum so the boundary/budget variants coexist with the per-primitive
/// `Step` rows.
///
/// TODO(phase-2a-G3-A / G4-A / G5-B): wire `SuspendBoundary`, `ResumeBoundary`,
/// `BudgetExhausted` firing + `attribution` threading onto every trace row.
#[derive(Debug, Clone)]
pub enum TraceStep {
    /// A single primitive execution row (Phase 1 baseline shape preserved
    /// as struct-variant).
    Step {
        /// Operation-node id within the handler.
        node_id: String,
        /// Duration in microseconds.
        duration_us: u64,
        /// Inputs to the primitive.
        inputs: Value,
        /// Outputs produced by the primitive.
        outputs: Value,
        /// Optional error code if the step failed.
        error: Option<ErrorCode>,
        /// Inv-14 attribution (G5-B-ii wires this). Phase-2a default
        /// constructs to `None` until the runtime attribution threader lands.
        attribution: Option<AttributionFrame>,
    },
    /// WAIT primitive drove the evaluator to suspension. Emitted as the
    /// terminal step for the suspended invocation (§9.1 G3-A).
    SuspendBoundary {
        /// CID of the persisted `ExecutionStateEnvelope`.
        state_cid: Cid,
    },
    /// Resume re-entered a suspended execution. Emitted as the first step
    /// after `Engine::resume_from_bytes` (§9.1 G3-A).
    ResumeBoundary {
        /// CID of the `ExecutionStateEnvelope` that was resumed.
        state_cid: Cid,
        /// Value handed to the resumed frame as the signal payload.
        signal_value: Value,
    },
    /// Invariant-8 / Phase-2b SANDBOX-fuel budget exhausted (§9.12).
    BudgetExhausted {
        /// `"inv_8_iteration"` | `"sandbox_fuel"`.
        budget_type: &'static str,
        /// How much budget was consumed before firing.
        consumed: u64,
        /// Configured limit.
        limit: u64,
        /// Path of operation-node ids that produced the exhaustion.
        path: Vec<String>,
    },
}

impl TraceStep {
    /// Convenience: return the primitive's `node_id` for `Step` rows;
    /// `None` for boundary / budget rows.
    #[must_use]
    pub fn node_id(&self) -> Option<&str> {
        match self {
            TraceStep::Step { node_id, .. } => Some(node_id.as_str()),
            _ => None,
        }
    }

    /// Inv-14 attribution accessor. `None` for boundary / budget rows in
    /// Phase 2a; will be `Some` once G5-B-ii wires runtime threading.
    #[must_use]
    pub fn attribution(&self) -> Option<&AttributionFrame> {
        match self {
            TraceStep::Step { attribution, .. } => attribution.as_ref(),
            _ => None,
        }
    }

    /// Phase-1 compat: the `duration_us` field on `Step` rows; `0` for
    /// boundary / budget rows.
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        match self {
            TraceStep::Step { duration_us, .. } => *duration_us,
            _ => 0,
        }
    }

    /// Phase-1 compat: the `error` field on `Step` rows; `None` otherwise.
    #[must_use]
    pub fn error(&self) -> Option<&ErrorCode> {
        match self {
            TraceStep::Step { error, .. } => error.as_ref(),
            _ => None,
        }
    }
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
