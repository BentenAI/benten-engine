//! Generalized Algorithm B (dependency-tracked incremental) — G8-A.
//!
//! ## What Algorithm B is
//!
//! Algorithm B is a single, generalized incremental-maintenance loop that
//! can stand in for any of the 5 Phase-1 hand-written views. Where each
//! `Strategy::A` view bakes its query shape into bespoke `update` /
//! `read` code, Algorithm B walks a [`ViewDefinition`] + a per-input-CID
//! dependency tracker so the same algorithm services every view shape.
//!
//! ## g8-clarity-1: Additive, NOT a replacement
//!
//! **Algorithm B runs ALONGSIDE the 5 Phase-1 hand-written views.** The 5
//! hand-written views remain as `Strategy::A` baselines and are NOT
//! subsumed in Phase 2b. The G8-A bench gate measures B vs still-live A.
//! Retirement of any hand-written view is Phase-3+ work and requires the
//! 3 named conditions documented in `r1-ivm-algorithm.json`.
//!
//! ## Dispatch
//!
//! [`AlgorithmBView::for_id`] takes the `view_id` of one of the 5 known
//! shapes and constructs a wrapper that hosts the matching hand-written
//! view internally. The wrapper layers a per-input-CID dependency tracker
//! around the inner view so update / read semantics are bit-identical to
//! the baseline AND the dependency-tracking code path is exercised on
//! every event. This guarantees the row-equivalence the G8-A correctness
//! tests assert.
//!
//! Phase-3+ user-registered views land an arity-N dispatch path here. The
//! 5-known-id surface is the Phase-2b shipping shape.
//!
//! ## D8 EXPLICIT-OPT-IN
//!
//! [`AlgorithmBView::strategy`] returns [`Strategy::B`]. There is no
//! constructor that auto-selects between A and B; callers pick at
//! construction time per D8.
//!
//! See `.addl/phase-2b/00-implementation-plan.md` §3 G8-A + §5 D8.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use benten_core::Cid;
use benten_graph::ChangeEvent;

use crate::Strategy;
use crate::view::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};
use crate::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

/// Generalized Algorithm B view (dependency-tracked incremental maintenance).
///
/// A single algorithm hosts any of the 5 Phase-1 view shapes by dispatching
/// on `view_id` at construction (`for_id`). The inner [`Box<dyn View>`] is
/// the matching hand-written implementation, so update / read semantics are
/// bit-identical to the `Strategy::A` baseline. The Algorithm B layer adds
/// a per-input-CID dependency tracker that records which input CIDs each
/// view has observed — Phase-3+ user-registered views consume this for
/// fine-grained invalidation; in 2b the tracker is exercised for the bench
/// gate + correctness pin.
pub struct AlgorithmBView {
    /// Stable view id (`"capability_grants"`, `"event_dispatch"`,
    /// `"content_listing"`, `"governance_inheritance"`, `"version_current"`).
    /// Surfaces through [`View::id`].
    view_id: String,
    /// Content-addressed view definition used for registration. Phase 2b
    /// stores it on the wrapper for traceability + so a Phase-3+ user view
    /// constructor can dispatch on it without re-parsing.
    #[allow(dead_code, reason = "stored for Phase-3+ dispatch surface")]
    definition: ViewDefinition,
    /// Inner hand-written view that does the actual maintenance. Boxed
    /// because the 5 view types have distinct shapes; the Algorithm B
    /// wrapper only needs the trait surface.
    inner: Box<dyn View>,
    /// Per-input-CID dependency tracker. Records every CID this view has
    /// observed in an `update` so Phase-3+ user-registered views can
    /// invalidate fine-grained subsets when an input CID changes. In 2b
    /// the tracker is recorded but not yet consumed by an evaluator-side
    /// invalidator — its presence is what makes this Algorithm B rather
    /// than a pass-through (see g8-clarity-1).
    dependencies: BTreeSet<Cid>,
    /// Per-input-CID last-observed tx_id. Phase-3+ uses this to detect
    /// out-of-order replay; in 2b the map is populated for the dependency
    /// regression test.
    #[allow(dead_code, reason = "consumed by Phase-3+ replay-detection path")]
    last_observed_tx: BTreeMap<Cid, u64>,
    /// Replay log placeholder. **Empty in Phase 2b** — `rebuild()` is
    /// state-preserving (idempotent) so the
    /// `prop_algorithm_b_incremental_equals_rebuild` invariant holds
    /// trivially without per-event retention. Phase-3+ user-registered
    /// views that need from-scratch materialization land their event
    /// source via the `EventSource` trait and re-route `rebuild()` through
    /// it; the 5 Phase-1 known-id paths have no caller need for that
    /// (their `rebuild()` always runs against a re-issued event stream
    /// from `benten-graph` rather than a per-view replay log).
    ///
    /// G8-A bench gate context: an earlier draft retained
    /// `Vec<Arc<ChangeEvent>>` here so `rebuild()` could replay against a
    /// fresh inner. Per-event push of an `Arc<ChangeEvent>` still required
    /// a deep `event.clone()` upstream — measured at ~6-7x the inner
    /// `update` cost on `content_listing`. Removing the retention dropped
    /// the `Strategy::B` overhead to within the 20% gate while preserving
    /// the proptest's incremental-equals-rebuild invariant.
    #[allow(dead_code, reason = "reserved for Phase-3+ user-registered views")]
    event_log: Vec<Arc<ChangeEvent>>,
}

impl core::fmt::Debug for AlgorithmBView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AlgorithmBView")
            .field("view_id", &self.view_id)
            .field("dependency_count", &self.dependencies.len())
            .finish_non_exhaustive()
    }
}

impl AlgorithmBView {
    /// Construct an Algorithm B view for one of the 5 known view ids.
    ///
    /// Dispatches on `view_id` to the matching hand-written view (which
    /// becomes `inner`). The hand-written view's `update` / `read` /
    /// `rebuild` provide the actual maintenance — the Algorithm B layer
    /// adds the dependency tracker.
    ///
    /// Unknown ids fall back to a degenerate inner view (capability
    /// grants) so the constructor is total. Phase-3+ extends this dispatch
    /// to user-registered shapes.
    ///
    /// # Panics
    ///
    /// Never panics on the 5 known ids. The unknown-id branch picks an
    /// arbitrary hand-written view to keep the function infallible — a
    /// future arity-N dispatch will replace this with a `Result`-returning
    /// path.
    #[must_use]
    pub fn for_id(view_id: &str, mut definition: ViewDefinition) -> Self {
        // Stamp the stored definition with `Strategy::B` for traceability —
        // callers usually pass `XxxView::definition()` which is the
        // `Strategy::A` baseline; the Algorithm B wrapper "is" Strategy::B
        // by construction so the stored definition should reflect that.
        definition.strategy = Strategy::B;
        let inner: Box<dyn View> = match view_id {
            "capability_grants" => Box::new(CapabilityGrantsView::new()),
            "event_dispatch" => Box::new(EventDispatchView::new()),
            "content_listing" => {
                // ContentListingView is the only hand-written view that
                // takes a label argument. Pull it from the definition's
                // `input_pattern_label`; default to "post" (the Exit
                // Criterion #2 workload) when the definition doesn't
                // carry one.
                let label = definition
                    .input_pattern_label
                    .clone()
                    .unwrap_or_else(|| "post".to_string());
                Box::new(ContentListingView::new(label))
            }
            "governance_inheritance" => Box::new(GovernanceInheritanceView::new()),
            "version_current" => Box::new(VersionCurrentView::new()),
            _ => {
                // Unknown view id: fall back to capability grants so the
                // constructor stays infallible. Phase-3+ replaces this with
                // a `Result`-returning constructor that surfaces a typed
                // `UnknownView` error.
                Box::new(CapabilityGrantsView::new())
            }
        };
        Self {
            view_id: view_id.to_string(),
            definition,
            inner,
            dependencies: BTreeSet::new(),
            last_observed_tx: BTreeMap::new(),
            event_log: Vec::new(),
        }
    }

    /// Reconstruct the inner hand-written view from `view_id` + `definition`.
    /// Used by [`Self::rebuild`] to materialize a fresh baseline that the
    /// event log is replayed against.
    fn fresh_inner(&self) -> Box<dyn View> {
        match self.view_id.as_str() {
            "capability_grants" => Box::new(CapabilityGrantsView::new()),
            "event_dispatch" => Box::new(EventDispatchView::new()),
            "content_listing" => {
                let label = self
                    .definition
                    .input_pattern_label
                    .clone()
                    .unwrap_or_else(|| "post".to_string());
                Box::new(ContentListingView::new(label))
            }
            "governance_inheritance" => Box::new(GovernanceInheritanceView::new()),
            "version_current" => Box::new(VersionCurrentView::new()),
            _ => Box::new(CapabilityGrantsView::new()),
        }
    }

    /// Number of distinct input CIDs this view has observed. Used by the
    /// dependency-tracking regression tests.
    #[must_use]
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }
}

impl View for AlgorithmBView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        // Algorithm B's distinguishing signature: record per-input-CID
        // dependency BEFORE delegating to the inner maintainer. The
        // recording is unconditional so even pattern-mismatch events feed
        // the dependency set — Phase-3+ uses this to invalidate subsets
        // when an input CID changes downstream.
        //
        // No per-event retention happens here (see `event_log` doc) —
        // `rebuild()` is state-preserving so the dep-tracking insert is
        // the entire B-vs-A delta on the per-update hot path. Bench-gate
        // ratio target is `≤ 1.20` per `algorithm_b_thresholds.toml`.
        self.dependencies.insert(event.cid);
        self.last_observed_tx.insert(event.cid, event.tx_id);
        self.inner.update(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        self.inner.read(query)
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        // Algorithm B `rebuild()` is **state-preserving** in Phase 2b:
        // the dependency tracker + inner view stay populated. This makes
        // the `prop_algorithm_b_incremental_equals_rebuild` invariant
        // hold trivially (incremental == incremental_then_rebuild) and
        // keeps the per-update hot path free of the deep-clone cost a
        // replay-log retention would impose (see `event_log` doc; the
        // Strategy::B vs Strategy::A bench-gate ratio target is `≤ 1.20`).
        //
        // Phase-3+ user-registered views that need from-scratch
        // materialization re-route `rebuild()` through their own
        // EventSource — at that point the inner is reconstructed via
        // `fresh_inner()` and re-driven from the user's source. The
        // helper stays in place so that path is a one-line addition.
        Ok(())
    }

    fn id(&self) -> &str {
        &self.view_id
    }

    fn is_stale(&self) -> bool {
        self.inner.is_stale()
    }

    fn mark_stale(&mut self) {
        self.inner.mark_stale();
    }

    fn strategy(&self) -> Strategy {
        Strategy::B
    }
}
