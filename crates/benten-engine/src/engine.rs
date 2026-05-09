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
use benten_eval::{
    InvariantConfig, PrimitiveHost, RegistrationError, SubgraphBuilderExt, SubgraphExt,
};
use benten_graph::{ChangeEvent, GraphBackend, GraphError, MutexExt};

// G13-C BLOCKER-2 fix-pass: builder + engine_transaction are gated to
// NOT-`browser-backend`; mirror the gate at the import here.
#[cfg(not(feature = "browser-backend"))]
use crate::builder::EngineBuilder;
use crate::change::ChangeBroadcast;
use crate::change_probe::ChangeProbe;
#[cfg(not(feature = "browser-backend"))]
use crate::engine_transaction::{EngineTransaction, GraphTxLike};
use crate::error::EngineError;
use crate::outcome::{
    AnchorHandle, DiagnosticInfo, HandlerPredecessors, Outcome, ReadViewOptions,
    RegisterReplaceOutcome, Trace, TraceStep, ViewCreateOptions,
};
// G13-C BLOCKER-2 fix-pass: primitive_host gated to NOT-`browser-backend`.
#[cfg(not(feature = "browser-backend"))]
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
    /// Phase-3 G19-E (wave-7b) per-handler TRANSFORM AST cache, keyed on
    /// `(handler_cid, node_id)`. Closes `phase-2-backlog.md` §9.2.
    /// Populated at `register_subgraph` / `register_subgraph_replace`
    /// time; consumed via the `PrimitiveHost::cached_transform_ast`
    /// override at TRANSFORM dispatch.
    pub(crate) ast_cache: crate::ast_cache::AstCache,
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
    /// G12-A test-only: override for the evaluator's cumulative iteration
    /// budget. `None` = use [`benten_eval::DEFAULT_ITERATION_BUDGET`]; `Some(n)`
    /// caps every subsequent `engine.trace(...)` / `engine.call(...)` walk at
    /// `n` steps so a small chained-primitive subgraph can deterministically
    /// trip the Inv-8 cumulative-step guard inside `Evaluator::run_inner`.
    /// Set via `Engine::testing_set_iteration_budget`; read in
    /// `dispatch_call_with_mode_and_trace`. Only meaningful under `cfg(any(
    /// test, feature = "test-helpers"))`; field stays present unconditionally
    /// to keep `EngineInner`'s layout cfg-invariant per arch-pre-r1-2 sibling
    /// "no cfg-conditional struct shapes" rule.
    pub(crate) test_iteration_budget: std::sync::Mutex<Option<u64>>,
    /// Phase 2b Wave-8f test-only call gate. When `Some(barrier)`, every
    /// `dispatch_call_with_mode_and_trace` invocation parks on the barrier
    /// AFTER resolving `handler_cid` from the handlers map but BEFORE
    /// constructing/walking the Subgraph. The harness uses this to land an
    /// `Engine::register_subgraph_replace` between the two points so the
    /// in-flight call's pre-swap `handler_cid` is the cache key feeding
    /// `dispatch_call_inner`'s `subgraph_for_spec` lookup. Cleared (set to
    /// `None`) on completion so production calls never block. Only meaningful
    /// under `cfg(any(test, feature = "test-helpers"))`; field stays present
    /// unconditionally per the same arch-pre-r1-2 "no cfg-conditional struct
    /// shapes" rule that applies to `test_iteration_budget`.
    pub(crate) test_pre_dispatch_gate: std::sync::Mutex<Option<std::sync::Arc<std::sync::Barrier>>>,
    /// Phase 2b G10-B: in-memory active set of installed module manifests
    /// keyed by their canonical-bytes CID. Mirrored to durable storage
    /// via the `system:ModuleManifest` zone (see
    /// [`crate::engine_modules`]). The active set lets the engine answer
    /// `is_module_installed` and capability-retraction queries without a
    /// backend round-trip.
    ///
    /// G13-C BLOCKER-2 fix-pass: gated to NOT-`browser-backend` since
    /// `engine_modules::InstalledModule` lives in the redb-coupled
    /// `engine_modules` module that is itself gated out. Browser thin
    /// clients do not run SANDBOX (per CLAUDE.md baked-in #16, the full
    /// peer is the SANDBOX execution surface) so the active-module set
    /// is unused on wasm32.
    #[cfg(not(feature = "browser-backend"))]
    pub(crate) installed_modules:
        std::sync::Mutex<std::collections::BTreeMap<Cid, crate::engine_modules::InstalledModule>>,
    /// Phase 2b Wave-8b: in-memory registry of WebAssembly module bytes
    /// keyed by the module's canonical CID. Wired through
    /// `Engine::register_module_bytes(cid, bytes)`; consumed by the
    /// `impl PrimitiveHost::execute_sandbox` override at SANDBOX dispatch
    /// time when the SANDBOX OperationNode's `module` property names a
    /// CID.
    ///
    /// Phase-3 generalises this to durable backend-resident WASM blob
    /// storage; the in-memory map keeps the wiring shape simple under
    /// Phase-2b's "no separate blob store yet" constraint and matches the
    /// in-memory `installed_modules` discipline (Compromise #N+8 wasm32
    /// in-memory-only narrative). The map is plain `BTreeMap` (not a
    /// Mutex of Arc<Vec<u8>>) because module bytes are read-mostly +
    /// cloning a Vec<u8> on lookup is cheap relative to wasmtime
    /// compilation cost.
    pub(crate) module_bytes: std::sync::Mutex<std::collections::BTreeMap<Cid, Vec<u8>>>,
    /// Phase 2b Wave-8f: per-handler version chain — newest-first list of
    /// CIDs registered under each handler_id. Populated lazily; the FIRST
    /// `register_subgraph` for a handler seeds the chain with the
    /// registered CID, every subsequent `register_subgraph_replace`
    /// prepends the new CID. The chain itself is bounded only by the
    /// process lifetime; Phase-3 promotes this to a durable Anchor +
    /// Version-Node chain (the `core::version` API). The in-memory chain
    /// keeps the hot-replace surface honest under Phase-2b's "no separate
    /// version store yet" constraint and gives `RegisterReplaceOutcome`
    /// the predecessor CID without a backend round-trip.
    pub(crate) handler_version_chain:
        std::sync::Mutex<std::collections::BTreeMap<String, Vec<Cid>>>,
    /// Wave-8h audit-gap fix — EMIT broadcast channel. `impl
    /// PrimitiveHost::emit_event` publishes here; consumers subscribe
    /// via [`Engine::subscribe_emit_events`]. See
    /// `crate::emit_broadcast` module docs for the rationale on a
    /// separate channel rather than extending [`benten_graph::ChangeEvent`].
    pub(crate) emit_broadcast: Arc<crate::emit_broadcast::EmitBroadcast>,
    /// Phase 2b Wave-8c-subscribe-infra: in-memory set of revoked actor
    /// CIDs. The SUBSCRIBE delivery-time cap-recheck consults this set;
    /// an actor present here has its in-flight ad-hoc onChange
    /// subscriptions auto-cancelled at the next delivery (D5
    /// cap-recheck-at-delivery contract). The set is populated by the
    /// testing helper `testing_revoke_actor_for_subscribe` (ESC-7) and
    /// by future grant-revocation flows once the GrantBackedPolicy
    /// rear-loads SUBSCRIBE-shape grant queries.
    ///
    /// Phase-3 promotion: the source-of-truth becomes the engine's
    /// grant store; this in-memory set degenerates to a cache hint.
    ///
    /// Phase-3 wave-5c §6.1-followup task #5 — held in `Arc<Mutex<...>>`
    /// so the SANDBOX `live_cap_check` callback (constructed at
    /// `execute_sandbox` dispatch time) can clone the Arc + observe
    /// revocations that arrive mid-call. Closes ESC-9 r1-wsa-3 MAJOR.
    pub(crate) revoked_actors_for_subscribe: Arc<std::sync::Mutex<std::collections::HashSet<Cid>>>,

    /// Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure): in-memory mirror of
    /// the revoked `(actor_cid, scope)` pairs the engine has observed
    /// since construction. Consulted by the per-row cap-recheck inside
    /// [`Engine::apply_atrium_merge`] for the structural-always-on
    /// recheck mirror of the SUBSCRIBE-side revocation surface.
    ///
    /// Production grant-revocation will rear-load this from the durable
    /// `system:CapabilityRevocation` write path; the in-memory set is
    /// the wave-5a-style bridge so the per-row recheck has a synchronous
    /// surface to consult without re-reading the backend on every
    /// merged row. Symmetric with `revoked_actors_for_subscribe` for
    /// SUBSCRIBE delivery-time recheck — but keyed on
    /// `(actor_cid, scope_string)` so the per-zone WRITE recheck can
    /// scope precisely (a peer revoked from `/zone/posts` may still be
    /// permitted to write `/zone/calendar`).
    pub(crate) revoked_actor_zone_pairs: std::sync::Mutex<std::collections::HashSet<(Cid, String)>>,

    /// Wave-8c fix-pass cr-w8c-fp-3: opaque test-only marker set.
    /// Decoupled from `revoked_actors_for_subscribe` so production
    /// cap-revocation semantics stay distinct from test-helper
    /// sideband signaling. Consumed by `testing_register_uncounted_host_fn`
    /// (ESC-13 helper smoke test); production paths NEVER read this set.
    /// Cfg-gated under `cfg(any(test, feature = "test-helpers"))` so the
    /// production engine layout does not carry the field at all.
    #[cfg(any(test, feature = "test-helpers"))]
    pub(crate) test_markers: std::sync::Mutex<std::collections::HashSet<Cid>>,

    /// Phase-3 G19-C1 (phase-3-backlog §7.1.3) — in-memory map of
    /// user-view input-label hints captured at
    /// [`Engine::register_user_view`] time. Used by
    /// [`Engine::user_view_on_update`] to derive the label filter for
    /// the returned [`ChangeProbe`] without round-tripping through the
    /// persisted `system:IVMView` Node. Canonical hand-written views
    /// have their input label resolved via
    /// [`benten_ivm::hardcoded_label_for_id`]; this map covers the
    /// generic user-defined fallback path. Maps `view_id` → input
    /// label string (the same value persisted as
    /// `input_pattern_label` on the `system:IVMView` Node).
    pub(crate) user_view_input_labels: std::sync::Mutex<BTreeMap<String, String>>,

    /// Phase-3 G19-C2 wave-7 (§7.1 SANDBOX execution metrics
    /// propagation): per-handler-id cumulative-high-water tracker for
    /// SANDBOX `fuel_consumed`, `output_consumed`, and the most-recent
    /// invocation's wall-clock duration. Populated at the
    /// `primitive_host.rs::execute_sandbox` boundary AFTER the eval-side
    /// `SandboxResult` returns; consumed by
    /// `engine_sandbox.rs::describe_sandbox_node` so the diagnostic
    /// accessor returns real metrics rather than the legacy `Unknown`
    /// placeholder.
    ///
    /// Per stream-r1-8: high-water values are PER-INVOCATION updates
    /// against the high-water mark within a single Engine instance —
    /// the cross-process WAIT-resume envelope does NOT carry in-flight
    /// SANDBOX metrics across the suspend boundary. A second-process
    /// resume that re-enters the SANDBOX node sees the second
    /// invocation's measurement (the fresh Engine has an empty
    /// metrics map).
    ///
    /// Keyed by handler_id (string) rather than node CID because the
    /// Phase-2b/3 dispatch surface tracks SANDBOX entries at the
    /// handler boundary; per-node sub-aggregation is a future
    /// devtools refinement once Phase 3+ adds richer node-level
    /// resolution. The shape is the resolved-defaults
    /// `SandboxNodeDescription` triple plus the high-water + last
    /// invocation-ms readings.
    pub(crate) sandbox_metrics:
        std::sync::Mutex<std::collections::BTreeMap<String, SandboxNodeMetrics>>,

    /// Phase-3 G20-A3 wave-8a (phase-3-backlog §7.3.A.9 sub-cluster
    /// 9c): test-only registry of SubscriberIds whose persistent
    /// cursors are driven by the `testing_register_persistent_subscriber`
    /// + `testing_emit_n_synthetic_events` test helpers. The helpers
    /// pin the SuspensionStore put_cursor → get_cursor round-trip
    /// under caller-controlled SubscriberIds; this map is the
    /// side-channel they walk on emit-events to locate registered
    /// subscribers. Production SUBSCRIBE ack flow does NOT consult
    /// this map — it lives behind the same cfg-gating as
    /// `test_markers` so the production cdylib does not ship it.
    #[cfg(any(test, feature = "test-helpers"))]
    pub(crate) testing_persistent_subscribers: std::sync::Mutex<Vec<benten_core::SubscriberId>>,

    /// Phase-3 G16-B-prime (§6.12 item 1): in-memory anchor store keyed
    /// by anchor name. Each entry carries a [`benten_core::version::Anchor`]
    /// (the prior-threaded chain) plus the most-recently-appended head CID
    /// (the CURRENT pointer). Populated by [`crate::Engine::create_anchor`]
    /// + [`crate::Engine::append_version`] / the engine-side Loro merge
    /// callback ([`crate::Engine::apply_atrium_merge`]).
    ///
    /// Production-grade durability of this map (survives engine restart)
    /// is gated on §1.1 GraphBackend umbrella trait — pre-§1.1 it lives
    /// in-memory only. The anchor names map 1:1 with logical Anchor
    /// identities; cloning an anchor across this map's iteration shares
    /// chain state (the inner `Arc<Mutex<...>>` discipline of
    /// [`benten_core::version::Anchor`]).
    pub(crate) anchor_store: std::sync::Mutex<BTreeMap<String, AnchorEntry>>,

    /// Phase-3 G16-B-prime (§6.12 item 3): the local engine's
    /// device-DID-attestation CID, used to populate
    /// [`benten_caps::WriteContext::device_cid`] +
    /// [`benten_caps::ReadContext::device_cid`] at engine-internal
    /// construction sites. `None` for non-attested / legacy / pre-
    /// device-attestation engines; `Some(cid)` for engines whose owner
    /// wired in a [`benten_id::device_attestation::DeviceAttestation`]
    /// CID via [`crate::Engine::set_device_cid`].
    pub(crate) device_cid: std::sync::Mutex<Option<Cid>>,

    /// Phase-3 G16-B-prime fp (cap-g16bp-1 closure / Ben's RATIFIED
    /// Option A 2026-05-08): the engine's logical-actor identity for
    /// sync-merge AttributionFrame minting. Decouples actor identity
    /// from device identity so AttributionFrame.actor_cid carries the
    /// PRINCIPAL identity (parent / handler-attributed agent) while
    /// AttributionFrame.device_did carries the DEVICE identity. Falls
    /// back to [`Self::device_cid`] when unset, preserving Phase-3
    /// single-user single-device behavior. Phase-4+ AI-agent /
    /// handler-attribution-flow callers set this explicitly via
    /// [`crate::Engine::set_actor_cid`] to retain principal identity
    /// across sync merges.
    pub(crate) actor_cid: std::sync::Mutex<Option<Cid>>,
}

/// Phase-3 G16-B-prime (§6.12 item 1): one entry in the engine's
/// in-memory anchor store. Pairs the prior-threaded
/// [`benten_core::version::Anchor`] with the most-recent head CID
/// (the CURRENT pointer the engine answers from
/// [`crate::Engine::read_current_version`]).
#[derive(Clone)]
pub(crate) struct AnchorEntry {
    /// The prior-threaded Anchor identity (chain history lives behind the
    /// inner `Arc<Mutex<...>>`).
    pub(crate) anchor: benten_core::version::Anchor,
    /// Most-recently-appended head CID. Equals `anchor.head` immediately
    /// after [`crate::Engine::create_anchor`]; advances with every
    /// successful [`crate::Engine::append_version`] call.
    pub(crate) current: Cid,
}

/// Phase-3 G19-C2 wave-7 (§7.1): per-invocation high-water tracker
/// for SANDBOX execution metrics. Populated at
/// `primitive_host.rs::execute_sandbox` AFTER the eval-side
/// `SandboxResult` returns; consumed by
/// `engine_sandbox.rs::describe_sandbox_node`.
///
/// `fuel_consumed_high_water` and `output_consumed_high_water` are
/// monotonically non-decreasing across invocations within a single
/// Engine instance; `last_invocation_ms` is the wall-clock duration
/// of the MOST-RECENT invocation only (NOT a high-water).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SandboxNodeMetrics {
    pub(crate) module_cid: Option<Cid>,
    pub(crate) manifest_id: Option<String>,
    pub(crate) fuel: u64,
    pub(crate) wallclock_ms: u64,
    pub(crate) output_limit_bytes: u64,
    pub(crate) fuel_consumed_high_water: Option<u64>,
    pub(crate) output_consumed_high_water: Option<u64>,
    pub(crate) last_invocation_ms: Option<u64>,
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
            ast_cache: crate::ast_cache::AstCache::new(),
            cap_write_committed: std::sync::Mutex::new(BTreeMap::new()),
            cap_write_denied: std::sync::Mutex::new(BTreeMap::new()),
            writes_committed_total: std::sync::atomic::AtomicU64::new(0),
            writes_denied_total: std::sync::atomic::AtomicU64::new(0),
            test_iteration_budget: std::sync::Mutex::new(None),
            test_pre_dispatch_gate: std::sync::Mutex::new(None),
            #[cfg(not(feature = "browser-backend"))]
            installed_modules: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            module_bytes: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            handler_version_chain: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            emit_broadcast: Arc::new(crate::emit_broadcast::EmitBroadcast::new()),
            revoked_actors_for_subscribe: Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            revoked_actor_zone_pairs: std::sync::Mutex::new(std::collections::HashSet::new()),
            #[cfg(any(test, feature = "test-helpers"))]
            test_markers: std::sync::Mutex::new(std::collections::HashSet::new()),
            user_view_input_labels: std::sync::Mutex::new(BTreeMap::new()),
            sandbox_metrics: std::sync::Mutex::new(BTreeMap::new()),
            #[cfg(any(test, feature = "test-helpers"))]
            testing_persistent_subscribers: std::sync::Mutex::new(Vec::new()),
            anchor_store: std::sync::Mutex::new(BTreeMap::new()),
            device_cid: std::sync::Mutex::new(None),
            actor_cid: std::sync::Mutex::new(None),
        }
    }

    /// Wave-8c fix-pass cr-w8c-fp-3: insert an opaque test marker into
    /// the cfg-gated `test_markers` sideband. Used by ESC-13's
    /// helper-smoke test (`testing_register_uncounted_host_fn`) so
    /// integration tests can assert helpers stamp markers without
    /// overloading the production cap-revocation set.
    #[cfg(any(test, feature = "test-helpers"))]
    pub(crate) fn insert_test_marker(&self, cid: &Cid) {
        let mut g = self
            .test_markers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.insert(*cid);
    }

    /// Wave-8c fix-pass cr-w8c-fp-3: query the opaque test-marker
    /// sideband set. Returns `true` iff `cid` was previously inserted
    /// via `insert_test_marker`.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub(crate) fn has_test_marker(&self, cid: &Cid) -> bool {
        let g = self
            .test_markers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.contains(cid)
    }

    /// Phase 2b Wave-8c-subscribe-infra: return `true` iff the actor
    /// has NOT been added to the revoked-actors set. The SUBSCRIBE
    /// delivery-time cap-recheck closure built in
    /// [`Engine::on_change_as_with_cursor`] calls this.
    pub(crate) fn is_actor_active(&self, actor: &Cid) -> bool {
        let g = self
            .revoked_actors_for_subscribe
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !g.contains(actor)
    }

    /// Phase 2b Wave-8c-subscribe-infra: mark `actor` as revoked. The
    /// next ad-hoc onChange delivery for any subscription registered
    /// under this actor will fail the cap-recheck and auto-cancel the
    /// subscription per D5 contract. Public via the `testing_*`
    /// helper on `Engine`; production grant revocation will hook this
    /// when the GrantBackedPolicy rear-loads SUBSCRIBE-shape grants.
    pub(crate) fn mark_actor_revoked_for_subscribe(&self, actor: &Cid) {
        let mut g = self
            .revoked_actors_for_subscribe
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.insert(*actor);
    }

    /// Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure): mark `(actor, scope)`
    /// as revoked at the in-memory mirror. The next sync-replica merge
    /// whose row would write under `scope` from `actor` fails the
    /// per-row cap-recheck and surfaces
    /// [`crate::error::EngineError::SyncRevokedDuringSession`].
    ///
    /// Symmetric with [`Self::mark_actor_revoked_for_subscribe`] but
    /// scope-keyed: a peer revoked from `/zone/posts` may still write
    /// `/zone/calendar` if the scope-pair set does not name the latter.
    pub(crate) fn mark_actor_revoked_for_zone(&self, actor: &Cid, scope: impl Into<String>) {
        let mut g = self
            .revoked_actor_zone_pairs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.insert((*actor, scope.into()));
    }

    /// Phase-3 G16-B-F: query whether `(actor, zone)` is in the in-memory
    /// revocation set. Consulted by [`Engine::apply_atrium_merge`]'s
    /// per-row cap-recheck loop. Match is "either an exact `(actor, zone)`
    /// pair OR an `(actor, "")` zone-wildcard pair was revoked"; the
    /// wildcard form lets a single-call revoke cover every zone the
    /// peer might write under without enumerating each.
    pub(crate) fn is_actor_revoked_for_zone(&self, actor: &Cid, zone: &str) -> bool {
        let g = self
            .revoked_actor_zone_pairs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.contains(&(*actor, zone.to_string())) || g.contains(&(*actor, String::new()))
    }

    /// Phase-3 G19-C2 wave-7 (§7.1): record a SANDBOX-execution metric
    /// observation against the per-handler high-water tracker.
    ///
    /// `fuel_consumed` + `output_consumed` update the `*_high_water`
    /// fields monotonically (max-of-prior-and-new); `last_invocation_ms`
    /// is overwritten with the most-recent invocation's wall-clock
    /// duration (NOT a high-water — per stream-r1-8 the "last" semantic
    /// is intentional). The resolved-defaults triple
    /// (`fuel`/`wallclock_ms`/`output_limit_bytes`) plus
    /// `module_cid`/`manifest_id` are folded in unconditionally so a
    /// fresh entry carries the resolved-defaults snapshot from the
    /// first invocation.
    ///
    /// Called from `primitive_host.rs::execute_sandbox` immediately
    /// after the eval-side `SandboxResult` returns Ok; the engine-side
    /// is the structural place to track because the eval-side
    /// `SandboxResult` is dropped at the `StepResult` boundary
    /// (Phase-2b/3 keeps `StepResult` slim by design — see
    /// `crates/benten-eval/src/lib.rs::StepResult`).
    pub(crate) fn record_sandbox_metric(&self, handler_id: &str, observation: SandboxNodeMetrics) {
        let mut guard = self.sandbox_metrics.lock_recover();
        let entry = guard
            .entry(handler_id.to_string())
            .or_insert_with(SandboxNodeMetrics::default);
        // Resolved-defaults snapshot — overwrite (the latest invocation's
        // resolved values are the freshest).
        entry.module_cid = observation.module_cid.or(entry.module_cid);
        entry.manifest_id = observation.manifest_id.or_else(|| entry.manifest_id.take());
        entry.fuel = observation.fuel;
        entry.wallclock_ms = observation.wallclock_ms;
        entry.output_limit_bytes = observation.output_limit_bytes;
        // Monotonic high-water max — never regresses across invocations.
        entry.fuel_consumed_high_water = match (
            entry.fuel_consumed_high_water,
            observation.fuel_consumed_high_water,
        ) {
            (None, x) => x,
            (Some(prev), None) => Some(prev),
            (Some(prev), Some(new)) => Some(prev.max(new)),
        };
        entry.output_consumed_high_water = match (
            entry.output_consumed_high_water,
            observation.output_consumed_high_water,
        ) {
            (None, x) => x,
            (Some(prev), None) => Some(prev),
            (Some(prev), Some(new)) => Some(prev.max(new)),
        };
        // Last-invocation overwrite — most-recent semantic is intentional
        // per stream-r1-8 (NOT cumulative across resumes).
        entry.last_invocation_ms = observation.last_invocation_ms.or(entry.last_invocation_ms);
    }

    /// Phase-3 G19-C2 wave-7 (§7.1): snapshot the metrics record for the
    /// named handler. Returns `None` when no SANDBOX invocation has
    /// occurred for this handler yet (the entry is created lazily on
    /// first record). Consumed by
    /// `engine_sandbox.rs::describe_sandbox_node`.
    pub(crate) fn sandbox_metric_snapshot(&self, handler_id: &str) -> Option<SandboxNodeMetrics> {
        let guard = self.sandbox_metrics.lock_recover();
        guard.get(handler_id).cloned()
    }

    /// R6 fp Wave C2 (obs-r6r1-2 closure): snapshot every per-handler
    /// SANDBOX metrics record so `engine_diagnostics::metrics_snapshot`
    /// can fan out namespaced keys for operator dashboards. Returns the
    /// (handler_id → metrics) map cloned out from under the lock so the
    /// caller can format keys without holding the metric lock during
    /// f64 conversion + string formatting.
    pub(crate) fn sandbox_metric_snapshot_all(&self) -> BTreeMap<String, SandboxNodeMetrics> {
        let guard = self.sandbox_metrics.lock_recover();
        guard.clone()
    }

    /// Phase-3 wave-5c §6.1-followup task #5 — clone the Arc'd
    /// revoked-actors set so the SANDBOX `live_cap_check` callback
    /// can observe revocations that arrive mid-call. Closes ESC-9
    /// r1-wsa-3 MAJOR (cap-revoke mid-call cadence: the callback
    /// fires BEFORE every host-fn invocation per cadence (a) per
    /// r1-wsa-3 disposition + r4-r1-wsa-4).
    pub(crate) fn revoked_actors_arc(
        &self,
    ) -> Arc<std::sync::Mutex<std::collections::HashSet<Cid>>> {
        Arc::clone(&self.revoked_actors_for_subscribe)
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
// Engine — G13-B generic-cascade per D-PHASE-3-1 RESOLVED + arch-r1-1 BLOCKER
// ---------------------------------------------------------------------------
//
// `EngineGeneric<B: GraphBackend>` is the generic engine struct introduced at
// G13-B (Phase-3 R5 wave-2). The default alias
//   `pub type Engine = EngineGeneric<RedbBackend>;`
// preserves API stability for every existing caller — the napi binding,
// integration tests, and `EngineBuilder::open` all continue to construct an
// `Engine` (= `EngineGeneric<RedbBackend>`) without changes.
//
// **D-PHASE-3-1 RESOLVED scope contract:** the engine consumes `GraphBackend`
// EXCLUSIVELY via the generic-cascade direction — `<B: GraphBackend>`
// parameters, never `dyn GraphBackend` / `Box<dyn>` / `Arc<dyn>`. The
// non-object-safety of `GraphBackend` (`type Error` + `type Snapshot` +
// `type Transaction` associated types) enforces this at compile time;
// `crates/benten-engine/tests/engine_no_dyn_graph_backend.rs::engine_does_not_reference_dyn_graph_backend_at_engine_boundary`
// pins it via syntactic grep so a future refactor that drops the associated
// types (re-enabling object-safety) cannot silently slip dyn-erasure into
// the engine boundary.
//
// **Where redb lives now (post-G13-B):** the resolved `Engine` alias still
// has the redb-specific `pub fn open(path)` constructor + the
// `from_snapshot_blob` rehydration path on a specialized `impl Engine`
// block (see `engine_snapshot.rs`). Those are convenience constructors
// that legitimately know they want `RedbBackend`. The cascade pin
// `crates/benten-engine/tests/engine_generic.rs::engine_generic_cascade_no_inherent_redb_references_outside_default_alias`
// scans this file (`engine.rs`) line-by-line and rejects any `RedbBackend`
// reference outside of:
//   1. The `pub type Engine = EngineGeneric<RedbBackend>;` line.
//   2. Lines inside `impl Engine { ... }` blocks (the resolved-alias
//      specialized-impl side; the pin's allowed-list).
// `impl<B: GraphBackend> EngineGeneric<B> { ... }` blocks (the generic
// cascade side) are forbidden from referencing `RedbBackend` inherently
// — every backend operation goes through the `B: GraphBackend` bound.

/// The Benten engine handle, generic over a [`GraphBackend`] storage layer.
///
/// `EngineGeneric<B>` is the generic shape introduced at G13-B (Phase-3 R5
/// wave-2). The default alias [`Engine`] resolves to
/// `EngineGeneric<RedbBackend>` and is what every existing caller, integration
/// test, and napi binding consumes.
///
/// ## Generic-cascade contract
///
/// Methods that need backend operations are bounded on `<B: GraphBackend>`
/// and live on the `impl<B: GraphBackend> EngineGeneric<B>` block. Methods
/// that legitimately specialize for redb (the convenience `Engine::open(path)`
/// constructor, `from_snapshot_blob` rehydration, and the engine modules
/// that consume the closure-based `RedbBackend::transaction(|tx| ...)`
/// surface) live on the resolved-alias `impl Engine` block.
///
/// See the module-level rationale block above for the full design contract.
pub struct EngineGeneric<B: GraphBackend> {
    pub(crate) backend: Arc<B>,
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
    ///
    /// G13-C BLOCKER-2 fix-pass: gated to NOT-`browser-backend` since
    /// `ActiveCall` lives in the cfg-gated `primitive_host` module
    /// (browser thin clients don't run handlers locally).
    #[cfg(not(feature = "browser-backend"))]
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
    /// Phase-2b G10-A-wasip1 (D10-RESOLVED): when `true`, the engine was
    /// constructed via [`Engine::from_snapshot_blob`] and is read-mostly
    /// — the underlying redb backend is a tempdir-resident materialization
    /// of the snapshot, and any user-facing mutation method
    /// (`create_node`, `update_node`, `delete_node`, `create_edge`,
    /// `delete_edge`) surfaces [`ErrorCode::BackendReadOnly`]
    /// (`E_BACKEND_READ_ONLY`) instead of corrupting the snapshot's
    /// canonical-bytes invariant the blob's CID is computed over.
    ///
    /// Defaults to `false`. Set only by `from_snapshot_blob`. The check
    /// lives at the user-facing surface (engine_crud.rs) rather than at
    /// the storage layer because the snapshot-blob engine still needs
    /// internal puts at construction time to hydrate the tempdir
    /// backend; gating at the user surface keeps the construction path
    /// straightforward.
    pub(crate) read_only_snapshot: bool,
    /// Phase-2b G12-E: durable store for WAIT suspend metadata,
    /// suspended `ExecutionStateEnvelope` bytes, and SUBSCRIBE
    /// persistent-cursor `max_delivered_seq` values. Populated at
    /// engine construction with a [`crate::RedbSuspensionStore`] over
    /// the engine's existing graph backend on the redb path (closes
    /// Phase-2a Compromise #10). The trait-object boundary keeps the
    /// generic engine free from coupling to the concrete backend's
    /// suspension-store wiring.
    pub(crate) suspension_store: Arc<dyn benten_eval::SuspensionStore>,
    /// G14-D wave-5a: per-actor live UCAN proof-chain CID list,
    /// consumed by [`Self::chain_for_actor`] at WAIT-resume time to
    /// recompute the cap_snapshot_hash and reject mismatches per
    /// CLR-2 §11.
    ///
    /// The eventual production-grade source of truth for this map is
    /// the durable UCAN backend (G14-B `UCANBackend::chain_for_audience`);
    /// G14-D wires the bridging slot here so the WAIT-resume hash
    /// recompute has a consultable accessor end-to-end. Tests +
    /// pre-G14-B-promotion deployments populate it via
    /// [`Self::testing_register_actor_proof_chain`].
    pub(crate) actor_chain_for_resume: std::sync::Mutex<BTreeMap<Cid, Vec<Cid>>>,
    /// G14-D wave-5a: per-device-DID revocation set. SUBSCRIBE
    /// subscriptions bound to a device-DID auto-cancel on the next
    /// delivery once the device-DID is added here per crypto-major-6
    /// + exploration-device-mesh. Production grant-revocation will
    /// rear-load this from the engine's durable cap store; the
    /// in-memory set is the wave-5a bridge.
    pub(crate) revoked_device_dids: std::sync::Mutex<std::collections::HashSet<String>>,
    /// G14-D wave-5a: thin-client outbound metrics surface. Counts
    /// events delivered to thin-client subscribers post-filter +
    /// events suppressed by F6 filtering at the full-peer edge.
    /// Reset to zero at engine construction.
    pub(crate) thin_client_metrics:
        std::sync::Mutex<crate::thin_client_subscribe::ThinClientMetrics>,
    /// Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure / cap-r4-3 +
    /// r4b-cap-4 reinforcement): sync-replica per-write cap-recheck
    /// counter. Incremented once per per-row recheck call inside
    /// [`Engine::apply_atrium_merge`]'s structural-always-on
    /// per-write loop (NOT once per merge — once per row written by
    /// the merge). Surfaced via [`Engine::sync_replica_cap_recheck_calls`]
    /// so test pins can assert the recheck observably fires.
    pub(crate) sync_replica_cap_recheck_count: std::sync::atomic::AtomicU64,
    /// G14-D wave-5a: per-thin-client-subscription state for the
    /// thin-client-subscribe protocol. Keyed by opaque subscription
    /// id; each entry carries the device-DID and the F6 cap-recheck
    /// closure consulted at delivery time.
    pub(crate) thin_client_subscriptions: std::sync::Mutex<
        std::collections::HashMap<
            crate::thin_client_subscribe::ThinClientSubId,
            crate::thin_client_subscribe::ThinClientSubscriptionState,
        >,
    >,
    /// G14-D wave-5a: handler-id-router log surface (per seq-major-8 +
    /// stream-r1-2). Records every routing decision made by SUBSCRIBE
    /// + EMIT producers so the integration test pins can assert that
    /// `Named(handler_id)` produces observably different traces from
    /// `DefaultFanOut`.
    pub(crate) handler_route_log: Arc<crate::handler_router::HandlerRouteLog>,
    /// Phase-3 G20-A2 (D12 wave-8a): WAIT TTL wall-clock test override
    /// (UNIX-epoch milliseconds). When `Some`, the engine consults this
    /// value at TTL deadline checks instead of `SystemTime::now()` —
    /// lets `testing_advance_wait_clock` simulate TTL expiry without
    /// real wallclock latency. Production engines never set this; the
    /// `&Engine`-receiving setters live under `cfg(any(test, feature =
    /// "test-helpers"))`.
    pub(crate) wait_wall_clock_override_ms: std::sync::Mutex<Option<u64>>,
    /// Phase-3 G20-A2 (D12 wave-8a): runtime GC machinery — observable
    /// reaped-count stats for the `testing_wait_ttl_gc_stats` helper +
    /// the GC pass entry point.
    pub(crate) wait_ttl_gc_stats: std::sync::Mutex<crate::WaitTtlGcStats>,
    /// Phase-3 G20-A2 (D12 wave-8a): when `true` the engine's WAIT-side
    /// event-driven GC sweep is suppressed (suspend / resume operations
    /// do NOT opportunistically reap expired siblings). The 1h interval
    /// backstop + the Engine::drop final sweep still fire. Set via
    /// EngineBuilder::gc_event_driven(false). Default `false`.
    pub(crate) wait_ttl_gc_event_driven_disabled: std::sync::atomic::AtomicBool,
    /// Phase-3 G20-A2 (D12 wave-8a): tracked envelope CIDs we've stamped
    /// WAIT metadata for. Used to drive GC sweeps without enumerating
    /// the entire suspension store key space (the redb store does not
    /// currently expose a prefix-scan API). Populated by suspend; pruned
    /// by GC + delete.
    pub(crate) wait_ttl_tracked_envelopes:
        std::sync::Mutex<std::collections::HashSet<benten_core::Cid>>,
}

/// Default engine alias resolving to the redb-backed specialization on
/// native targets, OR to the in-RAM thin-client cache on
/// `wasm32-unknown-unknown` browser targets when the `browser-backend`
/// cargo feature is opted in.
///
/// Existing callers (napi binding, integration tests, `EngineBuilder::open`)
/// continue to use `Engine` unchanged after the G13-B generic cascade —
/// API stability per D-PHASE-3-1a / arch-r1-1 BLOCKER closure.
///
/// ## Feature gating (G13-C wave-3 + CLAUDE.md baked-in #17)
///
/// - **Default features (native):** `Engine = EngineGeneric<RedbBackend>`.
/// - **`--features browser-backend`:** `Engine = EngineGeneric<BrowserBackend>`.
///   The browser-target napi cdylib build opts in via this feature so the
///   alias re-points without churning every `Engine` call site.
///
/// The two arms are mutually exclusive at the alias level — the cargo-
/// feature flip declared in `crates/benten-engine/Cargo.toml` (the
/// `browser-backend` feature) activates the second arm. Trying to
/// compile both arms at once is rejected at the type level (the cfg
/// gates produce a duplicate-type definition error).
#[cfg(not(feature = "browser-backend"))]
pub type Engine = EngineGeneric<benten_graph::RedbBackend>;

/// G13-C wave-3 (Phase-3 R5; CLAUDE.md baked-in #17): browser-target
/// alias resolving to the in-RAM thin-client cache. Enabled by the
/// `browser-backend` cargo feature. See the docstring on the
/// non-feature-gated `Engine` alias above for the full design narrative.
#[cfg(feature = "browser-backend")]
pub type Engine = EngineGeneric<benten_graph::BrowserBackend>;

impl<B: GraphBackend> std::fmt::Debug for EngineGeneric<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("caps_enabled", &self.caps_enabled)
            .field("ivm_enabled", &self.ivm_enabled)
            .finish_non_exhaustive()
    }
}

// G13-C wave-3: redb-specific convenience constructors live on the
// resolved-alias `impl Engine` block. Gated to NON `browser-backend`
// feature — the BrowserBackend alias gets its own thin-client-shape
// constructors (currently just `Engine::new(BrowserBackend::new())`
// via the generic-cascade `EngineBuilder` path).
#[cfg(not(feature = "browser-backend"))]
impl Engine {
    /// Open or create an engine backed by a redb database at `path`.
    ///
    /// Specialized constructor for the resolved-alias `Engine =
    /// EngineGeneric<RedbBackend>` — legitimately knows the caller wants
    /// a redb-backed engine because `path` is a filesystem location. The
    /// generic `EngineGeneric<B>` cascade does not have a uniform
    /// `open(path)` entry point because each backend defines its own
    /// construction shape (`BrowserBackend` consumes a JS-side IndexedDB
    /// handle; `SnapshotBlobBackend` consumes pre-loaded bytes).
    ///
    /// **Error-shape contract (D-PHASE-3-1a / D-B / arch-r1-1 BLOCKER
    /// closure):** backend-construction failures (path-not-found, redb
    /// corruption, invalid-blob etc.) surface as
    /// [`EngineError::Backend`] with the typed
    /// [`benten_graph::GraphError`] preserved inside the
    /// `Box<dyn std::error::Error + Send + Sync>` source-chain. Callers
    /// that need backend-specific telemetry recover via
    /// `err.source().and_then(|s| s.downcast_ref::<benten_graph::GraphError>())`
    /// — preserves API stability when alternative backends land while
    /// keeping the typed error path open for diagnostics. Pinned at
    /// `crates/benten-engine/tests/engine_error_boundary.rs`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        EngineBuilder::new()
            .open(path)
            .map_err(EngineError::erase_backend_at_public_boundary)
    }

    /// Begin a new builder. The current `EngineBuilder` is specialized for
    /// the redb-backed default path (`Engine = EngineGeneric<RedbBackend>`);
    /// each future backend (G13-C BrowserBackend, G13-D SnapshotBlobBackend)
    /// adds its own constructor shape.
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Open an Atrium peer-to-peer sync session bound to this engine.
    ///
    /// Phase-3 G16-B wave-6b. Native-only per CLAUDE.md baked-in #17;
    /// browser tabs participate via authenticated thin-client views,
    /// NOT as full Atrium peers.
    ///
    /// Returns the [`crate::engine_sync::AtriumHandle`] session
    /// handle (per Ben's D1 session-handle B-prime ratification);
    /// the handle carries the iroh transport endpoint, the per-zone
    /// Loro CRDT documents, and the merge-dispatch surface
    /// implementing the Inv-13 row-4 SPLIT classifier per ds-4.
    ///
    /// G16-B canary scope: the returned handle is logically
    /// independent of the engine's own state — a future R6-FP /
    /// G14-D integration will wire change-broadcast + Version-Node
    /// mint paths through the handle's merge-dispatch hooks.
    ///
    /// # Errors
    ///
    /// Returns [`crate::engine_sync::AtriumError`] if the iroh
    /// `Endpoint` binding fails.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn open_atrium(
        &self,
        config: crate::atrium_api::AtriumConfig,
    ) -> Result<crate::engine_sync::AtriumHandle, crate::engine_sync::AtriumError> {
        crate::engine_sync::AtriumHandle::open(config).await
    }

    /// Phase-3 G16-B-prime (§6.12 item 1): apply an inbound Loro CRDT
    /// merge frame against `zone` under `anchor`, mint a new Version
    /// Node carrying the resulting [`benten_eval::AttributionFrame`],
    /// and advance the anchor's CURRENT pointer.
    ///
    /// This is the engine-side completion of the row-4a (user-data) sync
    /// path the [`crate::engine_sync::AtriumHandle::merge_remote_change_with_hop_depth`]
    /// canary surfaced. The flow:
    ///
    /// 1. Apply the CRDT merge via `merge_remote_change_with_hop_depth`,
    ///    receiving a [`crate::engine_sync::SyncMergeAttribution`] seed
    ///    (peer node-ids + new sync_hop_depth).
    /// 2. Resolve peer node-ids → peer-DIDs via the local trust-store
    ///    ([`crate::engine_sync::AtriumHandle::resolve_peer_dids`]); fall
    ///    back to raw `node-id:NNN` strings for unresolvable ids.
    /// 3. Read the post-merge zone state via Loro's `all_writes` (the
    ///    full key/value snapshot the merge produced).
    /// 4. Construct a new "version" Node carrying the merged props +
    ///    a serialized [`benten_eval::AttributionFrame`] populated with
    ///    `peer_did_set`, `device_did` (from
    ///    [`Self::device_cid`] indirectly — the engine's own device-DID
    ///    if attestation-bound), and `sync_hop_depth`.
    /// 5. Persist the Node via the engine's
    ///    [`crate::Engine::append_version`] which (a) puts the Node bytes
    ///    via `GraphBackend::put_node`, (b) calls
    ///    [`benten_core::version::append_version`] to advance the chain
    ///    refusing forks, (c) updates the anchor entry's CURRENT.
    ///
    /// Returns the CID of the newly-minted merge Version Node.
    ///
    /// # Errors
    ///
    /// - [`crate::engine_sync::AtriumError`] on merge / hop-depth /
    ///   row-4b reject failures (mapped to [`EngineError::Other`] with
    ///   the underlying [`crate::engine_sync::AtriumError::code`]).
    /// - [`EngineError::Other`] (carrying [`ErrorCode::NotFound`]) when
    ///   the anchor handle is unknown to this engine.
    /// - [`EngineError::Graph`] / [`EngineError::Other`]
    ///   ([`ErrorCode::VersionBranched`] / [`ErrorCode::VersionUnknownPrior`])
    ///   on backend put or version-chain append failure.
    /// - [`EngineError::SyncRevokedDuringSession`] when the per-row
    ///   cap-recheck (sec-r4r1-2 BLOCKER closure) catches a
    ///   mid-session revocation against the originating peer.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(
        clippy::too_many_lines,
        reason = "G16-B-F (sec-r4r1-2 BLOCKER closure) inlines the per-row cap-recheck loop \
                  + structural-always-on policy.check_write hook + peer-DID resolution \
                  inside the merge orchestrator; splitting into helpers would scatter the \
                  defense-in-depth narrative across multiple call sites and obscure the \
                  before/after merge ordering contract"
    )]
    pub async fn apply_atrium_merge(
        &self,
        atrium: &crate::engine_sync::AtriumHandle,
        anchor: &crate::outcome::AnchorHandle,
        zone: &str,
        bytes: &[u8],
        incoming_hop_depth: u32,
    ) -> Result<Cid, EngineError> {
        // Phase-3 G16-B-prime fp (cap-g16bp-5 D10 closure): refuse
        // sync-merged writes against a read-only-snapshot engine
        // (e.g. one constructed via `Engine::from_snapshot_blob`).
        // Defense-in-depth: prevents an Atrium merge from reaching
        // `backend.put_node` on a snapshot engine — symmetrical with
        // the `engine_crud.rs::create_node` / `update_node` /
        // `delete_node` / `create_edge` / `delete_edge` guards.
        if self.is_read_only_snapshot() {
            return Err(EngineError::Other {
                code: ErrorCode::BackendReadOnly,
                message: "apply_atrium_merge: backend is read-only (snapshot engine)".into(),
            });
        }

        // 1. Apply the CRDT merge.
        let seed = atrium
            .merge_remote_change_with_hop_depth(zone, bytes, incoming_hop_depth)
            .await
            .map_err(|e| EngineError::Other {
                code: e.code(),
                message: e.to_string(),
            })?;

        // 2. Resolve peer node-ids → DIDs via the trust-store.
        let peer_did_set = atrium.resolve_peer_dids(&seed.peer_node_ids).await;

        // 3. Snapshot the post-merge zone state (Loro all_writes).
        let writes = atrium
            .with_zone(zone, |doc| doc.all_writes())
            .await
            .map_err(|e| EngineError::Other {
                code: e.code(),
                message: e.to_string(),
            })?;

        // 3.5. sec-r4r1-2 BLOCKER closure (Phase-3 G16-B-F): per-row
        //      cap-recheck-at-delivery. Mirrors the SUBSCRIBE-side
        //      delivery-time recheck per CLR-2 dual-layer recheck
        //      architecture. STRUCTURAL-ALWAYS-ON per Ben's RATIFIED
        //      Option (a) — no source enum / no caller bypass.
        //
        //      For each row produced by the Loro merge we (a) bump the
        //      observability counter so tests can assert the recheck
        //      observably fires, and (b) consult the in-memory
        //      `(actor_cid, scope)` revocation set (populated by
        //      `EngineCapsHandle::revoke`) — if the originating peer's
        //      grant for this zone has been revoked, fire the typed
        //      `SyncRevokedDuringSession` rejection BEFORE the merge
        //      Version Node is minted. Mirrors the SUBSCRIBE-side
        //      `E_SUBSCRIBE_REVOKED_MID_STREAM` shape.
        //
        //      Peer-actor-cid resolution: pre-G14-B durable identity
        //      backend, the engine derives a deterministic actor CID
        //      from the FIRST resolved peer-DID via the canonical
        //      blake3-of-utf8-bytes shape (matches the convention
        //      `EngineCapsHandle::install_proof` uses when the test
        //      installs a grant under a known actor CID — the test
        //      uses the same hashing path so the in-memory revocation
        //      pair set keys agree across the install/revoke + the
        //      per-row recheck). When no peer_did is present, falls
        //      back to the engine's `effective_actor_cid()` (single-
        //      peer single-device test fixture path).
        let peer_actor_cid = atrium
            .resolve_peer_dids(&seed.peer_node_ids)
            .await
            .first()
            .map(|did| Cid::from_blake3_digest(*blake3::hash(did.as_bytes()).as_bytes()))
            .or_else(|| self.effective_actor_cid());
        for (key, _stamped) in &writes {
            // Bump the per-row recheck counter — observable via
            // `Engine::sync_replica_cap_recheck_calls`. Pinned in
            // `sync_replica_attribution.rs::sync_replica_write_cap_recheck_at_delivery_against_local_grant_store`.
            self.sync_replica_cap_recheck_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Per-row revocation check: if `(actor_cid, scope)` is in
            // the in-memory revocation set under EITHER the bare zone
            // form OR the `<zone>:write` action-scoped form, reject
            // the entire merge before the Version Node is minted. The
            // merge is atomic — partial application would leave the
            // zone in an inconsistent state relative to the
            // originating peer's expected post-merge bytes, so a
            // single revoked row vetoes the whole merge. Checking
            // BOTH zone-shapes lets `EngineCapsHandle::revoke` be
            // ergonomic with a `:write`-scoped CapProof while the
            // merge boundary names just the bare zone.
            let zone_write = format!("{zone}:write");
            if let Some(actor_cid) = peer_actor_cid
                && (self.is_actor_revoked_for_zone(&actor_cid, zone)
                    || self.is_actor_revoked_for_zone(&actor_cid, &zone_write))
            {
                let peer_did = atrium
                    .resolve_peer_dids(&seed.peer_node_ids)
                    .await
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| "<unresolved-peer>".to_string());
                return Err(EngineError::SyncRevokedDuringSession {
                    peer_did,
                    zone: zone.to_string(),
                    cid: Cid::from_blake3_digest(*blake3::hash(key.as_bytes()).as_bytes()),
                });
            }

            // Additional structural recheck via the
            // `CapabilityPolicy::check_write` hook — preserves the
            // policy's view of the cap-recheck for backends that
            // route revocation through `system:CapabilityRevocation`
            // Node writes (the durable UCAN backend at G14-B). The
            // hook is consulted with a synthetic `WriteContext`
            // shaped per the row's zone + the resolved peer
            // actor_cid; a `Denied` / `Revoked` / `DeniedRead`
            // verdict surfaces the same typed
            // `SyncRevokedDuringSession` rejection as the in-memory
            // mirror.
            if let Some(policy) = self.policy.as_ref() {
                let scope = format!("{zone}:write");
                let ctx = benten_caps::WriteContext {
                    label: zone.to_string(),
                    actor_cid: peer_actor_cid,
                    scope: scope.clone(),
                    is_privileged: false,
                    actor_hint: None,
                    pending_ops: Vec::new(),
                    authority: benten_caps::WriteAuthority::User,
                    device_cid: None,
                };
                if let Err(cap_err) = policy.check_write(&ctx) {
                    use benten_caps::CapError;
                    if matches!(cap_err, CapError::Revoked | CapError::Denied { .. }) {
                        let peer_did = atrium
                            .resolve_peer_dids(&seed.peer_node_ids)
                            .await
                            .into_iter()
                            .next()
                            .unwrap_or_else(|| "<unresolved-peer>".to_string());
                        return Err(EngineError::SyncRevokedDuringSession {
                            peer_did,
                            zone: zone.to_string(),
                            cid: Cid::from_blake3_digest(*blake3::hash(key.as_bytes()).as_bytes()),
                        });
                    }
                    // Other CapError variants (NotImplemented, etc.)
                    // surface via the typed Cap pass-through —
                    // preserves existing semantics for non-revocation
                    // policy verdicts.
                    return Err(EngineError::Cap(cap_err));
                }
            }
        }

        // 4. Build the merge Version Node. Properties carry the merged
        //    string values + a serialized AttributionFrame slot ("attribution_frame_cid")
        //    that future readers consult to verify peer-DID provenance.
        //
        //    Phase-3 G16-D wave-6b: the device-DID slot reflects the
        //    ORIGINATING device's identity (the peer that produced the
        //    inbound writes), preferring the on-the-wire
        //    DeviceAttestationEnvelope's declared `device_did` when
        //    present (the typical production / multi-device path).
        //    Falls back to the local engine's `device_cid` slot when
        //    the inbound envelope did not declare a device-DID (legacy
        //    pre-G16-D peer / test fixture that bypasses the wire
        //    envelope), preserving Phase-3 single-user single-device
        //    behavior + the receiver-side device-CID introspection
        //    used by post-fix doc-coupling pim-2 end-to-end pins.
        //
        //    Closes plan §1 exit-criterion 16 (multi-device support
        //    for a single identity): two devices on the SAME identity
        //    sync as a single principal while AttributionFrame
        //    preserves DEVICE-grain provenance per Inv-14.
        let device_did = match seed.remote_device_did.clone() {
            Some(remote) => Some(remote),
            None => self.device_cid().map(|cid| {
                // Hex-render the device-CID bytes as the device DID
                // surface fallback. Pre-G16-D handshake protocol body
                // landing, this is the internal-engine convention for
                // the device-DID-string slot; the production
                // trust-store promotes to `did:key:` resolution once
                // the device-attestation envelope flow has surfaced
                // the originating-peer's declared DID (the preferred
                // branch above).
                format!("device-cid:{cid}")
            }),
        };
        // G16-D wave-6b: clear the per-zone remote-device-DID slot so a
        // subsequent merge that did NOT receive a fresh wire envelope
        // cannot inherit the prior envelope's device-DID. The slot is
        // re-populated by the next `sync_subgraph` /
        // `accept_sync_subgraph` exchange.
        atrium.clear_last_received_remote_device_did(zone).await;
        // Phase-3 G16-B-prime fp (cap-g16bp-1 / Ben's RATIFIED Option A
        // 2026-05-08): source the AttributionFrame.actor_cid from
        // `effective_actor_cid` — falls back to device_cid when no
        // explicit actor identity has been set, preserving Phase-3
        // single-user single-device behavior. Phase-4+ AI-agent /
        // handler-attribution flows that call `set_actor_cid(...)`
        // observably retain principal identity across sync merges.
        let attribution = benten_eval::AttributionFrame {
            actor_cid: self
                .effective_actor_cid()
                .unwrap_or_else(|| Cid::from_blake3_digest([0u8; 32])),
            handler_cid: Cid::from_blake3_digest([0u8; 32]),
            capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
            sandbox_depth: 0,
            peer_did_set: if peer_did_set.is_empty() {
                None
            } else {
                Some(peer_did_set)
            },
            device_did,
            sync_hop_depth: seed.sync_hop_depth,
        };
        let attribution_cid = attribution.cid().map_err(|e| EngineError::Other {
            code: ErrorCode::Serialize,
            message: format!("AttributionFrame::cid encode failed: {e}"),
        })?;

        // 5. Build + persist the Node.
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        for (key, stamped) in writes {
            props.insert(format!("loro:{key}"), Value::text(stamped.value));
        }
        props.insert(
            "attribution_frame_cid".into(),
            Value::Bytes(attribution_cid.as_bytes().to_vec()),
        );
        props.insert(
            "sync_hop_depth".into(),
            Value::Int(i64::from(seed.sync_hop_depth)),
        );
        props.insert("zone".into(), Value::text(zone.to_string()));
        let merge_node = Node::new(vec!["version".into()], props);
        let cid = self.append_version(anchor, &merge_node)?;
        Ok(cid)
    }

    /// Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure / cap-r4-3 +
    /// r4b-cap-4 reinforcement): cumulative count of per-row cap-recheck
    /// calls fired by [`Self::apply_atrium_merge`]'s structural-always-on
    /// per-write loop.
    ///
    /// Mirrors the SUBSCRIBE-side per-event cap-recheck observability
    /// surface; tests assert this counter increments when a sync-replica
    /// merge applies, and that the rejection arm fires
    /// [`EngineError::SyncRevokedDuringSession`] when the originating
    /// peer's grant was revoked locally between the Atrium handshake and
    /// the next sync round.
    #[must_use]
    pub fn sync_replica_cap_recheck_calls(&self) -> u64 {
        self.sync_replica_cap_recheck_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Phase-3 G16-B-F — capability-grant mutation handle, returned by
    /// the `caps()` accessor on the engine. Wraps a borrow of the
    /// engine + exposes thin `install_proof` / `revoke` surfaces that
    /// route through the engine's existing privileged
    /// [`Engine::grant_capability`] / [`Engine::revoke_capability`]
    /// paths.
    ///
    /// The handle is the surface the sec-r4r1-2 RED-PHASE pins
    /// consume (per
    /// `crates/benten-engine/tests/sync_replica_attribution.rs`); it
    /// names the production-equivalent surface for grant install /
    /// revoke without re-routing the existing system-zone
    /// `system:CapabilityGrant` / `system:CapabilityRevocation` Node
    /// writes.
    ///
    /// Cfg-gated to NOT-`browser-backend` because the underlying
    /// privileged `grant_capability` / `revoke_capability` paths live
    /// in `engine_caps` which is itself NOT-`browser-backend` (the
    /// browser thin client routes grant mutations through the full
    /// peer per CLAUDE.md baked-in #17).
    #[cfg(not(feature = "browser-backend"))]
    #[must_use]
    pub fn caps(&self) -> crate::engine_caps::EngineCapsHandle<'_> {
        crate::engine_caps::EngineCapsHandle { engine: self }
    }

    /// Phase-3 G16-B-F — current-epoch snapshot of every revoked
    /// `(actor_cid, scope)` pair that the in-memory cap layer has
    /// observed on this engine.
    ///
    /// Consulted by [`Self::apply_atrium_merge`]'s per-row
    /// cap-recheck. The Phase-1-stub [`crate::engine_caps`] grant
    /// path does NOT today maintain a typed in-memory revocation
    /// store (revocations land as `system:CapabilityRevocation`
    /// Nodes that the durable backend reads at write-check time);
    /// the in-memory mirror lives on [`EngineInner::revoked_actors`]
    /// so the per-row recheck has a synchronous surface to consult
    /// without re-reading the backend on every merged row.
    pub(crate) fn is_actor_revoked_for_zone(&self, actor_cid: &Cid, zone: &str) -> bool {
        self.inner.is_actor_revoked_for_zone(actor_cid, zone)
    }

    /// Phase-3 G21-T2 — dispatch a typed-CALL op directly through the
    /// engine's `dispatch_typed_call` arm without first registering a
    /// CALL-bearing subgraph.
    ///
    /// This is the inherent-method convenience path that the napi
    /// binding's `engine.typedCall(...)` surface routes through. The
    /// returned `Value` is the op's typed output (per
    /// [`benten_eval::TypedCallOp`] per-op rustdoc).
    ///
    /// Internally calls
    /// `<Engine as benten_eval::PrimitiveHost>::dispatch_typed_call` so
    /// the same dispatch arm exercised by the production CALL primitive
    /// fork is exercised here. Eval-side errors are mapped to
    /// [`EngineError`] via the existing
    /// `crate::primitive_host::eval_error_to_engine_error` catch-all
    /// (TypedCall variants surface as `EngineError::Other` carrying
    /// the stable `ErrorCode::TypedCall*` discriminant).
    ///
    /// # Errors
    ///
    /// Returns the typed [`EngineError`] for input validation /
    /// dispatch / cap-denial failures.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn dispatch_typed_call_public(
        &self,
        op: benten_eval::TypedCallOp,
        input: &benten_core::Value,
    ) -> Result<benten_core::Value, EngineError> {
        // `eval_error_to_engine_error` already imported at top of file
        // (R6 Wave 2 split). PrimitiveHost trait already in scope as well.
        //
        // G21-T2 fp-mini-review BLOCKER-1 closure: gate the napi entry
        // through the same per-op cap-check the eval-side
        // `execute_typed_call` (CALL primitive's typed-CALL fork) runs
        // BEFORE invoking `dispatch_typed_call`. Pre-fix the napi
        // `engine.typedCall(...)` path bypassed the cap-check entirely
        // — under `PolicyKind::Ucan` / `PolicyKind::GrantBacked` an
        // actor with NO grant for `cap:typed:crypto-sign` could still
        // run `engine.typedCall('ed25519_sign', ...)` and obtain a
        // valid signature. The fix mirrors `execute_typed_call`'s
        // cap-check arm + maps a generic `Capability` denial back to
        // the typed `TypedCallCapDenied` so the catalog code is
        // identical to the subgraph route (`E_TYPED_CALL_CAP_DENIED`,
        // not the broader `E_CAP_DENIED`).
        if let Err(e) = <Self as PrimitiveHost>::check_capability(self, op.required_cap(), None) {
            if matches!(e, benten_eval::EvalError::Capability(_)) {
                return Err(eval_error_to_engine_error(
                    benten_eval::EvalError::TypedCallCapDenied {
                        op_name: op.name(),
                        required: op.required_cap().to_string(),
                    },
                ));
            }
            return Err(eval_error_to_engine_error(e));
        }
        self.dispatch_typed_call(op, input)
            .map_err(eval_error_to_engine_error)
    }
}

/// G13-B generic-cascade: every constructor / accessor that does NOT
/// require backend-specific machinery lives on this generic-bound impl
/// block. Methods that need redb-specific surfaces (the closure-based
/// `transaction(|tx| ...)` execution path on CRUD writes, the
/// `from_snapshot_blob` rehydration that opens a fresh redb file, the
/// system-zone privileged write path) stay on the resolved-alias `impl
/// Engine` blocks (this file's `impl Engine { open / builder }` block
/// above; `engine_snapshot.rs::impl Engine`).
impl<B: GraphBackend> EngineGeneric<B> {
    /// Builder-only constructor used by `EngineBuilder::assemble`. Not part
    /// of the public API.
    ///
    /// Delegates to [`Self::from_parts_with_clocks`] with the Phase-1
    /// default clocks (`InstantMonotonicSource` + `HlcTimeSource`). Kept
    /// as a convenience for older call sites that predate the G9-A-cont
    /// clock injection; new call sites should use
    /// [`Self::from_parts_with_clocks`].
    ///
    /// The caller is responsible for supplying a [`benten_eval::SuspensionStore`]
    /// — the generic-cascade engine does not bake in the
    /// `RedbSuspensionStore` default (which is redb-specific). The
    /// resolved-alias `impl Engine` builder path injects
    /// [`crate::suspension_store::RedbSuspensionStore`] for the redb
    /// default; alternative backends supply their own suspension-store
    /// adapter.
    #[allow(
        clippy::too_many_arguments,
        dead_code,
        reason = "builder plumbing; retained for symmetry"
    )]
    pub(crate) fn from_parts(
        backend: Arc<B>,
        policy: Option<Box<dyn CapabilityPolicy>>,
        caps_enabled: bool,
        ivm_enabled: bool,
        broadcast: Arc<ChangeBroadcast>,
        inner: Arc<EngineInner>,
        ivm: Option<Arc<benten_ivm::Subscriber>>,
        suspension_store: Arc<dyn benten_eval::SuspensionStore>,
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
            suspension_store,
        )
    }

    /// Builder-only constructor used by `EngineBuilder::assemble` (Phase
    /// 2a G9-A-cont variant). Threads the clock sources onto the Engine
    /// struct so `impl PrimitiveHost` can consult them at refresh-point
    /// #3 without additional argument threading.
    #[allow(clippy::too_many_arguments, reason = "builder plumbing")]
    pub(crate) fn from_parts_with_clocks(
        backend: Arc<B>,
        policy: Option<Box<dyn CapabilityPolicy>>,
        caps_enabled: bool,
        ivm_enabled: bool,
        broadcast: Arc<ChangeBroadcast>,
        inner: Arc<EngineInner>,
        ivm: Option<Arc<benten_ivm::Subscriber>>,
        monotonic_source: Arc<dyn benten_eval::MonotonicSource>,
        time_source: Arc<dyn benten_eval::TimeSource>,
        suspension_store: Arc<dyn benten_eval::SuspensionStore>,
    ) -> Self {
        Self {
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
            #[cfg(not(feature = "browser-backend"))]
            active_call: std::sync::Mutex::new(Vec::new()),
            monotonic_source,
            time_source,
            revoke_at_iteration: std::sync::Mutex::new(BTreeMap::new()),
            read_only_snapshot: false,
            suspension_store,
            actor_chain_for_resume: std::sync::Mutex::new(BTreeMap::new()),
            revoked_device_dids: std::sync::Mutex::new(std::collections::HashSet::new()),
            thin_client_metrics: std::sync::Mutex::new(
                crate::thin_client_subscribe::ThinClientMetrics::default(),
            ),
            sync_replica_cap_recheck_count: std::sync::atomic::AtomicU64::new(0),
            thin_client_subscriptions: std::sync::Mutex::new(std::collections::HashMap::new()),
            handler_route_log: Arc::new(crate::handler_router::HandlerRouteLog::new()),
            wait_wall_clock_override_ms: std::sync::Mutex::new(None),
            wait_ttl_gc_stats: std::sync::Mutex::new(crate::WaitTtlGcStats::default()),
            wait_ttl_gc_event_driven_disabled: std::sync::atomic::AtomicBool::new(false),
            wait_ttl_tracked_envelopes: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// Phase-2b G10-A-wasip1: mark the engine as a read-only snapshot
    /// view. Set by [`Engine::from_snapshot_blob`] after the tempdir
    /// backend is hydrated; ungated otherwise. Crate-private so user
    /// code cannot retroactively flip a normal redb engine read-only
    /// (which would mask a write that committed).
    pub(crate) fn set_read_only_snapshot(&mut self) {
        self.read_only_snapshot = true;
    }

    /// Phase-2b G10-A-wasip1: returns `true` when this engine is a
    /// read-only view over a snapshot blob (constructed via
    /// [`Engine::from_snapshot_blob`]). User code can consult this to
    /// decide whether to attempt mutations or to thread reads through
    /// the engine.
    #[must_use]
    pub fn is_read_only_snapshot(&self) -> bool {
        self.read_only_snapshot
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

    /// G13-B generic-cascade: backend accessor returning the typed
    /// `Arc<B>` so consumer modules (`primitive_host.rs`,
    /// `engine_modules.rs`, etc.) can call backend-typed methods through
    /// the `B: GraphBackend` bound. The resolved-alias `Engine`
    /// (= `EngineGeneric<RedbBackend>`) sees this as `&Arc<RedbBackend>`
    /// — preserves every Phase-2b call-site signature unchanged.
    pub(crate) fn backend(&self) -> &Arc<B> {
        &self.backend
    }

    /// Phase 2a G5-B-i test-only backend accessor.
    ///
    /// The user-facing [`EngineGeneric::get_node`] now collapses system-zone
    /// reads to `None` under the Inv-11 runtime probe. Tests that need
    /// to assert an engine-privileged write actually landed (e.g.
    /// `grant_capability_only_via_engine_api`) reach through this
    /// accessor so the privileged back-channel is explicit.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn backend_for_test(&self) -> &Arc<B> {
        &self.backend
    }

    pub(crate) fn policy(&self) -> Option<&dyn CapabilityPolicy> {
        self.policy.as_deref()
    }

    /// Phase-2b G12-E: handle to the engine's [`benten_eval::SuspensionStore`].
    ///
    /// The store backs WAIT suspend metadata, suspended
    /// `ExecutionStateEnvelope` bytes, and SUBSCRIBE persistent-cursor
    /// `max_delivered_seq` values. Returned as a cheap `Arc::clone` so
    /// the caller can hand it to `EvalContext::with_suspension_store`
    /// when driving primitives outside the engine's own dispatch path
    /// (test fixtures, advanced consumers).
    #[must_use]
    pub fn suspension_store(&self) -> Arc<dyn benten_eval::SuspensionStore> {
        Arc::clone(&self.suspension_store)
    }

    pub(crate) fn ivm(&self) -> Option<&Arc<benten_ivm::Subscriber>> {
        self.ivm.as_ref()
    }

    /// Phase-3 G14-D wave-5a: return the live UCAN proof-chain CID list
    /// for `actor_cid` from the engine's durable cap surface. Used by
    /// `resume_from_bytes_inner` to recompute the cap_snapshot_hash and
    /// reject mismatches per CLR-2 §11. Returns the empty Vec when the
    /// engine has no policy configured (NoAuthBackend / placeholder
    /// deployments) or when no chain was registered for the actor —
    /// both cases produce a well-defined hash that differs from any
    /// non-empty chain bound at suspend.
    ///
    /// Not exposed to user code (the snapshot-hash flow is internal to
    /// resume); pub(crate) so the resume path in `engine_wait` can
    /// invoke it.
    pub(crate) fn chain_for_actor(&self, actor_cid: &Cid) -> Vec<Cid> {
        // The capability policy hook is the seam that surfaces the
        // chain. Phase-3 G14-B wired the durable UCAN backend; until
        // the chain accessor lands at the policy-hook surface (which
        // requires a CapabilityPolicy trait extension that's beyond
        // G14-D scope), the engine consults the in-memory
        // `actor_chain_for_resume` table the engine builder primes.
        let g = self
            .actor_chain_for_resume
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.get(actor_cid).cloned().unwrap_or_default()
    }

    /// Phase-3 G14-D wave-5a (test surface): register the proof-chain
    /// CID list to surface from `Self::chain_for_actor` (engine-private
    /// accessor) for `actor_cid`. Production code routes through the
    /// `CapabilityPolicy` chain accessor (G14-B durable UCAN backend);
    /// this helper exists for the WAIT-resume snapshot-hash test pins
    /// that need to pre-populate the live chain.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_register_actor_proof_chain(&self, actor_cid: Cid, chain: Vec<Cid>) {
        let mut g = self
            .actor_chain_for_resume
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.insert(actor_cid, chain);
    }

    /// Phase-3 G14-D wave-5a: handler-id-router log accessor. Used by
    /// integration tests + operator observability to inspect the
    /// routing decisions made by SUBSCRIBE + EMIT producers.
    #[must_use]
    pub fn handler_route_log(&self) -> Arc<crate::handler_router::HandlerRouteLog> {
        Arc::clone(&self.handler_route_log)
    }

    /// Phase-3 G19-B (§7.8): standalone `emit_event` surface — publish
    /// `payload` on `channel` directly through the engine's
    /// [`crate::emit_broadcast::EmitBroadcast`] without going through
    /// a handler dispatch. Mirrors the in-handler EMIT primitive's
    /// publish path so `Engine::subscribe_emit_events` consumers
    /// observe both standalone + handler-driven events on the same
    /// channel.
    ///
    /// Pre-G19-B the napi `emit_event` surface returned
    /// `E_PRIMITIVE_NOT_IMPLEMENTED` with a "named-destination-deferred
    /// to Phase 3 §7.8" hint. G19-B wires this directly through
    /// `EmitBroadcast::publish` so JS callers calling
    /// `engine.emitEvent(channel, payload)` see the event delivered to
    /// every `engine.onEmit(channel, ...)` consumer end-to-end.
    ///
    /// # Errors
    ///
    /// Currently infallible — the publish path is panic-isolated and
    /// the EmitBroadcast accepts any `(channel, payload)` pair. The
    /// `Result<(), EngineError>` return type is reserved for
    /// future-proofing (cap-recheck / rate-limiting hooks).
    pub fn emit_event(
        &self,
        channel: &str,
        payload: benten_core::Value,
    ) -> Result<(), EngineError> {
        let event = crate::emit_broadcast::EmitEvent {
            channel: channel.to_string(),
            payload,
        };
        self.inner.emit_broadcast.publish(&event);
        Ok(())
    }

    /// Phase-3 G14-D wave-5a: emit `payload` on `channel` with explicit
    /// [`crate::handler_router::HandlerRoute`] routing per seq-major-8.
    /// `Named(handler_id)` routes the emit-event through the named
    /// handler subgraph; `DefaultFanOut` falls through to the
    /// `EmitBroadcast` fan-out (the pre-G14-D behaviour).
    ///
    /// # Errors
    /// Returns [`EngineError`] when the named handler isn't registered
    /// (`E_NOT_FOUND`).
    pub fn emit_with_handler(
        &self,
        channel: &str,
        payload: benten_core::Value,
        route: crate::handler_router::HandlerRoute,
    ) -> Result<(), EngineError> {
        match &route {
            crate::handler_router::HandlerRoute::DefaultFanOut => {
                self.handler_route_log.record_default_fan_out();
                // Default fan-out — broadcast through the EmitBroadcast.
                let event = crate::emit_broadcast::EmitEvent {
                    channel: channel.to_string(),
                    payload,
                };
                self.inner.emit_broadcast.publish(&event);
                Ok(())
            }
            crate::handler_router::HandlerRoute::Named(handler_id) => {
                // Verify the handler is registered.
                let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
                if !handlers.contains_key(handler_id.as_str()) {
                    // R6 fp Wave C2 (dx-r6-r1-1).
                    return Err(EngineError::Other {
                        code: benten_errors::ErrorCode::DslUnregisteredHandler,
                        message: format!("emit_with_handler: handler not registered: {handler_id}"),
                    });
                }
                drop(handlers);
                self.handler_route_log
                    .record_named(&format!("emit:{channel}"), handler_id);
                // ENGINE-SIDE vs EVAL-SIDE NAMED-ARM ASYMMETRY (as-1
                // mini-review explanation; pim-4 §3.10 wave-pairing
                // destination: G16-D Atrium peer wave).
                //
                // The Named-route path here records the routing
                // decision into `HandlerRouteLog` and bypasses default
                // fan-out — the log divergence (`default_fan_out_count`
                // does NOT bump) is the load-bearing per-stream-r1-2
                // observable. It does NOT invoke the named handler
                // subgraph.
                //
                // The eval-side `benten_eval::primitives::emit::execute`
                // Named arm DOES invoke the handler subgraph via
                // `host.call_handler`. The asymmetry is intentional and
                // wave-paired (pim-4 §3.10): the engine-side seam ships
                // here at G14-D wave-5a; the engine-surface dispatch
                // into the named subgraph wires at G16-D once the
                // call-handler surface composes with the broadcast bus.
                // The G16-D brief carries the follow-on closed-claim
                // pin asserting subgraph-dispatch fires from this
                // entry point.
                Ok(())
            }
        }
    }

    /// Phase-3 G14-D wave-5a: register a SUBSCRIBE consumer with
    /// explicit [`crate::handler_router::HandlerRoute`] routing per
    /// seq-major-8 LOAD-BEARING. `Named(handler_id)` routes change
    /// events through the named handler subgraph; `DefaultFanOut`
    /// uses the existing on_change broadcast.
    ///
    /// Returns the engine-side [`crate::engine_subscribe::Subscription`]
    /// handle for `Named(_)` routes; for `DefaultFanOut` callers are
    /// expected to use the existing [`Self::on_change`] entry point.
    ///
    /// # Errors
    /// Returns [`EngineError`] when the named handler isn't registered
    /// (`E_NOT_FOUND`) or when the pattern is empty
    /// (`E_SUBSCRIBE_PATTERN_INVALID`).
    pub fn subscribe_with_handler(
        &self,
        pattern: &str,
        route: crate::handler_router::HandlerRoute,
    ) -> Result<(), EngineError> {
        if pattern.is_empty() {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::SubscribePatternInvalid,
                message: "subscribe_with_handler: pattern must be non-empty".into(),
            });
        }
        match &route {
            crate::handler_router::HandlerRoute::DefaultFanOut => {
                self.handler_route_log.record_default_fan_out();
                Ok(())
            }
            crate::handler_router::HandlerRoute::Named(handler_id) => {
                let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
                if !handlers.contains_key(handler_id.as_str()) {
                    // R6 fp Wave C2 (dx-r6-r1-1).
                    return Err(EngineError::Other {
                        code: benten_errors::ErrorCode::DslUnregisteredHandler,
                        message: format!(
                            "subscribe_with_handler: handler not registered: {handler_id}"
                        ),
                    });
                }
                drop(handlers);
                self.handler_route_log
                    .record_named(&format!("subscribe:{pattern}"), handler_id);
                Ok(())
            }
        }
    }

    /// Phase-3 G14-D wave-5a: persist a [`benten_eval::suspension_store::CapSnapshot`]
    /// for the suspended envelope at `envelope_cid` so a later
    /// `resume_from_bytes_*` can re-validate the bound UCAN-proof-chain
    /// hash + historical-policy metadata. The cap_snapshot_hash is
    /// computed via [`crate::cap_snapshot_hash::compute`]
    /// `(actor_cid, proof_chain_cids)`.
    ///
    /// # Errors
    /// Surfaces [`EngineError::Other`] with code
    /// [`benten_errors::ErrorCode::Serialize`] on suspension-store
    /// persistence failure.
    pub fn put_cap_snapshot_for_envelope(
        &self,
        envelope_cid: Cid,
        actor_cid: &Cid,
        proof_chain_cids: &[Cid],
        historical_policy_metadata: Vec<u8>,
    ) -> Result<(), EngineError> {
        // Phase-3 G16-B canary (r4b-cap-2 transition): legacy 2-input
        // call shape preserved while suspend-side capture of
        // revocation_set + policy_backend_tag is wired in the
        // post-canary wave. Identical to
        // `compute(actor, chain, empty_revocations, no_auth_tag)`.
        let cap_snapshot_hash =
            crate::cap_snapshot_hash::compute_legacy(actor_cid, proof_chain_cids);
        let snapshot = benten_eval::suspension_store::CapSnapshot {
            cap_snapshot_hash,
            historical_policy_metadata,
        };
        self.suspension_store
            .put_cap_snapshot(envelope_cid, snapshot)
            .map_err(|e| EngineError::Other {
                code: benten_errors::ErrorCode::Serialize,
                message: format!("put_cap_snapshot: {e}"),
            })?;
        Ok(())
    }

    #[cfg(not(feature = "browser-backend"))]
    pub(crate) fn active_call(&self) -> &std::sync::Mutex<Vec<ActiveCall>> {
        &self.active_call
    }

    /// Phase-3 G14-C accessor — locked guard for the in-memory
    /// per-handler version-chain map (newest-first `Vec<Cid>` per
    /// `handler_id`). Consumed by [`crate::handler_versions`]'s
    /// rehydrate path so the chain can be rebuilt from durable
    /// `system:HandlerVersion` zone Nodes at engine open.
    ///
    /// Public to crate (not crate::handler_versions only) so the
    /// rehydrate impl can write directly to the same map
    /// `register_subgraph_replace` mutates.
    #[cfg(not(feature = "browser-backend"))]
    pub(crate) fn handler_version_chain_in_memory_lock(
        &self,
    ) -> std::sync::MutexGuard<'_, std::collections::BTreeMap<String, Vec<Cid>>> {
        self.inner.handler_version_chain.lock_recover()
    }

    /// Phase 2b G10-B accessor — the in-memory active set of installed
    /// module manifests keyed by canonical-bytes CID. Used by
    /// [`crate::engine_modules`] for install / uninstall lifecycle
    /// queries and by tests asserting cap-retraction behavior.
    ///
    /// G13-C BLOCKER-2 fix-pass: gated to NOT-`browser-backend` —
    /// browser thin clients do not run SANDBOX (CLAUDE.md baked-in #16).
    #[cfg(not(feature = "browser-backend"))]
    pub(crate) fn installed_modules(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<Cid, crate::engine_modules::InstalledModule>>
    {
        &self.inner.installed_modules
    }

    /// Phase 2b Wave-8b — look up registered WebAssembly module bytes by
    /// CID. `None` when the CID is not in the registry (the SANDBOX
    /// dispatch path surfaces this as
    /// [`benten_eval::EvalError::PrimitiveNotImplemented`] with the
    /// SANDBOX kind plus a descriptive message — see the
    /// `impl PrimitiveHost for Engine::execute_sandbox` override).
    ///
    /// Returns a clone of the bytes — the caller (the executor) takes
    /// ownership for the duration of the per-call wasmtime instance.
    /// Compilation is cached at the wasmtime layer
    /// (`benten_eval::sandbox::instance::module_for_bytes`), so the
    /// per-dispatch clone cost amortises to zero across hot-loop
    /// invocations of the same CID.
    pub(crate) fn module_bytes_for(&self, cid: &Cid) -> Option<Vec<u8>> {
        let guard = self.inner.module_bytes.lock_recover();
        guard.get(cid).cloned()
    }
}

// ---------------------------------------------------------------------------
// Engine — resolved-alias specialized impl block (G13-B generic-cascade
// boundary)
// ---------------------------------------------------------------------------
//
// Per the cite-pin
// `crates/benten-engine/tests/engine_generic.rs::engine_generic_cascade_no_inherent_redb_references_outside_default_alias`,
// every method body that names `RedbBackend` directly OR consumes the
// closure-based `transaction(|tx| ...)` execution surface (which is
// inherent on `RedbBackend`, not on `GraphBackend`) lives on the
// resolved-alias `impl Engine` block below — `Engine` =
// `EngineGeneric<RedbBackend>`, so methods here see the concrete redb
// surface even though the struct itself is generic.
//
// Future waves (G13-C BrowserBackend / G13-D SnapshotBlobBackend / G14-A
// onward) introduce alternative `Engine = EngineGeneric<<other-backend>>`
// aliases gated by cargo features; methods that should work across ALL
// backends would migrate up to `impl<B: GraphBackend> EngineGeneric<B>`
// (the generic block above) at that time. For Phase-3, the redb path
// remains the only fully-wired backend, so the cost-of-migration is
// deferred without sacrificing the generic-cascade contract: the engine
// boundary IS generic at the type level, and the constructor/accessor
// surface that an alternative backend would need is already on the
// generic block.
//
// G13-C BLOCKER-2 fix-pass (browser-backend cfg-gating): this block
// uses `RedbBackend` inherent methods (`writes_committed`,
// `get_by_label`, `get_by_property`, `transaction(|tx| ...)`,
// `register_module_bytes`, etc.) that are NOT on the umbrella
// `GraphBackend` trait. Gated to NOT-`browser-backend` so the
// wasm32-unknown-unknown thin-client target compiles. The full
// per-method re-export of this surface to alternative backends is
// deferred to phase-3-backlog §1.2-followup (impl-block cascade).

#[cfg(not(feature = "browser-backend"))]
impl Engine {
    /// Phase 2a G5-A / G11-A: monotonic per-engine audit sequence.
    ///
    /// Returns the current value of the storage-layer commit counter —
    /// the number of times the privileged put path produced a real
    /// commit (corresponds to redb's `writes_committed` AtomicU64;
    /// `RedbBackend::put_node_with_context` increments it on first-put
    /// commits per §9.11 row 3). Names this as the observable sequence
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

    /// Phase-3 G14-C — register WebAssembly module bytes under their
    /// canonical CID for SANDBOX dispatch.
    ///
    /// **Compromise #17 — CLOSED at G14-C wave-4b.** Bytes are now
    /// persisted into the redb-backed
    /// [`benten_graph::backends::RedbBlobBackend`] (`system:ModuleBytes`
    /// zone Nodes) AND mirrored into the in-memory cache for hot-path
    /// SANDBOX dispatch. Across an `Engine::open` cycle the durable
    /// blobs are rehydrated by the crate-internal
    /// `rehydrate_module_bytes_from_zone` accessor
    /// (called from `EngineBuilder::assemble`), so a process restart no
    /// longer requires the operator to re-call `register_module_bytes`.
    /// See `docs/SECURITY-POSTURE.md` "Compromise #17" for the full
    /// closure narrative.
    ///
    /// **D-PHASE-3-12 RESOLVED — strict CID validation at the entry
    /// point.** This API recomputes
    /// `Cid::from_blake3_digest(BLAKE3(bytes))` and rejects with a
    /// typed `E_MODULE_BYTES_CID_MISMATCH` error when the
    /// caller-supplied CID does not match. Defense in depth: the
    /// concrete redb [`benten_graph::backends::RedbBlobBackend::put_sync`]
    /// rechecks (so direct-to-storage writes are also defended), but
    /// the engine boundary is the authoritative gate.
    ///
    /// Re-registering the same CID with identical bytes is idempotent
    /// (the redb-side Inv-13 dedup early-return; the in-memory map
    /// overwrites with the same value).
    ///
    /// Cap-policy is NOT consulted at this entrypoint: registering wasm
    /// bytes does not authorise any caller to invoke them. Authority
    /// flows through the SANDBOX node's manifest cap-set + dispatching
    /// grant, both of which are checked at execute time inside
    /// `benten_eval::sandbox::execute`.
    ///
    /// # Errors
    ///
    /// - [`EngineError::Other`] with `code:
    ///   ErrorCode::SandboxUnavailableOnWasm` when called on
    ///   `target_arch = "wasm32"` (D3 entry-point 2 uniformity per
    ///   r4b-wsa-2; module bytes are exclusively consumed by the
    ///   SANDBOX runtime which is compile-time absent on wasm32 per
    ///   CLAUDE.md baked-in #17).
    /// - [`EngineError::Other`] with `code:
    ///   ErrorCode::Unknown("E_MODULE_BYTES_CID_MISMATCH")` when
    ///   `BLAKE3(bytes) != cid`.
    /// - [`EngineError::Graph`] when the underlying redb privileged-
    ///   write surface surfaces a backend error.
    pub fn register_module_bytes(&self, cid: &Cid, bytes: &[u8]) -> Result<(), EngineError> {
        // r4b-wsa-2 D3 entry-point 2 uniformity (Ben 2026-05-04 D3
        // LOAD-BEARING). On wasm32-unknown-unknown the SANDBOX
        // runtime (wasmtime) is compile-time absent, so registered
        // module bytes have no execution path. Surface the typed
        // E_SANDBOX_UNAVAILABLE_ON_WASM error at registration so a
        // browser thin-client caller observes the actionable failure
        // immediately, rather than an opaque "module accepted but
        // never executes" surface. Mirrors `Engine::execute_sandbox`
        // wasm32 stub at `primitive_host.rs::execute_sandbox`.
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (cid, bytes);
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::SandboxUnavailableOnWasm,
                message: crate::engine_sandbox::SANDBOX_UNAVAILABLE_ON_WASM_TEXT.to_string(),
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // D-PHASE-3-12 — recompute BLAKE3 over the bytes and compare
            // against the caller-supplied CID. Reject mismatch as a typed
            // error before any side-effecting work.
            let recomputed = Cid::from_blake3_digest(*blake3::hash(bytes).as_bytes());
            if &recomputed != cid {
                return Err(EngineError::Other {
                    code: benten_errors::ErrorCode::Unknown(
                        "E_MODULE_BYTES_CID_MISMATCH".to_string(),
                    ),
                    message: format!(
                        "register_module_bytes: caller-supplied CID does not match BLAKE3(bytes); \
                         expected={expected} computed={computed}",
                        expected = cid.to_base32(),
                        computed = recomputed.to_base32(),
                    ),
                });
            }

            // Compromise #17 closure: persist via the durable BlobBackend
            // FIRST so a crash between in-memory insert and disk-flush
            // doesn't leave the in-memory map advertising bytes the
            // restart can't rehydrate.
            let blob_backend =
                benten_graph::backends::RedbBlobBackend::new(std::sync::Arc::clone(&self.backend));
            blob_backend.put_sync(cid, bytes).map_err(|e| match e {
                benten_graph::backends::BlobError::CidMismatch { .. } => EngineError::Other {
                    code: benten_errors::ErrorCode::Unknown(
                        "E_MODULE_BYTES_CID_MISMATCH".to_string(),
                    ),
                    message: format!("blob backend re-check tripped for cid={}", cid.to_base32()),
                },
                benten_graph::backends::BlobError::Graph(g) => EngineError::Graph(g),
                other => EngineError::Other {
                    code: other.code(),
                    message: other.to_string(),
                },
            })?;

            // Mirror into the in-memory cache so the SANDBOX dispatch hot
            // path stays free of disk reads. The cache is rebuilt at engine
            // open via `rehydrate_module_bytes_from_zone`.
            let mut guard = self.inner.module_bytes.lock_recover();
            guard.insert(*cid, bytes.to_vec());
            Ok(())
        }
    }

    /// Phase-3 G14-C — public accessor for previously-registered module
    /// bytes by CID. Returns `Some(Vec<u8>)` if the bytes have been
    /// registered (or rehydrated from a prior engine open) and `None`
    /// otherwise.
    ///
    /// Pinned by `crates/benten-engine/tests/module_bytes_cid.rs::module_bytes_durable_across_engine_restart`
    /// (Compromise #17 closure end-to-end pin per §3.6b pim-2):
    /// re-opening the engine at the same store path resurrects the
    /// registered bytes via the crate-internal
    /// `rehydrate_module_bytes_from_zone` accessor.
    #[must_use]
    pub fn fetch_module_bytes(&self, cid: &Cid) -> Option<Vec<u8>> {
        // Fast path: in-memory cache (populated by `register_module_bytes`
        // and rehydrate_module_bytes_from_zone at open time).
        self.module_bytes_for(cid)
    }

    /// Phase-3 G14-C (Compromise #17 closure) — rebuild the in-memory
    /// module-bytes cache from the durable
    /// [`benten_graph::backends::RedbBlobBackend`]'s `system:ModuleBytes`
    /// zone.
    ///
    /// Mirrors `Self::rehydrate_installed_modules_from_zone` (G10-B
    /// R6FP-Group-1 r6-arch-1) for the manifest side. Called from
    /// `EngineBuilder::assemble` once after the backend opens + the
    /// engine is constructed. Failures during rehydration log via
    /// tracing and are non-fatal — a corrupt or partial Node does not
    /// block engine startup.
    ///
    /// # Errors
    ///
    /// [`EngineError::Graph`] when the backend's
    /// `get_by_label("system:ModuleBytes")` accessor errors.
    pub(crate) fn rehydrate_module_bytes_from_zone(&self) -> Result<usize, EngineError> {
        let blob_backend =
            benten_graph::backends::RedbBlobBackend::new(std::sync::Arc::clone(&self.backend));
        let blob_cids = blob_backend.list_blob_cids().map_err(EngineError::Graph)?;
        let mut guard = self.inner.module_bytes.lock_recover();
        let mut count = 0usize;
        for cid in blob_cids {
            match blob_backend.get_sync(&cid) {
                Ok(Some(bytes)) => {
                    guard.insert(cid, bytes);
                    count += 1;
                }
                Ok(None) | Err(_) => {
                    // Best-effort: skip a blob whose Node was indexed
                    // under the label but couldn't be re-read.
                }
            }
        }
        Ok(count)
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

    /// G12-A test-only: cap the evaluator's cumulative iteration budget at
    /// `budget` steps for every subsequent `engine.call` / `engine.trace`
    /// invocation on this engine. `None` clears the override, restoring
    /// [`benten_eval::evaluator::DEFAULT_ITERATION_BUDGET`]. Used by the
    /// `budget_exhausted_runtime_trace_emission` integration test (and its
    /// napi companion) to drive the Inv-8 cumulative-step guard within a
    /// CI-friendly subgraph size without mutating the production default.
    ///
    /// Gated behind the narrow `iteration-budget-test-grade` feature so the
    /// napi cdylib can opt in without dragging the rest of the
    /// `test-helpers` surface across the JS boundary. The broader
    /// `test-helpers` feature implies it so existing in-tree test
    /// invocations work unchanged.
    #[cfg(any(test, feature = "iteration-budget-test-grade"))]
    pub fn testing_set_iteration_budget(&self, budget: Option<u64>) {
        let mut guard = self.inner.test_iteration_budget.lock_recover();
        *guard = budget;
    }

    /// Wave-8f test-only: install a `Barrier` that every subsequent
    /// `Engine::call`/`Engine::trace` invocation parks on AFTER resolving
    /// `handler_cid` from the handlers map but BEFORE walking the
    /// reconstructed Subgraph. Pair with another thread that calls
    /// `testing_clear_pre_dispatch_gate` (or releases the same barrier
    /// directly) once the harness has landed an intervening
    /// `register_subgraph_replace`. Used exclusively by the in-flight
    /// hot-replace contract test in `tests/register_subgraph_replace.rs`.
    ///
    /// Pass `None` to clear the gate so subsequent calls dispatch
    /// without parking.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_set_pre_dispatch_gate(&self, gate: Option<std::sync::Arc<std::sync::Barrier>>) {
        let mut guard = self.inner.test_pre_dispatch_gate.lock_recover();
        *guard = gate;
    }

    // -------- CRUD surface (Node + Edge) --------
    //
    // CRUD methods (`create_node`, `get_node`, `update_node`, `delete_node`,
    // `create_edge`, `get_edge`, `delete_edge`, `edges_from`, `edges_to`) live
    // in [`crate::engine_crud`].

    // -------- Registration / invariants --------

    /// Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1):
    /// validate that every SANDBOX node in the supplied [`Subgraph`]
    /// declares a manifest reference that resolves through the
    /// engine's manifest registry (codegen defaults + colon-joined
    /// `<manifest>:<entry>` keys + bare `<entry>` keys from installed
    /// modules).
    ///
    /// A SANDBOX node has TWO valid manifest-reference shapes:
    ///
    /// 1. Explicit `manifest` property — a Text key naming the manifest
    ///    entry (e.g. `manifest: "compute-basic"` or
    ///    `manifest: "echo:identity"` for the colon-joined DSL form).
    /// 2. Colon-joined `module` property — a Text key whose value is
    ///    `<manifest>:<entry>` (the TS DSL surface
    ///    `subgraph(...).sandbox({ module: "echo:identity" })` writes
    ///    this shape; primitive_host.rs reads `module` as a CID first
    ///    and falls back to manifest lookup when the parse fails).
    /// 3. Inline `caps` property (bypasses the registry — escape hatch);
    ///    not validated here because there is no name to resolve.
    ///
    /// Returns [`EngineError::SandboxManifestUnknown`] for the first
    /// SANDBOX node whose manifest name does not resolve (operator-
    /// actionable: caller-supplied name + the public hint about the
    /// codegen-default names lives in the Display impl).
    #[cfg(not(target_arch = "wasm32"))]
    fn validate_sandbox_manifest_names(
        &self,
        sg: &benten_eval::Subgraph,
    ) -> Result<(), EngineError> {
        let known = self.manifest_registry_known_names();
        for node in &sg.nodes {
            if !matches!(node.kind, benten_eval::PrimitiveKind::Sandbox) {
                continue;
            }
            // Pick the manifest reference. `manifest` wins; otherwise a
            // colon-joined `module` Text falls back to the named lookup
            // path (the eval-side primitive_host applies the same
            // precedence at dispatch).
            let manifest_name: Option<String> = match node.properties.get("manifest") {
                Some(Value::Text(name)) => Some(name.clone()),
                _ => match node.properties.get("module") {
                    Some(Value::Text(s)) => {
                        // Heuristic: a colon-joined string that does
                        // not parse as a base32 CID is a manifest name
                        // (`<manifest>:<entry>` shape from the TS DSL).
                        // A successful CID parse means the caller used
                        // the inline-caps escape hatch (or supplied a
                        // raw module CID directly); skip in that case
                        // since there is no name to resolve.
                        if s.contains(':') && Cid::from_str(s).is_err() {
                            Some(s.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
            };
            if let Some(name) = manifest_name
                && !known.contains(&name)
            {
                return Err(EngineError::SandboxManifestUnknown {
                    manifest_name: name,
                });
            }
        }
        Ok(())
    }

    /// Register a subgraph. Runs the G6 invariant battery (1/2/3/5/6/9/10/12)
    /// and stores the handler id → CID association. Idempotent: re-registering
    /// a subgraph with the same handler id and identical content returns the
    /// same CID. Different content under the same handler id returns
    /// [`EngineError::DuplicateHandler`].
    #[allow(
        clippy::too_many_lines,
        reason = "register_subgraph is the engine's central registration boundary — \
                  invariant validation walk + WAIT TTL check + TRANSFORM parse + \
                  SANDBOX manifest validation + persist + in-memory swap all live \
                  here by single-source-of-truth design (matches register_subgraph_replace). \
                  Phase-3 G21-T3 added the §2.5(d) reserved-namespace guard which pushed \
                  past the 100-line clippy threshold."
    )]
    pub fn register_subgraph<S>(&self, spec: S) -> Result<String, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        // Capture an owned SubgraphSpec view for dispatch-time use when the
        // input is one (idiomatic DSL path). Non-SubgraphSpec inputs get an
        // empty spec recorded — `call()` falls through to CRUD dispatch.
        let stored_spec = spec.as_subgraph_spec();
        let sg = spec.into_eval_subgraph()?;

        // Phase-3 G21-T3 §2.5(d) (corr-minor-3 fold-in): hard reject
        // any registration whose handler_id starts with the reserved
        // `engine:typed:` namespace. Fires BEFORE invariant validation
        // so a misnamed registration has zero observable side effect
        // on engine state. The eval-side dispatch fork
        // (`crates/benten-eval/src/primitives/call.rs::execute`)
        // pre-empts user-handler routing for this prefix — typed-CALL
        // registry is closed; extension is a Rust-only engine concern
        // per CLAUDE.md baked-in commitment #16. Without this guard,
        // the user registration would be silent dead code; the
        // registration-time reject surfaces the user-error sooner
        // than the eval-time `E_TYPED_CALL_UNKNOWN_OP` would.
        if sg.handler_id().starts_with(benten_eval::TYPED_CALL_PREFIX) {
            return Err(EngineError::Other {
                code: ErrorCode::ReservedHandlerNamespace,
                message: format!(
                    "register_subgraph: handler_id `{}` is in the reserved \
                     `engine:typed:` namespace; this prefix is the typed-CALL registry \
                     (see CLAUDE.md baked-in #16 + phase-3-backlog §2.5(d)). \
                     E_RESERVED_HANDLER_NAMESPACE",
                    sg.handler_id()
                ),
            });
        }

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
        // Phase-3 G20-A2 (D12 wave-8a): WAIT TTL validation walk. Every
        // WAIT node carrying a `ttl_hours` property must satisfy
        // `1 <= ttl_hours <= 720` (30 days max). 0 would expire
        // immediately on suspend (a footgun); >720 is the documented
        // ceiling. Both reject at registration time with the typed
        // E_WAIT_TTL_INVALID error rather than at suspend time so a
        // miswritten spec doesn't survive into running state.
        for node in sg.nodes() {
            if !matches!(node.kind, benten_eval::PrimitiveKind::Wait) {
                continue;
            }
            let Some(prop) = node.property("ttl_hours") else {
                continue;
            };
            let Value::Int(raw) = prop else {
                return Err(EngineError::Other {
                    code: ErrorCode::WaitTtlInvalid,
                    message: format!(
                        "register_subgraph: WAIT node {} has non-integer ttl_hours property; \
                         expected integer in [1, 720]: E_WAIT_TTL_INVALID",
                        node.id
                    ),
                });
            };
            if (*raw < 1) || (*raw > 720) {
                return Err(EngineError::Other {
                    code: ErrorCode::WaitTtlInvalid,
                    message: format!(
                        "register_subgraph: WAIT node {} has out-of-range ttl_hours={}; \
                         expected integer in [1, 720]: E_WAIT_TTL_INVALID",
                        node.id, raw
                    ),
                });
            }
        }
        // R6 fp Wave C2 (dx-r6-r1-1 MAJOR closure half): SANDBOX
        // numeric-budget shape validation. Mirrors the dsl-compiler's
        // `validate_shapes` pass on the engine boundary so non-DSL
        // registration paths (programmatic `Subgraph::with_node`
        // construction, fixture loaders) reject mis-typed properties
        // with the same typed `E_DSL_INVALID_SHAPE` surface. Properties
        // listed in `docs/SANDBOX-LIMITS.md` §2 MUST be non-negative
        // integers; anything else trips the typed error rather than
        // surfacing as an opaque wasmtime config rejection downstream.
        for node in sg.nodes() {
            if !matches!(node.kind, benten_eval::PrimitiveKind::Sandbox) {
                continue;
            }
            for &key in &["fuel", "wallclock_ms", "wallclock", "output_limit_bytes"] {
                let Some(prop) = node.property(key) else {
                    continue;
                };
                match prop {
                    Value::Int(n) if *n >= 0 => {}
                    Value::Int(n) => {
                        return Err(EngineError::Other {
                            code: ErrorCode::DslInvalidShape,
                            message: format!(
                                "register_subgraph: SANDBOX node `{}` property `{}` must be a non-negative integer (got {}); see docs/SANDBOX-LIMITS.md §2: E_DSL_INVALID_SHAPE",
                                node.id, key, n,
                            ),
                        });
                    }
                    other => {
                        return Err(EngineError::Other {
                            code: ErrorCode::DslInvalidShape,
                            message: format!(
                                "register_subgraph: SANDBOX node `{}` property `{}` must be a non-negative integer (got {:?}); see docs/SANDBOX-LIMITS.md §2: E_DSL_INVALID_SHAPE",
                                node.id, key, other,
                            ),
                        });
                    }
                }
            }
        }
        // 5d-J workstream 3: parse every TRANSFORM node's expression at
        // registration time so an unparseable grammar trips `register_*`
        // rather than surviving to `engine.call`. The runtime executor
        // path now consults [`crate::ast_cache::AstCache`] for the
        // pre-parsed AST (G19-E / wave-7b — closes phase-2-backlog
        // §9.2): the cache is populated below via
        // `ast_cache.populate_for_handler` once the `handler_cid` is
        // known.
        benten_eval::invariants::validate_transform_expressions(&sg).map_err(|e| {
            EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            }
        })?;
        // Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1):
        // SANDBOX manifest-name validation walk. Every SANDBOX node that
        // declares a named manifest reference (via `manifest` property
        // OR a colon-joined `<manifest>:<entry>` `module` property) MUST
        // resolve through the engine's manifest registry — otherwise
        // a misspelled name + post-uninstall residual reference would
        // hide as a wallclock-after-zero-progress shape at execution
        // time. The validation walk is cfg-gated NOT-wasm32 to match
        // `manifest_registry_known_names()` (CLAUDE.md baked-in #16:
        // browser thin clients do not run SANDBOX).
        #[cfg(not(target_arch = "wasm32"))]
        self.validate_sandbox_manifest_names(&sg)?;
        let cid = sg.cid().map_err(EngineError::Core)?;
        let handler_id = sg.handler_id().to_string();

        // g14-c-mr-4 BLOCKER fix-pass: persist BEFORE the in-memory
        // swap. The same crash-window risk as register_subgraph_replace
        // applies here on the first-register path. Preview the chain
        // state under a short read-only lock; persist outside the
        // lock; then commit to the in-memory tables.
        //
        // Two early-exit shapes WITHOUT persist:
        //   (a) handler already registered at the same CID — idempotent
        //   (b) handler already registered at a DIFFERENT CID — error
        // Only the "first registration" branch needs persist.
        let preview_existing: Option<Cid>;
        {
            let handlers_guard = self.inner.handlers.lock_recover();
            preview_existing = handlers_guard.get(&handler_id).copied();
        }
        let preview_replaced = match preview_existing {
            Some(existing) if existing == cid => false,
            Some(_) => {
                return Err(EngineError::DuplicateHandler { handler_id });
            }
            None => true,
        };

        if preview_replaced {
            // seq=0 (first registration), no predecessor.
            self.persist_handler_version_entry(&handler_id, &cid, None, 0)?;
        }

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
        let mut vc = self.inner.handler_version_chain.lock_recover();
        let chain = vc.entry(handler_id.clone()).or_default();
        if chain.first() != Some(&cid) {
            chain.insert(0, cid);
        }
        drop(vc);

        // G19-E (phase-2-backlog §9.2 closure): populate the per-handler
        // TRANSFORM AST cache so subsequent dispatches via `Engine::call`
        // skip the per-call parse step. Idempotent re-registration
        // (same CID) re-inserts the same Expr instances under the same
        // `(handler_cid, node_id)` key — cheap and correct.
        self.inner.ast_cache.populate_for_handler(&cid, &sg);

        Ok(handler_id)
    }

    /// Replace the subgraph registered under the same `handler_id` with a
    /// new content-addressed body. Phase 2b Wave-8f.
    ///
    /// Unlike [`Engine::register_subgraph`], this method DOES accept a
    /// different CID under the same `handler_id` and bumps the handler's
    /// version chain accordingly — the caller's intent is "hot-reload this
    /// handler". The previous CID becomes the new CID's predecessor on the
    /// in-memory `handler_version_chain` so callers (devserver hot-reload,
    /// audit consumers) can name what was replaced.
    ///
    /// Idempotency: re-registering identical content (same CID under the
    /// same `handler_id`) returns the same CID and does NOT bump the
    /// version chain — the call is a no-op except for re-running the
    /// invariant battery. This matches `register_subgraph`'s idempotence
    /// shape so dev-server fingerprint races (file mtime tick without
    /// content change) don't grow the chain.
    ///
    /// The same G6 invariant battery + TRANSFORM-syntax fail-fast that
    /// `register_subgraph` runs is run here. The newly-registered CID
    /// becomes the live dispatch target on `Engine::call(handler_id, ...)`.
    ///
    /// **In-flight evaluation contract:** in-flight `Engine::call`
    /// invocations DO NOT see the swap. The dispatch path resolves
    /// `handler_cid` once at call entry; the spec Mutex re-lookup at
    /// `dispatch_call_inner` uses that CID as the third axis of the
    /// subgraph-cache key, so the reconstructed `Subgraph` reflects the
    /// CID resolved at call entry rather than the post-swap CID. The swap
    /// is observed by NEXT calls only. (Full subscription-store-style
    /// pre-swap-suspension drain is out of scope for Phase 2b — the
    /// registered CID swap is atomic from the table's perspective; mid-call
    /// behaviour is bounded by the call's own snapshot.)
    ///
    /// **Concurrency / lock-ordering invariant:** the `handlers` Mutex,
    /// the `specs` Mutex, and the `handler_version_chain` Mutex are
    /// acquired in that fixed order and held jointly across the
    /// swap+spec-update+chain-prepend sequence. This keeps the
    /// `handlers`-table swap order in lockstep with the version-chain
    /// prepend order under concurrent writers (without it, two racing
    /// `register_subgraph_replace` calls against the same `handler_id`
    /// could land their handler-table swap in one order while their
    /// version-chain prepend lands in the other, violating the chain's
    /// newest-first invariant that `handler_version_chain()` reports).
    ///
    /// **Version chain in Phase 2b (Compromise #18):** in-memory only.
    /// The chain is a process-local `BTreeMap<HandlerId, Vec<Cid>>` that
    /// is NOT written to redb; on `Engine::open` it starts empty
    /// regardless of how many replace calls happened in the prior
    /// process. Phase 3 lifts this to a durable
    /// `core::version::Anchor` + Version-Node chain so reload audit
    /// survives engine restart. See `docs/SECURITY-POSTURE.md`
    /// "Compromise #18 — In-memory handler-version chain" for the
    /// full narrative + sibling relationship to Compromise #17 (the
    /// module-bytes registry; same Phase-3 promotion path, different
    /// audit class).
    ///
    /// # Errors
    /// - [`EngineError::Invariant`] / [`EngineError::Other`] on the same
    ///   conditions `register_subgraph` raises.
    /// - This method does NOT raise [`EngineError::DuplicateHandler`] —
    ///   that's the whole point.
    pub fn register_subgraph_replace<S>(
        &self,
        spec: S,
    ) -> Result<RegisterReplaceOutcome, EngineError>
    where
        S: IntoSubgraphSpec,
    {
        let stored_spec = spec.as_subgraph_spec();
        let sg = spec.into_eval_subgraph()?;

        // Phase-3 G21-T3 §2.5(d) (corr-minor-3 fold-in): hard reject
        // re-registration of a handler whose handler_id is in the
        // reserved `engine:typed:` namespace; same rationale as
        // `register_subgraph` above. Defense-in-depth: even if a
        // future change accidentally bypassed the `register_subgraph`
        // check, the replace path still rejects. Fires BEFORE
        // invariant validation so a misnamed replace has zero side
        // effect.
        if sg.handler_id().starts_with(benten_eval::TYPED_CALL_PREFIX) {
            return Err(EngineError::Other {
                code: ErrorCode::ReservedHandlerNamespace,
                message: format!(
                    "register_subgraph_replace: handler_id `{}` is in the reserved \
                     `engine:typed:` namespace; this prefix is the typed-CALL registry \
                     (see CLAUDE.md baked-in #16 + phase-3-backlog §2.5(d)). \
                     E_RESERVED_HANDLER_NAMESPACE",
                    sg.handler_id()
                ),
            });
        }

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
        benten_eval::invariants::validate_transform_expressions(&sg).map_err(|e| {
            EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            }
        })?;
        let cid = sg.cid().map_err(EngineError::Core)?;
        let handler_id = sg.handler_id().to_string();

        // g14-c-mr-4 BLOCKER fix-pass: persist BEFORE the in-memory
        // swap so a backend write failure surfaces as a clean Err to
        // the caller and the in-memory chain stays consistent with
        // disk. Pre-fix the durable write happened AFTER lock release
        // — a process crash between in-memory swap and durable write
        // produced a "ghost" current-version: the caller saw Ok, but
        // a subsequent Engine::open rebuild from disk silently
        // dropped the most recent replace entry (the audit-trail
        // erasure shape Compromise #18 was supposed to close).
        //
        // Mechanism: under a SHORT preview lock we read the previous
        // CID (if any) + the current chain depth, decide whether
        // replace would grow the chain, and compute the seq the new
        // entry would occupy. We THEN persist outside the locks (the
        // privileged write is a single put_node_with_context — no
        // long redb transaction). Only after persist succeeds do we
        // re-acquire the locks and apply the in-memory swap. If a
        // racing writer commits between the preview and the apply,
        // we re-detect via a second preview-and-apply pass under the
        // joint lock; the seq we just persisted may end up not
        // matching the in-memory chain if a racing writer prepended
        // first, but that's safe because the rehydrate path sorts by
        // seq + the racing writer's persist will land its own seq.
        //
        // Lock ordering invariant preserved: handlers → specs →
        // version_chain (the joint-lock acquisition for the mutation
        // pass keeps the swap order consistent under concurrent
        // writers).
        let previous_cid: Option<Cid>;
        let replaced;
        let chain_depth_before;
        {
            let handlers_guard = self.inner.handlers.lock_recover();
            let chain_guard = self.inner.handler_version_chain.lock_recover();
            previous_cid = handlers_guard.get(&handler_id).copied();
            replaced = previous_cid != Some(cid);
            chain_depth_before = chain_guard.get(&handler_id).map_or(0, std::vec::Vec::len);
        }

        // Compromise #18 closure: persist FIRST. Idempotent re-register
        // (same CID under same handler_id) is a no-op for the chain;
        // skip the persist call to match the in-memory contract.
        if replaced {
            let persist_predecessor = previous_cid;
            // seq is the position the new entry occupies AFTER insert
            // (newest-first map; seq=0 is the oldest, seq=len-1 is
            // the newest just inserted). chain_depth_before is the
            // length BEFORE the prepend; the new entry's seq is
            // chain_depth_before (zero-indexed) — i.e. the count of
            // entries that already exist for this handler.
            let persist_seq = u64::try_from(chain_depth_before).unwrap_or(u64::MAX);
            self.persist_handler_version_entry(
                &handler_id,
                &cid,
                persist_predecessor.as_ref(),
                persist_seq,
            )?;
        }

        // Atomically: read the previous CID (if any), swap the handlers
        // entry, update the spec table, then prepend onto the version
        // chain. The three locks are acquired in fixed order — handlers,
        // specs, version_chain — and HELD JOINTLY across the entire
        // mutation so the handlers-table swap order matches the version-
        // chain prepend order under concurrent writers. Without this,
        // racing `register_subgraph_replace` calls against the same
        // handler_id could land their handler-table swap in one order
        // and their chain prepend in the other, violating the
        // newest-first invariant `handler_version_chain()` reports +
        // the 7 dedicated tests assume.
        let chain_depth: usize;
        {
            let mut handlers_guard = self.inner.handlers.lock_recover();
            let mut specs_guard = self.inner.specs.lock_recover();
            let mut chain_guard = self.inner.handler_version_chain.lock_recover();

            let prev = handlers_guard.get(&handler_id).copied();
            let did_replace = match prev {
                Some(existing) if existing == cid => false,
                _ => {
                    handlers_guard.insert(handler_id.clone(), cid);
                    true
                }
            };

            if let Some(spec) = stored_spec {
                specs_guard.insert(handler_id.clone(), spec);
            }

            if did_replace {
                let chain = chain_guard.entry(handler_id.clone()).or_default();
                // Idempotency guard: don't re-prepend if the chain already
                // leads with this CID (defence against racing replace-
                // with-same-content writers — the handlers swap above is
                // the arbiter).
                if chain.first() != Some(&cid) {
                    chain.insert(0, cid);
                }
            }

            // Chain depth — used by audit consumers to name "v1 / v2 /
            // v3 / …" at a glance; matches `tools/benten-dev`'s legacy
            // version-tag convention. Read while still under the joint
            // lock to keep the reported depth consistent with the swap
            // we just performed.
            chain_depth = chain_guard.get(&handler_id).map_or(1, std::vec::Vec::len);
        }

        // G19-E (phase-2-backlog §9.2 closure): the AST cache is keyed
        // on `handler_cid`, so the replaced version's entries become
        // unreachable on key change. Drop them explicitly so a future
        // call into the OLD CID via the version-chain audit surface
        // doesn't reach a stale parse, then re-populate for the new CID.
        // Idempotent same-CID re-register is a no-op for the cache —
        // populate_for_handler simply re-inserts the same Expr instances
        // under the same key.
        if let Some(prev_cid) = previous_cid
            && prev_cid != cid
        {
            self.inner.ast_cache.invalidate_handler(&prev_cid);
        }
        self.inner.ast_cache.populate_for_handler(&cid, &sg);

        Ok(RegisterReplaceOutcome {
            handler_id,
            cid,
            previous_cid: if replaced { previous_cid } else { None },
            chain_depth,
        })
    }

    /// Return the in-memory version chain for `handler_id` (newest first),
    /// or an empty slice when the handler isn't registered. Phase 2b
    /// Wave-8f introspection helper for devserver + audit consumers.
    #[must_use]
    pub fn handler_version_chain(&self, handler_id: &str) -> Vec<Cid> {
        self.inner
            .handler_version_chain
            .lock_recover()
            .get(handler_id)
            .cloned()
            .unwrap_or_default()
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
    // TODO(phase-3 — register_crud_with_grants GrantBackedPolicy
    // routing): route through GrantBackedPolicy registration so the
    // handler honours grants at call-time. Carried from Phase-2
    // generic marker; pairs with the broader Phase-3 grant-backed
    // policy work (§2.1 Durable UCAN backend).
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
            // R6 fp Wave C2 (dx-r6-r1-1): typed `DslUnregisteredHandler`
            // mirrors the TS-side `EDslUnregisteredHandler` contract so
            // operators routing on `ON_NOT_FOUND` see the same typed
            // dispatch from Rust + TS code paths.
            return Err(EngineError::Other {
                code: ErrorCode::DslUnregisteredHandler,
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
                code: ErrorCode::DslUnregisteredHandler,
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
            // R6 fp Wave C2 (dx-r6-r1-1): typed `DslUnregisteredHandler`
            // routes through `ON_NOT_FOUND` mirroring the TS-side
            // `EDslUnregisteredHandler` contract.
            return Err(EngineError::Other {
                code: ErrorCode::DslUnregisteredHandler,
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
                code: ErrorCode::DslUnregisteredHandler,
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
            // R6 fp Wave C2 (dx-r6-r1-1): typed `DslUnregisteredHandler`
            // mirrors the TS-side `EDslUnregisteredHandler` thrown by
            // `packages/engine/src/engine.ts::call`'s
            // `knownHandlers.has(handlerId)` short-circuit. Routes via
            // `ON_NOT_FOUND`.
            return Err(EngineError::Other {
                code: ErrorCode::DslUnregisteredHandler,
                message: format!("handler not registered: {handler_id}"),
            });
        };

        // Reentrancy guard — set the active-call state so `impl PrimitiveHost`
        // can pick up the actor / op metadata without threading it through
        // the trait methods.
        {
            let mut guard = self.active_call.lock_recover();
            // R6FP-Group-1 (r6-cr-1 / r6-mpc-4 / r6-wsa-1): inherit
            // sandbox_depth from the parent frame (when there is one)
            // so a SANDBOX→handler→SANDBOX chain advances the count
            // correctly at each handler entry. The execute_sandbox
            // override layers a `+1` at SANDBOX entry on top of this
            // baseline; together they produce the cumulative nest
            // depth the eval-side runtime arm consults to fire
            // `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED`.
            let parent_sandbox_depth = guard.last().map_or(0, |f| f.sandbox_depth);
            guard.push(ActiveCall {
                handler_id: handler_id.to_string(),
                op: op.to_string(),
                actor,
                handler_cid: Some(handler_cid),
                pending_ops: Vec::new(),
                inject_failure: false,
                last_refresh: None,
                iteration: 0,
                sandbox_depth: parent_sandbox_depth,
            });
        }

        // Wave-8f test-only call gate: park here so the harness can land
        // a `register_subgraph_replace` between the handler_cid capture
        // (above) and the Subgraph reconstruction (below). Production
        // callers never set the gate so the lookup is a single Mutex
        // peek that returns `None`. The clone keeps the gate Mutex
        // unlocked while waiting so the harness thread can re-acquire
        // it without deadlocking.
        let pre_dispatch_gate = self.inner.test_pre_dispatch_gate.lock_recover().clone();
        if let Some(barrier) = pre_dispatch_gate {
            barrier.wait();
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
    // (the actor is now read from `self.active_call` inside
    // `dispatch_call_inner` rather than passed in by the caller).
    // Removed; the callable dispatch helper now
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
            // R6 fp Wave C2 (dx-r6-r1-1): typed `DslUnregisteredHandler`
            // — handler-id has no spec AND is not a `crud:` synth.
            return Err(EngineError::Other {
                code: ErrorCode::DslUnregisteredHandler,
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
        // G12-A: read the test-only iteration-budget override (set by
        // `Engine::testing_set_iteration_budget`); fall back to
        // `benten_eval::DEFAULT_ITERATION_BUDGET` when unset. Lets the
        // `budget_exhausted_runtime_trace_emission` integration test trip
        // the Inv-8 cumulative-step guard within a small chained subgraph.
        let iteration_budget = {
            let guard = self.inner.test_iteration_budget.lock_recover();
            (*guard).unwrap_or(benten_eval::evaluator::DEFAULT_ITERATION_BUDGET)
        };
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
                sandbox_depth: 0,
                ..Default::default()
            };
            // G12-A: route through the budget-aware capturing variant so
            // the terminal `TraceStep::BudgetExhausted` row pushed by the
            // evaluator before short-circuiting on Inv-8 cumulative-step
            // exhaustion (and any future runtime emissions on error paths)
            // reaches the user-visible `engine.trace(...)` consumer instead
            // of being dropped here. The `iteration_budget` resolves to the
            // production default unless `Engine::testing_set_iteration_budget`
            // applied a test override.
            evaluator.run_with_trace_attributed_capturing_with_budget(
                &subgraph,
                input_value,
                self as &dyn PrimitiveHost,
                frame,
                iteration_budget,
            )
        } else {
            (
                evaluator.run_with_budget(
                    &subgraph,
                    input_value,
                    self as &dyn PrimitiveHost,
                    iteration_budget,
                ),
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
                // G12-A: in trace mode, surface the Inv-8 cumulative-step
                // exhaustion as an Ok(error_outcome) so `engine.trace(...)`
                // returns the captured trace (which already carries the
                // terminal `TraceStep::BudgetExhausted` row pushed by the
                // evaluator) instead of dropping it on an `Err` return.
                // Non-trace `engine.call` paths still surface the typed
                // `EngineError` for back-compat.
                if trace_mode
                    && matches!(
                        &e,
                        benten_eval::EvalError::Invariant(
                            benten_eval::InvariantViolation::IterateBudget,
                        )
                    )
                {
                    return Ok(crate::primitive_host::inv_iterate_budget_to_outcome());
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

    // The body is a 4-pass pipeline (cache lookup → walk primitives →
    // post-build WRITE bag → post-build non-WRITE bag → cache insert) over
    // shared state (`sb`, `last`, `had_terminal_respond`, `write_ops`,
    // `other_ops`, `sg`). Extracting any single pass into a helper would
    // require threading 5+ parameters through and would not improve
    // readability. The 104-line length is intrinsic to the pipeline shape.
    #[allow(clippy::too_many_lines)]
    fn subgraph_for_spec(
        &self,
        spec: &SubgraphSpec,
        op: &str,
        _input: &Node,
        handler_cid: &Cid,
    ) -> Result<benten_eval::Subgraph, EngineError> {
        // Phase 2b G12-D: walk `spec.primitives` (the widened storage) to
        // materialise the runnable Subgraph. Each PrimitiveSpec contributes
        // one OperationNode of its declared kind; consecutive primitives
        // chain via `add_edge` so the walker steps through them in
        // registration order.
        //
        // Cache eligibility (Phase 2a G2-B / arch-r1-5): WRITE-free specs
        // only — WRITE-bearing specs stamp per-call `createdAt`, which would
        // poison a content-addressed cache. Determined by scanning the
        // widened `primitives` storage for any kind=Write entry.
        let has_write = spec
            .primitives
            .iter()
            .any(|p| matches!(p.kind, benten_eval::PrimitiveKind::Write));
        let cache_eligible = !has_write;
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
        // Track WRITE configs for post-build property-bag population. We
        // must walk write-spec rehydration via `WriteSpec::from_primitive_spec`
        // because the per-node `op`/`label`/`properties`/`createdAt` keys
        // the evaluator dispatch path expects live in the OperationNode's
        // properties map, NOT in the PrimitiveSpec bag (which uses the
        // `WRITE_PROP_*` keys for inter-spec config carry).
        let mut write_ops: Vec<(String, WriteSpec)> = Vec::new();
        // Two post-build property-population passes are needed because
        // SubgraphBuilder doesn't surface per-node property setters to
        // outside-the-crate callers. WRITE entries take the WriteSpec
        // rehydration shape (op/label/properties/createdAt — see the loop
        // below this walker). Non-WRITE entries with a `PrimitiveSpec.properties`
        // bag get a flat per-key copy onto the OperationNode (no inter-spec
        // shape transform); collect them here so future G6/G7/G10 SUBSCRIBE
        // /WAIT/SANDBOX entries can declare config alongside their `kind` and
        // have it visible at dispatch time.
        let mut other_ops: Vec<(String, BTreeMap<String, Value>)> = Vec::new();
        let mut had_terminal_respond = false;
        for ps in &spec.primitives {
            let h = match ps.kind {
                benten_eval::PrimitiveKind::Write => {
                    let h = sb.write(ps.id.clone());
                    write_ops.push((ps.id.clone(), WriteSpec::from_primitive_spec(ps)));
                    h
                }
                benten_eval::PrimitiveKind::Read => sb.read(ps.id.clone()),
                benten_eval::PrimitiveKind::Respond => {
                    // RESPOND must follow a predecessor; if it's first, fall
                    // through to the empty-spec branch below by leaving
                    // `last` unchanged. (Registered SubgraphSpec callers
                    // should never produce this; the .respond() builder is
                    // typically chained AFTER .write() entries.)
                    let Some(prev) = last else {
                        // Skip a leading-RESPOND defensively — it would
                        // produce a degenerate single-node subgraph; the
                        // empty-spec fallback below handles cleanly.
                        continue;
                    };
                    had_terminal_respond = true;
                    sb.respond(prev)
                }
                other => {
                    if !ps.properties.is_empty() {
                        other_ops.push((ps.id.clone(), ps.properties.clone()));
                    }
                    sb.push_primitive(ps.id.clone(), other)
                }
            };
            if let Some(prev) = last {
                // RESPOND already chained via sb.respond(prev); skip the
                // extra add_edge to avoid a duplicate edge.
                if !matches!(ps.kind, benten_eval::PrimitiveKind::Respond) {
                    sb.add_edge(prev, h);
                }
            }
            last = Some(h);
        }
        // If the builder didn't already produce a terminal RESPOND, supply
        // one so the dispatch path always has a terminator. Empty `primitives`
        // → noop_read+respond (preserves the prior empty-write_specs
        // semantics). Non-empty without a trailing RESPOND → append.
        if !had_terminal_respond {
            let terminal = match last {
                Some(prev) => sb.respond(prev),
                None => {
                    let r = sb.read("noop_read".to_string());
                    sb.respond(r)
                }
            };
            let _ = terminal;
        }
        let mut sg = sb.build_unvalidated_for_test();
        // Populate WRITE property bags post-build — SubgraphBuilder doesn't
        // surface per-node property setters for callers outside the crate.
        // Reads `(id, WriteSpec)` pairs collected during the walk so each
        // OperationNode's `op` / `label` / `properties` keys reflect the
        // caller's WriteSpec configuration.
        for (id, w) in &write_ops {
            if let Some(node) = sg.op_by_id_mut(id) {
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
        // Non-WRITE primitive property propagation (FP fix cr-g12d-mr-1):
        // SUBSCRIBE/WAIT/SANDBOX entries declared via `primitive_with_props`
        // carry their config in `PrimitiveSpec.properties`; propagate the
        // bag flat onto the OperationNode so dispatch sees what registration
        // declared. WRITE keeps its own loop above (different shape: op +
        // label + nested properties + createdAt stamping).
        for (id, props) in &other_ops {
            if let Some(node) = sg.op_by_id_mut(id) {
                for (key, val) in props {
                    node.properties.insert(key.clone(), val.clone());
                }
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
//
// G13-C BLOCKER-2 fix-pass: gated to NOT-`browser-backend` since
// `benten_graph::Transaction` is itself cfg-gated out on wasm32 per
// CLAUDE.md baked-in #17 (BrowserBackend has a no-op transaction runner).
#[cfg(not(feature = "browser-backend"))]
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
///
/// **Phase 2b G8-B:** user-registered IVM views (registered via
/// [`Engine::create_user_view`]) are recognized by the live-subscriber
/// branch in `read_view_with` (the subscriber's `view_ids()` set tracks
/// every registered view). This whitelist remains for the 5 hand-written
/// canonical ids that don't auto-register a live subscriber on engine
/// open; full removal waits on the G8-A Algorithm B port exposing a
/// definition-registry API on the IVM subscriber.
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

// ---------------------------------------------------------------------------
// Phase-3 G20-A2 (D12 wave-8a): WAIT TTL GC machinery — Engine impl + Drop.
//
// The GC implementation lives in `crate::wait_ttl_gc`; the methods below
// expose the entry points that the wait surface (engine_wait.rs) calls
// from the suspend / resume hot paths plus the test-only helpers under
// `cfg(any(test, feature = "test-helpers"))`.
//
// Scheduling correctness: the three sweep paths (event-driven, interval-
// backstop, drop-final) are documented in `wait_ttl_gc.rs`; the
// integration-correctness pin is `tests/wait_ttl_runtime_expiry_path_gc_machinery_correct`.
// ---------------------------------------------------------------------------

impl<B: GraphBackend> EngineGeneric<B> {
    /// Phase-3 G20-A2 (D12 wave-8a): record `cid` in the engine's tracked
    /// envelope set so subsequent GC sweeps reach it. Idempotent.
    /// Crate-private — only the suspend hooks in engine_wait.rs call this.
    pub(crate) fn wait_ttl_track_envelope(&self, cid: Cid) {
        let mut tracked = self.wait_ttl_tracked_envelopes.lock_recover();
        tracked.insert(cid);
    }

    /// Phase-3 G20-A2 (D12 wave-8a): drop `cid` from the engine's tracked
    /// envelope set. Called from the resume hot path after a successful
    /// reap (so the next sweep doesn't re-walk the already-gone entry).
    pub(crate) fn wait_ttl_untrack_envelope(&self, cid: &Cid) {
        let mut tracked = self.wait_ttl_tracked_envelopes.lock_recover();
        tracked.remove(cid);
    }

    /// Phase-3 G20-A2 (D12 wave-8a): event-driven sweep entry point.
    /// No-op when event-driven GC has been disabled
    /// (`EngineBuilder::gc_event_driven(false)` in production; the
    /// disabled-path is exercised by the
    /// `wait_gc_disabled_event_driven_still_works_via_interval` test).
    pub(crate) fn wait_ttl_run_event_driven_sweep_if_enabled(&self) -> u64 {
        if self
            .wait_ttl_gc_event_driven_disabled
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return 0;
        }
        let now =
            crate::wait_ttl_gc::wallclock_now_ms(*self.wait_wall_clock_override_ms.lock_recover());
        let mut tracked = self.wait_ttl_tracked_envelopes.lock_recover();
        let mut stats = self.wait_ttl_gc_stats.lock_recover();
        crate::wait_ttl_gc::run_event_driven_sweep(
            &self.suspension_store,
            &mut tracked,
            now,
            &mut stats,
        )
    }

    /// Phase-3 G20-A2 (D12 wave-8a): interval-backstop sweep entry point.
    /// Tests drive it synchronously via
    /// `testing_run_wait_ttl_gc_pass`. Production tokio-interval wiring
    /// (a 1h timer registered at `EngineBuilder::build`) is documented-
    /// deferred to `docs/future/phase-3-backlog.md §7.14` per G20-A2
    /// wave-8a mr-6 — the resume-time deadline check at
    /// `engine_wait.rs::resume_from_bytes_inner` is the load-bearing
    /// correctness mechanism (fires `E_WAIT_TTL_EXPIRED` independently
    /// of whether GC ran first); the interval backstop hardens
    /// disk-usage on idle engines but does not gate correctness.
    pub fn wait_ttl_run_interval_tick(&self) -> u64 {
        let now =
            crate::wait_ttl_gc::wallclock_now_ms(*self.wait_wall_clock_override_ms.lock_recover());
        let mut tracked = self.wait_ttl_tracked_envelopes.lock_recover();
        let mut stats = self.wait_ttl_gc_stats.lock_recover();
        crate::wait_ttl_gc::run_interval_tick(&self.suspension_store, &mut tracked, now, &mut stats)
    }

    /// Phase-3 G20-A2 (D12 wave-8a): drop-final sweep entry point.
    /// Called from `Engine::drop` so an explicit shutdown leaves the
    /// suspension store in the same shape an interval-tick would
    /// eventually produce.
    pub(crate) fn wait_ttl_run_drop_final_sweep(&self) -> u64 {
        let now =
            crate::wait_ttl_gc::wallclock_now_ms(*self.wait_wall_clock_override_ms.lock_recover());
        let mut tracked = self.wait_ttl_tracked_envelopes.lock_recover();
        let mut stats = self.wait_ttl_gc_stats.lock_recover();
        crate::wait_ttl_gc::run_drop_final_sweep(
            &self.suspension_store,
            &mut tracked,
            now,
            &mut stats,
        )
    }

    /// Phase-3 G20-A2 (D12 wave-8a): test-only — observable WAIT TTL GC
    /// stats snapshot. Enables `wait_ttl_runtime_expiry_path_gc_machinery_correct`
    /// to assert "the GC actually ran + reaped N entries" rather than
    /// only sentinel state.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_wait_ttl_gc_stats(&self) -> crate::WaitTtlGcStats {
        self.wait_ttl_gc_stats.lock_recover().clone()
    }

    /// Phase-3 G20-A2 (D12 wave-8a): test-only — drive the GC interval
    /// backstop synchronously. Returns the number of envelopes reaped
    /// by the call.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_run_wait_ttl_gc_pass(&self) -> u64 {
        self.wait_ttl_run_interval_tick()
    }

    /// Phase-3 G20-A2 (D12 wave-8a): test-only — set / unset the wall-
    /// clock override (UNIX-epoch ms). Drives the
    /// `testing_advance_wait_clock` helper so tests can simulate TTL
    /// expiry without real wall-clock latency.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_set_wait_wall_clock_override_ms(&self, ms: Option<u64>) {
        let mut g = self.wait_wall_clock_override_ms.lock_recover();
        *g = ms;
    }

    /// Phase-3 G20-A2 (D12 wave-8a): test-only — advance the wall-clock
    /// override by `delta`. If no override is set, seeds from the
    /// current system time. Used by
    /// `benten_engine::testing::testing_advance_wait_clock`.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_advance_wait_clock_by(&self, delta: std::time::Duration) {
        let mut g = self.wait_wall_clock_override_ms.lock_recover();
        let base = g.unwrap_or_else(|| crate::wait_ttl_gc::wallclock_now_ms(None));
        let delta_ms = u64::try_from(delta.as_millis()).unwrap_or(u64::MAX);
        *g = Some(base.saturating_add(delta_ms));
    }

    /// Phase-3 G20-A2 (D12 wave-8a): test-only — toggle event-driven GC
    /// path. When set to `true` the event-driven sweep on suspend /
    /// resume is suppressed; the interval backstop + Engine::drop final
    /// sweeps still fire. Used by
    /// `wait_gc_disabled_event_driven_still_works_via_interval`.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_set_event_driven_gc_disabled(&self, disabled: bool) {
        self.wait_ttl_gc_event_driven_disabled
            .store(disabled, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<B: GraphBackend> Drop for EngineGeneric<B> {
    fn drop(&mut self) {
        // Phase-3 G20-A2 (D12 wave-8a): final WAIT TTL GC sweep before
        // the SuspensionStore handle releases. Best-effort — we discard
        // the reap-count return value (a `u64` from
        // `wait_ttl_run_drop_final_sweep`; the sweep helpers are
        // infallible and `lock_recover` recovers poisoned mutexes
        // explicitly so no panic crosses this boundary in practice). If
        // a future change introduces a fallible path here it must wrap
        // the call in `std::panic::catch_unwind` rather than relying on
        // implicit silencing — Drop panics abort the process under the
        // C++-style two-panic rule.
        let _: u64 = self.wait_ttl_run_drop_final_sweep();
    }
}
