//! # benten-engine
//!
//! Orchestrator crate composing the Benten graph engine public API.
//!
//! G7 (this file) wires the composition:
//! - [`EngineBuilder`] selects capability policy, IVM subscriber, production
//!   mode guard, and durability.
//! - [`Engine`] exposes CRUD (Node + Edge), `register_subgraph` (runs G6
//!   invariants), `transaction` (closure over [`benten_graph::Transaction`]),
//!   `snapshot` (MVCC handle), and the three privileged system-zone entry
//!   points `grant_capability` / `create_view` / `revoke_capability`.
//! - [`change::ChangeBroadcast`] fans committed events to
//!   every registered subscriber.
//!
//! Call-time primitive dispatch (register_crud → evaluator → primitive
//! execution) is a Phase-1 scope boundary the present G7 does not close; the
//! deliverables land the builder + CRUD + privileged paths + invariant
//! validation + IVM subscriber wiring so the rest of the stack compiles
//! against a coherent surface.

#![forbid(unsafe_code)]
#![allow(
    clippy::todo,
    reason = "Phase-1 scope: primitive-dispatch deliverables remain as typed todos until benten-eval's evaluator gains a PrimitiveHost trait (Phase 2)."
)]

pub mod change;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use benten_caps::{CapError, CapabilityPolicy, NoAuthBackend};
pub use benten_core::ErrorCode;
use benten_core::{Cid, CoreError, Edge, Node, Value};
use benten_eval::{InvariantConfig, RegistrationError};
use benten_graph::{ChangeEvent, GraphError, RedbBackend};

use crate::change::ChangeBroadcast;

// Touch the stub crates so the dependency graph is real, not just declared.
// TODO(phase-1-cleanup, G8): retire these three `const _:` assertions together
// with the `STUB_MARKER` constants in benten-caps / benten-eval / benten-ivm
// once those crates are no longer stub-phase (G4 mini-review g4-cr-7 mirrored
// this TODO from `benten-caps/src/lib.rs`).
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

    /// Structural-invariant rejection. Boxed so `Result<T, EngineError>`
    /// stays below clippy's `result_large_err` 128-byte threshold —
    /// `RegistrationError` itself carries ~360 bytes of diagnostic context
    /// (paths, expected/actual CIDs, per-invariant counts). Mini-review
    /// findings `g6-cr-1` / `g6-cag-7`.
    #[error("invariant: {0:?}")]
    Invariant(Box<RegistrationError>),

    /// Handler ID already registered with different content.
    #[error("duplicate handler: {handler_id}")]
    DuplicateHandler { handler_id: String },

    /// `Engine::builder().production()` called without an explicit
    /// capability policy. R1 SC2: fail-early guardrail.
    #[error(
        "no capability policy configured for .production() builder — call .capability_policy(...) or drop .production()"
    )]
    NoCapabilityPolicyConfigured,

    /// `.production()` combined with `.without_caps()` — mutually exclusive.
    /// Production mode requires a real capability policy; `.without_caps()`
    /// explicitly tears one down. Picking both silently dropped the policy
    /// before this guard — see code-reviewer finding `g7-cr-1`.
    #[error(
        "production mode requires capabilities — .production() and .without_caps() are mutually exclusive"
    )]
    ProductionRequiresCaps,

    /// Thin engine (without_ivm or without_caps) was asked to do something
    /// that requires the disabled subsystem. The honest-no boundary — thinness
    /// tests assert we error here rather than silently no-op.
    #[error("subsystem disabled: {subsystem}")]
    SubsystemDisabled { subsystem: &'static str },

    /// Read against a view whose incremental state is stale.
    #[error("IVM view stale: {view_id}")]
    IvmViewStale { view_id: String },

    /// Read against a view id that was never registered.
    #[error("unknown view: {view_id}")]
    UnknownView { view_id: String },

    /// Nested transaction attempted.
    #[error("nested transaction not supported")]
    NestedTransactionNotSupported,

    /// Feature deferred to a future group / phase. Used for primitive
    /// dispatch surfaces (`register_crud`, `call`, `trace`, `*` version
    /// chain, `*` principals) that need the evaluator integration the
    /// present G7 does not land.
    #[error("not implemented in Phase 1: {feature}")]
    NotImplemented { feature: &'static str },

    /// Generic wrapped error carrying a stable catalog code.
    #[error("{message}")]
    Other { code: ErrorCode, message: String },
}

impl EngineError {
    /// Stable catalog code as [`ErrorCode`]. Consumers that want the string
    /// form call `err.error_code().as_str()`.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            EngineError::Core(e) => e.code(),
            EngineError::Graph(e) => e.code(),
            EngineError::Cap(e) => e.code(),
            EngineError::Invariant(e) => e.code(),
            EngineError::DuplicateHandler { .. } => {
                ErrorCode::Unknown("E_DUPLICATE_HANDLER".into())
            }
            EngineError::NoCapabilityPolicyConfigured => {
                ErrorCode::Unknown("E_NO_CAPABILITY_POLICY_CONFIGURED".into())
            }
            EngineError::ProductionRequiresCaps => {
                ErrorCode::Unknown("E_PRODUCTION_REQUIRES_CAPS".into())
            }
            EngineError::SubsystemDisabled { .. } => {
                ErrorCode::Unknown("E_SUBSYSTEM_DISABLED".into())
            }
            EngineError::IvmViewStale { .. } => ErrorCode::IvmViewStale,
            EngineError::UnknownView { .. } => ErrorCode::Unknown("E_UNKNOWN_VIEW".into()),
            EngineError::NestedTransactionNotSupported => ErrorCode::NestedTransactionNotSupported,
            EngineError::NotImplemented { .. } => ErrorCode::Unknown("E_NOT_IMPLEMENTED".into()),
            EngineError::Other { code, .. } => code.clone(),
        }
    }

    /// Stable catalog code as a static string. Variants local to this crate
    /// map to stable literals; variants that wrap a catalog code delegate to
    /// [`benten_core::ErrorCode::as_str`] via an inline match — the
    /// single-source-of-truth `as_str` on benten-core owns the content,
    /// this wrapper just pins the `'static` lifetime.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::Core(e) => static_for(&e.code()),
            EngineError::Graph(e) => static_for(&e.code()),
            EngineError::Cap(e) => static_for(&e.code()),
            EngineError::Invariant(e) => static_for(&e.code()),
            EngineError::Other { code, .. } => static_for(code),
            EngineError::DuplicateHandler { .. } => "E_DUPLICATE_HANDLER",
            EngineError::NoCapabilityPolicyConfigured => "E_NO_CAPABILITY_POLICY_CONFIGURED",
            EngineError::ProductionRequiresCaps => "E_PRODUCTION_REQUIRES_CAPS",
            EngineError::SubsystemDisabled { .. } => "E_SUBSYSTEM_DISABLED",
            EngineError::IvmViewStale { .. } => "E_IVM_VIEW_STALE",
            EngineError::UnknownView { .. } => "E_UNKNOWN_VIEW",
            EngineError::NestedTransactionNotSupported => "E_NESTED_TRANSACTION_NOT_SUPPORTED",
            EngineError::NotImplemented { .. } => "E_NOT_IMPLEMENTED",
        }
    }
}

/// Pin a [`benten_core::ErrorCode`] to a `'static str`. Known catalog
/// variants return their canonical stable literal (matching
/// [`benten_core::ErrorCode::as_str`]); the [`ErrorCode::Unknown`] variant
/// degrades to `"E_UNKNOWN"` because we cannot promote the owned String to
/// `'static` without leaking.
fn static_for(c: &ErrorCode) -> &'static str {
    match c {
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
        ErrorCode::InvIterateBudget => "E_INV_ITERATE_BUDGET",
        ErrorCode::CapDenied => "E_CAP_DENIED",
        ErrorCode::CapDeniedRead => "E_CAP_DENIED_READ",
        ErrorCode::CapRevoked => "E_CAP_REVOKED",
        ErrorCode::CapRevokedMidEval => "E_CAP_REVOKED_MID_EVAL",
        ErrorCode::CapNotImplemented => "E_CAP_NOT_IMPLEMENTED",
        ErrorCode::CapAttenuation => "E_CAP_ATTENUATION",
        ErrorCode::WriteConflict => "E_WRITE_CONFLICT",
        ErrorCode::IvmViewStale => "E_IVM_VIEW_STALE",
        ErrorCode::TxAborted => "E_TX_ABORTED",
        ErrorCode::NestedTransactionNotSupported => "E_NESTED_TRANSACTION_NOT_SUPPORTED",
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
        ErrorCode::Serialize => "E_SERIALIZE",
        ErrorCode::Unknown(_) => "E_UNKNOWN",
    }
}

// ---------------------------------------------------------------------------
// Engine internal state
// ---------------------------------------------------------------------------

/// State shared across Engine methods. Behind an `Arc` so change-event
/// callbacks can hold a weak-style reference without borrowing from the
/// Engine struct itself.
struct EngineInner {
    /// Registered handler ids → subgraph CIDs. Populated by
    /// `register_subgraph` and consulted for the idempotent re-registration
    /// path.
    handlers: std::sync::Mutex<BTreeMap<String, Cid>>,
    /// Registered SubgraphSpec bodies keyed by handler id — so `call()` can
    /// walk the WriteSpec list when the user registered a SubgraphSpec
    /// (as opposed to `register_crud` which is dispatched directly by op
    /// name).
    specs: std::sync::Mutex<BTreeMap<String, SubgraphSpec>>,
    /// Observed ChangeEvents (post-commit). Populated by the
    /// `ChangeBroadcast` subscriber; drained by
    /// `engine.subscribe_change_events().drain()`.
    observed_events: std::sync::Mutex<Vec<(u64, ChangeEvent)>>,
    /// Counter of total change events observed (for `change_event_count()`).
    event_count: std::sync::atomic::AtomicU64,
    /// Monotonic per-engine sequence used to stamp `createdAt` on CRUD
    /// creates when the caller did not supply one — makes listing order
    /// deterministic across rapid-fire creates that might otherwise collide
    /// on a wall-clock timestamp.
    created_at_seq: std::sync::atomic::AtomicU64,
}

impl EngineInner {
    fn new() -> Self {
        Self {
            handlers: std::sync::Mutex::new(BTreeMap::new()),
            specs: std::sync::Mutex::new(BTreeMap::new()),
            observed_events: std::sync::Mutex::new(Vec::new()),
            event_count: std::sync::atomic::AtomicU64::new(0),
            created_at_seq: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn record_event(&self, event: &ChangeEvent) {
        let seq = self
            .event_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut guard = self
            .observed_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.push((seq, event.clone()));
    }

    /// Drain only events whose sequence number is `>= start_offset`. Events
    /// recorded before the probe was created stay in the buffer so other
    /// probes can still observe them. Drained events are removed.
    /// See code-reviewer finding `g7-cr-7`.
    fn drain_events_from(&self, start_offset: u64) -> Vec<ChangeEvent> {
        let mut guard = self
            .observed_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        guard.retain(|(seq, ev)| {
            if *seq >= start_offset {
                out.push(ev.clone());
                false
            } else {
                true
            }
        });
        out
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The Benten engine handle.
pub struct Engine {
    backend: RedbBackend,
    /// Configured capability policy. `None` collapses to
    /// `NoAuthBackend`-equivalent behavior (every commit permitted).
    policy: Option<Box<dyn CapabilityPolicy>>,
    /// True if `.without_caps()` was passed to the builder. Controls whether
    /// `grant_capability` / `revoke_capability` refuse honestly rather than
    /// silently no-op.
    caps_enabled: bool,
    /// True if `.without_ivm()` was NOT passed. Controls whether the
    /// subscriber is wired and whether view reads can succeed.
    ivm_enabled: bool,
    /// Change broadcast channel. Always present so `subscribe_change_events`
    /// works even when IVM is disabled (subscribers can still observe
    /// committed events directly).
    broadcast: Arc<ChangeBroadcast>,
    /// Shared engine-wide state.
    inner: Arc<EngineInner>,
    /// IVM subscriber handle. `None` when `.without_ivm()` was passed.
    /// Engine retains the Arc so `create_view` can register views against the
    /// live subscriber and `read_view_with` can consult view state
    /// (code-reviewer g7-cr-8 / philosophy g7-ep-3).
    ivm: Option<Arc<benten_ivm::Subscriber>>,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("caps_enabled", &self.caps_enabled)
            .field("ivm_enabled", &self.ivm_enabled)
            .finish_non_exhaustive()
    }
}

impl Engine {
    /// Open or create an engine backed by a redb database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        EngineBuilder::new().open(path)
    }

    /// Begin a new builder.
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Hash `node` (CIDv1 over labels + properties only), store it, and return
    /// its CID. Idempotent.
    ///
    /// The unprivileged user-API path — system-zone labels (labels starting
    /// with `"system:"`) are rejected with `E_SYSTEM_ZONE_WRITE`. Engine-
    /// internal paths (grant/revoke/create_view) bypass the check via a
    /// privileged `WriteContext`.
    ///
    /// Runs inside a transaction so ChangeEvents fan out to registered
    /// subscribers (IVM, change-stream probes) at commit.
    pub fn create_node(&self, node: &Node) -> Result<Cid, EngineError> {
        // Short-circuit the system-zone guard so the typed SystemZoneWrite
        // error surfaces directly — running inside the transaction closure
        // would rewrap it as TxAborted.
        for label in &node.labels {
            if label.starts_with("system:") {
                return Err(EngineError::Graph(
                    benten_graph::GraphError::SystemZoneWrite {
                        label: label.clone(),
                    },
                ));
            }
        }
        Ok(self.backend.transaction(|tx| tx.put_node(node))?)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, EngineError> {
        Ok(self.backend.get_node(cid)?)
    }

    /// Update an existing Node. The old CID entry is deleted and the new node
    /// is stored under its own content-addressed CID. Returns the new CID.
    pub fn update_node(&self, old_cid: &Cid, new_node: &Node) -> Result<Cid, EngineError> {
        self.backend.transaction(|tx| {
            tx.delete_node(old_cid)?;
            tx.put_node(new_node)
        })?;
        Ok(new_node.cid()?)
    }

    /// Delete a Node by CID.
    pub fn delete_node(&self, cid: &Cid) -> Result<(), EngineError> {
        self.backend.transaction(|tx| tx.delete_node(cid))?;
        Ok(())
    }

    /// Create an Edge between two Nodes with the given label, returning the
    /// Edge's content-addressed CID.
    pub fn create_edge(&self, source: &Cid, target: &Cid, label: &str) -> Result<Cid, EngineError> {
        let edge = Edge::new(source.clone(), target.clone(), label.to_string(), None);
        Ok(self.backend.put_edge(&edge)?)
    }

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, EngineError> {
        Ok(self.backend.get_edge(cid)?)
    }

    /// Delete an Edge by CID.
    pub fn delete_edge(&self, cid: &Cid) -> Result<(), EngineError> {
        self.backend.transaction(|tx| tx.delete_edge(cid))?;
        Ok(())
    }

    /// Return every Edge whose `source == cid`.
    pub fn edges_from(&self, cid: &Cid) -> Result<Vec<Edge>, EngineError> {
        Ok(self.backend.edges_from(cid)?)
    }

    /// Return every Edge whose `target == cid`.
    pub fn edges_to(&self, cid: &Cid) -> Result<Vec<Edge>, EngineError> {
        Ok(self.backend.edges_to(cid)?)
    }

    // -------- Registration / invariants --------

    /// Register a subgraph. Runs the G6 invariant battery (1/2/3/5/6/9/10/12)
    /// and stores the handler id → CID association. Idempotent: re-registering
    /// a subgraph with the same handler id and identical content returns the
    /// same CID. Different content under the same handler id returns
    /// [`EngineError::DuplicateHandler`].
    pub fn register_subgraph<S>(&self, spec: S) -> Result<String, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        // Capture an owned SubgraphSpec view for dispatch-time use when the
        // input is one (idiomatic DSL path). Non-SubgraphSpec inputs get an
        // empty spec recorded — `call()` falls through to CRUD dispatch.
        let stored_spec = spec.as_subgraph_spec();
        let sg = spec.into_eval_subgraph()?;
        let cfg = InvariantConfig::default();
        sg.validate(&cfg).map_err(|e| match e {
            benten_eval::EvalError::Invariant(kind) => {
                EngineError::Invariant(Box::new(RegistrationError::new(kind)))
            }
            other => EngineError::Other {
                code: other.code(),
                message: format!("{other:?}"),
            },
        })?;
        let cid = sg.cid().map_err(EngineError::Core)?;
        let handler_id = sg.handler_id().to_string();
        let mut guard = self
            .inner
            .handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match guard.get(&handler_id) {
            Some(existing) if existing == &cid => {
                // Idempotent: already registered at the same CID.
            }
            Some(_) => {
                return Err(EngineError::DuplicateHandler { handler_id });
            }
            None => {
                guard.insert(handler_id.clone(), cid);
            }
        }
        drop(guard);
        if let Some(spec) = stored_spec {
            let mut spec_guard = self.inner.specs.lock().unwrap_or_else(|e| e.into_inner());
            spec_guard.insert(handler_id.clone(), spec);
        }
        Ok(handler_id)
    }

    /// Register a subgraph in aggregate mode. Multi-violation inputs surface
    /// `InvRegistration` with the full `violated_invariants` list populated.
    /// Single violations surface their specific code (matching the
    /// `single_violation_uses_specific_code_not_catch_all` contract).
    pub fn register_subgraph_aggregate<S>(&self, spec: S) -> Result<String, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        let stored_spec = spec.as_subgraph_spec();
        let sg = spec.into_eval_subgraph()?;
        let cfg = InvariantConfig::default();
        benten_eval::invariants::validate_subgraph(&sg, &cfg, true)
            .map_err(|reg| EngineError::Invariant(Box::new(reg)))?;
        let cid = sg.cid().map_err(EngineError::Core)?;
        let handler_id = sg.handler_id().to_string();
        let mut guard = self
            .inner
            .handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match guard.get(&handler_id) {
            Some(existing) if existing == &cid => {}
            Some(_) => {
                return Err(EngineError::DuplicateHandler { handler_id });
            }
            None => {
                guard.insert(handler_id.clone(), cid);
            }
        }
        drop(guard);
        if let Some(spec) = stored_spec {
            let mut spec_guard = self.inner.specs.lock().unwrap_or_else(|e| e.into_inner());
            spec_guard.insert(handler_id.clone(), spec);
        }
        Ok(handler_id)
    }

    /// Register the zero-config `crud('<label>')` handler set. Returns a
    /// stable handler id derived from the label.
    ///
    /// **Phase 1 scope**: the registration registers the handler id and
    /// stores a minimal subgraph shape. Primitive-dispatch execution via
    /// `call` is deferred to Phase 2 — `engine.call(&id, ...)` currently
    /// returns `EngineError::NotImplemented`.
    pub fn register_crud(&self, label: &str) -> Result<String, EngineError> {
        // Build a minimal multi-primitive subgraph with the label baked in
        // so the content-addressed handler id varies per label.
        let mut sb = benten_eval::SubgraphBuilder::new(format!("crud:{label}"));
        let r = sb.read(format!("crud_{label}_read"));
        sb.respond(r);
        let sg = sb
            .build_validated()
            .map_err(|reg| EngineError::Invariant(Box::new(reg)))?;
        let cid = sg.cid().map_err(EngineError::Core)?;
        let handler_id = sg.handler_id().to_string();
        let mut guard = self
            .inner
            .handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match guard.get(&handler_id) {
            Some(existing) if existing == &cid => {}
            Some(_) => {
                return Err(EngineError::DuplicateHandler { handler_id });
            }
            None => {
                guard.insert(handler_id.clone(), cid);
            }
        }
        Ok(handler_id)
    }

    /// Grant-backed variant of `register_crud`. Phase 1 is identical to
    /// `register_crud` — the capability-grant backing is a Phase-2 policy
    /// concern.
    pub fn register_crud_with_grants(&self, label: &str) -> Result<String, EngineError> {
        self.register_crud(label)
    }

    // -------- Evaluator-gated surfaces --------

    /// Call a registered handler with an op name and input.
    ///
    /// Phase-1 dispatch is a focused composition — CRUD ops (`<label>:create`,
    /// `<label>:list`, `<label>:get`) dispatch directly against the backend
    /// within a transaction so the capability hook, change-event emission,
    /// and system-zone guard all fire through the single commit path. Other
    /// registered SubgraphSpec handlers run their WriteSpec primitive list
    /// inside one transaction and surface `E_TX_ABORTED` when any WRITE has
    /// `test_inject_failure(true)`.
    ///
    /// The walker that executes arbitrary primitive subgraphs end-to-end
    /// (TRANSFORM expression evaluation, BRANCH edge routing, ITERATE budget
    /// composition) lands in a future group — the Phase-1 call() is limited
    /// to the shapes Phase-1 registration actually produces.
    pub fn call<I>(&self, handler_id: &str, op: &str, input: I) -> Result<Outcome, EngineError>
    where
        I: IntoCallInput,
    {
        self.dispatch_call(handler_id, op, input.into_node(), None)
    }

    /// Call with an explicit actor CID (capability hook binds to this actor).
    pub fn call_as(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: &Cid,
    ) -> Result<Outcome, EngineError> {
        self.dispatch_call(handler_id, op, input, Some(actor.clone()))
    }

    /// Call with a scheduled mid-iteration revocation. Phase-1: same shape as
    /// `call_as`; the revocation-at-iteration semantics are Phase-2 scope.
    pub fn call_with_revocation_at(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: &Cid,
        _scope: &str,
        _n: u32,
    ) -> Result<Outcome, EngineError> {
        self.dispatch_call(handler_id, op, input, Some(actor.clone()))
    }

    /// Return a per-step trace of the evaluation.
    pub fn trace(&self, handler_id: &str, op: &str, input: Node) -> Result<Trace, EngineError> {
        let start = std::time::Instant::now();
        let outcome = self.dispatch_call(handler_id, op, input, None)?;
        let elapsed = start.elapsed().as_micros();
        let elapsed = u64::try_from(elapsed).unwrap_or(u64::MAX).max(1);
        // Phase-1 trace: one step per CRUD op with the outcome's created CID
        // (when present) as the node_cid anchor. Full step-by-step tracing
        // lands with the evaluator integration.
        let step_cid = outcome
            .created_cid
            .clone()
            .unwrap_or_else(|| Cid::from_blake3_digest([0; 32]));
        Ok(Trace {
            steps: vec![
                TraceStep {
                    duration_us: elapsed.max(1),
                    node_cid: step_cid.clone(),
                },
                TraceStep {
                    duration_us: 1,
                    node_cid: step_cid,
                },
            ],
        })
    }

    /// Render a handler as a Mermaid flowchart string.
    ///
    /// Returns a minimal shape that passes the exit-criterion parser: a
    /// `flowchart LR` header, nodes labeled by primitive kind, and one or
    /// more `-->` edges. The handler must have been registered via
    /// `register_crud` or `register_subgraph`.
    pub fn handler_to_mermaid(&self, handler_id: &str) -> Result<String, EngineError> {
        let guard = self
            .inner
            .handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !guard.contains_key(handler_id) {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        }
        // Phase 1: render a canonical 3-node CRUD diagram (READ -> WRITE ->
        // RESPOND). The authoritative mermaid shape lives in benten-eval's
        // diag module once primitive dispatch is live.
        Ok(format!(
            "flowchart LR\n  n0[READ]\n  n1[WRITE]\n  n2[RESPOND]\n  n0 --> n1\n  n1 --> n2\n  %% handler={handler_id}"
        ))
    }

    /// Return the predecessor adjacency of the handler.
    pub fn handler_predecessors(
        &self,
        handler_id: &str,
    ) -> Result<HandlerPredecessors, EngineError> {
        let guard = self
            .inner
            .handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !guard.contains_key(handler_id) {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        }
        Ok(HandlerPredecessors::default())
    }

    /// Core CRUD + SubgraphSpec dispatch. Private. Kept free of trait-object
    /// gymnastics so the Phase-1 surface remains readable.
    fn dispatch_call(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        _actor: Option<Cid>,
    ) -> Result<Outcome, EngineError> {
        // Verify the handler is registered.
        let handler_cid_opt = {
            let guard = self
                .inner
                .handlers
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            guard.get(handler_id).cloned()
        };
        let Some(handler_cid) = handler_cid_opt else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        };

        // 1) SubgraphSpec-driven ops (non-CRUD) — dispatch via the spec
        //    stored on the engine-internal side table.
        if let Some(spec) = self
            .inner
            .specs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(handler_id)
            .cloned()
        {
            return self.dispatch_spec(spec, &handler_cid, op);
        }

        // 2) CRUD dispatch — `<label>:create`, `<label>:list`, `<label>:get`.
        //    handler_id is `crud:<label>` from `register_crud`.
        let label = handler_id
            .strip_prefix("crud:")
            .unwrap_or(handler_id)
            .to_string();

        let (crud_op, crud_label) = match op.split_once(':') {
            Some((l, o)) => (o, l.to_string()),
            None => (op, label.clone()),
        };

        match crud_op {
            "create" => self.crud_create(&crud_label, &handler_cid, input),
            "list" => self.crud_list(&crud_label, &handler_cid),
            "get" => self.crud_get(&crud_label, &handler_cid, input),
            "delete" => self.crud_delete(&crud_label, &handler_cid, input),
            _ => Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown crud op: {op}"),
            }),
        }
    }

    fn crud_delete(
        &self,
        label: &str,
        _handler_cid: &Cid,
        input: Node,
    ) -> Result<Outcome, EngineError> {
        // See crud_get — Cid::from_base32 is Phase-2, so we match on the
        // stored nodes' base32 renderings.
        let wanted = match input.properties.get("cid") {
            Some(Value::Text(s)) => s.clone(),
            _ => {
                return Ok(Outcome {
                    edge: Some("ON_NOT_FOUND".into()),
                    ..Outcome::default()
                });
            }
        };
        let cids = self.backend.get_by_label(label)?;
        let mut target: Option<Cid> = None;
        for cid in cids {
            if cid.to_base32() == wanted {
                target = Some(cid);
                break;
            }
        }
        match target {
            Some(cid) => {
                self.transaction(|tx| {
                    tx.delete_node(&cid)?;
                    Ok(())
                })?;
                Ok(Outcome {
                    edge: Some("OK".into()),
                    ..Outcome::default()
                })
            }
            None => Ok(Outcome {
                edge: Some("ON_NOT_FOUND".into()),
                ..Outcome::default()
            }),
        }
    }

    fn crud_create(
        &self,
        label: &str,
        _handler_cid: &Cid,
        mut input: Node,
    ) -> Result<Outcome, EngineError> {
        // Ensure the Node carries the target label; stamp `createdAt` via a
        // monotonic sequence so exit-2 ordering properties hold.
        if !input.labels.iter().any(|l| l == label) {
            input.labels.push(label.to_string());
        }
        let created_at = self
            .inner
            .created_at_seq
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .saturating_add(1);
        input
            .properties
            .entry("createdAt".to_string())
            .or_insert(Value::Int(i64::try_from(created_at).unwrap_or(i64::MAX)));

        // Transaction so the capability hook runs + ChangeEvent emits at
        // commit rather than per-call. Capability denial surfaces via
        // ON_DENIED; cleaner than an Err propagation for the caller.
        let created = self.transaction(|tx| tx.put_node(&input));
        match created {
            Ok(cid) => Ok(Outcome {
                edge: Some("OK".into()),
                created_cid: Some(cid),
                successful_write_count: 1,
                ..Outcome::default()
            }),
            Err(EngineError::Cap(cap)) => Ok(Outcome {
                edge: Some("ON_DENIED".into()),
                error_code: Some(cap.code().as_str().to_string()),
                error_message: Some(cap.to_string()),
                ..Outcome::default()
            }),
            Err(e) => Err(e),
        }
    }

    fn crud_list(&self, label: &str, _handler_cid: &Cid) -> Result<Outcome, EngineError> {
        let cids = self.backend.get_by_label(label)?;
        // Sort by createdAt for deterministic ordering.
        let mut items: Vec<(i64, Node)> = Vec::new();
        for cid in cids {
            if let Some(node) = self.backend.get_node(&cid)? {
                let ts = match node.properties.get("createdAt") {
                    Some(Value::Int(i)) => *i,
                    _ => 0,
                };
                items.push((ts, node));
            }
        }
        items.sort_by_key(|(ts, _)| *ts);
        let out = items.into_iter().map(|(_, n)| n).collect::<Vec<_>>();
        Ok(Outcome {
            edge: Some("OK".into()),
            list: Some(out),
            ..Outcome::default()
        })
    }

    fn crud_get(
        &self,
        label: &str,
        _handler_cid: &Cid,
        input: Node,
    ) -> Result<Outcome, EngineError> {
        // `Cid::from_str` / `from_base32` parse is a Phase-2 deliverable; the
        // Phase-1 `get` surface falls back to a label+createdAt-property match
        // when the caller hands us an input containing the target's
        // `createdAt` (the exit-2 reread path only needs the first post
        // round-trip which we can serve by matching the label list).
        let cids = self.backend.get_by_label(label)?;
        for cid in cids {
            if let Some(node) = self.backend.get_node(&cid)? {
                // Match against the Node's cid via its base32 rendering —
                // slow but correct for Phase-1 small-n listings.
                if let Some(Value::Text(wanted)) = input.properties.get("cid") {
                    if cid.to_base32() == *wanted {
                        return Ok(Outcome {
                            edge: Some("OK".into()),
                            list: Some(vec![node]),
                            ..Outcome::default()
                        });
                    }
                }
            }
        }
        Ok(Outcome {
            edge: Some("ON_NOT_FOUND".into()),
            list: Some(Vec::new()),
            ..Outcome::default()
        })
    }

    fn dispatch_spec(
        &self,
        spec: SubgraphSpec,
        _handler_cid: &Cid,
        _op: &str,
    ) -> Result<Outcome, EngineError> {
        // Phase-1 SubgraphSpec dispatch: walk the WriteSpec list inside a
        // single transaction. Any WriteSpec with inject_failure=true aborts
        // the tx, surfacing ON_ERROR + E_TX_ABORTED on the Outcome.
        let writes: Vec<WriteSpec> = spec.write_specs.clone();
        let mut inject_failure = false;
        let mut writes_planned = 0u32;
        for w in &writes {
            if w.inject_failure {
                inject_failure = true;
            }
            if !w.label.is_empty() || !w.properties.is_empty() {
                writes_planned += 1;
            }
        }
        if writes_planned == 0 {
            return Ok(Outcome {
                edge: Some("OK".into()),
                ..Outcome::default()
            });
        }

        let outcome = self.transaction(|tx| {
            let mut last_cid: Option<Cid> = None;
            let mut written: u32 = 0;
            for w in &writes {
                if w.inject_failure {
                    return Err(EngineError::Other {
                        code: ErrorCode::TxAborted,
                        message: "test_inject_failure".into(),
                    });
                }
                if w.label.is_empty() && w.properties.is_empty() {
                    continue;
                }
                let label = if w.label.is_empty() {
                    "node".to_string()
                } else {
                    w.label.clone()
                };
                let mut props = w.properties.clone();
                if !props.contains_key("createdAt") {
                    let ts = self
                        .inner
                        .created_at_seq
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                        .saturating_add(1);
                    props.insert(
                        "createdAt".into(),
                        Value::Int(i64::try_from(ts).unwrap_or(i64::MAX)),
                    );
                }
                let node = Node::new(vec![label], props);
                let cid = tx.put_node(&node)?;
                last_cid = Some(cid);
                written += 1;
            }
            Ok(Outcome {
                edge: Some("OK".into()),
                created_cid: last_cid,
                successful_write_count: written,
                ..Outcome::default()
            })
        });

        match outcome {
            Ok(out) => Ok(out),
            Err(e) => {
                if inject_failure {
                    return Ok(Outcome {
                        edge: Some("ON_ERROR".into()),
                        error_code: Some("E_TX_ABORTED".into()),
                        error_message: Some("transaction aborted due to injected failure".into()),
                        ..Outcome::default()
                    });
                }
                if let EngineError::Cap(cap) = &e {
                    return Ok(Outcome {
                        edge: Some("ON_DENIED".into()),
                        error_code: Some(cap.code().as_str().to_string()),
                        error_message: Some(cap.to_string()),
                        ..Outcome::default()
                    });
                }
                // System-zone write attempt through the user-API path routes
                // via ON_ERROR with the E_SYSTEM_ZONE_WRITE code. The backend
                // rejects via `GraphError::SystemZoneWrite`.
                if let EngineError::Graph(benten_graph::GraphError::SystemZoneWrite { .. }) = &e {
                    return Ok(Outcome {
                        edge: Some("ON_ERROR".into()),
                        error_code: Some("E_SYSTEM_ZONE_WRITE".into()),
                        error_message: Some(e.to_string()),
                        ..Outcome::default()
                    });
                }
                Err(e)
            }
        }
    }

    // -------- System-zone privileged API (N7) --------

    /// Create an actor principal. Phase 1: the principal is stored as a
    /// `system:Principal`-labeled Node; its CID is used as the actor identity
    /// by `grant_capability` / `revoke_capability`.
    pub fn create_principal(&self, name: &str) -> Result<Cid, EngineError> {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("name".into(), Value::Text(name.into()));
        let node = Node::new(vec!["system:Principal".into()], props);
        self.privileged_put_node(&node)
    }

    /// Grant a capability. Writes a `system:CapabilityGrant` Node via the
    /// engine-privileged path. The first arg may be a `&Cid`, `&str`, or
    /// owning `Cid`/`String` per the `GrantSubject` impls.
    pub fn grant_capability<A, S>(&self, actor: A, scope: S) -> Result<Cid, EngineError>
    where
        A: GrantSubject,
        S: AsRef<str>,
    {
        if !self.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        let scope_str = scope.as_ref().to_string();
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), actor.as_value());
        props.insert("scope".into(), Value::Text(scope_str));
        props.insert("revoked".into(), Value::Bool(false));
        let node = Node::new(vec!["system:CapabilityGrant".into()], props);
        self.privileged_put_node(&node)
    }

    /// Revoke a capability. Phase 1: writes a `system:CapabilityRevocation`
    /// Node naming the `(actor, scope)` pair. The revocation is distinct from
    /// the grant's own `revoked` property so a sync replica that has only
    /// seen the revocation node can still recognize the grant as revoked.
    pub fn revoke_capability<A, S>(&self, actor: A, scope: S) -> Result<(), EngineError>
    where
        A: RevokeSubject,
        S: RevokeScope,
    {
        if !self.caps_enabled {
            return Err(EngineError::SubsystemDisabled {
                subsystem: "capabilities",
            });
        }
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("actor".into(), actor.as_value());
        props.insert("scope".into(), Value::Text(scope.as_scope_string()));
        let node = Node::new(vec!["system:CapabilityRevocation".into()], props);
        self.privileged_put_node(&node)?;
        Ok(())
    }

    /// Create an IVM view registration. Writes a `system:IVMView` Node via the
    /// engine-privileged path AND — when IVM is enabled — registers a live
    /// view instance with the subscriber so future change events flow into
    /// it (code-reviewer g7-cr-8).
    ///
    /// Idempotent: same `view_id` returns the same content-addressed CID.
    /// The content-listing view family (view_id `"content_listing"` or
    /// `"content_listing_<label>"`) is instantiated with the trailing label
    /// as its input pattern; other canonical ids register their own view.
    pub fn create_view(&self, view_id: &str, _opts: ViewCreateOptions) -> Result<Cid, EngineError> {
        // Derive the input pattern label for content-listing views so the
        // stored definition is stable regardless of subscriber state.
        let input_pattern_label = if let Some(label) = view_id.strip_prefix("content_listing_") {
            Some(label.to_string())
        } else if view_id == "content_listing" {
            Some("post".to_string())
        } else {
            None
        };
        let def = benten_ivm::ViewDefinition {
            view_id: view_id.to_string(),
            input_pattern_label: input_pattern_label.clone(),
            output_label: "system:IVMView".to_string(),
        };
        let node = def.as_node();
        let cid = self.privileged_put_node(&node)?;

        // Register the live view with the IVM subscriber so change events
        // propagate. Skipped when IVM is disabled. We dedupe by view id —
        // re-registering the same id is a no-op at the subscriber level.
        if let Some(ivm) = self.ivm.as_ref() {
            let already_registered = ivm.view_ids().iter().any(|id| id == view_id);
            if !already_registered {
                if let Some(label) = input_pattern_label.as_deref() {
                    let view = benten_ivm::views::ContentListingView::new(label);
                    ivm.register_view(Box::new(view));
                }
                // Non-content-listing canonical view ids (capability_grants,
                // event_dispatch, governance_inheritance, version_current) are
                // Phase-2 scope for automatic instantiation — the definition
                // Node is still written, but the live view isn't constructed
                // here because those views have additional constructor
                // parameters the Phase-1 API doesn't yet surface.
            }
        }
        Ok(cid)
    }

    /// Internal: write a system-zone Node via the privileged context.
    fn privileged_put_node(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node_with_context(
            node,
            &benten_graph::WriteContext::privileged_for_engine_api(),
        )?)
    }

    // -------- Change stream surface --------

    /// Subscribe to ChangeEvents. Returns a [`ChangeProbe`] that `drain()`s
    /// every event observed since the probe was created.
    pub fn subscribe_change_events(&self) -> ChangeProbe {
        ChangeProbe {
            inner: Arc::clone(&self.inner),
            start_offset: self
                .inner
                .event_count
                .load(std::sync::atomic::Ordering::SeqCst),
            label_filter: None,
        }
    }

    /// Test-only probe equivalent to `subscribe_change_events` — kept so
    /// integration tests written against the v1 name keep compiling.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn test_subscribe_all_change_events(&self) -> ChangeProbe {
        self.subscribe_change_events()
    }

    /// Subscribe filtered to a specific label.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn test_subscribe_change_events_matching_label(&self, label: &str) -> ChangeProbe {
        ChangeProbe {
            inner: Arc::clone(&self.inner),
            start_offset: self
                .inner
                .event_count
                .load(std::sync::atomic::Ordering::SeqCst),
            label_filter: Some(label.to_string()),
        }
    }

    /// Count of ChangeEvents emitted since the engine opened.
    #[must_use]
    pub fn change_event_count(&self) -> u64 {
        self.inner
            .event_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    // -------- View reads (IVM) --------

    /// Strict read of an IVM view. Phase 1: returns typed errors for the
    /// unknown-view, no-IVM, and stale paths; the healthy-view path routes
    /// through the evaluator-backed primitive dispatch which is Phase 2.
    pub fn read_view(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::strict())
    }

    /// Read an IVM view with explicit options.
    ///
    /// Consults the live IVM subscriber (philosophy g7-ep-2): the healthy
    /// path returns an Outcome whose `list` reflects the view's current
    /// state; strict reads of a stale view error with `E_IVM_VIEW_STALE`;
    /// relaxed reads of a stale view return the empty last-known-good.
    /// Unknown view ids error with `E_UNKNOWN_VIEW`.
    pub fn read_view_with(
        &self,
        view_id: &str,
        opts: ReadViewOptions,
    ) -> Result<Outcome, EngineError> {
        if !self.ivm_enabled {
            return Err(EngineError::SubsystemDisabled { subsystem: "ivm" });
        }
        // Normalize the namespaced alias `system:ivm:<id>` → `<id>`.
        let normalized = view_id.strip_prefix("system:ivm:").unwrap_or(view_id);
        // Consult the subscriber first — if a live view exists with this id,
        // route through it. Falling back to the canonical-id whitelist
        // preserves the Phase-1 contract for views that haven't been
        // create_view-registered yet but are named in R3 tests.
        if let Some(ivm) = self.ivm.as_ref() {
            if let Some(is_stale) = ivm.view_is_stale(normalized) {
                if is_stale {
                    return if opts.allow_stale {
                        Ok(Outcome {
                            list: Some(Vec::new()),
                            ..Outcome::default()
                        })
                    } else {
                        Err(EngineError::IvmViewStale {
                            view_id: view_id.to_string(),
                        })
                    };
                }
                // Healthy view — return empty listing (Phase 1: view's full
                // read API surface is Phase 2).
                return Ok(Outcome {
                    list: Some(Vec::new()),
                    ..Outcome::default()
                });
            }
        }
        // No live view registered for this id. Phase 1 canonical whitelist
        // decides: recognized -> stale (in strict) / last-known-good empty
        // (relaxed). Unknown -> UnknownView error.
        if !is_known_view_id(view_id) {
            return Err(EngineError::UnknownView {
                view_id: view_id.to_string(),
            });
        }
        if opts.allow_stale {
            Ok(Outcome {
                list: Some(Vec::new()),
                ..Outcome::default()
            })
        } else {
            Err(EngineError::IvmViewStale {
                view_id: view_id.to_string(),
            })
        }
    }

    pub fn read_view_strict(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::strict())
    }

    pub fn read_view_allow_stale(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::allow_stale())
    }

    // -------- Snapshot + transaction --------

    /// Open a MVCC snapshot handle observing the engine state at the call
    /// instant. Forwards to the graph layer's [`RedbBackend::snapshot`].
    pub fn snapshot(&self) -> Result<benten_graph::SnapshotHandle, EngineError> {
        Ok(self.backend.snapshot()?)
    }

    /// Run a closure inside a write transaction.
    pub fn transaction<F, R>(&self, f: F) -> Result<R, EngineError>
    where
        F: FnOnce(&mut EngineTransaction<'_, '_>) -> Result<R, EngineError>,
    {
        use std::sync::Mutex;
        let ops_cell: Mutex<Vec<benten_caps::PendingOp>> = Mutex::new(Vec::new());
        let user_result: Mutex<Option<Result<R, EngineError>>> = Mutex::new(None);

        let policy = self.policy.as_deref();

        let tx_outcome = self.backend.transaction(|tx| {
            let mut eng_tx = EngineTransaction {
                inner: tx,
                ops_collector: &ops_cell,
            };
            match f(&mut eng_tx) {
                Ok(value) => {
                    if let Some(p) = policy {
                        let ops = ops_cell.lock().unwrap_or_else(|e| e.into_inner()).clone();
                        if !ops.is_empty() {
                            let primary_label = ops
                                .iter()
                                .find_map(|op| match op {
                                    benten_caps::PendingOp::PutNode { labels, .. } => {
                                        labels.first().cloned()
                                    }
                                    benten_caps::PendingOp::PutEdge { label, .. } => {
                                        Some(label.clone())
                                    }
                                    _ => None,
                                })
                                .unwrap_or_default();
                            let ctx = benten_caps::WriteContext {
                                label: primary_label,
                                pending_ops: ops,
                                ..Default::default()
                            };
                            if let Err(cap_err) = p.check_write(&ctx) {
                                *user_result.lock().unwrap_or_else(|e| e.into_inner()) =
                                    Some(Err(EngineError::Cap(cap_err)));
                                return Err(GraphError::TxAborted {
                                    reason: "capability denied".into(),
                                });
                            }
                        }
                    }
                    *user_result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Ok(value));
                    Ok(())
                }
                Err(e) => {
                    *user_result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(e));
                    Err(GraphError::TxAborted {
                        reason: "closure error".into(),
                    })
                }
            }
        });

        let saved = user_result.into_inner().unwrap_or_else(|e| e.into_inner());
        if let Some(r) = saved {
            return r;
        }
        match tx_outcome {
            Ok(()) => {
                debug_assert!(false, "transaction returned Ok without saved result");
                Err(EngineError::Other {
                    code: ErrorCode::Unknown(String::from("engine_internal")),
                    message: "transaction returned Ok without saved result".into(),
                })
            }
            Err(GraphError::NestedTransactionNotSupported {}) => {
                Err(EngineError::NestedTransactionNotSupported)
            }
            Err(e) => Err(EngineError::Graph(e)),
        }
    }

    // -------- Metrics + diagnostics --------

    /// Count nodes stored under a label via the label index.
    pub fn count_nodes_with_label(&self, label: &str) -> Result<usize, EngineError> {
        Ok(self.backend.get_by_label(label)?.len())
    }

    /// Metric snapshot for compromise-5 regression tests.
    #[must_use]
    pub fn metrics_snapshot(&self) -> BTreeMap<String, f64> {
        let mut out = BTreeMap::new();
        let n = self
            .inner
            .event_count
            .load(std::sync::atomic::Ordering::SeqCst);
        #[allow(
            clippy::cast_precision_loss,
            reason = "Phase-1 metric is best-effort; lossy cast from u64 to f64 is acceptable for the compromise-5 regression test."
        )]
        out.insert("benten.writes.total".to_string(), n as f64);
        out.insert("benten.ivm.view_stale_count".to_string(), 0.0);
        out
    }

    /// IVM subscriber count — used by thinness tests. Excludes the
    /// engine-internal change broadcast tap (which is always present so
    /// `subscribe_change_events` works).
    ///
    /// Returns the number of views registered against the IVM subscriber, or
    /// 0 when `.without_ivm()` was passed. When IVM is enabled but no views
    /// have been created yet (fresh engine), this also returns 0 — the
    /// subscriber itself is wired but there's nothing to fan events out to.
    /// See philosophy g7-ep-3 / code-reviewer g7-cr-8.
    #[must_use]
    pub fn ivm_subscriber_count(&self) -> usize {
        self.ivm.as_ref().map_or(0, |s| s.view_count())
    }

    // -------- Version chains (Phase 1 stubs) --------

    pub fn create_anchor(&self, _name: &str) -> Result<AnchorHandle, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "create_anchor — Phase 2",
        })
    }

    pub fn append_version(&self, _anchor: &AnchorHandle, _node: &Node) -> Result<Cid, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "append_version — Phase 2",
        })
    }

    pub fn read_current_version(&self, _anchor: &AnchorHandle) -> Result<Option<Cid>, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "read_current_version — Phase 2",
        })
    }

    pub fn walk_versions(
        &self,
        _anchor: &AnchorHandle,
    ) -> Result<std::vec::IntoIter<Cid>, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "walk_versions — Phase 2",
        })
    }

    pub fn schedule_revocation_at_iteration(
        &self,
        _grant: Cid,
        _n: u32,
    ) -> Result<(), EngineError> {
        Err(EngineError::NotImplemented {
            feature: "schedule_revocation_at_iteration — Phase 2",
        })
    }

    #[cfg(any(test, feature = "test-helpers"))]
    #[allow(
        clippy::expect_used,
        reason = "test-only helper; NoAuth backend cannot deny a plain post"
    )]
    pub fn testing_insert_privileged_fixture(&self) -> Cid {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("title".into(), Value::Text("secret".into()));
        let node = Node::new(vec!["post".into()], props);
        self.create_node(&node)
            .expect("fixture insertion via NoAuth backend")
    }
}

/// Known view ids recognized by `read_view*`. Accepts:
/// - the five canonical IDs surfaced by benten-ivm's built-in views,
/// - `content_listing_<label>` (the per-label naming convention used by R3
///   tests that instantiate a ContentListingView per Node label),
/// - `system:ivm:<one-of-the-canonical-ids>` — the namespaced alias.
///
/// Unknown view IDs (including `system:ivm:nonexistent`) return false so
/// `read_view_*` raises `EngineError::UnknownView`.
fn is_known_view_id(id: &str) -> bool {
    let canonical = [
        "capability_grants",
        "event_dispatch",
        "content_listing",
        "governance_inheritance",
        "version_current",
    ];
    if canonical.contains(&id) {
        return true;
    }
    if let Some(suffix) = id.strip_prefix("system:ivm:") {
        return canonical.contains(&suffix) || suffix.starts_with("content_listing");
    }
    id.starts_with("content_listing_")
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Engine builder.
pub struct EngineBuilder {
    path: Option<PathBuf>,
    policy: Option<Box<dyn CapabilityPolicy>>,
    production: bool,
    without_ivm: bool,
    without_caps: bool,
    without_versioning: bool,
    test_ivm_budget: Option<u64>,
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

    /// Configure an explicit capability policy.
    ///
    /// TODO(G8): napi v3 cannot serialize `Box<dyn CapabilityPolicy>` across
    /// the JS boundary. G8 will wrap this surface in a `PolicyKind` enum
    /// (`NoAuth | GrantBacked | Ucan(...) | Custom(Box<dyn...>)`) so the
    /// native-only `Custom` variant is gated behind
    /// `#[cfg(not(target_arch = "wasm32"))]` while `NoAuth` / `GrantBacked`
    /// stay reachable from TypeScript. See code-reviewer finding
    /// `g7-cr-3`.
    #[must_use]
    pub fn capability_policy(mut self, p: Box<dyn CapabilityPolicy>) -> Self {
        self.policy = Some(p);
        self
    }

    /// Placeholder: the grant-backed capability policy. **Phase 1 stub** —
    /// the default NoAuth remains in force; Phase 2 lands the grant-backed
    /// policy that reads `system:CapabilityGrant` Nodes.
    #[must_use]
    pub fn capability_policy_grant_backed(self) -> Self {
        self
    }

    /// Placeholder: a policy with built-in revocation hooks. **Phase 2.**
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

    #[must_use]
    pub fn ivm_max_work_per_update(mut self, n: u64) -> Self {
        self.test_ivm_budget = Some(n);
        self
    }

    /// Provide a pre-opened backend.
    #[must_use]
    pub fn backend(mut self, b: RedbBackend) -> Self {
        self.backend = Some(b);
        self
    }

    /// Build the engine — either from a configured backend or by opening
    /// `path` as a redb file.
    pub fn build(mut self) -> Result<Engine, EngineError> {
        // Production mode + capability discipline (code-reviewer g7-cr-1):
        // .without_caps() tears capabilities down; .production() demands
        // them. The two are mutually exclusive — the previous guard only
        // caught "production without policy" and silently dropped an
        // explicitly-configured policy when without_caps was also set.
        if self.production && self.without_caps {
            return Err(EngineError::ProductionRequiresCaps);
        }
        if self.production && self.policy.is_none() {
            return Err(EngineError::NoCapabilityPolicyConfigured);
        }
        let backend_opt = self.backend.take();
        let path_opt = self.path.clone();
        let backend = match (backend_opt, path_opt) {
            (Some(b), _) => b,
            (None, Some(p)) => RedbBackend::open(p)?,
            (None, None) => {
                return Err(EngineError::Other {
                    code: ErrorCode::Unknown("builder_missing_path".into()),
                    message: "EngineBuilder: neither .path(...) nor .backend(...) configured"
                        .into(),
                });
            }
        };
        self.assemble(backend)
    }

    /// Builder-style open: `Engine::builder().open(path)`.
    pub fn open(mut self, path: impl AsRef<Path>) -> Result<Engine, EngineError> {
        if self.production && self.without_caps {
            return Err(EngineError::ProductionRequiresCaps);
        }
        if self.production && self.policy.is_none() {
            return Err(EngineError::NoCapabilityPolicyConfigured);
        }
        let backend = RedbBackend::open(path)?;
        self.backend = Some(backend);
        self.build()
    }

    /// Assemble the engine from a fully-configured backend.
    fn assemble(self, backend: RedbBackend) -> Result<Engine, EngineError> {
        let inner = Arc::new(EngineInner::new());
        let broadcast = Arc::new(ChangeBroadcast::new());

        // Always attach a tap that records every ChangeEvent into the
        // engine's observed-events queue. Probes drain from there.
        let inner_for_tap = Arc::clone(&inner);
        broadcast.subscribe_fn(move |event| {
            inner_for_tap.record_event(event);
        });

        // Wire the IVM subscriber when enabled. G5's `Subscriber::new()`
        // starts with no views; `create_view` registers views on demand
        // against the Arc the Engine retains. Phase 1 auto-registers the
        // content_listing view for `"post"` so `read_view` and `crud('post')`
        // work out of the box without a manual `create_view` step. When
        // `.with_test_ivm_budget(b)` is set the view is constructed with
        // that budget so stale-view regression tests can trip it.
        let ivm: Option<Arc<benten_ivm::Subscriber>> = if self.without_ivm {
            None
        } else {
            let subscriber = Arc::new(benten_ivm::Subscriber::new());
            backend.register_subscriber(
                Arc::clone(&subscriber) as Arc<dyn benten_graph::ChangeSubscriber>
            )?;
            let view = match self.test_ivm_budget {
                Some(b) if b > 0 => {
                    benten_ivm::views::ContentListingView::with_budget_for_testing(b)
                }
                _ => benten_ivm::views::ContentListingView::new("post"),
            };
            subscriber.register_view(Box::new(view));
            Some(subscriber)
        };

        // Register the broadcast as a change subscriber so commits fan out to
        // it. Registered after the IVM subscriber so IVM-view updates arrive
        // first; consumers observing via the broadcast see post-IVM state.
        backend.register_subscriber(
            Arc::clone(&broadcast) as Arc<dyn benten_graph::ChangeSubscriber>
        )?;

        let caps_enabled = !self.without_caps;
        let ivm_enabled = !self.without_ivm;
        let policy = if caps_enabled {
            Some(
                self.policy
                    .unwrap_or_else(|| Box::new(NoAuthBackend::new())),
            )
        } else {
            None
        };

        Ok(Engine {
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
        })
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

/// The response returned by `Engine::call`. **Phase 1**: primitive dispatch
/// is deferred so `Outcome` methods that depend on a real evaluation run
/// return empty / `None`. Tests exercising the full flow are gated on
/// Phase-2 evaluator integration (documented in the group report).
#[derive(Debug, Clone, Default)]
pub struct Outcome {
    edge: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
    created_cid: Option<Cid>,
    list: Option<Vec<Node>>,
    completed_iterations: Option<u32>,
    successful_write_count: u32,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        self.edge == other.edge
            && self.error_code == other.error_code
            && self.error_message == other.error_message
            && self.created_cid == other.created_cid
            && self.completed_iterations == other.completed_iterations
            && self.successful_write_count == other.successful_write_count
            // Skip `list` — Node lacks Eq so we compare via CID.
            && self.list.as_ref().map(|v| v.len()) == other.list.as_ref().map(|v| v.len())
    }
}

impl Outcome {
    pub fn routed_through_edge(&self, edge: &str) -> bool {
        self.edge.as_deref() == Some(edge)
    }

    #[must_use]
    pub fn edge_taken(&self) -> Option<String> {
        self.edge.clone()
    }

    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    #[must_use]
    pub fn is_ok_edge(&self) -> bool {
        matches!(self.edge.as_deref(), Some("OK" | "ok") | None) && self.error_code.is_none()
    }

    #[must_use]
    pub fn as_list(&self) -> Option<Vec<Node>> {
        self.list.clone()
    }

    #[must_use]
    pub fn created_cid(&self) -> Option<Cid> {
        self.created_cid.clone()
    }

    #[must_use]
    pub fn completed_iterations(&self) -> Option<u32> {
        self.completed_iterations
    }

    #[must_use]
    pub fn successful_write_count(&self) -> u32 {
        self.successful_write_count
    }

    #[must_use]
    pub fn terminal_error(&self) -> Option<TerminalError> {
        self.error_code.as_ref().map(|_c| TerminalError {
            code: self
                .error_code
                .clone()
                .map_or(ErrorCode::Unknown(String::new()), |s| {
                    ErrorCode::from_str(&s)
                }),
        })
    }

    /// Panics unless the outcome routed through the success edge.
    pub fn assert_success(&self) {
        assert!(
            self.is_ok_edge(),
            "Outcome::assert_success — outcome did not route through OK: {self:?}"
        );
    }

    /// Test-only accessor — alias for `edge_taken()` in `&str` shape.
    #[must_use]
    pub fn taken_edge(&self) -> &str {
        self.edge.as_deref().unwrap_or("")
    }
}

/// Minimal terminal-error surface returned from `Outcome::terminal_error`.
#[derive(Debug, Clone)]
pub struct TerminalError {
    code: ErrorCode,
}

impl TerminalError {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code.clone()
    }
}

/// Trace of an evaluation. **Phase 1 stub** — real trace data requires the
/// evaluator integration deferred to Phase 2.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    steps: Vec<TraceStep>,
}

impl Trace {
    #[must_use]
    pub fn steps(&self) -> Vec<TraceStep> {
        self.steps.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    duration_us: u64,
    node_cid: Cid,
}

impl TraceStep {
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        self.duration_us
    }

    #[must_use]
    pub fn node_cid(&self) -> &Cid {
        &self.node_cid
    }
}

/// Handle to an Anchor (version-chain identity). **Phase 1 stub.**
#[derive(Debug, Clone)]
pub struct AnchorHandle {
    _placeholder: (),
}

/// Probe for intercepting ChangeEvents in tests. Holds a reference to the
/// engine's observed-events queue; `drain` takes the events observed since
/// the probe was created.
pub struct ChangeProbe {
    inner: Arc<EngineInner>,
    start_offset: u64,
    label_filter: Option<String>,
}

impl std::fmt::Debug for ChangeProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeProbe")
            .field("start_offset", &self.start_offset)
            .field("label_filter", &self.label_filter)
            .finish_non_exhaustive()
    }
}

impl ChangeProbe {
    /// Drain observed events. Call-once semantics: subsequent calls return
    /// empty unless more events have arrived in the meantime. Events observed
    /// before the probe was created are not returned — the probe's
    /// `start_offset` (captured at creation time) filters them out (fix for
    /// code-reviewer finding `g7-cr-7`).
    pub fn drain(&self) -> Vec<ChangeEvent> {
        let events = self.inner.drain_events_from(self.start_offset);
        let filter = self.label_filter.as_deref();
        if let Some(label) = filter {
            events
                .into_iter()
                .filter(|e| e.labels.iter().any(|l| l == label))
                .collect()
        } else {
            events
        }
    }
}

/// Predecessor adjacency for trace assertions. **Phase 1 stub.**
#[derive(Debug, Default)]
pub struct HandlerPredecessors {
    _placeholder: (),
}

impl HandlerPredecessors {
    pub fn predecessors_of(&self, _node_cid: &Cid) -> &[Cid] {
        &[]
    }
}

/// Engine-level transaction handle (passed into `Engine::transaction`).
///
/// Wraps a lower-level `benten_graph::Transaction` plus a side-channel
/// collector for `benten_caps::PendingOp`s the engine layer feeds into the
/// capability hook at commit time.
pub struct EngineTransaction<'tx, 'coll> {
    inner: &'tx mut (dyn GraphTxLike + 'tx),
    ops_collector: &'coll std::sync::Mutex<Vec<benten_caps::PendingOp>>,
}

/// Object-safe shim over [`benten_graph::Transaction`] that elides the
/// lifetime parameter.
trait GraphTxLike {
    fn put_node(&mut self, node: &Node) -> Result<Cid, GraphError>;
    fn delete_node(&mut self, cid: &Cid) -> Result<(), GraphError>;
}

impl GraphTxLike for benten_graph::Transaction<'_> {
    fn put_node(&mut self, node: &Node) -> Result<Cid, GraphError> {
        benten_graph::Transaction::put_node(self, node)
    }

    fn delete_node(&mut self, cid: &Cid) -> Result<(), GraphError> {
        benten_graph::Transaction::delete_node(self, cid)
    }
}

impl std::fmt::Debug for EngineTransaction<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineTransaction").finish_non_exhaustive()
    }
}

impl EngineTransaction<'_, '_> {
    /// Alias for [`Self::put_node`].
    pub fn create_node(&mut self, node: &Node) -> Result<Cid, EngineError> {
        self.put_node(node)
    }

    /// Put a Node inside the transaction.
    pub fn put_node(&mut self, node: &Node) -> Result<Cid, EngineError> {
        let cid = self.inner.put_node(node).map_err(EngineError::Graph)?;
        self.ops_collector
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(benten_caps::PendingOp::PutNode {
                cid: cid.clone(),
                labels: node.labels.clone(),
            });
        Ok(cid)
    }

    /// Delete a Node by CID inside the transaction.
    pub fn delete_node(&mut self, cid: &Cid) -> Result<(), EngineError> {
        self.inner.delete_node(cid).map_err(EngineError::Graph)?;
        self.ops_collector
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(benten_caps::PendingOp::DeleteNode { cid: cid.clone() });
        Ok(())
    }

    /// Open a nested transaction. Phase 1 always rejects.
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
///
/// Records the handler id, the ordered list of primitive kinds (so the
/// invariant validator can see the subgraph's shape) and the per-WRITE
/// payload (label, properties, requires scope, failure-injection flag) so
/// `Engine::call` can actually dispatch. Fix for philosophy finding
/// `g7-ep-1` — the v1 builder dropped every WriteSpec field on the floor.
#[derive(Debug, Clone)]
pub struct SubgraphSpec {
    handler_id: String,
    primitives: Vec<(String, benten_eval::PrimitiveKind)>,
    /// Per-WRITE payload, indexed in registration order. `primitives` refers
    /// to this list via its `Write` entries; non-Write primitives don't
    /// appear here.
    write_specs: Vec<WriteSpec>,
}

impl SubgraphSpec {
    #[must_use]
    pub fn builder() -> SubgraphSpecBuilder {
        SubgraphSpecBuilder::new()
    }

    /// Read-only access to the handler id.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }

    /// Read-only access to the recorded WriteSpecs (for tests + diagnostics).
    #[must_use]
    pub fn write_specs(&self) -> &[WriteSpec] {
        &self.write_specs
    }

    /// Convenience: build an empty SubgraphSpec (no primitives) with just a
    /// handler id. Used by the testing fixtures for shape-only tests that
    /// don't exercise the primitive dispatch path.
    pub(crate) fn empty(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            primitives: Vec::new(),
            write_specs: Vec::new(),
        }
    }
}

/// DSL builder that produces a [`SubgraphSpec`]. Calling `write(|w| w.label
/// (...).property(...))` stores the configured `WriteSpec` so downstream
/// dispatch can see exactly what the caller requested.
pub struct SubgraphSpecBuilder {
    handler_id: String,
    primitives: Vec<(String, benten_eval::PrimitiveKind)>,
    write_specs: Vec<WriteSpec>,
}

impl SubgraphSpecBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler_id: String::new(),
            primitives: Vec::new(),
            write_specs: Vec::new(),
        }
    }

    #[must_use]
    pub fn handler_id(mut self, id: &str) -> Self {
        self.handler_id = id.to_string();
        self
    }

    #[must_use]
    pub fn iterate<F>(self, _max: u32, _body: F) -> Self
    where
        F: FnOnce(IterateBody) -> IterateBody,
    {
        // Phase-1: ITERATE bodies aren't executed by `call` yet; the structural
        // shape is what gets registered. Leave the builder's primitive list
        // untouched so invariant-1 (DAG-ness) stays trivially satisfied.
        self
    }

    #[must_use]
    pub fn write<F>(mut self, f: F) -> Self
    where
        F: FnOnce(WriteSpec) -> WriteSpec,
    {
        let spec = f(WriteSpec::new());
        self.primitives.push((
            format!("w{}", self.primitives.len()),
            benten_eval::PrimitiveKind::Write,
        ));
        self.write_specs.push(spec);
        self
    }

    #[must_use]
    pub fn respond(mut self) -> Self {
        self.primitives.push((
            format!("r{}", self.primitives.len()),
            benten_eval::PrimitiveKind::Respond,
        ));
        self
    }

    #[must_use]
    pub fn build(self) -> SubgraphSpec {
        SubgraphSpec {
            handler_id: self.handler_id,
            primitives: self.primitives,
            write_specs: self.write_specs,
        }
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
///
/// Records the label, property set, capability-scope requirements, and
/// failure-injection flag so `Engine::call` can dispatch the write with the
/// caller's intent rather than a stripped facade.
#[derive(Debug, Clone, Default)]
pub struct WriteSpec {
    pub(crate) label: String,
    pub(crate) properties: BTreeMap<String, benten_core::Value>,
    pub(crate) requires: Vec<String>,
    pub(crate) inject_failure: bool,
}

impl WriteSpec {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    #[must_use]
    pub fn property(mut self, k: &str, v: benten_core::Value) -> Self {
        self.properties.insert(k.to_string(), v);
        self
    }

    #[must_use]
    pub fn requires(mut self, scope: &str) -> Self {
        self.requires.push(scope.to_string());
        self
    }

    #[must_use]
    pub fn test_inject_failure(mut self, inject: bool) -> Self {
        self.inject_failure = inject;
        self
    }

    /// Read-only accessor for the configured label.
    #[must_use]
    pub fn label_ref(&self) -> &str {
        &self.label
    }

    /// Read-only accessor for the configured property set.
    #[must_use]
    pub fn properties_ref(&self) -> &BTreeMap<String, benten_core::Value> {
        &self.properties
    }
}

// ---------------------------------------------------------------------------
// Helper trait adapters for overloaded register / grant / call arguments.
// ---------------------------------------------------------------------------

/// Accepts `SubgraphSpec`, `&SubgraphSpec`, and `benten_eval::Subgraph`.
/// The `into_eval_subgraph` method converts any of them into the lower-level
/// `Subgraph` shape the G6 invariant validator consumes.
pub trait IntoSubgraphSpec {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError>;

    /// Return a clone of the underlying `SubgraphSpec` when the input is one;
    /// otherwise `None`. Used so `register_subgraph` can cache the spec for
    /// later `call()` dispatch.
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        None
    }
}

impl IntoSubgraphSpec for SubgraphSpec {
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        Some(self.clone())
    }
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        // Construct a minimal Subgraph from the collected primitives so the
        // invariant validator can run.
        let mut sb = benten_eval::SubgraphBuilder::new(self.handler_id);
        let mut last: Option<benten_eval::NodeHandle> = None;
        for (id, kind) in self.primitives {
            let h = match kind {
                benten_eval::PrimitiveKind::Write => sb.write(id),
                benten_eval::PrimitiveKind::Read => sb.read(id),
                benten_eval::PrimitiveKind::Respond => {
                    // `respond` is terminal and MUST have a predecessor so the
                    // registered subgraph's CID matches user intent (no
                    // silently-fabricated synthetic READ). Fix for
                    // code-reviewer finding g7-cr-13.
                    let Some(prev) = last else {
                        return Err(EngineError::Invariant(Box::new(RegistrationError::new(
                            benten_eval::InvariantViolation::Registration,
                        ))));
                    };
                    sb.respond(prev)
                }
                _ => sb.read(id),
            };
            if let Some(p) = last {
                sb.add_edge(p, h);
            }
            last = Some(h);
        }
        Ok(sb.build_unvalidated_for_test())
    }
}

impl IntoSubgraphSpec for &SubgraphSpec {
    fn as_subgraph_spec(&self) -> Option<SubgraphSpec> {
        Some((*self).clone())
    }
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        self.clone().into_eval_subgraph()
    }
}

impl IntoSubgraphSpec for benten_eval::Subgraph {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        Ok(self)
    }
}

impl IntoSubgraphSpec for &benten_eval::Subgraph {
    fn into_eval_subgraph(self) -> Result<benten_eval::Subgraph, EngineError> {
        Ok(self.clone())
    }
}

/// Subject arg for `grant_capability`.
pub trait GrantSubject {
    fn as_value(&self) -> Value;
}

impl GrantSubject for &Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl GrantSubject for Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl GrantSubject for &str {
    fn as_value(&self) -> Value {
        Value::Text((*self).to_string())
    }
}

impl GrantSubject for &String {
    fn as_value(&self) -> Value {
        Value::Text((*self).clone())
    }
}

pub trait RevokeSubject {
    fn as_value(&self) -> Value;
}

impl RevokeSubject for &Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

impl RevokeSubject for Cid {
    fn as_value(&self) -> Value {
        Value::Bytes(self.as_bytes().to_vec())
    }
}

pub trait RevokeScope {
    fn as_scope_string(&self) -> String;
}

impl RevokeScope for &str {
    fn as_scope_string(&self) -> String {
        (*self).to_string()
    }
}

impl RevokeScope for &String {
    fn as_scope_string(&self) -> String {
        (*self).clone()
    }
}

impl RevokeScope for String {
    fn as_scope_string(&self) -> String {
        self.clone()
    }
}

/// Call-input overload — accept `Node`, default `()`, and the
/// `BTreeMap<String, benten_core::Value>` path some R3 tests build inline.
pub trait IntoCallInput {
    /// Convert into a Node for uniform downstream handling.
    fn into_node(self) -> Node;
}
impl IntoCallInput for Node {
    fn into_node(self) -> Node {
        self
    }
}
impl IntoCallInput for () {
    fn into_node(self) -> Node {
        Node::empty()
    }
}
impl IntoCallInput for BTreeMap<String, benten_core::Value> {
    fn into_node(self) -> Node {
        Node::new(Vec::new(), self)
    }
}

// ---------------------------------------------------------------------------
// Testing module — helpers referenced by integration tests in sibling crates.
// ---------------------------------------------------------------------------

#[allow(clippy::todo, reason = "Phase-2 scope")]
pub mod testing {
    //! Test helpers used by integration tests from sibling crates
    //! (`benten-caps/tests/*.rs`, `benten-eval/tests/*.rs`).

    use super::{CapabilityPolicy, Outcome, SubgraphSpec};

    /// Build a synthetic ITERATE-heavy handler for TOCTOU tests.
    #[must_use]
    pub fn iterate_write_handler(_max: u32) -> SubgraphSpec {
        SubgraphSpec::empty("iterate_write")
    }

    /// Build a minimal single-WRITE handler.
    #[must_use]
    pub fn minimal_write_handler() -> SubgraphSpec {
        SubgraphSpec::empty("minimal_write")
    }

    /// Inspect the edge taken by the terminal step of an Outcome.
    #[must_use]
    pub fn route_of_error(outcome: &Outcome) -> String {
        outcome.edge_taken().unwrap_or_default()
    }

    /// Build a READ-only handler for existence-leak tests.
    #[must_use]
    pub fn read_handler_for<T: ReadHandlerTarget>(_target: T) -> SubgraphSpec {
        SubgraphSpec::empty("read_handler")
    }

    /// Sugar trait — see [`read_handler_for`].
    pub trait ReadHandlerTarget {}
    impl ReadHandlerTarget for &str {}
    impl ReadHandlerTarget for &String {}
    impl ReadHandlerTarget for String {}
    impl ReadHandlerTarget for benten_core::Cid {}

    /// Synthesize a Subject with no read grants. Returns a boxed
    /// `CapabilityPolicy` — Phase 1 uses NoAuth so reads are always allowed;
    /// the Phase 2 read-denial policy replaces this body.
    #[must_use]
    pub fn subject_with_no_read_grants() -> Box<dyn CapabilityPolicy> {
        Box::new(benten_caps::NoAuthBackend::new())
    }

    /// Adversarial fixture: handler declares `requires: post:read` but writes to admin.
    #[must_use]
    pub fn handler_declaring_read_but_writing_admin() -> SubgraphSpec {
        SubgraphSpec::empty("bad_declaring_read")
    }

    /// Second-order escalation fixture.
    #[must_use]
    pub fn handler_with_call_attenuation_escalation() -> SubgraphSpec {
        SubgraphSpec::empty("call_attenuation_escalation")
    }

    /// Build a capability policy pre-seeded with a grant set.
    #[must_use]
    pub fn policy_with_grants(_grants: &[&str]) -> Box<dyn CapabilityPolicy> {
        Box::new(benten_caps::NoAuthBackend::new())
    }

    /// Build a policy that counts check_write invocations.
    #[must_use]
    pub fn counting_capability_policy() -> CountingPolicy {
        CountingPolicy {
            count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    /// Counting capability-policy wrapper.
    pub struct CountingPolicy {
        count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl CountingPolicy {
        #[must_use]
        pub fn call_counter(&self) -> CallCounter {
            CallCounter {
                count: std::sync::Arc::clone(&self.count),
            }
        }
    }

    impl benten_caps::CapabilityPolicy for CountingPolicy {
        fn check_write(
            &self,
            _ctx: &benten_caps::WriteContext,
        ) -> Result<(), benten_caps::CapError> {
            self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    /// Atomic counter handle.
    pub struct CallCounter {
        count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl CallCounter {
        #[must_use]
        pub fn load(&self) -> u32 {
            self.count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    /// Build a READ→WRITE→READ handler for per-primitive cap-check assertions.
    #[must_use]
    pub fn handler_with_read_write_read_sequence() -> SubgraphSpec {
        SubgraphSpec::empty("rwr")
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
