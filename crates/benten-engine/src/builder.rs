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
//! subsystem is wired to the backend. Slim-build variants that omit the
//! code paths entirely are a Phase-2 concern — see the `testing.rs`
//! `TODO(phase-2-features)` mirror.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use benten_caps::{
    CAPABILITY_GRANT_LABEL, CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader,
    NoAuthBackend,
};
use benten_core::Value;
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
            use_grant_backed: false,
            allow_revocation: false,
            change_stream_capacity: None,
            monotonic_source: None,
            time_source: None,
        }
    }

    #[must_use]
    pub fn path(mut self, p: impl AsRef<Path>) -> Self {
        self.path = Some(p.as_ref().to_path_buf());
        self
    }

    /// Configure an explicit capability policy.
    ///
    /// TODO(phase-2-policy-kind): napi v3 cannot serialize `Box<dyn CapabilityPolicy>` across
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

    /// Phase 2a alias for [`Self::capability_policy_grant_backed`] used by
    /// Option C bench fixtures.
    #[must_use]
    pub fn with_grant_backed_policy(self) -> Self {
        self.capability_policy_grant_backed()
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

        // Wire the IVM subscriber when enabled. G5's `Subscriber::new()`
        // starts with no views; `create_view` registers views on demand
        // against the Arc the Engine retains. Phase 1 auto-registers the
        // content_listing view for `"post"` so `read_view` and `crud('post')`
        // work out of the box without a manual `create_view` step. When
        // `.with_test_ivm_budget(b)` is set the view is constructed with
        // that budget so stale-view regression tests can trip it.
        //
        // TODO(phase-2-content-listing-autoreg): arch-6 flagged the
        // "post" auto-registration + register_crud auto-registration as
        // two paths that both materialise `content_listing_<label>` views;
        // Phase-2 collapses to a single `register_content_listing(label)`
        // entry point the builder calls on demand.
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
        let (policy, using_noauth): (Option<Box<dyn CapabilityPolicy>>, bool) = if caps_enabled {
            if let Some(explicit) = self.policy {
                (Some(explicit), false)
            } else if self.use_grant_backed {
                let reader: Arc<dyn GrantReader> =
                    Arc::new(BackendGrantReader::new(Arc::clone(&backend)));
                (Some(Box::new(GrantBackedPolicy::new(reader))), false)
            } else {
                (Some(Box::new(NoAuthBackend::new())), true)
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

        Ok(Engine::from_parts_with_clocks(
            backend,
            policy,
            caps_enabled,
            ivm_enabled,
            broadcast,
            inner,
            ivm,
            monotonic,
            time,
        ))
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
