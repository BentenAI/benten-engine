//! # benten-engine
//!
//! Orchestrator crate composing the Benten graph engine public API.
//!
//! The spike shipped with a minimal `Engine::open` + `create_node` + `get_node`
//! surface. Phase 1 R3 tests drive a much larger API — registration,
//! capability-gated `call`, IVM view reads, version chains, and system-zone
//! privileged paths. This file is the R3 stub scaffold for that surface; every
//! method lands as `todo!()` so tests compile red. R5 fills in the bodies.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use benten_caps::{CapError, CapabilityPolicy};
pub use benten_core::ErrorCode;
use benten_core::{Cid, CoreError, Node};
use benten_eval::RegistrationError;
use benten_graph::{GraphError, RedbBackend};

// Touch the stub crates so the dependency graph is real, not just declared.
const _: &str = benten_caps::STUB_MARKER;
const _: &str = benten_eval::STUB_MARKER;
const _: &str = benten_ivm::STUB_MARKER;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by the engine orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("core: {0}")]
    Core(#[from] CoreError),

    #[error("graph: {0}")]
    Graph(#[from] GraphError),

    #[error("capability: {0}")]
    Cap(#[from] CapError),

    #[error("invariant: {0:?}")]
    Invariant(RegistrationError),

    /// Handler ID already registered with different content.
    #[error("duplicate handler: {handler_id}")]
    DuplicateHandler { handler_id: String },

    /// `Engine::builder().production()` called without an explicit
    /// capability policy. R1 SC2: fail-early guardrail.
    #[error(
        "no capability policy configured for .production() builder — call .capability_policy(...) or drop .production()"
    )]
    NoCapabilityPolicyConfigured,

    /// Read against a view whose incremental state is stale.
    #[error("IVM view stale: {view_id}")]
    IvmViewStale { view_id: String },

    /// Read against a view id that was never registered.
    #[error("unknown view: {view_id}")]
    UnknownView { view_id: String },

    /// Nested transaction attempted.
    #[error("nested transaction not supported")]
    NestedTransactionNotSupported,

    /// Generic wrapped error carrying a stable catalog code.
    #[error("{message}")]
    Other { code: ErrorCode, message: String },
}

impl EngineError {
    /// Stable catalog code as `&str` (Phase 1 test surface).
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            EngineError::Core(e) => e.code().as_str_owned_leaked(),
            EngineError::Graph(e) => e.code().as_str_owned_leaked(),
            EngineError::Cap(e) => e.code().as_str_owned_leaked(),
            EngineError::Invariant(e) => e.code().as_str_owned_leaked(),
            EngineError::DuplicateHandler { .. } => "E_DUPLICATE_HANDLER",
            EngineError::NoCapabilityPolicyConfigured => "E_NO_CAPABILITY_POLICY_CONFIGURED",
            EngineError::IvmViewStale { .. } => "E_IVM_VIEW_STALE",
            EngineError::UnknownView { .. } => "E_UNKNOWN_VIEW",
            EngineError::NestedTransactionNotSupported => "E_NESTED_TRANSACTION_NOT_SUPPORTED",
            EngineError::Other { code, .. } => code.as_str_owned_leaked(),
        }
    }
}

/// Small extension trait that widens [`ErrorCode`] into an owned `'static str`
/// suitable for the `EngineError::code()` surface. Leaks the string on the
/// `Unknown` path — acceptable at the R3 stub level since production code
/// uses `as_str()` directly.
trait ErrorCodeStaticStr {
    fn as_str_owned_leaked(&self) -> &'static str;
}

impl ErrorCodeStaticStr for ErrorCode {
    fn as_str_owned_leaked(&self) -> &'static str {
        match self {
            ErrorCode::Unknown(_s) => "E_UNKNOWN",
            other => {
                // Re-walk through the non-Unknown arms of `as_str` to get
                // a `'static` lifetime. This matches the full enum.
                match other {
                    ErrorCode::InvCycle => "E_INV_CYCLE",
                    ErrorCode::InvDepthExceeded => "E_INV_DEPTH_EXCEEDED",
                    ErrorCode::InvFanoutExceeded => "E_INV_FANOUT_EXCEEDED",
                    ErrorCode::InvTooManyNodes => "E_INV_TOO_MANY_NODES",
                    ErrorCode::InvTooManyEdges => "E_INV_TOO_MANY_EDGES",
                    ErrorCode::InvDeterminism => "E_INV_DETERMINISM",
                    ErrorCode::InvContentHash => "E_INV_CONTENT_HASH",
                    ErrorCode::InvRegistration => "E_INV_REGISTRATION",
                    ErrorCode::InvIterateNestDepth => "E_INV_ITERATE_NEST_DEPTH",
                    ErrorCode::InvIterateMaxMissing => "E_INV_ITERATE_MAX_MISSING",
                    ErrorCode::CapDenied => "E_CAP_DENIED",
                    ErrorCode::CapDeniedRead => "E_CAP_DENIED_READ",
                    ErrorCode::CapRevoked => "E_CAP_REVOKED",
                    ErrorCode::CapRevokedMidEval => "E_CAP_REVOKED_MID_EVAL",
                    ErrorCode::CapNotImplemented => "E_CAP_NOT_IMPLEMENTED",
                    ErrorCode::CapAttenuation => "E_CAP_ATTENUATION",
                    ErrorCode::WriteConflict => "E_WRITE_CONFLICT",
                    ErrorCode::IvmViewStale => "E_IVM_VIEW_STALE",
                    ErrorCode::TxAborted => "E_TX_ABORTED",
                    ErrorCode::NestedTransactionNotSupported => {
                        "E_NESTED_TRANSACTION_NOT_SUPPORTED"
                    }
                    ErrorCode::PrimitiveNotImplemented => "E_PRIMITIVE_NOT_IMPLEMENTED",
                    ErrorCode::SystemZoneWrite => "E_SYSTEM_ZONE_WRITE",
                    ErrorCode::ValueFloatNan => "E_VALUE_FLOAT_NAN",
                    ErrorCode::ValueFloatNonFinite => "E_VALUE_FLOAT_NONFINITE",
                    ErrorCode::CidParse => "E_CID_PARSE",
                    ErrorCode::CidUnsupportedCodec => "E_CID_UNSUPPORTED_CODEC",
                    ErrorCode::CidUnsupportedHash => "E_CID_UNSUPPORTED_HASH",
                    ErrorCode::VersionBranched => "E_VERSION_BRANCHED",
                    ErrorCode::BackendNotFound => "E_BACKEND_NOT_FOUND",
                    ErrorCode::TransformSyntax => "E_TRANSFORM_SYNTAX",
                    ErrorCode::InputLimit => "E_INPUT_LIMIT",
                    ErrorCode::NotFound => "E_NOT_FOUND",
                    ErrorCode::Unknown(_) => "E_UNKNOWN",
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The Benten engine handle.
pub struct Engine {
    backend: RedbBackend,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}

impl Engine {
    /// Open or create an engine backed by a redb database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let backend = RedbBackend::open(path)?;
        Ok(Self { backend })
    }

    /// Begin a new builder.
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Hash `node` (CIDv1 over labels + properties only), store it, and return
    /// its CID. Idempotent.
    pub fn create_node(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node(node)?)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, EngineError> {
        Ok(self.backend.get_node(cid)?)
    }

    // -------- Phase 1 stubs below — R5 fills bodies. --------

    /// Register the zero-config `crud('<label>')` handler set. Returns the
    /// handler id (a stable string derived from the subgraph's CID).
    pub fn register_crud(&self, _label: &str) -> Result<String, EngineError> {
        todo!("Engine::register_crud — N4 (Phase 1)")
    }

    /// Register crud with grant-backed capability policy as default.
    pub fn register_crud_with_grants(&self, _label: &str) -> Result<String, EngineError> {
        todo!("Engine::register_crud_with_grants — N4 (Phase 1)")
    }

    /// Register an arbitrary subgraph. Accepts anything coercible into a
    /// [`SubgraphSpec`] (including `&SubgraphSpec` references via blanket impls).
    pub fn register_subgraph<S>(&self, _spec: S) -> Result<String, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        todo!("Engine::register_subgraph — N4 (Phase 1)")
    }

    /// Register a subgraph, returning the aggregate-mode error on multi-violation.
    pub fn register_subgraph_aggregate<S>(&self, _spec: S) -> Result<String, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        todo!("Engine::register_subgraph_aggregate — N4 (Phase 1)")
    }

    /// Create an actor principal (returns a CID that identifies them).
    pub fn create_principal(&self, _name: &str) -> Result<Cid, EngineError> {
        todo!("Engine::create_principal — N7 (Phase 1)")
    }

    /// Grant a capability. Writes a `system:CapabilityGrant` Node via the
    /// engine-privileged path.
    pub fn grant_capability<A, S>(&self, _actor: A, _scope: S) -> Result<Cid, EngineError>
    where
        A: GrantSubject,
        S: AsRef<str>,
    {
        todo!("Engine::grant_capability — N7 (Phase 1)")
    }

    /// Revoke a capability for `(actor, scope)`.
    pub fn revoke_capability<A, S>(&self, _actor: A, _scope: S) -> Result<(), EngineError>
    where
        A: RevokeSubject,
        S: RevokeScope,
    {
        todo!("Engine::revoke_capability — N7 (Phase 1)")
    }

    /// Create a `system:IVMView` Node via the engine-privileged path.
    pub fn create_view(
        &self,
        _view_id: &str,
        _opts: ViewCreateOptions,
    ) -> Result<Cid, EngineError> {
        todo!("Engine::create_view — N7 (Phase 1)")
    }

    /// Call a registered handler with the given operation and input.
    /// `handler_id: &str` keeps the caller's owned `String` un-moved across
    /// repeated invocations (Rust auto-derefs `&String` to `&str`, and
    /// `String` deref-coerces in the temporary-binding form).
    pub fn call<I>(&self, _handler_id: &str, _op: &str, _input: I) -> Result<Outcome, EngineError>
    where
        I: IntoCallInput,
    {
        todo!("Engine::call — N6 (Phase 1)")
    }

    /// Call a registered handler as a specific principal.
    pub fn call_as(
        &self,
        _handler_id: &str,
        _op: &str,
        _input: Node,
        _actor: &Cid,
    ) -> Result<Outcome, EngineError> {
        todo!("Engine::call_as — N6 (Phase 1)")
    }

    /// Call a handler with an injected revocation hook at iteration `n`.
    pub fn call_with_revocation_at(
        &self,
        _handler_id: &str,
        _op: &str,
        _input: Node,
        _actor: &Cid,
        _scope: &str,
        _n: u32,
    ) -> Result<Outcome, EngineError> {
        todo!("Engine::call_with_revocation_at — N6 (Phase 1)")
    }

    /// Trace evaluation of a handler. Returns per-step timings + errors.
    pub fn trace(&self, _handler_id: &str, _op: &str, _input: Node) -> Result<Trace, EngineError> {
        todo!("Engine::trace — N6 (Phase 1)")
    }

    /// Mermaid flowchart serialization of a registered handler.
    pub fn handler_to_mermaid(&self, _handler_id: &str) -> Result<String, EngineError> {
        todo!("Engine::handler_to_mermaid — N6 (Phase 1)")
    }

    /// Adjacency accessor for traces (topological-order assertions).
    pub fn handler_predecessors(
        &self,
        _handler_id: &str,
    ) -> Result<HandlerPredecessors, EngineError> {
        todo!("Engine::handler_predecessors — N6 (Phase 1)")
    }

    /// Register a test-only probe receiving every ChangeEvent.
    pub fn test_subscribe_all_change_events(&self) -> ChangeProbe {
        todo!("Engine::test_subscribe_all_change_events — N6 (Phase 1)")
    }

    /// Register a test-only probe filtered to a given label.
    pub fn test_subscribe_change_events_matching_label(&self, _label: &str) -> ChangeProbe {
        todo!("Engine::test_subscribe_change_events_matching_label — N6 (Phase 1)")
    }

    /// Metric snapshot for compromise-5 regression tests.
    pub fn metrics_snapshot(&self) -> BTreeMap<String, f64> {
        todo!("Engine::metrics_snapshot — N6 (Phase 1)")
    }

    /// Count nodes stored under a label.
    pub fn count_nodes_with_label(&self, _label: &str) -> Result<usize, EngineError> {
        todo!("Engine::count_nodes_with_label — N6 (Phase 1)")
    }

    /// Count of ChangeEvents emitted.
    pub fn change_event_count(&self) -> u64 {
        todo!("Engine::change_event_count — N6 (Phase 1)")
    }

    /// Read an IVM view (strict mode — error on stale).
    pub fn read_view(&self, _view_id: &str) -> Result<Outcome, EngineError> {
        todo!("Engine::read_view — N3 (Phase 1)")
    }

    /// Read an IVM view with explicit options (strict / allow-stale).
    pub fn read_view_with(
        &self,
        _view_id: &str,
        _opts: ReadViewOptions,
    ) -> Result<Outcome, EngineError> {
        todo!("Engine::read_view_with — N3 (Phase 1)")
    }

    /// Strict read — errors on stale.
    pub fn read_view_strict(&self, _view_id: &str) -> Result<Outcome, EngineError> {
        todo!("Engine::read_view_strict — N3 (Phase 1)")
    }

    /// Relaxed read — returns last-known-good on stale.
    pub fn read_view_allow_stale(&self, _view_id: &str) -> Result<Outcome, EngineError> {
        todo!("Engine::read_view_allow_stale — N3 (Phase 1)")
    }

    /// Subscriber count (thinness test).
    pub fn ivm_subscriber_count(&self) -> usize {
        todo!("Engine::ivm_subscriber_count — N6 (Phase 1)")
    }

    /// Anchor creation for version chains.
    pub fn create_anchor(&self, _name: &str) -> Result<AnchorHandle, EngineError> {
        todo!("Engine::create_anchor — N7 (Phase 1)")
    }

    /// Append a version to an Anchor chain.
    pub fn append_version(&self, _anchor: &AnchorHandle, _node: &Node) -> Result<Cid, EngineError> {
        todo!("Engine::append_version — N7 (Phase 1)")
    }

    /// Resolve the current-version CID for an anchor.
    pub fn read_current_version(&self, _anchor: &AnchorHandle) -> Result<Option<Cid>, EngineError> {
        todo!("Engine::read_current_version — N7 (Phase 1)")
    }

    /// Walk the full version chain oldest→newest.
    pub fn walk_versions(
        &self,
        _anchor: &AnchorHandle,
    ) -> Result<std::vec::IntoIter<Cid>, EngineError> {
        todo!("Engine::walk_versions — N7 (Phase 1)")
    }

    /// Run a closure inside a write transaction.
    pub fn transaction<F, R>(&self, _f: F) -> Result<R, EngineError>
    where
        F: FnOnce(&mut EngineTransaction<'_>) -> Result<R, EngineError>,
    {
        todo!("Engine::transaction — N6 (Phase 1)")
    }

    /// Schedule a capability revocation at iteration `n` (test hook).
    pub fn schedule_revocation_at_iteration(
        &self,
        _grant: Cid,
        _n: u32,
    ) -> Result<(), EngineError> {
        todo!("Engine::schedule_revocation_at_iteration — N7 (Phase 1)")
    }

    /// Test-only privileged-path Node insertion. Used by `read_denial.rs`
    /// to seed a Node the attacker can READ-with-cap-denied against.
    pub fn testing_insert_privileged_fixture(&self) -> Cid {
        todo!("Engine::testing_insert_privileged_fixture — N6 (Phase 1)")
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Engine builder. See tests under `benten-engine/tests/` for the full
/// surface it exposes.
pub struct EngineBuilder {
    path: Option<PathBuf>,
    policy: Option<Box<dyn CapabilityPolicy>>,
    production: bool,
    #[allow(dead_code)]
    without_ivm: bool,
    #[allow(dead_code)]
    without_caps: bool,
    #[allow(dead_code)]
    without_versioning: bool,
    #[allow(dead_code)]
    test_ivm_budget: Option<u64>,
    #[allow(dead_code)]
    backend: Option<RedbBackend>,
}

impl EngineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            path: None,
            policy: None,
            production: false,
            without_ivm: false,
            without_caps: false,
            without_versioning: false,
            test_ivm_budget: None,
            backend: None,
        }
    }

    #[must_use]
    pub fn path(mut self, p: impl AsRef<Path>) -> Self {
        self.path = Some(p.as_ref().to_path_buf());
        self
    }

    #[must_use]
    pub fn capability_policy(mut self, p: Box<dyn CapabilityPolicy>) -> Self {
        self.policy = Some(p);
        self
    }

    /// Placeholder: the grant-backed capability policy. R5 wires a real impl.
    #[must_use]
    pub fn capability_policy_grant_backed(self) -> Self {
        self
    }

    /// Placeholder: a policy with built-in revocation hooks. R5 wires it.
    #[must_use]
    pub fn with_policy_allowing_revocation(self) -> Self {
        self
    }

    #[must_use]
    pub fn production(mut self) -> Self {
        self.production = true;
        self
    }

    #[must_use]
    pub fn without_ivm(mut self) -> Self {
        self.without_ivm = true;
        self
    }

    #[must_use]
    pub fn without_caps(mut self) -> Self {
        self.without_caps = true;
        self
    }

    #[must_use]
    pub fn without_versioning(mut self) -> Self {
        self.without_versioning = true;
        self
    }

    #[must_use]
    pub fn with_test_ivm_budget(mut self, b: u64) -> Self {
        self.test_ivm_budget = Some(b);
        self
    }

    /// Provide a pre-opened backend (used by test fixtures that open redb
    /// independently and then hand the handle to the engine).
    #[must_use]
    pub fn backend(mut self, b: RedbBackend) -> Self {
        self.backend = Some(b);
        self
    }

    /// Configure an IVM max-work-per-update budget.
    #[must_use]
    pub fn ivm_max_work_per_update(self, _n: u64) -> Self {
        self
    }

    /// Build the engine by path. Used when `.path()` was supplied.
    pub fn build(self) -> Result<Engine, EngineError> {
        if self.production && self.policy.is_none() {
            return Err(EngineError::NoCapabilityPolicyConfigured);
        }
        if let Some(backend) = self.backend {
            return Ok(Engine { backend });
        }
        let path = self.path.ok_or(EngineError::NoCapabilityPolicyConfigured)?;
        Engine::open(path)
    }

    /// Builder-style open: `Engine::builder().open(path)`.
    pub fn open(self, path: impl AsRef<Path>) -> Result<Engine, EngineError> {
        if self.production && self.policy.is_none() {
            return Err(EngineError::NoCapabilityPolicyConfigured);
        }
        Engine::open(path)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Options passed to `Engine::create_view`. Currently a placeholder shape so
/// `Default::default()` resolves unambiguously at the call site.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewCreateOptions;

/// Options passed to `Engine::read_view_with`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadViewOptions {
    pub allow_stale: bool,
}

impl ReadViewOptions {
    #[must_use]
    pub fn strict() -> Self {
        Self { allow_stale: false }
    }

    #[must_use]
    pub fn allow_stale() -> Self {
        Self { allow_stale: true }
    }
}

/// Extension trait used by `tx_atomicity` integration tests.
pub trait OutcomeExt {
    fn as_outcome(&self) -> &Outcome;
}

/// The response returned by `Engine::call`. Exposes routing edge + error code
/// + CID of any created Node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    _placeholder: (),
}

impl Outcome {
    pub fn routed_through_edge(&self, _edge: &str) -> bool {
        todo!("Outcome::routed_through_edge — N6 (Phase 1)")
    }

    pub fn edge_taken(&self) -> Option<String> {
        todo!("Outcome::edge_taken — N6 (Phase 1)")
    }

    pub fn error_code(&self) -> Option<&str> {
        todo!("Outcome::error_code — N6 (Phase 1)")
    }

    pub fn error_message(&self) -> Option<String> {
        todo!("Outcome::error_message — N6 (Phase 1)")
    }

    pub fn is_ok_edge(&self) -> bool {
        todo!("Outcome::is_ok_edge — N6 (Phase 1)")
    }

    pub fn as_list(&self) -> Option<Vec<Node>> {
        todo!("Outcome::as_list — N6 (Phase 1)")
    }

    pub fn created_cid(&self) -> Option<Cid> {
        todo!("Outcome::created_cid — N6 (Phase 1)")
    }

    pub fn completed_iterations(&self) -> Option<u32> {
        todo!("Outcome::completed_iterations — N6 (Phase 1)")
    }

    pub fn successful_write_count(&self) -> u32 {
        todo!("Outcome::successful_write_count — N6 (Phase 1)")
    }

    pub fn terminal_error(&self) -> Option<TerminalError> {
        todo!("Outcome::terminal_error — N6 (Phase 1)")
    }

    /// Panics unless the outcome routed through the success edge. Test-only
    /// shortcut used by `requires_property_call_time_check.rs`.
    pub fn assert_success(&self) {
        todo!("Outcome::assert_success — N6 (Phase 1)")
    }

    /// Test-only accessor: the typed edge label the evaluator routed through.
    /// Aliased name some R3 writers used; identical to `edge_taken()`.
    pub fn taken_edge(&self) -> &str {
        todo!("Outcome::taken_edge — N6 (Phase 1)")
    }
}

/// Minimal terminal-error surface returned from `Outcome::terminal_error`.
#[derive(Debug, Clone)]
pub struct TerminalError {
    _placeholder: (),
}

impl TerminalError {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        todo!("TerminalError::code — N6 (Phase 1)")
    }
}

/// Trace of an evaluation.
#[derive(Debug, Clone)]
pub struct Trace {
    _placeholder: (),
}

impl Trace {
    pub fn steps(&self) -> Vec<TraceStep> {
        todo!("Trace::steps — N6 (Phase 1)")
    }
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    _placeholder: (),
}

impl TraceStep {
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        todo!("TraceStep::duration_us — N6 (Phase 1)")
    }

    #[must_use]
    pub fn node_cid(&self) -> &Cid {
        todo!("TraceStep::node_cid — N6 (Phase 1)")
    }
}

/// Handle to an Anchor (version-chain identity).
#[derive(Debug, Clone)]
pub struct AnchorHandle {
    _placeholder: (),
}

/// Probe for intercepting ChangeEvents in tests.
#[derive(Debug)]
pub struct ChangeProbe {
    _placeholder: (),
}

impl ChangeProbe {
    pub fn drain(&self) -> Vec<benten_graph::ChangeEvent> {
        todo!("ChangeProbe::drain — N6 (Phase 1)")
    }
}

/// Predecessor adjacency for trace assertions.
#[derive(Debug)]
pub struct HandlerPredecessors {
    _placeholder: (),
}

impl HandlerPredecessors {
    pub fn predecessors_of(&self, _node_cid: &Cid) -> &[Cid] {
        todo!("HandlerPredecessors::predecessors_of — N6 (Phase 1)")
    }
}

/// Engine-level transaction handle (passed into `Engine::transaction`).
#[derive(Debug)]
pub struct EngineTransaction<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> EngineTransaction<'a> {
    pub fn create_node(&mut self, _node: &Node) -> Result<Cid, EngineError> {
        todo!("EngineTransaction::create_node — N6 (Phase 1)")
    }

    pub fn put_node(&mut self, _node: &Node) -> Result<Cid, EngineError> {
        todo!("EngineTransaction::put_node — N6 (Phase 1)")
    }

    pub fn begin_nested(&mut self) -> Result<NestedTx, EngineError> {
        Err(EngineError::NestedTransactionNotSupported)
    }
}

#[derive(Debug)]
pub struct NestedTx {
    _placeholder: (),
}

// ---------------------------------------------------------------------------
// SubgraphSpec — the DSL-friendly builder for `engine.register_subgraph`
// ---------------------------------------------------------------------------

/// DSL-friendly specification passed to `Engine::register_subgraph`.
#[derive(Debug, Clone)]
pub struct SubgraphSpec {
    _placeholder: (),
}

impl SubgraphSpec {
    #[must_use]
    pub fn builder() -> SubgraphSpecBuilder {
        SubgraphSpecBuilder::new()
    }
}

pub struct SubgraphSpecBuilder {
    _placeholder: (),
}

impl SubgraphSpecBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    #[must_use]
    pub fn handler_id(self, _id: &str) -> Self {
        self
    }

    #[must_use]
    pub fn iterate<F>(self, _max: u32, _body: F) -> Self
    where
        F: FnOnce(IterateBody) -> IterateBody,
    {
        self
    }

    #[must_use]
    pub fn write<F>(self, _f: F) -> Self
    where
        F: FnOnce(WriteSpec) -> WriteSpec,
    {
        self
    }

    #[must_use]
    pub fn respond(self) -> Self {
        self
    }

    #[must_use]
    pub fn build(self) -> SubgraphSpec {
        SubgraphSpec { _placeholder: () }
    }
}

impl Default for SubgraphSpecBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// DSL body object handed to `iterate(|body| ...)`.
pub struct IterateBody;

impl IterateBody {
    #[must_use]
    pub fn write<F>(self, _f: F) -> Self
    where
        F: FnOnce(WriteSpec) -> WriteSpec,
    {
        self
    }
}

/// DSL object handed to `write(|w| ...)`.
pub struct WriteSpec;

impl WriteSpec {
    #[must_use]
    pub fn label(self, _label: &str) -> Self {
        self
    }

    #[must_use]
    pub fn property(self, _k: &str, _v: benten_core::Value) -> Self {
        self
    }

    #[must_use]
    pub fn requires(self, _scope: &str) -> Self {
        self
    }

    #[must_use]
    pub fn test_inject_failure(self, _inject: bool) -> Self {
        self
    }
}

// ---------------------------------------------------------------------------
// Helper trait adapters for overloaded `register_subgraph` / `grant_capability`
// / `revoke_capability` / `call` arguments.
// ---------------------------------------------------------------------------

/// Accepts `SubgraphSpec`, `&SubgraphSpec`, and `benten_eval::Subgraph` (the
/// lower-level type that `SubgraphBuilder::build_*` returns).
pub trait IntoSubgraphSpec {}
impl IntoSubgraphSpec for SubgraphSpec {}
impl IntoSubgraphSpec for &SubgraphSpec {}
impl IntoSubgraphSpec for benten_eval::Subgraph {}
impl IntoSubgraphSpec for &benten_eval::Subgraph {}

/// Subject arg for `grant_capability` — accepts `&Cid`, `&str`, and other
/// tests reference a `&Actor` style. Keep liberal.
pub trait GrantSubject {}
impl GrantSubject for &Cid {}
impl GrantSubject for &str {}
impl GrantSubject for &String {}
impl GrantSubject for Cid {}

pub trait RevokeSubject {}
impl RevokeSubject for &Cid {}
impl RevokeSubject for Cid {}

pub trait RevokeScope {}
impl RevokeScope for &str {}
impl RevokeScope for &String {}
impl RevokeScope for String {}

/// Call-input overload — accept `Node`, default `()`, and the
/// `BTreeMap<String, benten_core::Value>` path some R3 tests build inline.
pub trait IntoCallInput {}
impl IntoCallInput for Node {}
impl IntoCallInput for () {}
impl IntoCallInput for BTreeMap<String, benten_core::Value> {}

// ---------------------------------------------------------------------------
// Testing module — helpers referenced by integration tests in sibling crates.
// Every function returns `todo!()`; tests compile but panic at runtime, which
// is the TDD red phase we want.
// ---------------------------------------------------------------------------

#[allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]
pub mod testing {
    //! Test helpers used by integration tests from sibling crates
    //! (`benten-caps/tests/*.rs`, `benten-eval/tests/*.rs`).

    use super::{CapabilityPolicy, Outcome, SubgraphSpec};

    /// Build a synthetic ITERATE-heavy handler for TOCTOU tests.
    #[must_use]
    pub fn iterate_write_handler(_max: u32) -> SubgraphSpec {
        todo!("testing::iterate_write_handler — N6 (Phase 1)")
    }

    /// Build a minimal single-WRITE handler.
    #[must_use]
    pub fn minimal_write_handler() -> SubgraphSpec {
        todo!("testing::minimal_write_handler — N6 (Phase 1)")
    }

    /// Inspect the edge taken by the terminal step of an Outcome.
    #[must_use]
    pub fn route_of_error(_outcome: &Outcome) -> String {
        todo!("testing::route_of_error — N6 (Phase 1)")
    }

    /// Build a READ-only handler for existence-leak tests. Accepts a label,
    /// a `&str`, or a `Cid` via the [`ReadHandlerTarget`] sugar trait.
    #[must_use]
    pub fn read_handler_for<T: ReadHandlerTarget>(_target: T) -> SubgraphSpec {
        todo!("testing::read_handler_for — N6 (Phase 1)")
    }

    /// Sugar trait — see [`read_handler_for`].
    pub trait ReadHandlerTarget {}
    impl ReadHandlerTarget for &str {}
    impl ReadHandlerTarget for &String {}
    impl ReadHandlerTarget for String {}
    impl ReadHandlerTarget for benten_core::Cid {}

    /// Synthesize a Subject with no read grants. Returns a boxed
    /// `CapabilityPolicy` ready to plug into the builder.
    #[must_use]
    pub fn subject_with_no_read_grants() -> Box<dyn CapabilityPolicy> {
        todo!("testing::subject_with_no_read_grants — N6 (Phase 1)")
    }

    /// Adversarial fixture: handler declares `requires: post:read` but writes to admin.
    #[must_use]
    pub fn handler_declaring_read_but_writing_admin() -> SubgraphSpec {
        todo!("testing::handler_declaring_read_but_writing_admin — N6 (Phase 1)")
    }

    /// Second-order escalation fixture.
    #[must_use]
    pub fn handler_with_call_attenuation_escalation() -> SubgraphSpec {
        todo!("testing::handler_with_call_attenuation_escalation — N6 (Phase 1)")
    }

    /// Build a capability policy pre-seeded with a grant set.
    #[must_use]
    pub fn policy_with_grants(_grants: &[&str]) -> Box<dyn CapabilityPolicy> {
        todo!("testing::policy_with_grants — N6 (Phase 1)")
    }

    /// Build a policy that counts check_write invocations. Returns a wrapper
    /// implementing both `CapabilityPolicy` and exposing a `.call_counter()`
    /// accessor.
    #[must_use]
    pub fn counting_capability_policy() -> CountingPolicy {
        todo!("testing::counting_capability_policy — N6 (Phase 1)")
    }

    /// Counting capability-policy wrapper used by R3 per-primitive cap-check
    /// regression tests.
    pub struct CountingPolicy {
        _placeholder: (),
    }

    impl CountingPolicy {
        /// Atomic counter exposing the number of `check_write` invocations.
        #[must_use]
        pub fn call_counter(&self) -> CallCounter {
            todo!("CountingPolicy::call_counter — N6 (Phase 1)")
        }
    }

    impl benten_caps::CapabilityPolicy for CountingPolicy {
        fn check_write(
            &self,
            _ctx: &benten_caps::WriteContext,
        ) -> Result<(), benten_caps::CapError> {
            todo!("CountingPolicy::check_write — N6 (Phase 1)")
        }
    }

    /// Atomic counter handle. **Phase 1 stub** — `load` returns 0 today.
    pub struct CallCounter {
        _placeholder: (),
    }

    impl CallCounter {
        #[must_use]
        pub fn load(&self) -> u32 {
            todo!("CallCounter::load — N6 (Phase 1)")
        }
    }

    /// Build a READ→WRITE→READ handler for per-primitive cap-check assertions.
    #[must_use]
    pub fn handler_with_read_write_read_sequence() -> SubgraphSpec {
        todo!("testing::handler_with_read_write_read_sequence — N6 (Phase 1)")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let node = canonical_test_node();
        let cid = engine.create_node(&node).unwrap();
        let fetched = engine.get_node(&cid).unwrap().expect("node exists");
        assert_eq!(fetched, node);
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn missing_cid_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let cid = canonical_test_node().cid().unwrap();
        assert!(engine.get_node(&cid).unwrap().is_none());
    }
}
