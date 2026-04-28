//! Generalized Algorithm B (single-loop dispatch over `ViewDefinition`) — G8-A.
//!
//! ## What Algorithm B is in Phase 2b
//!
//! Algorithm B is a **single, view-id-dispatching wrapper** that hosts any of
//! the 5 Phase-1 hand-written views behind a uniform constructor surface. The
//! distinguishing property in 2b is that one constructor (`for_id`) returns
//! a `View` for any of the 5 known shapes — the maintenance loop itself is
//! delegated to the inner hand-written view so update / read / rebuild
//! semantics are bit-identical to the `Strategy::A` baseline.
//!
//! ## g8-clarity-1: Additive, NOT a replacement
//!
//! **Algorithm B runs ALONGSIDE the 5 Phase-1 hand-written views.** The 5
//! hand-written views remain as `Strategy::A` baselines and are NOT
//! subsumed in Phase 2b. The G8-A bench gate measures B vs still-live A.
//! Retirement of any hand-written view is Phase-3+ work and requires the
//! 3 named conditions documented in `r1-ivm-algorithm.json`.
//!
//! ## What was here and is intentionally NOT here in 2b
//!
//! An earlier draft carried a per-input-CID dependency tracker
//! (`BTreeSet<Cid>` + `BTreeMap<Cid, u64>` + an `event_log` Vec). It is
//! REMOVED in 2b per the project's
//! `feedback_engine_primitives_vs_application_layer` principle: paying
//! primitive-level update-hot-path cost (5-10 ns/event of `BTreeSet::insert`
//! that bench-gate measurements show is the entire B-vs-A delta) for a
//! tracker that has zero current consumer is precisely the
//! engine-primitive-vs-application-layer anti-pattern. When a Phase-3+
//! user-registered view lands a consumer that NEEDS the per-input-CID set,
//! the tracker re-lands ALONGSIDE that consumer in the same change so the
//! cost is paid for an actual user. See decision in
//! `.addl/phase-2b/r5-decisions-log.md` G8-A fix-pass entry.
//!
//! ## Dispatch
//!
//! [`AlgorithmBView::for_id`] takes the `view_id` of one of the 5 known
//! shapes and constructs a wrapper that hosts the matching hand-written
//! view internally. Phase-3+ user-registered views land an arity-N dispatch
//! path here — the 5-known-id surface is the Phase-2b shipping shape and is
//! `Result`-returning so unknown ids surface as a typed error rather than
//! silently selecting a fallback inner.
//!
//! ## D8 EXPLICIT-OPT-IN
//!
//! [`AlgorithmBView::strategy`] returns [`Strategy::B`]. There is no
//! constructor that auto-selects between A and B; callers pick at
//! construction time per D8.
//!
//! See `.addl/phase-2b/00-implementation-plan.md` §3 G8-A + §5 D8.

use alloc::boxed::Box;
use alloc::string::{String, ToString};

use benten_graph::ChangeEvent;

use crate::Strategy;
use crate::view::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};
use crate::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

/// Stable view ids the `for_id` dispatcher knows about. Kept as a slice so
/// the unknown-id error surfaces the full set as a diagnostic.
const KNOWN_VIEW_IDS: &[&str] = &[
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Generalized Algorithm B view — single-loop dispatch over the 5 Phase-1
/// view shapes.
///
/// In Phase 2b the wrapper's only job is to host any of the 5 Phase-1 view
/// shapes behind a uniform `for_id`-style constructor. The inner
/// [`Box<dyn View>`] is the matching hand-written implementation; update /
/// read / rebuild / is_stale all delegate to it so the B path is
/// bit-identical to the A baseline AND the bench gate measures the
/// dispatch overhead rather than any speculative tracker cost (see module
/// doc; the prior dep-tracker was removed per
/// `feedback_engine_primitives_vs_application_layer`).
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
}

impl core::fmt::Debug for AlgorithmBView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // `definition` is omitted from the Debug projection on purpose — it
        // is stored for Phase-3+ dispatch traceability but is not part of
        // the runtime-observable surface this Debug impl reflects (the id +
        // inner are what callers diagnose).
        f.debug_struct("AlgorithmBView")
            .field("view_id", &self.view_id)
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl AlgorithmBView {
    /// Construct an Algorithm B view for one of the 5 known view ids.
    ///
    /// Dispatches on `view_id` to the matching hand-written view (which
    /// becomes `inner`). The hand-written view's `update` / `read` /
    /// `rebuild` provide the actual maintenance.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::PatternMismatch`] when `view_id` is not one of
    /// the 5 known Phase-1 ids. The error message lists the valid set so a
    /// caller-side typo surfaces with a fix-it diagnostic rather than the
    /// prior silent-fallback-to-`capability_grants` foot-gun
    /// (cr-g8a-mr-6).
    pub fn for_id(view_id: &str, mut definition: ViewDefinition) -> Result<Self, ViewError> {
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
            unknown => {
                return Err(ViewError::PatternMismatch(format!(
                    "AlgorithmBView::for_id: unknown view id `{unknown}` \
                     (known ids in Phase 2b: {known:?}). Phase-3+ replaces \
                     this branch with arity-N user-registered dispatch.",
                    known = KNOWN_VIEW_IDS
                )));
            }
        };
        Ok(Self {
            view_id: view_id.to_string(),
            definition,
            inner,
        })
    }
}

impl View for AlgorithmBView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        // 2b update path is pure delegation to the inner hand-written
        // view. The single-loop dispatch (selecting `inner` at construction)
        // is what makes this Algorithm B in 2b; per-event work is therefore
        // exactly the inner view's update cost + one virtual call. The
        // bench gate at `tests/algorithm_b_within_20pct_gate.rs` measures
        // that overhead per-view.
        self.inner.update(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        self.inner.read(query)
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        // Delegate to the inner hand-written view's rebuild. The 5 Phase-1
        // views' rebuild() each clear state + reset the budget tracker
        // (see e.g. `content_listing.rs::rebuild_from_scratch` →
        // `BudgetTracker::rebuild`). Without this delegation the wrapper
        // would silently lie about recovery: a wrapped budget-tripped view
        // would stay `Stale` forever even after `rebuild()` returned
        // `Ok(())` (cr-g8a-mr-2).
        self.inner.rebuild()
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
