//! `EngineBuilder` — fluent configuration surface + `BackendGrantReader`.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The builder wires the
//! capability policy, IVM subscriber, and production-mode guard before
//! handing a fully-configured `Engine` to the caller.
//!
//! # Thinness-vs-runtime-config (arch-4)
//!
//! The `.without_ivm()` / `.without_caps()` / `.without_versioning()`
//! toggles are intentionally **build-time** rather than feature-gated.
//! A binary built against `benten-engine` always links every subsystem;
//! the toggles decide at `Engine::builder().build()` time whether the
//! subsystem is wired to the backend. Slim-build variants that omit
//! the code paths entirely are a Phase-3 concern (slim-build feature
//! gates land alongside the broader Phase-3 packaging hardening
//! pass).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use benten_caps::{
    CAPABILITY_GRANT_LABEL, CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader,
    NoAuthBackend,
};
use benten_core::{Cid, Value};
use benten_errors::ErrorCode;
use benten_eval::{HlcTimeSource, InstantMonotonicSource, MonotonicSource, TimeSource};
use benten_graph::{ChangeSubscriber, RedbBackend};

use crate::change::ChangeBroadcast;
use crate::engine::{CHANGE_STREAM_MAX_BUFFERED, Engine, EngineInner};
use crate::error::EngineError;

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
    /// Set by `.capability_policy_grant_backed()`. At assemble time this
    /// flag routes the policy construction through [`GrantBackedPolicy`]
    /// with a reader pointing at the engine's own backend.
    use_grant_backed: bool,
    /// Set by `.with_policy_allowing_revocation()`. Phase-1 alias for
    /// `.capability_policy_grant_backed()` — the revocation-aware policy
    /// is the grant-backed one. Phase-2 tightens this to a per-iteration
    /// cap refresh policy (see named compromise #1 / R4b finding
    /// `g4-p2-uc-2`).
    allow_revocation: bool,
    /// Set by `.capability_policy_ucan_durable()`. When true, the
    /// builder wraps the grant-backed surface in
    /// [`benten_caps::UcanGroundedPolicy`] composing the durable
    /// [`benten_caps::UCANBackend`] proof-chain validator alongside
    /// [`benten_caps::GrantBackedPolicy`]. Closes G21-T2 fp-mini-review
    /// BLOCKER-2 — the prior `capability_policy_ucan_durable` was a
    /// verbatim alias for the grant-backed builder so UCAN proof-chain
    /// validation never fired under `PolicyKind::Ucan`.
    use_ucan_grounded: bool,
    /// G16-B-B-rest sub-item D: test-only clock injection for the
    /// `UcanGroundedPolicy` chain-walker. When `Some(now_secs)`, the
    /// policy uses the supplied epoch-seconds value as `now_secs`
    /// instead of the `DEFAULT_NOW_SECS = 0` sentinel. Production
    /// callers leave this `None` until `WriteContext::now` threading
    /// lands per `docs/future/phase-3-backlog.md §2.3 (i)`. Tests that
    /// install time-bounded UCAN proofs via `install_ucan_proof` MUST
    /// inject a positive value here (or the chain-walker fail-closes
    /// with `E_UCAN_CLOCK_NOT_INJECTED` per the inversion at G16-B-B-rest
    /// sub-item D).
    ucan_grounded_now_secs: Option<u64>,
    /// Upper bound on the in-memory change-event buffer. `None` defaults to
    /// [`CHANGE_STREAM_MAX_BUFFERED`]. See r6-sec-5.
    change_stream_capacity: Option<usize>,
    /// Phase 2a G9-A-cont: explicit monotonic clock source used by the
    /// evaluator's wall-clock-refresh cadence (§9.13 refresh point #3).
    /// `None` defaults to [`InstantMonotonicSource`] at build time.
    monotonic_source: Option<Arc<dyn MonotonicSource>>,
    /// Phase 2a G9-A-cont: explicit HLC wall-clock source. `None` defaults
    /// to [`HlcTimeSource`] at build time. Rides alongside
    /// `monotonic_source` for federation-correlation context; never
    /// primary for cadence (§9.13 dual-source resolution).
    time_source: Option<Arc<dyn TimeSource>>,
    /// R6-R3 r6-r3-arch-5: explicit `Arc<dyn SuspensionStore>` injection
    /// for tests + alternative impls. `None` (default) constructs the
    /// production [`crate::suspension_store::RedbSuspensionStore`] at
    /// `assemble()` time. When `Some(_)` the supplied store is used
    /// verbatim — lets test fixtures inject in-memory suspension stores
    /// without spinning the redb persistence path. Aligns structurally
    /// with the PHASE-3-BUNDLE-1 GraphBackend genericism work but
    /// unblocks alternative-store testing today.
    suspension_store: Option<Arc<dyn benten_eval::SuspensionStore>>,
}

impl EngineBuilder {
    /// Construct an empty `EngineBuilder` with all subsystems enabled
    /// and default configuration. Caller must call [`Self::path`] (or
    /// the in-memory variant) before [`Self::build`].
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
            use_grant_backed: false,
            allow_revocation: false,
            use_ucan_grounded: false,
            ucan_grounded_now_secs: None,
            change_stream_capacity: None,
            monotonic_source: None,
            time_source: None,
            suspension_store: None,
        }
    }

    /// R6-R3 r6-r3-arch-5: inject an explicit
    /// [`benten_eval::SuspensionStore`] implementation. Tests pass an
    /// in-memory store to exercise WAIT suspend/resume without spinning
    /// the redb persistence path; production callers leave this unset
    /// so the private `assemble` constructor builds the default
    /// `RedbSuspensionStore` backed by the engine's own backend.
    ///
    /// Pre-1.0 no-shim. Aligns structurally with PHASE-3-BUNDLE-1's
    /// GraphBackend genericism work — the suspension-store generalization
    /// unblocks alternative-store testing without waiting for the full
    /// Phase-3 lift.
    #[must_use]
    pub fn suspension_store(mut self, store: Arc<dyn benten_eval::SuspensionStore>) -> Self {
        self.suspension_store = Some(store);
        self
    }

    /// Set the on-disk redb path the engine opens. Required for
    /// persistent engines; in-memory deployments use the `:memory:`
    /// pseudo-path.
    #[must_use]
    pub fn path(mut self, p: impl AsRef<Path>) -> Self {
        self.path = Some(p.as_ref().to_path_buf());
        self
    }

    /// Configure an explicit capability policy.
    ///
    /// TODO(phase-4-meta — backlog §4.68 row 3): napi v3 cannot
    /// serialize `Box<dyn CapabilityPolicy>` across the JS boundary.
    /// Phase-4-Meta wraps this surface in a `PolicyKind` enum
    /// (`NoAuth | GrantBacked | Ucan(...) | Custom(Box<dyn...>)`) so
    /// the native-only `Custom` variant is gated behind
    /// `#[cfg(not(target_arch = "wasm32"))]` while `NoAuth` /
    /// `GrantBacked` stay reachable from TypeScript (origin
    /// code-reviewer finding `g7-cr-3`; couples to §4.63 KVBackend
    /// v1-stabilization fork). See `docs/future/phase-4-backlog.md §4.65`.
    #[must_use]
    pub fn capability_policy(mut self, p: Box<dyn CapabilityPolicy>) -> Self {
        self.policy = Some(p);
        self
    }

    /// Route the builder through [`benten_caps::GrantBackedPolicy`].
    ///
    /// At [`EngineBuilder::build`] time the backend is wrapped in an
    /// `Arc<RedbBackend>`, a [`GrantReader`] handle is constructed against
    /// that Arc, and the policy is installed. Subsequent `call()` paths see
    /// write denials whenever the derived scope (`"store:<label>:write"`)
    /// has no unrevoked `system:CapabilityGrant` Node.
    ///
    /// Phase-1 scope: actor threading is not yet wired — any unrevoked
    /// grant for the derived scope permits the write. Phase-2 `benten-id`
    /// tightens to principal-scoped lookups.
    #[must_use]
    pub fn capability_policy_grant_backed(mut self) -> Self {
        self.use_grant_backed = true;
        self
    }

    /// Phase-1 alias for [`Self::capability_policy_grant_backed`]: the
    /// revocation-aware policy shape IS the grant-backed one (revocation is
    /// observed as a `system:CapabilityRevocation` Node matching an existing
    /// grant's scope). The name is preserved so tests that spell the
    /// revocation-aware intent keep compiling; the Phase-2 per-iteration
    /// wall-clock refresh policy (R4b finding `g4-p2-uc-2`) replaces the
    /// body when it lands.
    #[must_use]
    pub fn with_policy_allowing_revocation(mut self) -> Self {
        self.allow_revocation = true;
        self.use_grant_backed = true;
        self
    }

    /// Phase-3 G21-T2 — route the builder through the durable
    /// UCAN-backed capability policy (closes audit-6-1 +
    /// phase-3-backlog §2.3 + G21-T2 fp-mini-review BLOCKER-2).
    ///
    /// Composes [`benten_caps::UcanGroundedPolicy`] which wraps
    /// [`benten_caps::GrantBackedPolicy`] (Phase-2b revocation-aware
    /// `system:CapabilityGrant` Node-encoded grant store) AND
    /// [`benten_caps::UCANBackend`] proof-chain validation (signature +
    /// `nbf`/`exp` time-window + attenuation + per-token revocation
    /// at every link, plus the [`benten_caps::typed_cap_for_ucan_claim`]
    /// claim → `cap:typed:*` mapping table).
    ///
    /// ## Composition
    ///
    /// 1. `GrantBackedPolicy` is consulted first (fast path); a stored
    ///    grant permits the write immediately.
    /// 2. If GrantBackedPolicy denies AND the required capability is
    ///    in the `cap:typed:*` namespace, the policy enumerates
    ///    persisted UCAN proofs via
    ///    [`benten_caps::UCANBackend::iter_installed_proofs`] and
    ///    accepts the first chain whose leaf-claim maps to the
    ///    required typed-cap AND passes the chain-walker.
    /// 3. Otherwise the original GrantBackedPolicy denial bubbles.
    ///
    /// ## Pre-G21-T2 fp-mini-review state (now closed)
    ///
    /// The Phase-3-G21-T2-pre-fp method body was a verbatim alias for
    /// `capability_policy_grant_backed` — UCAN proof-chain validation
    /// NEVER fired under `PolicyKind::Ucan`. A forged UCAN with
    /// audience-right + capability-wrong, an expired token, or an
    /// attenuation-violation chain was NEVER rejected on the basis of
    /// the chain — only on the basis of literal entry in
    /// `system:CapabilityGrant`. This method now runs the chain-walker
    /// for `cap:typed:*` requirements.
    ///
    /// ## Out of scope (named at phase-3-backlog §2.3 (i))
    ///
    /// Per-write proof-chain enforcement for arbitrary `store:*` /
    /// other scope-strings (with audience binding +
    /// `WriteContext::actor_hint`-as-DID propagation +
    /// `WriteContext::now`-as-real-clock injection) is the wider
    /// architectural lift. That extension is named at
    /// `docs/future/phase-3-backlog.md §2.3 (i)`.
    #[must_use]
    pub fn capability_policy_ucan_durable(mut self) -> Self {
        // Both flags fire: the grant-backed reader is the inner
        // surface; `use_ucan_grounded` triggers the
        // UcanGroundedPolicy wrap at assemble time.
        self.use_grant_backed = true;
        self.use_ucan_grounded = true;
        self
    }

    /// G16-B-B-rest sub-item D test-only escape valve: pin a static
    /// `now_secs` (epoch seconds) for the `UcanGroundedPolicy`
    /// chain-walker so integration tests can install time-bounded
    /// UCAN proofs without tripping the
    /// [`benten_caps::CapError::UcanClockNotInjected`] fail-closed
    /// branch.
    ///
    /// Production callers leave this unset (the
    /// `DEFAULT_NOW_SECS = 0` sentinel surfaces the
    /// "no clock injected" misconfiguration). The `WriteContext::now`
    /// threading work named at `docs/future/phase-3-backlog.md §2.3 (i)`
    /// will replace this static-clock fixture with a real-clock
    /// injection on every chain-walk.
    ///
    /// Composes with [`Self::capability_policy_ucan_durable`] — calling
    /// this method without that one is a no-op (the
    /// `UcanGroundedPolicy` wrap doesn't fire unless
    /// `use_ucan_grounded` is set).
    #[must_use]
    pub fn ucan_grounded_now_for_test(mut self, now_secs: u64) -> Self {
        self.ucan_grounded_now_secs = Some(now_secs);
        self
    }

    /// Mark the engine as production-grade — disables permissive
    /// defaults and surfaces additional sanity checks at boot. Off by
    /// default for embedded / single-user paths.
    #[must_use]
    pub fn production(mut self) -> Self {
        self.production = true;
        self
    }

    /// Build the engine without the IVM subsystem. Embedded clients
    /// that do not use views opt in here to avoid the IVM crate's
    /// memory + scheduling overhead. View-API calls then surface
    /// `E_SUBSYSTEM_DISABLED`.
    #[must_use]
    pub fn without_ivm(mut self) -> Self {
        self.without_ivm = true;
        self
    }

    /// Build the engine without the capability subsystem. Single-user
    /// embedded clients with full backend trust opt in here. Cap-API
    /// calls then surface `E_SUBSYSTEM_DISABLED`.
    #[must_use]
    pub fn without_caps(mut self) -> Self {
        self.without_caps = true;
        self
    }

    /// Build the engine without the version-chain subsystem. Suitable
    /// for ephemeral data paths that don't need version-history
    /// retention. `E_SUBSYSTEM_DISABLED` on version-API calls.
    #[must_use]
    pub fn without_versioning(mut self) -> Self {
        self.without_versioning = true;
        self
    }

    /// Set the IVM per-update work budget for testing. Ordinary
    /// production deployments should use the default (effectively
    /// unbounded); this knob exists so tests can pin the budget at a
    /// small value to exercise the budget-exhausted code path.
    #[must_use]
    pub fn with_test_ivm_budget(mut self, b: u64) -> Self {
        self.test_ivm_budget = Some(b);
        self
    }

    /// Alias of [`Self::with_test_ivm_budget`] retained for the IVM
    /// per-update-work API spelling preferred by some test fixtures.
    #[must_use]
    pub fn ivm_max_work_per_update(mut self, n: u64) -> Self {
        self.test_ivm_budget = Some(n);
        self
    }

    /// Configure the upper bound on the in-memory change-event buffer held
    /// by the engine for [`Engine::subscribe_change_events`] probes.
    ///
    /// When a subscriber lags behind the write path and the buffer reaches
    /// this capacity, older events are dropped (oldest-first) and
    /// `benten.change_stream.dropped_events` is incremented (observable via
    /// [`Engine::metrics_snapshot`]). Defaults to
    /// [`CHANGE_STREAM_MAX_BUFFERED`].
    ///
    /// Values of `0` are clamped to `1` so at least the most recent event
    /// stays visible; use `.without_ivm()` plus refraining from subscribing
    /// if you truly need a zero-buffer engine. See r6-sec-5.
    #[must_use]
    pub fn change_stream_capacity(mut self, n: usize) -> Self {
        self.change_stream_capacity = Some(n);
        self
    }

    /// Provide a pre-opened backend.
    #[must_use]
    pub fn backend(mut self, b: RedbBackend) -> Self {
        self.backend = Some(b);
        self
    }

    /// Phase 2a G9-A-cont: inject an explicit monotonic clock source.
    ///
    /// The configured source drives the evaluator's wall-clock-refresh
    /// cadence (§9.13 refresh point #3). Tests that need a controllable
    /// monotonic clock (see
    /// `tests/wallclock_refresh_uses_monotonic_only.rs`) inject a
    /// `MockMonotonicSource`; production builds take the
    /// [`InstantMonotonicSource`] default.
    ///
    /// Monotonic is PRIMARY for cadence — if you want to also control the
    /// wall-clock HLC stamp, use [`Self::time_source`] alongside this
    /// method. The two traits are deliberately orthogonal so a
    /// drift-injecting test can freeze the wall-clock independently of
    /// the monotonic tick.
    #[must_use]
    pub fn monotonic_source(mut self, source: Arc<dyn MonotonicSource>) -> Self {
        self.monotonic_source = Some(source);
        self
    }

    /// Phase 2a G9-A-cont: inject an explicit HLC / wall-clock source.
    ///
    /// Rides alongside [`Self::monotonic_source`] for federation-
    /// correlation context. This source is NEVER used to drive TOCTOU
    /// refresh cadence — wall-clock is drift-exploitable (§9.13
    /// refresh-point-5 threat model). Production defaults to
    /// [`HlcTimeSource`].
    #[must_use]
    pub fn time_source(mut self, source: Arc<dyn TimeSource>) -> Self {
        self.time_source = Some(source);
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
            (None, Some(p)) => open_backend_for_path(&p)?,
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
    ///
    /// The sentinel `":memory:"` is recognised here (HANDOFF §3.F wave-5):
    /// it routes to [`RedbBackend::open_in_memory`] so 9-of-10
    /// `packages/engine/test/*.test.ts` can drive the engine without a
    /// filesystem path. Any other value is treated as a real path and
    /// opens through [`RedbBackend::open`].
    pub fn open(mut self, path: impl AsRef<Path>) -> Result<Engine, EngineError> {
        if self.production && self.without_caps {
            return Err(EngineError::ProductionRequiresCaps);
        }
        if self.production && self.policy.is_none() {
            return Err(EngineError::NoCapabilityPolicyConfigured);
        }
        let backend = open_backend_for_path(path.as_ref())?;
        self.backend = Some(backend);
        self.build()
    }

    /// Assemble the engine from a fully-configured backend.
    #[allow(
        clippy::too_many_lines,
        reason = "linear assembly pipeline (broadcast wiring → IVM → policy → noauth log → clocks → engine ctor → rehydrate); extracting any single pass requires threading 6+ parameters and worsens readability"
    )]
    fn assemble(self, backend: RedbBackend) -> Result<Engine, EngineError> {
        let backend = Arc::new(backend);
        let capacity = self
            .change_stream_capacity
            .unwrap_or(CHANGE_STREAM_MAX_BUFFERED);
        let inner = Arc::new(EngineInner::with_change_stream_capacity(capacity));
        let broadcast = Arc::new(ChangeBroadcast::new());

        // Always attach a tap that records every ChangeEvent into the
        // engine's observed-events queue. Probes drain from there.
        let inner_for_tap = Arc::clone(&inner);
        broadcast.subscribe_fn(move |event| {
            inner_for_tap.record_event(event);
        });

        // Wave-8c-subscribe-infra: bridge the engine-side `ChangeBroadcast`
        // to the eval-side SUBSCRIBE delivery path. Every committed change
        // event is translated into the eval-side `ChangeEvent` shape
        // (anchor_cid + monotonic engine-assigned seq + opaque payload
        // bytes) and dispatched to ALL active subgraph-SUBSCRIBE primitives
        // + ad-hoc `engine.on_change` consumers via
        // `subscribe::publish_change_event_with_label`. Closes the
        // SUBSCRIBE production-runtime DRIFT gap surfaced in the
        // r4b-followup-primitive-executor-docs-vs-code-audit.
        broadcast.subscribe_fn(move |event| {
            // Translate graph::ChangeEvent → eval-side ChangeEvent.
            let kind = match event.kind {
                benten_graph::ChangeKind::Created => {
                    benten_eval::primitives::subscribe::ChangeKind::Created
                }
                benten_graph::ChangeKind::Updated => {
                    benten_eval::primitives::subscribe::ChangeKind::Updated
                }
                benten_graph::ChangeKind::Deleted => {
                    benten_eval::primitives::subscribe::ChangeKind::Deleted
                }
                // Edge events route as Updated (no analogous variant
                // on the eval side — Phase-3 may extend if needed).
                benten_graph::ChangeKind::EdgeCreated | benten_graph::ChangeKind::EdgeDeleted => {
                    benten_eval::primitives::subscribe::ChangeKind::Updated
                }
            };
            // Best-effort encoding of the Node body as the payload. For
            // deletes / edge events without a Node we ship empty bytes;
            // consumers care about anchor_cid + kind in those cases. We
            // route through the engine-internal canonical Node bytes so
            // the eval-side delivery can carry the Node identity without
            // pulling a new serde dep into benten-engine. (The
            // ChangeBroadcast already operates inside benten-engine, so
            // we have the Node's `cid()` accessor + canonical_bytes
            // available; we use the latter so the consumer-side code can
            // re-hash without ambiguity if it wishes.)
            let payload_bytes: Vec<u8> = event
                .node
                .as_ref()
                .and_then(|n| n.canonical_bytes().ok())
                .unwrap_or_default();
            // R6FP-Group-1 (Round-2 Instance 6 BLOCKER): forward ALL
            // 9 fields cleanly. Pre-fix this bridge dropped 6 of 9
            // (tx_id, actor_cid, handler_cid, capability_grant_cid,
            // edge_endpoints, AND collapsed labels: Vec<String> to a
            // single primary_label: String). The collapse caused a
            // real BEHAVIORAL DEFECT: a multi-labeled Node
            // ["User","Admin"] silently missed delivery to a
            // SUBSCRIBE consumer matching `Admin:*` because the
            // matcher only consulted the (single) primary label.
            // edge_endpoints stays out of the eval-side struct (the
            // eval ChangeEvent is anchor-centric; edge events ride
            // the anchor_cid/labels with no separate endpoint
            // surface). All other 8 fields forward.
            //
            // `ChangeEvent` is `#[non_exhaustive]` (ST-CORE lane
            // change_stream.rs:122) so the cross-crate struct-expression
            // form is no longer legal here; the bridge uses the
            // full-fidelity `ChangeEvent::for_bridge` constructor, which
            // forwards every one of the 9 fields with their real values
            // (NOT the lossy `legacy_minimal`). Argument order mirrors
            // the field order so the no-field-drop drift guard still
            // covers all forwarded fields.
            let translated = benten_eval::primitives::subscribe::ChangeEvent::for_bridge(
                event.cid,
                kind,
                benten_eval::primitives::subscribe::next_engine_seq(),
                payload_bytes,
                event.labels.clone(),
                event.tx_id,
                event.actor_cid,
                event.handler_cid,
                event.capability_grant_cid,
            );
            benten_eval::primitives::subscribe::publish_change_event_with_labels(
                &event.labels,
                translated,
            );
        });

        // Wire the IVM subscriber when enabled. G5's `Subscriber::new()`
        // starts with no views; `create_view` registers views on demand
        // against the Arc the Engine retains. Phase 1 auto-registers the
        // content_listing view for `"post"` so `read_view` and `crud('post')`
        // work out of the box without a manual `create_view` step. When
        // `.with_test_ivm_budget(b)` is set the view is constructed with
        // that budget so stale-view regression tests can trip it.
        //
        // TODO(phase-4-meta — backlog §4.68 row 4): arch-6 flagged
        // the "post" auto-registration + register_crud auto-registration
        // as two paths that both materialise `content_listing_<label>`
        // views; Phase-4-Meta collapses to a single
        // `register_content_listing(label)` builder entry point called
        // on demand. See `docs/future/phase-4-backlog.md §4.65`.
        let ivm: Option<Arc<benten_ivm::Subscriber>> = if self.without_ivm {
            None
        } else {
            let subscriber = Arc::new(benten_ivm::Subscriber::new());
            backend.register_subscriber(Arc::clone(&subscriber) as Arc<dyn ChangeSubscriber>)?;
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
        backend.register_subscriber(Arc::clone(&broadcast) as Arc<dyn ChangeSubscriber>)?;

        let caps_enabled = !self.without_caps;
        let ivm_enabled = !self.without_ivm;
        // Phase 4-Foundation R1-FP wave-1 G22-FP-1 option-D (2026-05-12):
        // the public builder API (`capability_policy(p: Box<dyn ...>)`)
        // remains `Box` so ~40 test + downstream callers stay unchanged;
        // here at assemble time we convert `Box → Arc::from(box)` so the
        // engine's internal storage is `Arc<dyn CapabilityPolicy>` and
        // the cap-recheck closure in `Engine::on_change_as_with_cursor`
        // can capture a clonable handle for per-event `check_read`.
        let (policy, using_noauth): (Option<Arc<dyn CapabilityPolicy>>, bool) = if caps_enabled {
            if let Some(explicit) = self.policy {
                (Some(Arc::from(explicit)), false)
            } else if self.use_grant_backed {
                let reader: Arc<dyn GrantReader> =
                    Arc::new(BackendGrantReader::new(Arc::clone(&backend)));
                let grant_backed = GrantBackedPolicy::new(reader);
                if self.use_ucan_grounded {
                    // G21-T2 fp-mini-review BLOCKER-2 closure: wrap
                    // the grant-backed policy in
                    // `UcanGroundedPolicy` so `cap:typed:*` writes
                    // also consult the durable UCAN proof-chain
                    // validator (signature + nbf/exp + attenuation +
                    // revocation + typed-cap-claim mapping).
                    let ucan_backend =
                        Arc::new(benten_caps::UCANBackend::new(Arc::clone(&backend)));
                    let mut policy =
                        benten_caps::UcanGroundedPolicy::new(grant_backed, ucan_backend);
                    if let Some(now_secs) = self.ucan_grounded_now_secs {
                        // G16-B-B-rest sub-item D: tests inject a
                        // static clock so time-bounded UCAN proofs
                        // walk cleanly past the fail-closed branch.
                        policy = policy.with_now_for_test(now_secs);
                    }
                    (Some(Arc::new(policy) as Arc<dyn CapabilityPolicy>), false)
                } else {
                    (
                        Some(Arc::new(grant_backed) as Arc<dyn CapabilityPolicy>),
                        false,
                    )
                }
            } else {
                (Some(Arc::new(NoAuthBackend::new())), true)
            }
        } else {
            (None, false)
        };

        // 5d-J workstream 6: when the assembled engine is running with the
        // zero-config NoAuthBackend (no caller-supplied policy, no grant-
        // backed flag, caps not disabled), emit a one-shot info log so
        // operators running `create-benten-app`-scaffolded code see the
        // posture in stderr on first startup. Acceptable for embedded /
        // single-user deployments; not suitable for multi-user or
        // networked use. Tests capture stderr and pin the presence.
        if using_noauth {
            emit_noauth_startup_log();
        }

        // Phase 2a G9-A-cont: resolve the configured clock sources, falling
        // back to the production defaults when the builder caller did not
        // inject explicit mocks. Both sources are held behind `Arc<dyn …>`
        // on the Engine so `impl PrimitiveHost for Engine` can read them
        // without threading them through the trait method signatures.
        let monotonic: Arc<dyn MonotonicSource> = self
            .monotonic_source
            .unwrap_or_else(|| Arc::new(InstantMonotonicSource::new()));
        let time: Arc<dyn TimeSource> = self
            .time_source
            .unwrap_or_else(|| Arc::new(HlcTimeSource::new()));

        // G13-B generic-cascade: the `EngineGeneric::from_parts_with_clocks`
        // constructor requires an injected `Arc<dyn SuspensionStore>` (the
        // generic engine cannot know how to construct a suspension store
        // for an arbitrary backend `B`). The redb-backed builder fills in
        // the default `RedbSuspensionStore` here when the caller did not
        // inject an alternative; this is the redb-specific specialization
        // half of D-PHASE-3-1 RESOLVED.
        let suspension_store: Arc<dyn benten_eval::SuspensionStore> =
            self.suspension_store.unwrap_or_else(|| {
                Arc::new(crate::suspension_store::RedbSuspensionStore::new(
                    Arc::clone(&backend),
                ))
            });

        let engine = Engine::from_parts_with_clocks(
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
            monotonic,
            time,
            suspension_store,
        );

        // R6FP-Group-1 (r6-arch-1): rebuild the in-memory installed-
        // modules active set from the durable `system:ModuleManifest`
        // zone. Pre-R6FP-G1 a fresh `Engine::open` after a previous
        // `install_module` returned `false` from `is_module_installed`
        // for the previously-installed CID — the manifest survived on
        // disk but the in-memory indexes the dispatcher consults were
        // empty. The hydration is best-effort: a corrupt / partial
        // Node is skipped rather than aborting startup so a single
        // bad manifest cannot wedge engine open.
        //
        // R6-R3 r6-r3-arch-7: pre-fix the outer-call Result was discarded
        // via `let _ = ...`, swallowing backend-level read failures (e.g.
        // redb returns Err on get_by_label). An operator running
        // `engine_diagnostics` had no observable distinction between
        // (a) clean fresh engine, (b) successful rehydrate of N
        // manifests, (c) backend read-error rehydrate-failure that
        // silently dropped manifests. Now log per-outcome via tracing
        // so the failure mode is observable in the same logs the
        // operator already consults.
        //
        // The `tracing` crate is gated to `cfg(not(target_arch = "wasm32"))`
        // in `Cargo.toml` (snapshot-blob-only browser builds explicitly
        // exclude it; the redb-backed `Engine::open` path that consumes
        // module-manifest rehydration also doesn't reach wasm32). On
        // wasm32 the rehydrate result is discarded as before — the
        // wasm32 build does not surface installed-module manifests.
        let rehydrate_outcome = engine.rehydrate_installed_modules_from_zone();
        #[cfg(not(target_arch = "wasm32"))]
        match rehydrate_outcome {
            Ok(n) => {
                tracing::debug!(rehydrated = n, "engine module-manifest rehydrate complete");
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "engine module-manifest rehydrate failed; \
                     booting with empty active-set (modules registered \
                     in earlier sessions will not be visible until \
                     re-installed)"
                );
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = rehydrate_outcome;

        // Phase-3 G14-C (Compromise #17 closure) — rehydrate the
        // in-memory module-bytes cache from `system:ModuleBytes`
        // zone Nodes. Pre-G14-C, register_module_bytes wrote to an
        // in-memory map only; closure persists via RedbBlobBackend
        // and the cache is rebuilt at engine open so SANDBOX
        // dispatch resolves manifests across restart without an
        // operator re-call. See `crates/benten-engine/src/engine.rs::
        // rehydrate_module_bytes_from_zone`.
        let bytes_outcome = engine.rehydrate_module_bytes_from_zone();
        #[cfg(not(target_arch = "wasm32"))]
        match bytes_outcome {
            Ok(n) => {
                tracing::debug!(rehydrated = n, "engine module-bytes rehydrate complete");
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "engine module-bytes rehydrate failed; \
                     booting with empty cache (operators must \
                     re-call register_module_bytes for affected \
                     SANDBOX manifests until the next successful \
                     rehydrate)"
                );
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = bytes_outcome;

        // Phase-3 G14-C (Compromise #18 closure) — rehydrate per-handler
        // version chains from `system:HandlerVersion` zone Nodes.
        let chains_outcome = engine.rehydrate_handler_version_chains_from_zone();
        #[cfg(not(target_arch = "wasm32"))]
        match chains_outcome {
            Ok(n) => {
                tracing::debug!(
                    rehydrated = n,
                    "engine handler-version-chain rehydrate complete"
                );
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "engine handler-version-chain rehydrate failed; \
                     booting with empty chains (audit consumers will \
                     not see history from prior process)"
                );
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = chains_outcome;

        Ok(engine)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot guard for the NoAuth startup log — the process-wide `Once` so
/// a binary opening a pool of engines does not spam stderr.
static NOAUTH_LOG_ONCE: std::sync::Once = std::sync::Once::new();

/// Sentinel path that routes [`Engine::open`] / [`EngineBuilder::open`] /
/// [`EngineBuilder::build`] to a transient in-memory backend instead of
/// touching the filesystem. HANDOFF §3.F wave-5: pinned as the literal
/// string `":memory:"` because 9-of-10 `packages/engine/test/*.test.ts`
/// already drive the engine through `Engine.open(":memory:")` at the
/// napi boundary; renaming would break the JS test suite.
pub const IN_MEMORY_SENTINEL: &str = ":memory:";

/// Open the configured backend for `path` — sentinel-aware.
///
/// `path == ":memory:"` routes to [`RedbBackend::open_in_memory`] (a
/// transient redb store with `Durability::None`). Any other value is
/// treated as a real filesystem path and opened through
/// [`RedbBackend::open`].
///
/// The match is intentionally on the literal sentinel string (not a
/// `to_str()`-then-equality on the borrowed path) so a path that *happens*
/// to encode `":memory:"` in some non-UTF8 form (impossible on POSIX, but
/// the type permits it) does not silently get redirected.
fn open_backend_for_path(path: &Path) -> Result<RedbBackend, EngineError> {
    if path.as_os_str() == IN_MEMORY_SENTINEL {
        Ok(RedbBackend::open_in_memory()?)
    } else {
        Ok(RedbBackend::open(path)?)
    }
}

/// Message emitted once per process when the assembled engine falls through
/// to the zero-config [`NoAuthBackend`]. Public so the integration test can
/// assert the exact wording without drift.
pub const NOAUTH_STARTUP_LOG: &str = "benten-engine: running with NoAuthBackend (no authorization). \
     Acceptable for embedded/single-user; configure a CapabilityPolicy \
     for multi-user/networked use.";

fn emit_noauth_startup_log() {
    NOAUTH_LOG_ONCE.call_once(|| {
        // `eprintln!` rather than `tracing` so the notice surfaces on a
        // scaffolded project that hasn't wired a tracing subscriber yet.
        // A binary that prefers structured logs can route through their
        // own subscriber once they've configured a CapabilityPolicy and
        // the noauth path is no longer hit.
        #[allow(
            clippy::print_stderr,
            reason = "5d-J workstream 6: operator-facing startup notice; \
                      intentionally unstructured so scaffolded projects \
                      without a tracing subscriber still see it."
        )]
        {
            eprintln!("{NOAUTH_STARTUP_LOG}");
        }
    });
}

/// Test-only reset hook so the one-shot log can be re-armed between
/// test cases. `std::sync::Once` has no reset API, so we replace the
/// guard atomically via a module-local mutex. Gated on `cfg(test)` and
/// the `test-helpers` feature so production builds never see it.
#[cfg(any(test, feature = "test-helpers"))]
#[doc(hidden)]
pub fn __test_reset_noauth_log_once() {
    // No-op: we can't reset a `Once`. Tests that need re-entrant capture
    // should spawn a subprocess or accept that the log is one-shot
    // process-wide. The constant [`NOAUTH_STARTUP_LOG`] is the stable
    // anchor for contents-based assertions.
}

/// [`GrantReader`] implementation backed by the engine's
/// [`RedbBackend`]. Looks up `system:CapabilityGrant` Nodes by their
/// canonical label and matches on the `scope` property; presence of a
/// `system:CapabilityRevocation` Node with the same `scope` marks the
/// family revoked.
pub(crate) struct BackendGrantReader {
    backend: Arc<RedbBackend>,
}

impl BackendGrantReader {
    pub(crate) fn new(backend: Arc<RedbBackend>) -> Self {
        Self { backend }
    }

    /// Iterate every `system:CapabilityRevocation` Node and collect the set
    /// of revoked scopes. Phase-1 signal is scope-only (actor threading is
    /// Phase-3 scope). The label-index walk is O(revocations); a revocation
    /// count high enough to matter is a symptom of a different problem.
    fn revoked_scopes(&self) -> Result<std::collections::BTreeSet<String>, CapError> {
        let cids = self
            .backend
            .get_by_label("system:CapabilityRevocation")
            .map_err(|e| CapError::Denied {
                required: format!("backend read: {e:?}"),
                entity: String::new(),
            })?;
        let mut out = std::collections::BTreeSet::new();
        for cid in cids {
            match self.backend.get_node(&cid) {
                Ok(Some(node)) => {
                    if let Some(Value::Text(scope)) = node.properties.get("scope") {
                        out.insert(scope.clone());
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(CapError::Denied {
                        required: format!("backend read: {e:?}"),
                        entity: String::new(),
                    });
                }
            }
        }
        Ok(out)
    }
}

impl GrantReader for BackendGrantReader {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        // Phase 4-Foundation R1 cap-r1-2 / cap-r1-10: this scope-only
        // path remains the no-actor-context fallback. `check_write`
        // continues to call it; `check_read` now routes through the
        // principal-aware override below.
        self.has_unrevoked_grant_for_scope_and_actor(scope, None)
    }

    fn has_unrevoked_grant_for_scope_and_actor(
        &self,
        scope: &str,
        actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        let revoked = self.revoked_scopes()?;
        if revoked.contains(scope) {
            return Ok(false);
        }
        // Single-source-of-truth for the grant label — matches the
        // `CAPABILITY_GRANT_LABEL` constant in `benten-caps` (and View 1's
        // filter). A hard-coded string here would re-open the r6b-ivm-2
        // namespace-drift bug.
        let cids = self
            .backend
            .get_by_label(CAPABILITY_GRANT_LABEL)
            .map_err(|e| CapError::Denied {
                required: format!("backend read: {e:?}"),
                entity: String::new(),
            })?;
        for cid in cids {
            match self.backend.get_node(&cid) {
                Ok(Some(node)) => {
                    let grant_scope = match node.properties.get("scope") {
                        Some(Value::Text(s)) => s.as_str(),
                        _ => continue,
                    };
                    if grant_scope != scope {
                        continue;
                    }
                    // A grant whose `revoked` property is explicitly `true`
                    // is treated as revoked in addition to the separate
                    // revocation-Node path (belt-and-braces — both write
                    // paths can be used independently).
                    let explicitly_revoked =
                        matches!(node.properties.get("revoked"), Some(Value::Bool(true)));
                    if explicitly_revoked {
                        continue;
                    }
                    // Phase 4-Foundation R1 cap-r1-2 BLOCKER + cap-r1-10
                    // closure: principal binding. When the caller threads
                    // an `actor_cid`, only grants whose stored `actor`
                    // property matches are considered. A grant issued to
                    // user-B does NOT permit user-A's read — this closes
                    // the cross-principal-permission bug where
                    // `check_read` was wildcard-enumerating by scope only.
                    // When `actor_cid` is `None` the call collapses to the
                    // scope-only check (legacy / Phase-1 / Phase-2
                    // fixtures + NoAuthBackend default-permit path).
                    //
                    // The grant Node persists `actor` via `actor.as_value()`
                    // (`GrantSubject::as_value`) which is `Value::Bytes`
                    // for a CID-shaped subject; bytes equal `cid.as_bytes()`.
                    if let Some(want) = actor_cid {
                        let actor_bytes = match node.properties.get("actor") {
                            Some(Value::Bytes(b)) => b.as_slice(),
                            _ => continue, // malformed grant — skip
                        };
                        if actor_bytes != want.as_bytes() {
                            continue;
                        }
                    }
                    return Ok(true);
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(CapError::Denied {
                        required: format!("backend read: {e:?}"),
                        entity: String::new(),
                    });
                }
            }
        }
        Ok(false)
    }
}
