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
    reason = "TODO(phase-2b-docs): benten-eval has ~120 pub items (Subgraph builder, primitives, RegistrationError diagnostic fields, expr parser surface). Crate-root + module-root docs land Phase-1 R6; per-item sweep deferred to Phase-2b when the public surface is re-audited post-evaluator-completion."
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

pub use benten_core::{
    ATTRIBUTION_PROPERTY_KEY, NodeHandle, OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder,
};
use benten_core::{Cid, Value};
pub use benten_errors::ErrorCode;
use std::collections::BTreeMap;

pub mod chunk_sink;
pub mod context;
pub mod diag;
pub mod evaluator;
pub mod exec_state;
pub mod expr;
pub mod host;
pub mod host_error;
pub mod invariants;
pub mod primitives;
pub mod subgraph_ext;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod time_source;

pub use subgraph_ext::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};

pub use context::EvalContext;
pub use exec_state::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame};
pub use host::{NullHost, PrimitiveHost, ViewQuery};
pub use host_error::HostError;
pub use primitives::wait::{SignalShape, SuspendedHandle, WaitOutcome, WaitResumeSignal};
#[cfg(any(test, feature = "testing"))]
pub use time_source::MockMonotonicSource;
pub use time_source::{
    HlcTimeSource, InstantMonotonicSource, MockTimeSource, MonotonicSource, TimeSource,
    default_monotonic_source, default_time_source,
};

/// Phase 2a G4-A test harness: register a callee handler with a declared
/// iteration-budget bound. Consumed by `invariant_8_isolated_call` tests
/// so the Inv-8 multiplicative walker can look up the callee's bound at
/// registration time.
///
/// The registry is a process-global `RwLock<HashMap<String, u64>>`; the
/// table is append-only within a test process (a subsequent register of
/// the same name overwrites the prior entry). Real handler registration
/// is an engine-layer concern; this is purely a test-surface convenience
/// (Phase 2a testing-helpers contract — see plan §3 G4-A).
///
/// # Soundness gate (G4-A mini-review C1 + follow-up tightening)
/// This MUTATION surface is gated behind `cfg(any(test, feature =
/// "testing"))` so ANY non-test build (including dev-profile
/// `cargo build` / `cargo check`, release, bench, and custom deploy
/// profiles) cannot pre-seed the registry that `invariants::budget`
/// consults at registration-time validation. The READ path
/// (`lookup_test_callee`) stays unconditional — in a non-test build
/// the registry is always empty, and the Inv-8 validator rejects
/// unknown callees with `E_INV_REGISTRATION` (see G4-A mini-review M1
/// fix).
///
/// Integration tests that live in `crates/benten-eval/tests/*.rs` are
/// covered by the `cfg(test)` leg because cargo compiles the crate
/// with `--cfg test` when building them. Cross-crate test binaries
/// (e.g. a `benten-engine` integration test that wants to seed the
/// benten-eval registry) must explicitly opt into the `testing`
/// feature via `benten-eval = { path = "...", features = ["testing"]
/// }` under their `[dev-dependencies]`. Earlier iterations included a
/// `debug_assertions` leg for DX convenience; that leg was dropped
/// because `cargo build` (dev-profile) sets `debug_assertions = true`
/// and a compromised dep or accidentally-introduced production-path
/// call to `register_test_callee` would compile in that profile.
#[cfg(any(test, feature = "testing"))]
pub fn register_test_callee(name: &str, bound: u64) {
    let mut guard = TEST_CALLEE_REGISTRY
        .write()
        .expect("test callee registry poisoned");
    guard.insert(name.to_string(), bound);
}

/// Look up a previously-registered callee bound. Returns `None` when the
/// name has not been registered — the multiplicative walker treats a
/// CALL with a `handler` property naming an unregistered callee as an
/// Inv-8-rejectable registration error (Phase 2a G4-A M1 fix), so the
/// fallback is no longer "contribute factor 1."
#[must_use]
pub fn lookup_test_callee(name: &str) -> Option<u64> {
    TEST_CALLEE_REGISTRY
        .read()
        .expect("test callee registry poisoned")
        .get(name)
        .copied()
}

static TEST_CALLEE_REGISTRY: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<String, u64>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

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
/// # G3-B-cont guard (Complete-variant rejection)
///
/// The alias accepts only a `WaitOutcome::Suspended(_)` payload — passing
/// a `WaitOutcome::Complete(_)` is API misuse (there is no suspended
/// frame to resume, and the `Complete` arm's `state_cid()` returns a
/// zero-derived CID that would consult an unrelated metadata slot). We
/// surface `Outcome::Err(ErrorCode::InvRegistration)` for that case
/// rather than silently succeeding or panicking.
///
/// # Errors
/// See [`primitives::wait::resume`]. Additionally returns
/// `Outcome::Err(ErrorCode::InvRegistration)` when `handle` is a
/// `WaitOutcome::Complete(_)`.
pub fn resume(
    _sg: &Subgraph,
    ctx: &mut EvalContext,
    handle: WaitOutcome,
    signal: WaitResumeSignal,
) -> Outcome {
    if matches!(handle, WaitOutcome::Complete(_)) {
        return Outcome::Err(ErrorCode::InvRegistration);
    }
    let state_cid = handle.state_cid();
    let meta = primitives::wait::metadata_for_cid(&state_cid);
    match primitives::wait::resume_with_meta(meta, signal, ctx.elapsed_ms()) {
        Ok(WaitOutcome::Complete(v)) => Outcome::Complete(v),
        Ok(wo @ WaitOutcome::Suspended(_)) => Outcome::Suspended(wo),
        Err(e) => Outcome::Err(e.code()),
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
    // HostError's Display already includes the "host error (...)" prefix, so
    // a redundant "host: " in this attribute would render as "host: host
    // error (...)". Delegate the whole Display to HostError (G1-B mini-review
    // nit N1).
    #[error("{0}")]
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
    /// Runtime + registration-time cumulative iteration-budget violation
    /// (invariant 8). Phase-2a folds what was Phase-1's nest-depth stopgap
    /// (`IterateNestDepth`, now stripped) into a single multiplicative-
    /// through-CALL check via `benten-eval::invariants::budget`. Maps to
    /// [`ErrorCode::InvIterateBudget`] / `E_INV_ITERATE_BUDGET`.
    IterateBudget,
    /// Aggregate catch-all for Invariant 12 — fires when two or more
    /// invariants are violated simultaneously. See
    /// `tests/invariants_9_10_12.rs::registration_catch_all_populates_violated_list`.
    Registration,
    /// Invariant 14 (G5-B-ii): a primitive-type in the subgraph did not
    /// declare whether it consumes an `AttributionFrame`. Fires at
    /// registration-time. Maps to `ErrorCode::InvAttribution`.
    Attribution,
    /// Invariant 13 (G5-A): a WRITE primitive declares a literal CID target
    /// that is already registered as an immutable subgraph/Node. Fires at
    /// registration-time (declaration-layer reject). Maps to
    /// [`ErrorCode::InvImmutability`]. Runtime firing lives in
    /// `benten-graph` per plan §9.11.
    Immutability,
    /// Invariant 11 (G5-B-i): a user subgraph declares a READ or WRITE
    /// whose target label falls within a `system:*` system-zone prefix.
    /// Fires at registration-time via the literal-CID walker in
    /// [`crate::invariants::system_zone::validate_registration`]; the
    /// runtime counterpart lives in `benten-engine/src/primitive_host.rs`
    /// and reuses the `ErrorCode::InvSystemZone` code. Maps to
    /// [`ErrorCode::InvSystemZone`].
    SystemZone,
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
            InvariantViolation::IterateBudget => ErrorCode::InvIterateBudget,
            InvariantViolation::Registration => ErrorCode::InvRegistration,
            InvariantViolation::Attribution => ErrorCode::InvAttribution,
            InvariantViolation::Immutability => ErrorCode::InvImmutability,
            InvariantViolation::SystemZone => ErrorCode::InvSystemZone,
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

/// `Display` impl for `RegistrationError` — required so consumers (notably
/// `EngineError::Invariant(#[from] Box<RegistrationError>)`) participate in
/// the standard `std::error::Error::source()` chain via thiserror's
/// `{0}`-format expansion. Phase-2a R6FP catch-up EH4. The rendering is
/// deliberately compact: catalog code as the leading discriminant followed
/// by the first available diagnostic context field. Operators wanting the
/// full diagnostic structure use the typed accessors (`depth_actual()`,
/// `cycle_path()`, etc.) — `Display` is the one-line summary.
impl core::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.kind.code().as_static_str())?;
        if let (Some(actual), Some(max)) = (self.nodes_actual, self.nodes_max) {
            write!(f, " (nodes: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.edges_actual, self.edges_max) {
            write!(f, " (edges: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.depth_actual, self.depth_max) {
            write!(f, " (depth: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.fanout_actual, self.fanout_max) {
            write!(f, " (fanout: {actual}/{max}")?;
            if let Some(ref id) = self.fanout_node_id {
                write!(f, " at node {id}")?;
            }
            write!(f, ")")?;
        } else if let (Some(expected), Some(actual)) = (self.expected_cid, self.actual_cid) {
            write!(f, " (expected CID {expected}, actual {actual})")?;
        } else if let Some(ref violated) = self.violated_invariants {
            // R6 round-2 C2-R2-5: render the invariant numbers as a
            // Display-style comma-separated list rather than the
            // `{:?}` Debug-formatted `Vec<u8>`. Keeps the rest of the
            // impl's compact one-line summary style consistent.
            write!(f, " (violated invariants: ")?;
            for (i, n) in violated.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{n}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// `Error` impl for `RegistrationError` — paired with `Display` above so the
/// type satisfies `std::error::Error` and can be threaded through `#[from]`
/// / `#[source]` in downstream `thiserror` enums. R6FP catch-up EH4.
impl std::error::Error for RegistrationError {}

/// Borrowed snapshot of a [`SubgraphBuilder`] used by the invariant checker.
/// Kept separate so `invariants` never needs a mutable handle on the
/// builder. After Phase-2b G12-C-cont this snapshot is built by
/// [`crate::subgraph_ext`] from the builder's validator-accessor surface
/// (the builder itself moved to `benten-core`).
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

/// Configurable invariant thresholds.
#[derive(Debug, Clone)]
pub struct InvariantConfig {
    pub max_depth: u32,
    pub max_fanout: u32,
    pub max_nodes: u32,
    pub max_edges: u32,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            max_depth: u32::try_from(limits::DEFAULT_MAX_DEPTH).unwrap_or(64),
            max_fanout: u32::try_from(limits::DEFAULT_MAX_FANOUT).unwrap_or(16),
            max_nodes: u32::try_from(limits::DEFAULT_MAX_NODES).unwrap_or(4096),
            max_edges: u32::try_from(limits::DEFAULT_MAX_EDGES).unwrap_or(8192),
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
