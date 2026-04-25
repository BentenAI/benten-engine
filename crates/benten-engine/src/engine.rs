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
use benten_core::{Cid, Edge, Node, Value};
use benten_errors::ErrorCode;
use benten_eval::{InvariantConfig, PrimitiveHost, RegistrationError};
use benten_graph::{ChangeEvent, GraphError, MutexExt, RedbBackend};

use crate::builder::EngineBuilder;
use crate::change::ChangeBroadcast;
use crate::change_probe::ChangeProbe;
use crate::engine_transaction::{EngineTransaction, GraphTxLike};
use crate::error::EngineError;
use crate::outcome::{
    AnchorHandle, DiagnosticInfo, HandlerPredecessors, Outcome, ReadViewOptions, Trace, TraceStep,
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
    pub(crate) handlers: std::sync::Mutex<BTreeMap<String, Cid>>,
    /// Registered SubgraphSpec bodies keyed by handler id — so `call()` can
    /// walk the WriteSpec list when the user registered a SubgraphSpec
    /// (as opposed to `register_crud` which is dispatched directly by op
    /// name).
    pub(crate) specs: std::sync::Mutex<BTreeMap<String, SubgraphSpec>>,
    /// Observed ChangeEvents (post-commit). Populated by the
    /// `ChangeBroadcast` subscriber; drained by
    /// `engine.subscribe_change_events().drain()`.
    ///
    /// Bounded by [`EngineInner::change_stream_capacity`] — on overflow the
    /// oldest event is evicted and [`EngineInner::dropped_events`] is
    /// incremented. See r6-sec-5.
    pub(crate) observed_events: std::sync::Mutex<Vec<(u64, ChangeEvent)>>,
    /// Configured upper bound for `observed_events`. Populated from
    /// [`EngineBuilder::change_stream_capacity`] at engine-build time;
    /// defaults to [`CHANGE_STREAM_MAX_BUFFERED`].
    pub(crate) change_stream_capacity: usize,
    /// Counter of ChangeEvents dropped because the buffer reached
    /// `change_stream_capacity` before a subscriber drained. Surfaced via
    /// `metrics_snapshot["benten.change_stream.dropped_events"]`.
    pub(crate) dropped_events: std::sync::atomic::AtomicU64,
    /// Counter of total change events observed (for `change_event_count()`).
    pub(crate) event_count: std::sync::atomic::AtomicU64,
    /// Monotonic per-engine sequence used to stamp `createdAt` on CRUD
    /// creates when the caller did not supply one — makes listing order
    /// deterministic across rapid-fire creates that might otherwise collide
    /// on a wall-clock timestamp.
    pub(crate) created_at_seq: std::sync::atomic::AtomicU64,
    /// Pre-built subgraph templates keyed on `(handler_id, op, subgraph_cid)`.
    /// Phase 2a G2-B / arch-r1-5.
    pub(crate) subgraph_cache: SubgraphCache,
    /// Phase 2a G2-B / dx-r1: count of subgraph template builds (cache misses).
    pub(crate) parse_counter: std::sync::atomic::AtomicU64,
    /// Per-capability-scope tally of writes that passed the policy's
    /// `check_write` gate (i.e. committed). Keyed by the derived scope
    /// string (`store:<label>:write`). Closes named compromise #5 — the
    /// Phase-1 posture is "record, don't enforce"; Phase-2 adds the
    /// rate-limit enforcement pass on top of these counters.
    pub(crate) cap_write_committed: std::sync::Mutex<BTreeMap<String, u64>>,
    /// Per-capability-scope tally of writes the policy DENIED. Scope keys
    /// match the committed side; incremented when `check_write` returns
    /// `Err(CapError::Denied)` / `Err(CapError::DeniedRead)` so operators
    /// can spot abnormal denial patterns out-of-band.
    pub(crate) cap_write_denied: std::sync::Mutex<BTreeMap<String, u64>>,
    /// Aggregate count of capability-policy `check_write` calls that
    /// returned `Ok`. Surfaced via `metrics_snapshot["benten.writes.committed"]`.
    pub(crate) writes_committed_total: std::sync::atomic::AtomicU64,
    /// Aggregate count of capability-policy `check_write` calls that
    /// returned `Err`. Surfaced via `metrics_snapshot["benten.writes.denied"]`.
    pub(crate) writes_denied_total: std::sync::atomic::AtomicU64,
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
            parse_counter: std::sync::atomic::AtomicU64::new(0),
            cap_write_committed: std::sync::Mutex::new(BTreeMap::new()),
            cap_write_denied: std::sync::Mutex::new(BTreeMap::new()),
            writes_committed_total: std::sync::atomic::AtomicU64::new(0),
            writes_denied_total: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Increment the per-scope + aggregate committed-write counters once per
    /// transaction commit. `scopes` is the deduplicated list of
    /// `store:<label>:write` scopes the batch exercises; the aggregate
    /// `writes_committed_total` bumps by exactly 1 per commit regardless of
    /// scope-fan-out so the metric counts commits, not ops.
    pub(crate) fn record_cap_write_committed(&self, scopes: &[String]) {
        self.writes_committed_total
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut guard = self.cap_write_committed.lock_recover();
        for scope in scopes {
            let slot = guard.entry(scope.clone()).or_insert(0);
            *slot = slot.saturating_add(1);
        }
    }

    /// Increment the per-scope + aggregate denied-write counters when the
    /// capability policy rejects a batch. `scopes` is best-effort — an
    /// unstructured `WriteContext` may surface with an empty scope list, in
    /// which case only the aggregate tally moves.
    pub(crate) fn record_cap_write_denied(&self, scopes: &[String]) {
        self.writes_denied_total
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut guard = self.cap_write_denied.lock_recover();
        for scope in scopes {
            let slot = guard.entry(scope.clone()).or_insert(0);
            *slot = slot.saturating_add(1);
        }
    }

    /// Snapshot the per-scope committed-writes map. Returned clone so the
    /// caller can inspect without holding the Engine's lock.
    pub(crate) fn cap_write_committed_snapshot(&self) -> BTreeMap<String, u64> {
        self.cap_write_committed.lock_recover().clone()
    }

    /// Snapshot the per-scope denied-writes map.
    pub(crate) fn cap_write_denied_snapshot(&self) -> BTreeMap<String, u64> {
        self.cap_write_denied.lock_recover().clone()
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
pub(crate) struct SubgraphCache {
    entries: std::sync::RwLock<std::collections::HashMap<SubgraphCacheKey, benten_eval::Subgraph>>,
}

/// Cache key — Phase 2a G2-B / arch-r1-5: three-axis
/// `(handler_id, op, subgraph_cid)`.
#[derive(Clone, PartialEq, Eq, Hash)]
struct SubgraphCacheKey {
    handler_id: String,
    op: String,
    subgraph_cid: Cid,
}

impl SubgraphCache {
    fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get(
        &self,
        handler_id: &str,
        op: &str,
        cid: &Cid,
    ) -> Option<benten_eval::Subgraph> {
        let guard = benten_graph::RwLockExt::read_recover(&self.entries);
        guard
            .get(&SubgraphCacheKey {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
                subgraph_cid: *cid,
            })
            .cloned()
    }

    pub(crate) fn insert(&self, handler_id: &str, op: &str, cid: &Cid, sg: benten_eval::Subgraph) {
        let mut guard = benten_graph::RwLockExt::write_recover(&self.entries);
        guard.insert(
            SubgraphCacheKey {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
                subgraph_cid: *cid,
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
    pub(crate) backend: Arc<RedbBackend>,
    /// Configured capability policy. `None` collapses to
    /// `NoAuthBackend`-equivalent behavior (every commit permitted).
    pub(crate) policy: Option<Box<dyn CapabilityPolicy>>,
    /// True if `.without_caps()` was passed to the builder. Controls whether
    /// `grant_capability` / `revoke_capability` refuse honestly rather than
    /// silently no-op.
    pub(crate) caps_enabled: bool,
    /// True if `.without_ivm()` was NOT passed. Controls whether the
    /// subscriber is wired and whether view reads can succeed.
    pub(crate) ivm_enabled: bool,
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
    pub(crate) broadcast: Arc<ChangeBroadcast>,
    /// Shared engine-wide state.
    pub(crate) inner: Arc<EngineInner>,
    /// IVM subscriber handle. `None` when `.without_ivm()` was passed.
    /// Engine retains the Arc so `create_view` can register views against the
    /// live subscriber and `read_view_with` can consult view state
    /// (code-reviewer g7-cr-8 / philosophy g7-ep-3).
    pub(crate) ivm: Option<Arc<benten_ivm::Subscriber>>,
    /// Active `Engine::call` stack. Used by `impl PrimitiveHost` to pick up
    /// per-call context (actor, nested depth) without threading it through
    /// the trait-method signatures.
    pub(crate) active_call: std::sync::Mutex<Vec<ActiveCall>>,
    /// Phase 2a G9-A-cont: configured monotonic clock source. Drives
    /// TOCTOU wall-clock-refresh cadence inside `impl PrimitiveHost`
    /// (§9.13 refresh point #3). Always `Some` post-build —
    /// [`crate::builder::EngineBuilder::build`] installs
    /// [`benten_eval::InstantMonotonicSource`] when the caller didn't
    /// inject a mock. `Arc<dyn _>` rather than `Box<dyn _>` so tests can
    /// retain a clone of the handle to drive advances.
    pub(crate) monotonic_source: Arc<dyn benten_eval::MonotonicSource>,
    /// Phase 2a G9-A-cont: configured HLC / wall-clock source. NEVER used
    /// to drive TOCTOU cadence — rides alongside for federation-
    /// correlation stamping only.
    pub(crate) time_source: Arc<dyn benten_eval::TimeSource>,
    /// Phase 2a G9-A-cont: target-iteration revocation schedule. Populated
    /// by [`Engine::schedule_revocation_at_iteration`]; consulted by
    /// `impl PrimitiveHost` at iterate-batch boundaries so a test can
    /// assert that a cap revoked mid-walk fires
    /// `E_CAP_REVOKED_MID_EVAL`. Keyed by `grant_cid`; value is the
    /// target iteration number. Empty in production — the real
    /// revocation path runs through
    /// `benten-caps::GrantBackedPolicy::check_write` via a
    /// `system:CapabilityRevocation` Node write.
    pub(crate) revoke_at_iteration: std::sync::Mutex<BTreeMap<Cid, u64>>,
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
    ///
    /// Delegates to [`Self::from_parts_with_clocks`] with the Phase-1
    /// default clocks (`InstantMonotonicSource` + `HlcTimeSource`). Kept
    /// as a convenience for older call sites that predate the G9-A-cont
    /// clock injection; new call sites should use
    /// [`Self::from_parts_with_clocks`].
    #[allow(
        dead_code,
        reason = "retained for symmetry; all live call sites now use from_parts_with_clocks"
    )]
    pub(crate) fn from_parts(
        backend: Arc<RedbBackend>,
        policy: Option<Box<dyn CapabilityPolicy>>,
        caps_enabled: bool,
        ivm_enabled: bool,
        broadcast: Arc<ChangeBroadcast>,
        inner: Arc<EngineInner>,
        ivm: Option<Arc<benten_ivm::Subscriber>>,
    ) -> Self {
        Self::from_parts_with_clocks(
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
            Arc::new(benten_eval::InstantMonotonicSource::new()),
            Arc::new(benten_eval::HlcTimeSource::new()),
        )
    }

    /// Builder-only constructor used by `EngineBuilder::assemble` (Phase
    /// 2a G9-A-cont variant). Threads the clock sources onto the Engine
    /// struct so `impl PrimitiveHost` can consult them at refresh-point
    /// #3 without additional argument threading.
    #[allow(clippy::too_many_arguments, reason = "builder plumbing")]
    pub(crate) fn from_parts_with_clocks(
        backend: Arc<RedbBackend>,
        policy: Option<Box<dyn CapabilityPolicy>>,
        caps_enabled: bool,
        ivm_enabled: bool,
        broadcast: Arc<ChangeBroadcast>,
        inner: Arc<EngineInner>,
        ivm: Option<Arc<benten_ivm::Subscriber>>,
        monotonic_source: Arc<dyn benten_eval::MonotonicSource>,
        time_source: Arc<dyn benten_eval::TimeSource>,
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
            monotonic_source,
            time_source,
            revoke_at_iteration: std::sync::Mutex::new(BTreeMap::new()),
        }
    }

    /// Phase 2a G9-A-cont: accessor for the configured monotonic clock
    /// source. `impl PrimitiveHost::check_capability` consults this at
    /// refresh-point-#3 so a drift-resilient TOCTOU refresh cadence runs
    /// regardless of wall-clock jumps.
    #[must_use]
    pub fn monotonic_source(&self) -> &Arc<dyn benten_eval::MonotonicSource> {
        &self.monotonic_source
    }

    /// Phase 2a G9-A-cont: accessor for the configured HLC / wall-clock
    /// source. Rides alongside `monotonic_source` for federation-
    /// correlation stamping; never primary for cadence.
    #[must_use]
    pub fn time_source(&self) -> &Arc<dyn benten_eval::TimeSource> {
        &self.time_source
    }

    // -------- Cross-module accessors (used by primitive_host.rs) --------

    pub(crate) fn backend(&self) -> &Arc<RedbBackend> {
        &self.backend
    }

    /// Phase 2a G5-B-i test-only backend accessor.
    ///
    /// The user-facing [`Engine::get_node`] now collapses system-zone
    /// reads to `None` under the Inv-11 runtime probe. Tests that need
    /// to assert an engine-privileged write actually landed (e.g.
    /// `grant_capability_only_via_engine_api`) reach through this
    /// accessor so the privileged back-channel is explicit.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn backend_for_test(&self) -> &Arc<RedbBackend> {
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

    /// Phase 2a G5-A / G11-A: monotonic per-engine audit sequence.
    ///
    /// Returns the current value of the storage-layer commit counter —
    /// the number of times `RedbBackend::put_node_with_context` produced
    /// a real commit. §9.11 row 3 names this as the observable sequence
    /// the dedup path MUST NOT advance: re-putting identical bytes is a
    /// pure-read and must leave this counter alone.
    ///
    /// Wave-1 mini-review SEVERE-2 fix: previously this accessor read
    /// the engine-level `writes_committed_total` counter, which was
    /// only bumped by `Engine::transaction`'s capability-policy commit
    /// path. The privileged `grant_capability → privileged_put_node`
    /// route goes direct to the backend and bypasses that counter,
    /// making the `inv_13_dedup_path_does_not_advance_audit_sequence`
    /// assertion vacuous (0 == 0 before and after the grant). Pulling
    /// the counter from the storage layer closes the gap — the counter
    /// advances on every genuine first-put, whether that put originates
    /// from a user-authority transaction or an engine-privileged grant,
    /// and stays put on the dedup early-return branch.
    ///
    /// Surfaced publicly (not cfg-gated behind `test-helpers`) so the
    /// graph crate's `inv_13_dedup_path_does_not_advance_audit_sequence`
    /// test can observe the counter across the engine boundary. The
    /// accessor reads an `AtomicU64::SeqCst` load — safe to call from
    /// any thread, cost is a single atomic load per invocation.
    #[must_use]
    pub fn audit_sequence(&self) -> u64 {
        self.backend.writes_committed()
    }

    /// Phase 2a G11-A Wave 1 test-only alias spelled the way the R3 test
    /// suite (`inv_8_11_13_14_firing.rs`) names the counter — the
    /// integration tests imported the `testing_` prefix by convention
    /// before the public `audit_sequence` name was finalised. Kept as a
    /// thin alias rather than duplicated logic; if the test suite
    /// migrates to the public accessor, this alias can be deleted
    /// without touching any behaviour.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_audit_sequence(&self) -> u64 {
        self.audit_sequence()
    }

    // -------- CRUD surface (Node + Edge) --------
    //
    // CRUD methods (`create_node`, `get_node`, `update_node`, `delete_node`,
    // `create_edge`, `get_edge`, `delete_edge`, `edges_from`, `edges_to`) live
    // in [`crate::engine_crud`].

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
        // 5d-J workstream 3: parse every TRANSFORM node's expression at
        // registration time so an unparseable grammar trips `register_*`
        // rather than surviving to `engine.call`. The runtime executor
        // still re-parses per-call (Phase-2 completes the AST-cache
        // perf pass); this is the fail-fast guarantee only.
        benten_eval::invariants::validate_transform_expressions(&sg).map_err(|e| {
            EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            }
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
        // 5d-J workstream 3: same registration-time TRANSFORM parse as
        // `register_subgraph`. Aggregate mode collects structural
        // invariants (1/2/3/5/6/9/10/12); TRANSFORM syntax is a
        // separate hazard class and always fail-fast.
        benten_eval::invariants::validate_transform_expressions(&sg).map_err(|e| {
            EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            }
        })?;
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
    /// The stored handler is a minimal `READ → RESPOND` subgraph whose CID
    /// identifies the *handler family*. At [`Engine::call`] time the
    /// dispatcher synthesises a per-op Subgraph (five arms:
    /// `<label>:create`, `<label>:get`, `<label>:list`, `<label>:update`,
    /// `<label>:delete`) and walks it end-to-end through
    /// [`benten_eval::Evaluator::run_with_trace`] with `self as &dyn
    /// PrimitiveHost` as the backend/capability surface. Compromise #8
    /// (CRUD fast-path bypass) is CLOSED — `Engine::call` is the sole
    /// dispatch path and no handler arm short-circuits the evaluator.
    ///
    /// See the arch-10 module doc for the "registered CID vs walked CID"
    /// distinction — the CID stored under `handler_id` encodes only the
    /// zero-config READ → RESPOND shape, not the per-op CRUD shapes the
    /// walker materialises at call-time.
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
    /// Dispatch walks the handler's Subgraph end-to-end through
    /// [`benten_eval::Evaluator::run_with_trace`] with the Engine itself
    /// acting as the [`PrimitiveHost`]. CRUD handlers synthesise a per-op
    /// shape (`<label>:{create,get,list,update,delete}`); SubgraphSpec
    /// handlers walk their recorded primitive list. Buffered host-side
    /// WRITE / DELETE ops from the walk are replayed atomically inside a
    /// single transaction so the capability hook, change-event emission,
    /// IVM update, and system-zone guard all fire at one commit boundary;
    /// any WRITE flagged with `test_inject_failure(true)` surfaces
    /// `E_TX_ABORTED`.
    ///
    /// Closes Compromise #8: the evaluator is the sole dispatch path. No
    /// handler arm short-circuits the walker.
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
        self.dispatch_call(handler_id, op, input, Some(*actor))
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
        self.dispatch_call(handler_id, op, input, Some(*actor))
    }

    /// Return a per-step trace of the evaluation.
    ///
    /// Delegates to [`benten_eval::Evaluator::run_with_trace`], so every
    /// returned step is a real per-primitive record with a distinct
    /// `node_cid` (derived from the handler-scoped OperationNode id, not
    /// from the outcome's created CID) and a per-step duration sampled
    /// around the primitive executor. The terminal `Outcome` is attached
    /// to the Trace via [`Trace::outcome`] so callers don't need to
    /// re-invoke `Engine::call` just to recover the result (avoids the
    /// write-amplification footgun).
    ///
    /// # Side-effect-free (r6-dx-C4)
    ///
    /// Tracing runs the evaluator in "trace mode" — the buffered host
    /// write ops produced by the walk are *discarded* instead of being
    /// replayed into the backend. A traced `crud:create` therefore does
    /// NOT persist a Node, does NOT fire a ChangeEvent, does NOT
    /// disturb IVM views. The returned `Outcome.created_cid` is the
    /// *projected* CID the write would have landed under, so the tracer
    /// can display a realistic terminal result without polluting the
    /// graph.
    pub fn trace(&self, handler_id: &str, op: &str, input: Node) -> Result<Trace, EngineError> {
        // r6b-dx-C4 + r6b-dx-C6: delegate to the evaluator's real
        // `run_with_trace` so every returned step is a genuine per-primitive
        // record — distinct `node_cid` per OperationNode, distinct per-step
        // microsecond duration, primitive kind reflecting what the walk
        // actually executed. The per-step `node_cid` is derived from the
        // handler-scoped OperationNode id, so it cross-references the
        // Mermaid diagram's node identifiers rather than the outcome's
        // content-addressed CID.
        let mut trace_steps: Vec<TraceStep> = Vec::new();
        let outcome = self.dispatch_call_with_mode_and_trace(
            handler_id,
            op,
            input,
            None,
            true,
            Some(&mut trace_steps),
        )?;
        Ok(Trace {
            steps: trace_steps,
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
        let handler_cid = {
            let guard = self.inner.handlers.lock_recover();
            guard.get(handler_id).copied()
        };
        let Some(handler_cid) = handler_cid else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        };

        // Reconstruct the subgraph. DSL (SubgraphSpec) path takes priority;
        // CRUD synthesis falls back to the `:create` shape because mermaid
        // rendering is op-agnostic (all CRUD ops share the same structural
        // READ/WRITE/RESPOND shape for the diagram).
        let spec_opt = self.inner.specs.lock_recover().get(handler_id).cloned();
        let subgraph = if let Some(spec) = spec_opt {
            self.subgraph_for_spec(&spec, "default", &Node::empty(), &handler_cid)?
        } else if let Some(label) = handler_id.strip_prefix("crud:") {
            self.subgraph_for_crud(label, "create", &Node::empty(), &handler_cid)?
                .0
        } else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown handler: {handler_id}"),
            });
        };
        Ok(benten_eval::diag::mermaid::render(&subgraph))
    }

    /// Return the predecessor adjacency of the handler.
    ///
    /// Computes a `target_cid -> [predecessor_cids]` map by reconstructing
    /// the registered subgraph (DSL path when a SubgraphSpec was stored;
    /// CRUD-synthesised `:create` shape otherwise) and mapping each
    /// handler-scoped OperationNode id through the same BLAKE3 derivation
    /// `Engine::trace` uses for each TraceStep's `node_cid`, so callers
    /// can correlate a predecessor adjacency entry with a trace step
    /// without additional bookkeeping. See 5d-J workstream 5.
    pub fn handler_predecessors(
        &self,
        handler_id: &str,
    ) -> Result<HandlerPredecessors, EngineError> {
        let handler_cid = {
            let guard = self.inner.handlers.lock_recover();
            guard.get(handler_id).copied()
        };
        let Some(handler_cid) = handler_cid else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        };

        // Reconstruct the subgraph via the same code path `handler_to_mermaid`
        // uses so the node set + edge set match what the walker saw.
        let spec_opt = self.inner.specs.lock_recover().get(handler_id).cloned();
        let subgraph = if let Some(spec) = spec_opt {
            self.subgraph_for_spec(&spec, "default", &Node::empty(), &handler_cid)?
        } else if let Some(label) = handler_id.strip_prefix("crud:") {
            self.subgraph_for_crud(label, "create", &Node::empty(), &handler_cid)?
                .0
        } else {
            return Err(EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("unknown handler: {handler_id}"),
            });
        };

        // Walk (from, to, _label) tuples and resolve each endpoint through
        // the derive_op_node_cid BLAKE3-keyed derivation so the adjacency
        // map is keyed the same way TraceStep::node_cid is.
        let mut adjacency: BTreeMap<Cid, Vec<Cid>> = BTreeMap::new();
        for (from_id, to_id, _label) in subgraph.edges() {
            let from_cid = derive_op_node_cid(handler_id, from_id);
            let to_cid = derive_op_node_cid(handler_id, to_id);
            let list = adjacency.entry(to_cid).or_default();
            if !list.contains(&from_cid) {
                list.push(from_cid);
            }
        }
        // Stable ordering so test assertions over the predecessor list
        // don't depend on HashMap iteration order.
        for preds in adjacency.values_mut() {
            preds.sort_by_key(benten_core::Cid::to_base32);
        }
        Ok(HandlerPredecessors::from_adjacency(adjacency))
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
        self.dispatch_call_with_mode(handler_id, op, input, actor, false)
    }

    /// Internal dispatch that optionally runs in trace-mode.
    ///
    /// When `trace_mode` is `true`, the replay phase of the two-phase write
    /// is skipped entirely — buffered `PendingHostOp`s are dropped rather
    /// than applied inside a transaction. This gives `Engine::trace` a
    /// side-effect-free walk while still producing the same `Outcome`
    /// shape (via the pending ops' projected CIDs) so callers see a
    /// realistic terminal result. See r6-dx-C4.
    pub(crate) fn dispatch_call_with_mode(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: Option<Cid>,
        trace_mode: bool,
    ) -> Result<Outcome, EngineError> {
        self.dispatch_call_with_mode_and_trace(handler_id, op, input, actor, trace_mode, None)
    }

    /// Variant of [`Self::dispatch_call_with_mode`] that optionally populates
    /// a per-step trace buffer during the walk (r6b-dx-C4). When
    /// `trace_steps_out` is `Some`, the engine invokes
    /// [`benten_eval::Evaluator::run_with_trace`] and converts each recorded
    /// [`benten_eval::TraceStep`] into the engine-level [`TraceStep`] the
    /// public `Engine::trace` surface consumes. The per-step `node_cid` is
    /// derived from the handler-scoped OperationNode id via BLAKE3 so the
    /// CID uniquely identifies *which primitive* executed (cross-reference
    /// point for the Mermaid diagram) rather than echoing the outcome's
    /// created CID (r6b-dx-C6).
    ///
    /// The function is kept private to the engine crate; public callers
    /// reach it via `Engine::trace`.
    pub(crate) fn dispatch_call_with_mode_and_trace(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: Option<Cid>,
        trace_mode: bool,
        trace_steps_out: Option<&mut Vec<TraceStep>>,
    ) -> Result<Outcome, EngineError> {
        // Verify the handler is registered.
        let handler_cid_opt = {
            let guard = self.inner.handlers.lock_recover();
            guard.get(handler_id).copied()
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
                actor,
                handler_cid: Some(handler_cid),
                pending_ops: Vec::new(),
                inject_failure: false,
                last_refresh: None,
                iteration: 0,
            });
        }

        let result = self.dispatch_call_inner(
            handler_id,
            op,
            input,
            &handler_cid,
            trace_mode,
            trace_steps_out,
        );

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
    // R6 round-2 C2-R2-3: the `_actor: Option<Cid>` parameter became dead
    // after sec-r6r1-01 landed the actor-from-active-call lookup
    // (`engine.rs:1096-1102`). Removed; the callable dispatch helper now
    // takes 6 args, dropping under the default `clippy::too_many_arguments`
    // threshold so its allow attribute is no longer required either.
    fn dispatch_call_inner(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        handler_cid: &Cid,
        trace_mode: bool,
        trace_steps_out: Option<&mut Vec<TraceStep>>,
    ) -> Result<Outcome, EngineError> {
        // Phase 2a G2-B / arch-r1-5: `handler_cid` is the third axis on the
        // subgraph-cache key — re-registration flips this axis.
        let spec_opt = self.inner.specs.lock_recover().get(handler_id).cloned();

        let (subgraph, list_hint) = if let Some(spec) = spec_opt {
            (
                self.subgraph_for_spec(&spec, op, &input, handler_cid)?,
                None,
            )
        } else if let Some(label) = handler_id.strip_prefix("crud:") {
            self.subgraph_for_crud(label, op, &input, handler_cid)?
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
        //
        // When tracing (trace_steps_out.is_some()) we call
        // `run_with_trace` so the evaluator records a real per-step entry
        // for every primitive it executed; the resulting TraceStep list
        // replaces the synthetic-step fabrication r6b-dx-C4 retired.
        let input_value = Value::Map(input.properties.clone());
        let mut evaluator = benten_eval::Evaluator::new();
        let (eval_result, raw_trace) = if trace_steps_out.is_some() {
            // G5-B-ii / Inv-14: construct the runtime AttributionFrame from
            // the active call's `(actor, handler, grant)` triple and thread
            // it through `run_with_trace_attributed` so every emitted
            // `TraceStep::Step` carries the originating audit context.
            // Symmetric with the WRITE-path stamping in
            // `impl PrimitiveHost::put_node` — same `noauth_pseudo_actor_cid`
            // fallback when the caller did not supply an explicit actor and
            // the same zero-CID placeholder for the grant under
            // NoAuthBackend (no grant entity yet — populated Phase 3 when
            // UCAN lands).
            let actor_cid = {
                let guard = self.active_call.lock_recover();
                guard
                    .last()
                    .and_then(|f| f.actor)
                    .unwrap_or_else(crate::primitive_host::noauth_pseudo_actor_cid)
            };
            let frame = benten_eval::AttributionFrame {
                actor_cid,
                handler_cid: *handler_cid,
                capability_grant_cid: crate::primitive_host::noauth_zero_grant_cid(),
            };
            match evaluator.run_with_trace_attributed(
                &subgraph,
                input_value,
                self as &dyn PrimitiveHost,
                frame,
            ) {
                Ok((run, trace)) => (Ok(run), trace),
                Err(e) => (Err(e), Vec::new()),
            }
        } else {
            (
                evaluator.run(&subgraph, input_value, self as &dyn PrimitiveHost),
                Vec::new(),
            )
        };

        // Copy raw evaluator TraceSteps into the caller's buffer, mapping
        // each Step row to its stable per-OperationNode CID and preserving
        // boundary / budget rows verbatim. The CID is derived from the
        // handler_id + node_id pair so two ops with the same primitive
        // kind but different positions in the subgraph surface distinct
        // CIDs (r6b-dx-C6). G11-A Wave 2b: TraceStep is now an enum
        // mirroring the eval-side variant union.
        if let Some(out) = trace_steps_out {
            for rs in raw_trace {
                match rs {
                    benten_eval::TraceStep::Step {
                        node_id,
                        duration_us,
                        inputs,
                        outputs,
                        error,
                        attribution,
                    } => {
                        let primitive = primitive_kind_label(&subgraph, &node_id);
                        let node_cid = derive_op_node_cid(subgraph.handler_id(), &node_id);
                        out.push(TraceStep::Step {
                            duration_us: duration_us.max(1),
                            node_cid,
                            primitive,
                            node_id,
                            inputs,
                            outputs,
                            error,
                            attribution,
                        });
                    }
                    benten_eval::TraceStep::SuspendBoundary { state_cid } => {
                        out.push(TraceStep::SuspendBoundary { state_cid });
                    }
                    benten_eval::TraceStep::ResumeBoundary {
                        state_cid,
                        signal_value,
                    } => {
                        out.push(TraceStep::ResumeBoundary {
                            state_cid,
                            signal_value,
                        });
                    }
                    benten_eval::TraceStep::BudgetExhausted {
                        budget_type,
                        consumed,
                        limit,
                        path,
                    } => {
                        out.push(TraceStep::BudgetExhausted {
                            budget_type,
                            consumed,
                            limit,
                            path,
                        });
                    }
                }
            }
        }

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
                // Phase 2a G5-B-i mini-review C1: an evaluator-raised Inv-11
                // (from `impl PrimitiveHost::put_node`'s system-zone
                // short-circuit) routes through the same Outcome::ON_ERROR
                // shape as the Phase-1 storage-layer stopgap, but fires
                // the Phase-2a user-surface code `E_INV_SYSTEM_ZONE`
                // instead of `E_SYSTEM_ZONE_WRITE`. See
                // `primitive_host::inv_system_zone_to_outcome` for the
                // symmetry rationale. Pending ops are intentionally
                // dropped — the violating WRITE is the first thing the
                // evaluator attempts, so the buffer is empty at this
                // point (but defence-in-depth: even if a legal WRITE had
                // buffered first, dropping preserves all-or-nothing).
                if matches!(
                    &e,
                    benten_eval::EvalError::Invariant(benten_eval::InvariantViolation::SystemZone)
                ) {
                    return Ok(crate::primitive_host::inv_system_zone_to_outcome());
                }
                return Err(eval_error_to_engine_error(e));
            }
        };

        // Trace-mode short-circuit (r6-dx-C4): drop the pending ops rather
        // than replaying them. We still project the would-be `created_cid`
        // from the first `PutNode` so `Outcome.created_cid` stays realistic
        // for the tracer; nothing hits the backend. No transaction, no IVM
        // disturbance, no change-event emission.
        if trace_mode {
            let projected_cid = pending.iter().find_map(|op| match op {
                PendingHostOp::PutNode { projected_cid, .. } => Some(*projected_cid),
                PendingHostOp::DeleteNode { .. } => None,
            });
            return Ok(outcome_from_terminal_with_cid(
                self,
                &edge,
                output,
                list_hint,
                projected_cid,
            ));
        }

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
                                Some(*projected_cid),
                                *actor_cid,
                                *handler_cid,
                                *capability_grant_cid,
                            )?;
                            last_cid = Some(cid);
                        }
                        PendingHostOp::DeleteNode { cid } => {
                            tx.delete_node(cid)?;
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
        handler_cid: &Cid,
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
                    .get(&cache_handler, "create", handler_cid)
                    .unwrap_or_else(|| {
                        self.inner
                            .parse_counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
                        self.inner.subgraph_cache.insert(
                            &cache_handler,
                            "create",
                            handler_cid,
                            sg.clone(),
                        );
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
                    .get(&cache_handler, "list", handler_cid)
                    .unwrap_or_else(|| {
                        self.inner
                            .parse_counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
                        self.inner.subgraph_cache.insert(
                            &cache_handler,
                            "list",
                            handler_cid,
                            sg.clone(),
                        );
                        sg
                    });
                Ok((sg, Some(label.to_string())))
            }
            "get" => {
                let mut sg = self
                    .inner
                    .subgraph_cache
                    .get(&cache_handler, "get", handler_cid)
                    .unwrap_or_else(|| {
                        self.inner
                            .parse_counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let mut sb = benten_eval::SubgraphBuilder::new(format!("crud:{label}:get"));
                        let r = sb.read(format!("crud_{label}_get"));
                        let _ = sb.respond(r);
                        let sg = sb.build_unvalidated_for_test();
                        self.inner.subgraph_cache.insert(
                            &cache_handler,
                            "get",
                            handler_cid,
                            sg.clone(),
                        );
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
            "update" => {
                // r6b-dx-C2: add the update arm. Resolves the target CID,
                // reads the current Node properties, merges the caller's
                // `patch` Map onto them, and emits a subgraph that
                // (1) deletes the old Node and (2) writes the merged new
                // Node under a freshly-hashed CID. Both host ops land in the
                // same `pending_ops` batch and replay atomically in one tx.
                //
                // Input shape: `{ cid: <base32>, patch: <Map> }`. Missing
                // `cid` routes ON_NOT_FOUND via the delete executor's
                // delete_missing path; missing `patch` is treated as an
                // empty map (no-op update re-writes identical properties).
                let target = match input.properties.get("cid") {
                    Some(Value::Text(s)) => self.lookup_cid_by_base32(label, s)?,
                    _ => None,
                };
                // Read the existing Node so we can merge its properties with
                // the caller-supplied patch. A miss routes the delete
                // subgraph's `delete_missing` branch via ON_NOT_FOUND.
                let (old_props, resolved_target) = match target {
                    Some(cid) => match self.backend.get_node(&cid)? {
                        Some(node) => (node.properties, Some(cid)),
                        None => (BTreeMap::new(), None),
                    },
                    None => (BTreeMap::new(), None),
                };
                let patch = match input.properties.get("patch") {
                    Some(Value::Map(m)) => m.clone(),
                    _ => BTreeMap::new(),
                };
                // Merge: patch wins on key collisions; old properties fill
                // the rest. Preserve the stamped `createdAt` unless the
                // patch explicitly overrides it.
                let mut merged = old_props;
                for (k, v) in patch {
                    merged.insert(k, v);
                }

                // Build the two-step subgraph. We do NOT cache this one
                // because the delete target CID is per-call input; the
                // shape is small (2 WRITEs + RESPOND) so rebuilding is
                // cheap relative to the lookup work above.
                let mut sb = benten_eval::SubgraphBuilder::new(format!("crud:{label}:update"));
                let del = sb.write(format!("crud_{label}_update_delete"));
                let upd = sb.write(format!("crud_{label}_update_write"));
                sb.add_edge(del, upd);
                let _ = sb.respond(upd);
                let mut sg = sb.build_unvalidated_for_test();

                // Populate the delete node.
                if let Some(del_node) = sg.op_by_id_mut(&format!("crud_{label}_update_delete")) {
                    match resolved_target {
                        Some(cid) => {
                            del_node
                                .properties
                                .insert("op".into(), Value::text("delete"));
                            del_node
                                .properties
                                .insert("target_cid".into(), Value::Bytes(cid.as_bytes().to_vec()));
                        }
                        None => {
                            // Route the overall subgraph through
                            // ON_NOT_FOUND when the target doesn't resolve.
                            // The delete_missing executor emits the typed
                            // edge; without a follow-up WRITE the
                            // Outcome's edge reflects the miss.
                            del_node
                                .properties
                                .insert("op".into(), Value::text("delete_missing"));
                        }
                    }
                }

                // Populate the write (update) node only when the target
                // resolved. If it didn't, the delete_missing step routes
                // ON_NOT_FOUND and the walk terminates before reaching
                // the write — but we still populate defensively so
                // re-entering via `next` would not crash.
                if let Some(upd_node) = sg.op_by_id_mut(&format!("crud_{label}_update_write")) {
                    upd_node
                        .properties
                        .insert("op".into(), Value::text("update"));
                    upd_node
                        .properties
                        .insert("label".into(), Value::text(label));
                    upd_node
                        .properties
                        .insert("properties".into(), Value::Map(merged));
                }

                Ok((sg, None))
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
                    .get(&cache_handler, "delete", handler_cid)
                    .unwrap_or_else(|| {
                        self.inner
                            .parse_counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let mut sb =
                            benten_eval::SubgraphBuilder::new(format!("crud:{label}:delete"));
                        let w = sb.write(format!("crud_{label}_delete"));
                        let _ = sb.respond(w);
                        let sg = sb.build_unvalidated_for_test();
                        self.inner.subgraph_cache.insert(
                            &cache_handler,
                            "delete",
                            handler_cid,
                            sg.clone(),
                        );
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
        op: &str,
        _input: &Node,
        handler_cid: &Cid,
    ) -> Result<benten_eval::Subgraph, EngineError> {
        // Phase 2a G2-B / arch-r1-5: consult the AST cache (WRITE-free
        // specs only, since WRITE-bearing specs stamp per-call `createdAt`).
        let cache_eligible = spec.write_specs.is_empty();
        if cache_eligible
            && let Some(cached) = self
                .inner
                .subgraph_cache
                .get(&spec.handler_id, op, handler_cid)
        {
            return Ok(cached);
        }
        self.inner
            .parse_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        if cache_eligible {
            self.inner
                .subgraph_cache
                .insert(&spec.handler_id, op, handler_cid, sg.clone());
        }
        Ok(sg)
    }

    // -------- System-zone privileged API (N7) --------
    //
    // `create_principal`, `grant_capability`, `revoke_capability`,
    // `create_view`, and the private `privileged_put_node` helper live in
    // [`crate::engine_caps`].

    // -------- Change stream surface --------
    //
    // `subscribe_change_events`, the test-only probe variants, and
    // `change_event_count` live in [`crate::engine_views`].

    // -------- View reads (IVM) --------
    //
    // `read_view`, `read_view_with`, `read_view_strict`, and
    // `read_view_allow_stale` live in [`crate::engine_views`].

    // -------- Snapshot + transaction / metrics / diagnostics --------
    //
    // `snapshot`, `transaction`, `count_nodes_with_label`,
    // `metrics_snapshot`, `capability_writes_committed` /
    // `capability_writes_denied`, `change_stream_capacity`,
    // `ivm_subscriber_count`, `diagnose_read`, the Phase-2 version-chain
    // stubs, and `testing_insert_privileged_fixture` all live in
    // [`crate::engine_diagnostics`].
}

// `make_engine_tx` constructs an EngineTransaction from a graph Transaction.
// Kept out of the main flow so the closure reads cleanly.
pub(crate) fn make_engine_tx<'tx, 'coll>(
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
// Per-capability scope derivation (named compromise #5)
// ---------------------------------------------------------------------------

/// Derive the deduplicated list of `store:<label>:write` scopes a batch of
/// `PendingOp`s exercises. Mirrors [`benten_caps::GrantBackedPolicy`]'s
/// internal derivation so a NoAuth engine and a GrantBacked engine tally
/// commits under the same key space.
///
/// System-zone labels (`system:*`) are skipped — user subgraphs cannot
/// reach them, and crediting privileged grant/revoke writes to the
/// per-scope tally would make the metric misleading. Empty labels
/// collapse to `store:write` (matches the caps-side fallback).
pub(crate) fn derive_committed_scopes(ops: &[benten_caps::PendingOp]) -> Vec<String> {
    use benten_caps::PendingOp;
    let mut out: Vec<String> = Vec::new();
    let mut push = |label: &str| {
        if label.starts_with("system:") {
            return;
        }
        let scope = if label.is_empty() {
            "store:write".to_string()
        } else {
            format!("store:{label}:write")
        };
        if !out.contains(&scope) {
            out.push(scope);
        }
    };
    for op in ops {
        match op {
            PendingOp::PutNode { labels, .. } => {
                let primary = labels.first().map_or("", String::as_str);
                push(primary);
            }
            PendingOp::PutEdge { label, .. } => push(label),
            PendingOp::DeleteNode { labels, .. } => {
                let primary = labels.first().map_or("", String::as_str);
                push(primary);
            }
            PendingOp::DeleteEdge { label, .. } => {
                if let Some(l) = label.as_deref() {
                    push(l);
                }
            }
            // R6 fix-pass: `PendingOp` is `#[non_exhaustive]` (added in
            // commit 98d14fe). Phase-3 will add UCAN-attributed variants
            // (PutCapabilityGrant etc.); future variants surface here as
            // a no-op until the scope-mapping path is taught about them.
            _ => {}
        }
    }
    out
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
/// Derive a stable content-addressed identifier for a single OperationNode
/// inside a handler's subgraph. Used by `Engine::trace` so each trace step
/// carries a CID that cross-references the operation-node identifier
/// rendered by Mermaid (`subgraph.to_mermaid()`) — the trace's
/// `node_cid` stream is meaningfully distinct from the Outcome's
/// `created_cid` (r6b-dx-C6).
///
/// The derivation hashes `"<handler_id>\0<node_id>"` via BLAKE3; an empty
/// handler_id collapses to hashing the node_id alone, matching the
/// fallback shape the evaluator uses when a handler has no id.
fn derive_op_node_cid(handler_id: &str, node_id: &str) -> Cid {
    let mut material = Vec::with_capacity(handler_id.len() + 1 + node_id.len());
    material.extend_from_slice(handler_id.as_bytes());
    material.push(0);
    material.extend_from_slice(node_id.as_bytes());
    let digest: [u8; 32] = *blake3::hash(&material).as_bytes();
    Cid::from_blake3_digest(digest)
}

/// Look up the primitive kind label for a given operation-node id inside a
/// subgraph and lowercase it for surface parity with the DSL-side
/// primitive names (`"read"`, `"write"`, `"respond"`, …). Missing nodes
/// collapse to an empty string — the evaluator should never emit a
/// TraceStep with an unknown id, but the trace surface must not crash if
/// that invariant slips.
fn primitive_kind_label(subgraph: &benten_eval::Subgraph, node_id: &str) -> String {
    for node in subgraph.nodes() {
        if node.id == node_id {
            return format!("{:?}", node.kind).to_lowercase();
        }
    }
    String::new()
}

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
