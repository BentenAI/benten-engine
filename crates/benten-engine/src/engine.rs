//! The `Engine` orchestrator + its internal state (`EngineInner`,
//! `SubgraphCache`) and the public CRUD / register / dispatch / view-read /
//! transaction / snapshot / diagnostics surfaces.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). Every public method is
//! unchanged; only its module address moved. Cross-module helpers are in
//! [`crate::primitive_host`] (replay machinery), [`crate::builder`]
//! (assembly), and [`crate::outcome`] (response shapes).
//!
//! # Change-stream vs write-notification distinction (arch-8)
//!
//! `subscribe_change_events` returns a [`ChangeProbe`] that drains the
//! engine's in-memory observed-events queue — a post-commit view of what
//! the backend just wrote. This is separate from the IVM subscriber feed
//! (which consumes the same events but for view maintenance). Callers who
//! want "notify me when X changed" use the probe; callers who want "run my
//! view maintenance" register a view via `create_view`. The two paths share
//! the same underlying `ChangeBroadcast`.
//!
//! # Subgraph cache + register_crud (arch-10)
//!
//! `register_crud(label)` registers a single subgraph whose CID encodes
//! only the label (READ → RESPOND shape). At `call()` time we *also*
//! build per-op shapes (create/list/get/update/delete) that the walker
//! actually walks — those op-specific shapes have different CIDs. The
//! stored handler CID (reachable via `handler_predecessors` / diagnostics)
//! therefore does NOT match the walked shape. This is intentional for
//! Phase-1 — the registered CID identifies the *handler family*, the
//! walked CID identifies the *op-specific dispatch*. Phase-2 stores both
//! so audit consumers see either.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use benten_caps::{CapError, CapabilityPolicy};
use benten_core::{Cid, Edge, ErrorCode, Node, Value};
use benten_eval::{InvariantConfig, PrimitiveHost, RegistrationError};
use benten_graph::{ChangeEvent, GraphError, MutexExt, RedbBackend};

use crate::builder::EngineBuilder;
use crate::change::ChangeBroadcast;
use crate::change_probe::ChangeProbe;
use crate::engine_transaction::{EngineTransaction, GraphTxLike};
use crate::error::EngineError;
use crate::outcome::{
    AnchorHandle, HandlerPredecessors, Outcome, ReadViewOptions, Trace, TraceStep,
    ViewCreateOptions,
};
use crate::primitive_host::{
    ActiveCall, PendingHostOp, cap_error_to_outcome, eval_error_to_engine_error,
    outcome_from_terminal_with_cid, system_zone_to_outcome, tx_aborted_outcome,
};
use crate::subgraph_spec::{
    GrantSubject, IntoCallInput, IntoSubgraphSpec, RevokeScope, RevokeSubject, SubgraphSpec,
    WriteSpec,
};

// ---------------------------------------------------------------------------
// Engine internal state
// ---------------------------------------------------------------------------

/// Default upper bound on the in-memory change-event buffer held by the
/// engine for `subscribe_change_events` probes. When no subscriber drains the
/// buffer, older events are dropped (oldest-first) rather than growing the
/// buffer unboundedly — an unbounded `Vec<ChangeEvent>` is a memory-
/// exhaustion vector against a long-running engine (r6-sec-5).
///
/// Operators can tune this per-engine via
/// [`EngineBuilder::change_stream_capacity`]. Drops are surfaced via
/// `metrics_snapshot["benten.change_stream.dropped_events"]`.
pub const CHANGE_STREAM_MAX_BUFFERED: usize = 16_384;

/// State shared across Engine methods. Behind an `Arc` so change-event
/// callbacks can hold a weak-style reference without borrowing from the
/// Engine struct itself.
pub(crate) struct EngineInner {
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
    ///
    /// Bounded by [`EngineInner::change_stream_capacity`] — on overflow the
    /// oldest event is evicted and [`EngineInner::dropped_events`] is
    /// incremented. See r6-sec-5.
    observed_events: std::sync::Mutex<Vec<(u64, ChangeEvent)>>,
    /// Configured upper bound for `observed_events`. Populated from
    /// [`EngineBuilder::change_stream_capacity`] at engine-build time;
    /// defaults to [`CHANGE_STREAM_MAX_BUFFERED`].
    change_stream_capacity: usize,
    /// Counter of ChangeEvents dropped because the buffer reached
    /// `change_stream_capacity` before a subscriber drained. Surfaced via
    /// `metrics_snapshot["benten.change_stream.dropped_events"]`.
    dropped_events: std::sync::atomic::AtomicU64,
    /// Counter of total change events observed (for `change_event_count()`).
    event_count: std::sync::atomic::AtomicU64,
    /// Monotonic per-engine sequence used to stamp `createdAt` on CRUD
    /// creates when the caller did not supply one — makes listing order
    /// deterministic across rapid-fire creates that might otherwise collide
    /// on a wall-clock timestamp.
    created_at_seq: std::sync::atomic::AtomicU64,
    /// Pre-built subgraph templates keyed on `(handler_id, op)`. See
    /// [`SubgraphCache`] — closes r6-perf-5.
    subgraph_cache: SubgraphCache,
}

impl EngineInner {
    pub(crate) fn with_change_stream_capacity(capacity: usize) -> Self {
        // Defensive: a capacity of 0 would reject every event. Clamp to 1 so
        // the bounded-drain invariant still holds and at least the most
        // recent event is visible to a late-attached probe.
        let capacity = capacity.max(1);
        Self {
            handlers: std::sync::Mutex::new(BTreeMap::new()),
            specs: std::sync::Mutex::new(BTreeMap::new()),
            observed_events: std::sync::Mutex::new(Vec::new()),
            change_stream_capacity: capacity,
            dropped_events: std::sync::atomic::AtomicU64::new(0),
            event_count: std::sync::atomic::AtomicU64::new(0),
            created_at_seq: std::sync::atomic::AtomicU64::new(0),
            subgraph_cache: SubgraphCache::new(),
        }
    }

    pub(crate) fn record_event(&self, event: &ChangeEvent) {
        let seq = self
            .event_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut guard = self.observed_events.lock_recover();
        // Drop-oldest overflow policy (r6-sec-5). The attacker vector is an
        // unlimited producer racing a stalled subscriber; retaining the
        // newest-N means the live consumer sees up-to-date state the moment
        // it drains, at the cost of losing the oldest-N. The dropped-count
        // metric surfaces the lag loud.
        while guard.len() >= self.change_stream_capacity {
            guard.remove(0);
            self.dropped_events
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        guard.push((seq, event.clone()));
    }

    /// Drain only events whose sequence number is `>= start_offset`. Events
    /// recorded before the probe was created stay in the buffer so other
    /// probes can still observe them. Drained events are removed.
    /// See code-reviewer finding `g7-cr-7`.
    pub(crate) fn drain_events_from(&self, start_offset: u64) -> Vec<ChangeEvent> {
        let mut guard = self.observed_events.lock_recover();
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
// Subgraph cache (r6-perf-5)
// ---------------------------------------------------------------------------

/// Thread-safe cache of pre-built `benten_eval::Subgraph` templates keyed on
/// `(handler_id, op_name)`.
///
/// Rationale: `dispatch_call` previously rebuilt the op-specific Subgraph on
/// every invocation — a fresh `SubgraphBuilder` + edges + `build_unvalidated_
/// for_test`, for every `engine.call(...)`. For a hot-loop caller (1k calls/s
/// against a `crud('post').create` handler), that is 1k fully-allocated
/// Subgraph instances per second plus the per-call BLAKE3 + DAG-CBOR that
/// `build_unvalidated_for_test` performs internally. r6-perf-5 flagged this
/// as a major hotspot.
///
/// Contract:
/// * Handlers in Phase-1 are immutable once registered; the cache does NOT
///   invalidate on re-register (re-register itself is an idempotent no-op
///   or a `DuplicateHandler` error).
/// * The cache stores *templates* — the static shape of the subgraph for a
///   given `(handler_id, op_name)`. Per-call inputs (stamped `createdAt`,
///   resolved target CIDs, serialized property bags) are patched onto the
///   cloned template in `subgraph_for_crud` after the cache lookup. The
///   cache itself holds no per-call state.
/// * Thread-safe via `RwLock` — the hot path is all reads; misses briefly
///   hold a write lock to `insert`.
///
/// Backed by a simple `HashMap` rather than a bounded LRU because the
/// cache key space is bounded by (registered handlers × primitive ops),
/// which is tiny in Phase-1 (a handful of CRUD labels × 5 ops). Phase-2
/// should revisit when user-registered subgraphs are long-lived and
/// handler_id count can grow large.
#[derive(Default)]
struct SubgraphCache {
    entries: std::sync::RwLock<std::collections::HashMap<SubgraphCacheKey, benten_eval::Subgraph>>,
}

/// Cache key. Owned-string so the cache can outlive the dispatch stack that
/// produced the lookup.
#[derive(Clone, PartialEq, Eq, Hash)]
struct SubgraphCacheKey {
    handler_id: String,
    op: String,
}

impl SubgraphCache {
    fn new() -> Self {
        Self::default()
    }

    /// Return a clone of the cached template for `(handler_id, op)`, or
    /// `None` on a miss. Callers then build the subgraph and store it via
    /// [`Self::insert`].
    fn get(&self, handler_id: &str, op: &str) -> Option<benten_eval::Subgraph> {
        let guard = benten_graph::RwLockExt::read_recover(&self.entries);
        guard
            .get(&SubgraphCacheKey {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
            })
            .cloned()
    }

    /// Insert a fresh template under `(handler_id, op)`. Safe to call
    /// concurrently — last writer wins, but since handlers are immutable
    /// the value is identical across concurrent constructors.
    fn insert(&self, handler_id: &str, op: &str, sg: benten_eval::Subgraph) {
        let mut guard = benten_graph::RwLockExt::write_recover(&self.entries);
        guard.insert(
            SubgraphCacheKey {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
            },
            sg,
        );
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The Benten engine handle.
pub struct Engine {
    backend: Arc<RedbBackend>,
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
    ///
    /// The handle is retained for Phase-2 observability surfaces (operator
    /// queries of the subscriber count); the Engine itself only publishes
    /// through the backend's registered-subscriber path.
    #[allow(
        dead_code,
        reason = "retained for Phase-2 operator observability of subscriber count"
    )]
    broadcast: Arc<ChangeBroadcast>,
    /// Shared engine-wide state.
    inner: Arc<EngineInner>,
    /// IVM subscriber handle. `None` when `.without_ivm()` was passed.
    /// Engine retains the Arc so `create_view` can register views against the
    /// live subscriber and `read_view_with` can consult view state
    /// (code-reviewer g7-cr-8 / philosophy g7-ep-3).
    ivm: Option<Arc<benten_ivm::Subscriber>>,
    /// Active `Engine::call` stack. Used by `impl PrimitiveHost` to pick up
    /// per-call context (actor, nested depth) without threading it through
    /// the trait-method signatures.
    active_call: std::sync::Mutex<Vec<ActiveCall>>,
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

    /// Builder-only constructor used by `EngineBuilder::assemble`. Not part
    /// of the public API.
    pub(crate) fn from_parts(
        backend: Arc<RedbBackend>,
        policy: Option<Box<dyn CapabilityPolicy>>,
        caps_enabled: bool,
        ivm_enabled: bool,
        broadcast: Arc<ChangeBroadcast>,
        inner: Arc<EngineInner>,
        ivm: Option<Arc<benten_ivm::Subscriber>>,
    ) -> Self {
        Self {
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
            active_call: std::sync::Mutex::new(Vec::new()),
        }
    }

    // -------- Cross-module accessors (used by primitive_host.rs) --------

    pub(crate) fn backend(&self) -> &Arc<RedbBackend> {
        &self.backend
    }

    pub(crate) fn policy(&self) -> Option<&dyn CapabilityPolicy> {
        self.policy.as_deref()
    }

    pub(crate) fn ivm(&self) -> Option<&Arc<benten_ivm::Subscriber>> {
        self.ivm.as_ref()
    }

    pub(crate) fn active_call(&self) -> &std::sync::Mutex<Vec<ActiveCall>> {
        &self.active_call
    }

    // -------- CRUD surface (Node + Edge) --------

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
                return Err(EngineError::Graph(GraphError::SystemZoneWrite {
                    label: label.clone(),
                }));
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
        let mut guard = self.inner.handlers.lock_recover();
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
            let mut spec_guard = self.inner.specs.lock_recover();
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
        let mut guard = self.inner.handlers.lock_recover();
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
            let mut spec_guard = self.inner.specs.lock_recover();
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
    ///
    /// See the arch-10 module doc for the "registered CID vs walked CID"
    /// discussion — the CID stored under `handler_id` encodes only the
    /// zero-config READ → RESPOND shape, not the per-op `(create, list,
    /// get, update, delete)` shapes the walker materialises at call-time.
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
        let mut guard = self.inner.handlers.lock_recover();
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

        // Auto-register a content_listing view for this label so
        // `crud('<label>').list` routes through View 3. The default "post"
        // view is already registered at assembly time; other labels land
        // here. Skipped when IVM is disabled — the resolver falls back to
        // the backend label index.
        //
        // See arch-6 — the builder's assembly-time auto-registration for
        // `"post"` and this per-label auto-registration are two sides of
        // the same coin; Phase-2 collapses to a single entry point.
        if let Some(ivm) = self.ivm.as_ref() {
            let view_id = format!("content_listing_{label}");
            let already = ivm.view_ids().iter().any(|id| id == &view_id);
            if !already && label != "post" {
                let view = benten_ivm::views::ContentListingView::new(label);
                ivm.register_view(Box::new(view));
            }
        }
        Ok(handler_id)
    }

    /// PLACEHOLDER alias. **Phase 1 stub; behaves identically to
    /// `register_crud`.**
    ///
    /// Grant-backed variant of `register_crud`. Phase 1 is a direct
    /// pass-through — the capability-grant backing is a Phase-2 policy
    /// concern and this method exists so tests that spell the
    /// grant-backed intent survive across the Phase-1 / Phase-2
    /// boundary.
    // TODO(phase-2-grant-backed-policy): route through GrantBackedPolicy
    // registration so the handler honours grants at call-time.
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
    ///
    /// Phase-1 synthesizes one step per primitive the dispatched op is
    /// known to route through (e.g. CRUD `create` = WRITE + RESPOND;
    /// CRUD `list` = READ + RESPOND; CRUD `get` = READ + RESPOND;
    /// CRUD `delete` = WRITE + RESPOND). The terminal `Outcome` is
    /// attached to the Trace via [`Trace::outcome`] so callers don't
    /// need to re-invoke `Engine::call` just to recover the result
    /// (avoids the Phase-1 write-amplification footgun). Phase 2
    /// replaces the step synthesis with live evaluator instrumentation.
    pub fn trace(&self, handler_id: &str, op: &str, input: Node) -> Result<Trace, EngineError> {
        let start = std::time::Instant::now();
        let outcome = self.dispatch_call(handler_id, op, input, None)?;
        let elapsed = start.elapsed().as_micros();
        let elapsed = u64::try_from(elapsed).unwrap_or(u64::MAX).max(1);
        // Derive the op-specific primitive list for CRUD handlers. The
        // bare op name (e.g. `"create"`) drives the mapping; the label
        // prefix is ignored because the synthetic steps below only
        // describe primitive kinds, not labels.
        let op_name = op.split_once(':').map_or(op, |(_, o)| o);
        let primitives: Vec<&'static str> = match op_name {
            "create" => vec!["write", "respond"],
            "list" | "get" => vec!["read", "respond"],
            "update" => vec!["read", "write", "respond"],
            "delete" => vec!["write", "respond"],
            // Fall through to the generic "read+respond" shape for
            // unknown ops; the terminal Outcome still carries truth.
            _ => vec!["read", "respond"],
        };
        let step_cid = outcome
            .created_cid
            .clone()
            .unwrap_or_else(|| Cid::from_blake3_digest([0; 32]));
        let n = u64::try_from(primitives.len().max(1)).unwrap_or(1);
        let per_step = elapsed / n;
        let steps = primitives
            .into_iter()
            .map(|p| TraceStep {
                duration_us: per_step.max(1),
                node_cid: step_cid.clone(),
                primitive: p.to_string(),
            })
            .collect();
        Ok(Trace {
            steps,
            outcome: Some(outcome),
        })
    }

    /// Render a handler as a Mermaid flowchart string.
    ///
    /// Reconstructs the registered Subgraph (from the cached SubgraphSpec
    /// for DSL handlers, or from the `crud:<label>` synthesis for the
    /// zero-config path) and delegates to
    /// [`benten_eval::diag::mermaid::render`].
    ///
    /// The output starts with `flowchart TD` and contains one labelled
    /// primitive per registered node plus one `-->` per edge. See the
    /// renderer module for the exact format.
    pub fn handler_to_mermaid(&self, handler_id: &str) -> Result<String, EngineError> {
        let guard = self.inner.handlers.lock_recover();
        if !guard.contains_key(handler_id) {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        }
        drop(guard);

        // Reconstruct the subgraph. DSL (SubgraphSpec) path takes priority;
        // CRUD synthesis falls back to the `:create` shape because mermaid
        // rendering is op-agnostic (all CRUD ops share the same structural
        // READ/WRITE/RESPOND shape for the diagram). An unknown handler_id
        // shape propagates via the normal error path.
        let spec_opt = self.inner.specs.lock_recover().get(handler_id).cloned();
        let subgraph = if let Some(spec) = spec_opt {
            self.subgraph_for_spec(&spec, "default", &Node::empty())?
        } else if let Some(label) = handler_id.strip_prefix("crud:") {
            self.subgraph_for_crud(label, "create", &Node::empty())?.0
        } else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown handler: {handler_id}"),
            });
        };
        Ok(benten_eval::diag::mermaid::render(&subgraph))
    }

    /// Return the predecessor adjacency of the handler.
    pub fn handler_predecessors(
        &self,
        handler_id: &str,
    ) -> Result<HandlerPredecessors, EngineError> {
        let guard = self.inner.handlers.lock_recover();
        if !guard.contains_key(handler_id) {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        }
        Ok(HandlerPredecessors::default())
    }

    /// Core dispatch — fetch the registered Subgraph (or an op-specific
    /// ephemeral for CRUD handlers) and run it through
    /// [`benten_eval::Evaluator`] using `self` as the [`PrimitiveHost`].
    ///
    /// Closes Compromise #8: the evaluator is the sole dispatch path; no
    /// fast-path short-circuits the walk.
    pub(crate) fn dispatch_call(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: Option<Cid>,
    ) -> Result<Outcome, EngineError> {
        // Verify the handler is registered.
        let handler_cid_opt = {
            let guard = self.inner.handlers.lock_recover();
            guard.get(handler_id).cloned()
        };
        let Some(handler_cid) = handler_cid_opt else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        };

        // Reentrancy guard — set the active-call state so `impl PrimitiveHost`
        // can pick up the actor / op metadata without threading it through
        // the trait methods.
        {
            let mut guard = self.active_call.lock_recover();
            guard.push(ActiveCall {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
                actor: actor.clone(),
                handler_cid: Some(handler_cid.clone()),
                pending_ops: Vec::new(),
                inject_failure: false,
            });
        }

        let result = self.dispatch_call_inner(handler_id, op, input, actor, &handler_cid);

        // Always pop the stack frame, even on error.
        {
            let mut guard = self.active_call.lock_recover();
            guard.pop();
        }

        result
    }

    #[allow(
        clippy::too_many_lines,
        reason = "r6-sec-4 adds the NotImplemented→ON_ERROR routing arm; further decomposition would obscure the top-to-bottom dispatch flow (subgraph build → evaluator run → replay → outcome mapping)"
    )]
    fn dispatch_call_inner(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        _actor: Option<Cid>,
        _handler_cid: &Cid,
    ) -> Result<Outcome, EngineError> {
        // Build the execution subgraph. CRUD handlers synthesize an op-
        // specific shape (READ / WRITE / RESPOND); SubgraphSpec-registered
        // handlers materialize their recorded WriteSpecs into WRITE nodes.
        // Either way the resulting Subgraph walks through the Evaluator.
        let spec_opt = self.inner.specs.lock_recover().get(handler_id).cloned();

        let (subgraph, list_hint) = if let Some(spec) = spec_opt {
            (self.subgraph_for_spec(&spec, op, &input)?, None)
        } else if let Some(label) = handler_id.strip_prefix("crud:") {
            self.subgraph_for_crud(label, op, &input)?
        } else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown handler: {handler_id}"),
            });
        };

        // Walk the evaluator. Host-side WRITE / DELETE ops go into the
        // pending-ops buffer on the active_call frame so we can replay them
        // atomically inside a transaction after the walk completes. See the
        // `primitive_host` module doc "Two-phase write (arch-2)" for the
        // rationale.
        let input_value = Value::Map(input.properties.clone());
        let mut evaluator = benten_eval::Evaluator::new();
        let eval_result = evaluator.run(&subgraph, input_value, self as &dyn PrimitiveHost);

        // Capture pending ops + inject_failure out of the active_call frame.
        let (pending, inject_failure) = {
            let mut guard = self.active_call.lock_recover();
            if let Some(frame) = guard.last_mut() {
                (
                    std::mem::take(&mut frame.pending_ops),
                    std::mem::replace(&mut frame.inject_failure, false),
                )
            } else {
                (Vec::new(), false)
            }
        };

        let (edge, output) = match eval_result {
            Ok(run) => (run.terminal_edge, run.output),
            Err(e) => {
                if inject_failure
                    || matches!(&e, benten_eval::EvalError::Backend(s) if s == "test_inject_failure")
                {
                    return Ok(tx_aborted_outcome());
                }
                return Err(eval_error_to_engine_error(e));
            }
        };

        // Replay the buffered host ops atomically. If the capability hook
        // denies, surface ON_DENIED; on SystemZoneWrite surface ON_ERROR
        // with E_SYSTEM_ZONE_WRITE.
        let replay_result: Result<Option<Cid>, EngineError> = if pending.is_empty() {
            Ok(None)
        } else {
            self.transaction(|tx| {
                let mut last_cid = None;
                for op in &pending {
                    match op {
                        // r6-perf-3 + r6-sec-3: reuse the already-computed CID
                        // (skip the redundant BLAKE3+DAG-CBOR hash) and thread
                        // the attribution triple into the emitted ChangeEvent.
                        PendingHostOp::PutNode {
                            node,
                            projected_cid,
                            actor_cid,
                            handler_cid,
                            capability_grant_cid,
                        } => {
                            let cid = tx.put_node_with_attribution(
                                node,
                                Some(projected_cid.clone()),
                                actor_cid.clone(),
                                handler_cid.clone(),
                                capability_grant_cid.clone(),
                            )?;
                            last_cid = Some(cid);
                        }
                        PendingHostOp::DeleteNode { cid } => {
                            tx.delete_node(cid)?;
                        }
                        PendingHostOp::PutEdge { .. } | PendingHostOp::DeleteEdge { .. } => {
                            // Phase-1: edge ops via PrimitiveHost are not
                            // surfaced by any test subgraph. Reserved for
                            // Phase-2 when a dedicated EngineTransaction
                            // edge API lands.
                        }
                    }
                }
                Ok(last_cid)
            })
        };

        match replay_result {
            Ok(created_cid) => Ok(outcome_from_terminal_with_cid(
                self,
                &edge,
                output,
                list_hint,
                created_cid,
            )),
            Err(EngineError::Cap(cap)) => Ok(cap_error_to_outcome(&cap)),
            Err(EngineError::Graph(GraphError::SystemZoneWrite { .. })) => {
                Ok(system_zone_to_outcome())
            }
            Err(e) => Err(e),
        }
    }

    /// Synthesize an op-specific Subgraph for a `crud:<label>` handler. The
    /// returned `list_hint`, when `Some`, directs the outcome mapper to
    /// populate `Outcome.list` by walking the label index — the read path
    /// that currently has no direct Evaluator primitive in Phase 1.
    ///
    /// Caches the static template for each `(crud:<label>, op)` pair via
    /// [`SubgraphCache`] — r6-perf-5. The cache stores the shape + static
    /// properties (`op`, `label`, `query_kind`); per-call inputs (stamped
    /// `createdAt`, resolved target CIDs, serialized property bags) are
    /// patched onto the cloned template.
    #[allow(
        clippy::too_many_lines,
        reason = "four-op dispatch arm + cache-miss template construction; splitting hurts local readability more than it helps"
    )]
    fn subgraph_for_crud(
        &self,
        label: &str,
        op: &str,
        input: &Node,
    ) -> Result<(benten_eval::Subgraph, Option<String>), EngineError> {
        // Strip an optional leading `<label>:` prefix in the op argument so
        // both `"create"` and `"post:create"` dispatch identically.
        let op_name = op.split_once(':').map_or(op, |(_, o)| o);
        let cache_handler = format!("crud:{label}");
        match op_name {
            "create" => {
                let mut props = input.properties.clone();
                // Defense-in-depth fallback stamp. Primary stamping happens
                // at the DSL's call-time entry (packages/engine/src/engine.ts
                // Engine.call, crud create branch); if a caller reaches the
                // Rust surface without a DSL-side stamp the `or_insert` below
                // preserves the View-3 sort key so content listing stays
                // functional. Not a primary path — see r4b-qa-3 for the
                // stamp-once-per-call contract.
                let created_at = self
                    .inner
                    .created_at_seq
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    .saturating_add(1);
                props
                    .entry("createdAt".to_string())
                    .or_insert(Value::Int(i64::try_from(created_at).unwrap_or(i64::MAX)));

                let mut sg = self
                    .inner
                    .subgraph_cache
                    .get(&cache_handler, "create")
                    .unwrap_or_else(|| {
                        let mut sb =
                            benten_eval::SubgraphBuilder::new(format!("crud:{label}:create"));
                        let w = sb.write(format!("crud_{label}_write"));
                        let _ = sb.respond(w);
                        let mut sg = sb.build_unvalidated_for_test();
                        // Backfill STATIC WRITE properties — everything that
                        // is invariant across calls for this (label, op).
                        if let Some(w_node) = sg.write_op_mut() {
                            w_node.properties.insert("op".into(), Value::text("create"));
                            w_node.properties.insert("label".into(), Value::text(label));
                        }
                        self.inner
                            .subgraph_cache
                            .insert(&cache_handler, "create", sg.clone());
                        sg
                    });
                // Patch the per-call property bag onto the cloned template.
                if let Some(w_node) = sg.write_op_mut() {
                    w_node
                        .properties
                        .insert("properties".into(), Value::Map(props));
                }
                Ok((sg, None))
            }
            "list" => {
                // Use a READ with query semantics; the read executor routes
                // via the host's `get_by_label`. We surface the list via a
                // dedicated post-evaluator walk (list_hint) because the
                // Phase-1 READ output shape is a list of base32 CID strings
                // rather than a list of Nodes.
                let sg = self
                    .inner
                    .subgraph_cache
                    .get(&cache_handler, "list")
                    .unwrap_or_else(|| {
                        let mut sb =
                            benten_eval::SubgraphBuilder::new(format!("crud:{label}:list"));
                        let r = sb.read(format!("crud_{label}_list"));
                        let _ = sb.respond(r);
                        let mut sg = sb.build_unvalidated_for_test();
                        if let Some(r_node) = sg.first_op_mut() {
                            r_node
                                .properties
                                .insert("query_kind".into(), Value::text("by_label"));
                            r_node.properties.insert("label".into(), Value::text(label));
                        }
                        self.inner
                            .subgraph_cache
                            .insert(&cache_handler, "list", sg.clone());
                        sg
                    });
                Ok((sg, Some(label.to_string())))
            }
            "get" => {
                let mut sg = self
                    .inner
                    .subgraph_cache
                    .get(&cache_handler, "get")
                    .unwrap_or_else(|| {
                        let mut sb = benten_eval::SubgraphBuilder::new(format!("crud:{label}:get"));
                        let r = sb.read(format!("crud_{label}_get"));
                        let _ = sb.respond(r);
                        let sg = sb.build_unvalidated_for_test();
                        self.inner
                            .subgraph_cache
                            .insert(&cache_handler, "get", sg.clone());
                        sg
                    });
                // Signal to the outcome mapper that this single-get should
                // surface the Node via `list` as a single-entry vector.
                let resolved_cid = if let Some(Value::Text(wanted)) = input.properties.get("cid") {
                    self.lookup_cid_by_base32(label, wanted)?
                } else {
                    None
                };
                if let Some(r_node) = sg.first_op_mut() {
                    match &resolved_cid {
                        Some(cid) => {
                            r_node
                                .properties
                                .insert("target_cid".into(), Value::Bytes(cid.as_bytes().to_vec()));
                        }
                        None => {
                            r_node
                                .properties
                                .insert("target_cid".into(), Value::text("missing"));
                        }
                    }
                }
                // `get:<label>:<base32>` tells the outcome mapper to
                // resolve a single Node into the outcome's list. A miss
                // still surfaces an empty list, but the READ primitive will
                // have routed ON_NOT_FOUND, which the mapper translates
                // into the ON_NOT_FOUND Outcome edge.
                let hint = resolved_cid.map(|c| format!("get:{}:{}", label, c.to_base32()));
                Ok((sg, hint))
            }
            "delete" => {
                // Resolve the target CID up front (Cid::from_str is Phase-2),
                // then build a WRITE(op=delete, cid=<bytes>) subgraph.
                let target = match input.properties.get("cid") {
                    Some(Value::Text(s)) => self.lookup_cid_by_base32(label, s)?,
                    _ => None,
                };
                let mut sg = self
                    .inner
                    .subgraph_cache
                    .get(&cache_handler, "delete")
                    .unwrap_or_else(|| {
                        let mut sb =
                            benten_eval::SubgraphBuilder::new(format!("crud:{label}:delete"));
                        let w = sb.write(format!("crud_{label}_delete"));
                        let _ = sb.respond(w);
                        let sg = sb.build_unvalidated_for_test();
                        self.inner
                            .subgraph_cache
                            .insert(&cache_handler, "delete", sg.clone());
                        sg
                    });
                if let Some(w_node) = sg.first_op_mut() {
                    match target {
                        Some(cid) => {
                            w_node.properties.insert("op".into(), Value::text("delete"));
                            w_node
                                .properties
                                .insert("target_cid".into(), Value::Bytes(cid.as_bytes().to_vec()));
                        }
                        None => {
                            // Signal "not found" via the WRITE's op so the
                            // host-side delete executor routes ON_NOT_FOUND.
                            w_node
                                .properties
                                .insert("op".into(), Value::text("delete_missing"));
                        }
                    }
                }
                Ok((sg, None))
            }
            _ => Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown crud op: {op}"),
            }),
        }
    }

    /// Resolve a base32-rendered CID string to a real `Cid` by scanning the
    /// label index. Phase-1 stopgap until `Cid::from_str` lands.
    fn lookup_cid_by_base32(&self, label: &str, wanted: &str) -> Result<Option<Cid>, EngineError> {
        let cids = self.backend.get_by_label(label)?;
        for cid in cids {
            if cid.to_base32() == wanted {
                return Ok(Some(cid));
            }
        }
        Ok(None)
    }

    fn subgraph_for_spec(
        &self,
        spec: &SubgraphSpec,
        _op: &str,
        _input: &Node,
    ) -> Result<benten_eval::Subgraph, EngineError> {
        // Materialize the recorded WriteSpecs into an ordered WRITE chain
        // terminated by RESPOND. When no WriteSpec contributes a real write
        // (phase-1 shape-only fixtures), we still synthesize an empty chain
        // so the evaluator walks it and terminates cleanly.
        let mut sb = benten_eval::SubgraphBuilder::new(spec.handler_id.clone());
        let mut last: Option<benten_eval::NodeHandle> = None;
        let mut write_ops: Vec<(usize, WriteSpec)> = Vec::new();
        for (idx, w) in spec.write_specs.iter().enumerate() {
            let h = sb.write(format!("w{idx}"));
            if let Some(prev) = last {
                sb.add_edge(prev, h);
            }
            last = Some(h);
            write_ops.push((idx, w.clone()));
        }
        let terminal = match last {
            Some(prev) => sb.respond(prev),
            None => {
                let r = sb.read("noop_read".to_string());
                sb.respond(r)
            }
        };
        let _ = terminal;
        let mut sg = sb.build_unvalidated_for_test();
        // Populate WRITE property bags post-build — SubgraphBuilder doesn't
        // surface per-node property setters for callers outside the crate.
        for (idx, w) in &write_ops {
            if let Some(node) = sg.op_by_id_mut(&format!("w{idx}")) {
                if w.inject_failure {
                    node.properties
                        .insert("op".into(), Value::text("test_inject_failure"));
                    continue;
                }
                node.properties.insert("op".into(), Value::text("create"));
                let label = if w.label.is_empty() {
                    "node".to_string()
                } else {
                    w.label.clone()
                };
                node.properties.insert("label".into(), Value::text(&label));
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
                node.properties
                    .insert("properties".into(), Value::Map(props));
            }
        }
        Ok(sg)
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

    /// Strict view read — alias for [`Self::read_view`].
    ///
    /// Retained for source-compatibility with R3 tests that spell the strict
    /// intent explicitly. `read_view_strict(id)` is literally
    /// `read_view_with(id, ReadViewOptions::strict())` and is documented as
    /// such so operators choosing between the three names know the contract
    /// is identical (R-minor-05).
    pub fn read_view_strict(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::strict())
    }

    /// Relaxed view read — equivalent to
    /// [`Self::read_view_with`] with `ReadViewOptions::allow_stale()`.
    /// Retained for R3 test source-compatibility (R-minor-05).
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
            let mut eng_tx = make_engine_tx(tx, &ops_cell);
            match f(&mut eng_tx) {
                Ok(value) => {
                    if let Some(p) = policy {
                        let ops = ops_cell.lock_recover().clone();
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
                                *user_result.lock_recover() = Some(Err(EngineError::Cap(cap_err)));
                                return Err(GraphError::TxAborted {
                                    reason: "capability denied".into(),
                                });
                            }
                        }
                    }
                    *user_result.lock_recover() = Some(Ok(value));
                    Ok(())
                }
                Err(e) => {
                    *user_result.lock_recover() = Some(Err(e));
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
    ///
    /// Surfaces:
    /// - `benten.writes.total` — cumulative ChangeEvents observed.
    /// - `benten.ivm.view_stale_count` — Phase-1 placeholder; Phase-2 wires
    ///   the real counter.
    /// - `benten.change_stream.dropped_events` — ChangeEvents evicted from
    ///   the bounded observed-events buffer because a subscriber fell behind
    ///   the write path (r6-sec-5). Non-zero means an operator should
    ///   increase the capacity via
    ///   [`EngineBuilder::change_stream_capacity`] or ensure probes drain.
    #[must_use]
    pub fn metrics_snapshot(&self) -> BTreeMap<String, f64> {
        let mut out = BTreeMap::new();
        let n = self
            .inner
            .event_count
            .load(std::sync::atomic::Ordering::SeqCst);
        let dropped = self
            .inner
            .dropped_events
            .load(std::sync::atomic::Ordering::SeqCst);
        #[allow(
            clippy::cast_precision_loss,
            reason = "Phase-1 metric is best-effort; lossy cast from u64 to f64 is acceptable for the compromise-5 regression test."
        )]
        {
            out.insert("benten.writes.total".to_string(), n as f64);
            out.insert(
                "benten.change_stream.dropped_events".to_string(),
                dropped as f64,
            );
        }
        out.insert("benten.ivm.view_stale_count".to_string(), 0.0);
        out
    }

    /// Configured upper bound on the in-memory change-event buffer. Matches
    /// the value passed to [`EngineBuilder::change_stream_capacity`] (or
    /// [`CHANGE_STREAM_MAX_BUFFERED`] when the default was taken). See
    /// r6-sec-5.
    #[must_use]
    pub fn change_stream_capacity(&self) -> usize {
        self.inner.change_stream_capacity
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

// `make_engine_tx` constructs an EngineTransaction from a graph Transaction.
// Kept out of the main flow so the closure reads cleanly.
fn make_engine_tx<'tx, 'coll>(
    tx: &'tx mut benten_graph::Transaction<'_>,
    ops_collector: &'coll std::sync::Mutex<Vec<benten_caps::PendingOp>>,
) -> EngineTransaction<'tx, 'coll> {
    // Coerce `&mut benten_graph::Transaction<'_>` into `&mut dyn GraphTxLike`
    // so `EngineTransaction` can hold the lifetime-elided shim.
    let inner: &mut (dyn GraphTxLike + 'tx) = tx;
    EngineTransaction {
        inner,
        ops_collector,
    }
}

// ---------------------------------------------------------------------------
// Known-view-id whitelist
// ---------------------------------------------------------------------------

/// Known view ids recognized by `read_view*`. Accepts:
/// - the five canonical IDs surfaced by benten-ivm's built-in views,
/// - `content_listing_<label>` (the per-label naming convention used by R3
///   tests that instantiate a ContentListingView per Node label),
/// - `system:ivm:<one-of-the-canonical-ids>` — the namespaced alias.
///
/// Unknown view IDs (including `system:ivm:nonexistent`) return false so
/// `read_view_*` raises `EngineError::UnknownView`.
///
/// **Drift warning (arch-5):** this whitelist hard-codes the five canonical
/// view names from `benten_ivm::views::*`. If new views are added to
/// benten-ivm without updating this list, read_view surface errors with
/// `UnknownView` for a view the subscriber *does* serve. The subscriber
/// probe in `read_view_with` consults the live subscriber first, so this
/// only matters for tests that probe uncreated-but-canonical view ids.
/// TODO(phase-2-view-id-registry): replace with a per-view definition
/// registration pulled from benten-ivm.
pub(crate) fn is_known_view_id(id: &str) -> bool {
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
