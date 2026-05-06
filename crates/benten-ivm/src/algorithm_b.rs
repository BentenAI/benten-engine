//! Generalized Algorithm B kernel — Phase-3 G15-A.
//!
//! ## What Algorithm B is post-G15-A
//!
//! Algorithm B is the **single generic IVM kernel** that handles arbitrary
//! `(view_id, label_pattern, projection)` triples. The kernel is internally
//! routed by [`dispatch_for`]:
//!
//! - **Canonical view ids** (`capability_grants`, `event_dispatch`,
//!   `content_listing`, `governance_inheritance`, `version_current`) route
//!   to [`Strategy::A`] — the canonical fast-path classification. The 5
//!   hand-written Phase-1 views remain as the **inner kernels of
//!   Strategy::B** (NOT Strategy::A baselines per `ivm-disagree-1`); the
//!   `Strategy::A` enum variant at the dispatch-classification level is the
//!   "this view-id is on the canonical fast-path" marker.
//! - **User-defined view ids** route to [`Strategy::B`] — the generalized
//!   generic kernel keyed on `(label_pattern, projection)` per
//!   `ivm-major-1` architectural choice (a) `D-PHASE-3-28 RESOLVED`.
//!
//! ## D8-RESOLVED EXPLICIT-OPT-IN — Strategy::A vs Strategy::B router INTERNAL
//!
//! The dispatch router is INTERNAL to the IVM kernel. The engine REFUSES
//! `Strategy::A` user-view registration (per `ivm-major-5` + `D8-RESOLVED`):
//! a user attempting to register a user-view with `Strategy::A` hits
//! `benten_engine::EngineError::ViewStrategyARefused` at
//! `Engine::register_user_view` (plain code ref — `benten-ivm` does NOT
//! depend on `benten-engine` so cross-crate intra-doc links are stable
//! rustdoc errors). The 5 hand-written views are not registered through
//! the user-view surface; they live as inner kernels invoked by
//! Strategy::B's dispatch router for canonical view ids.
//!
//! ## Compromise #11 per-row READ gate composition
//!
//! [`Algorithm::register`] does NOT gate row-level READs itself — gate
//! composition lives at
//! `crates/benten-engine/src/ivm_view_read_gate.rs::IvmViewReadGate`
//! which composes label-hint extraction with the
//! `crates/benten-engine/src/cap_recheck.rs::CapRecheckFn` actor-cap-set
//! check at materialization time (plain code ref instead of intra-doc
//! link because `benten-ivm` does NOT depend on `benten-engine`). The
//! gate is separate from G14-D's delivery-time per-event gate at
//! SUBSCRIBE — both layers must permit a row before it is observable to
//! the actor (deny-from-either-layer wins per cap-r4-3 composition).
//!
//! See `.addl/phase-3/00-implementation-plan.md` §3 G15-A row.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::Strategy;
use crate::view::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};
use crate::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

/// Canonical view ids the dispatch router classifies as [`Strategy::A`]
/// (canonical fast-path). User-defined view ids route to [`Strategy::B`]
/// (generic kernel).
const CANONICAL_VIEW_IDS: &[&str] = &[
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Stable mapping from canonical view id → hardcoded `input_pattern_label`
/// for the four canonical views whose hand-written dispatch arms IGNORE
/// caller-supplied label and use a fixed value. `content_listing` is
/// intentionally absent — its arm honors `definition.input_pattern_label`.
///
/// Surfaced as a `pub` accessor (`hardcoded_label_for_id`) so the engine's
/// `register_user_view` boundary can fail-loud when a caller supplies a
/// canonical id + a mismatching label (Phase-2b R6-R3 `r6-r3-ivm-1` closure;
/// Phase-3 G15-A preserves the same fail-loud guard).
const CANONICAL_HARDCODED_LABELS: &[(&str, &str)] = &[
    ("capability_grants", "system:CapabilityGrant"),
    ("version_current", "NEXT_VERSION"),
    ("event_dispatch", "system:EventDispatch"),
    ("governance_inheritance", "system:GovernanceInheritance"),
];

/// Return the hardcoded `input_pattern_label` for one of the four canonical
/// view ids whose hand-written dispatch arm ignores caller-supplied label.
/// Returns `None` for `content_listing` (which honors the supplied label)
/// + for any user-defined id outside the canonical set.
///
/// Used by `Engine::register_user_view` to surface
/// `benten_engine::EngineError::ViewLabelMismatch` (catalog code
/// `E_VIEW_LABEL_MISMATCH`) when the caller supplies a canonical id +
/// a label that disagrees with the hardcoded value.
#[must_use]
pub fn hardcoded_label_for_id(view_id: &str) -> Option<&'static str> {
    CANONICAL_HARDCODED_LABELS
        .iter()
        .find_map(|(id, label)| (*id == view_id).then_some(*label))
}

/// Is `view_id` one of the 5 canonical Phase-1 view ids?
///
/// Used by [`dispatch_for`] to classify which strategy lane the view-id
/// routes to internally. NOT exposed at the engine boundary — the engine
/// only consumes [`Strategy`] (per CLAUDE.md baked-in #2: "the engine names
/// `benten_ivm::Strategy` as the dispatch type but no `View` / algorithm
/// internals leak through").
#[must_use]
pub fn is_canonical_view_id(view_id: &str) -> bool {
    CANONICAL_VIEW_IDS.contains(&view_id)
}

/// INTERNAL Strategy::A vs Strategy::B dispatch router.
///
/// Classifies a view id into the strategy lane the kernel will use:
///
/// - Canonical view ids → [`Strategy::A`] (canonical fast-path, hand-written
///   inner kernels).
/// - Non-canonical / user-defined view ids → [`Strategy::B`] (generalized
///   generic kernel keyed on `(label_pattern, projection)`).
///
/// Per `D8-RESOLVED` the router is INTERNAL: callers do not pick the
/// strategy at the engine boundary; user-view registration always runs under
/// Strategy::B (the engine refuses Strategy::A user-view registration per
/// `ivm-major-5`). The 5 hand-written canonical views are not user-view
/// registrations — they are inner kernels invoked by Strategy::B's
/// dispatch router when a canonical id is materialized. The `Strategy::A`
/// classification at this level is the "view-id is on the canonical
/// fast-path" marker.
#[must_use]
pub fn dispatch_for(view_id: &str) -> Strategy {
    if is_canonical_view_id(view_id) {
        Strategy::A
    } else {
        Strategy::B
    }
}

/// Label-pattern selector consumed by the generalized kernel.
///
/// Phase-3 G15-A ships `Exact` + `AnchorPrefix`; G15-B's `PrefixMatcher`
/// selector type lifts the engine-side surface for `AnchorPrefix` to
/// genuine prefix matching (the kernel here exposes the pattern surface
/// disjoint from the engine-side selector type per `seq-blocker-3`
/// repartition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelPattern {
    /// Exact label equality. `LabelPattern::Exact("post")` matches Nodes
    /// whose first label is `"post"`.
    Exact(String),
    /// Anchor-prefix match. `LabelPattern::AnchorPrefix("crud:")` matches
    /// Nodes whose first label starts with `"crud:"` (e.g. `"crud:post"`,
    /// `"crud:user"`). Genuine prefix semantics — NOT the Phase-2b stub
    /// that silently coerced to label equality.
    AnchorPrefix(String),
}

impl LabelPattern {
    /// Convenience constructor for an exact-label pattern.
    #[must_use]
    pub fn exact(label: impl Into<String>) -> Self {
        Self::Exact(label.into())
    }

    /// Convenience constructor for an anchor-prefix pattern.
    #[must_use]
    pub fn anchor_prefix(prefix: impl Into<String>) -> Self {
        Self::AnchorPrefix(prefix.into())
    }

    /// Does `label` match this pattern?
    #[must_use]
    pub fn matches(&self, label: &str) -> bool {
        match self {
            LabelPattern::Exact(target) => label == target.as_str(),
            LabelPattern::AnchorPrefix(prefix) => label.starts_with(prefix.as_str()),
        }
    }

    /// Surface as a stable string for `ViewDefinition.input_pattern_label`
    /// content-addressing. `Exact("post") -> "post"`,
    /// `AnchorPrefix("crud:") -> "crud:"`. The kind-disambiguation lives in
    /// a sibling `input_pattern_kind` field on the persisted Node (see
    /// `engine_views.rs::register_user_view`).
    #[must_use]
    pub fn as_label_str(&self) -> &str {
        match self {
            LabelPattern::Exact(s) | LabelPattern::AnchorPrefix(s) => s.as_str(),
        }
    }
}

/// Projection — Phase-3 G15-A ships the no-op `AllProps` projection (every
/// matched Node yielded as-is). Future shape narrowing (`PropSubset`,
/// `Computed`) lifts to a richer enum without breaking the kernel surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Projection {
    /// Yield matched Nodes unchanged.
    AllProps,
}

impl Projection {
    /// Convenience constructor for the no-op projection.
    #[must_use]
    pub fn all_props() -> Self {
        Self::AllProps
    }

    /// Apply the projection to a Node. `AllProps` is the identity.
    #[must_use]
    pub fn apply(&self, node: Node) -> Node {
        match self {
            Projection::AllProps => node,
        }
    }
}

/// Errors specific to the generalized Algorithm B kernel surface
/// (`Algorithm::register` / `Algorithm::try_register`). Distinct from
/// [`ViewError`] which surfaces from the per-event update path.
#[derive(Debug, thiserror::Error)]
pub enum AlgorithmError {
    /// Caller supplied a canonical view id with a label_pattern that
    /// disagrees with the canonical view's hardcoded label. The fail-loud
    /// guard prevents silently materialising a view that excludes its
    /// declared surface (e.g. `crud:post` with label_pattern `"user"`).
    #[error(
        "view-label mismatch: view_id `{view_id}` requires label `{expected_label}` but pattern `{got_pattern:?}` does not match"
    )]
    ViewLabelMismatch {
        /// The canonical view id supplied.
        view_id: String,
        /// The hardcoded label the canonical view requires.
        expected_label: String,
        /// The pattern the caller supplied.
        got_pattern: LabelPattern,
    },
}

/// Generic single-loop kernel for non-canonical view ids (Strategy::B
/// inner kernel for user-defined view ids per `D-PHASE-3-28 RESOLVED`).
///
/// Maintains a `BTreeSet<Cid>` of Nodes whose first label matches the
/// `label_pattern`. The set is rebuilt from scratch on
/// [`crate::View::rebuild`]; per-event `update` adds on `Created` /
/// `Modified` (when the new label still matches) and removes on `Deleted`.
///
/// **NOT exposed at the engine boundary** per CLAUDE.md baked-in #2 (the
/// engine names `Strategy` only). Engine-side construction goes through
/// [`AlgorithmBView::for_id`] / [`Algorithm::register`].
#[derive(Debug)]
struct GenericKernel {
    view_id: String,
    label_pattern: LabelPattern,
    #[allow(
        dead_code,
        reason = "projection currently no-op; placeholder for Phase-3+ narrowing"
    )]
    projection: Projection,
    /// Maintained set of matched Node CIDs sorted by `Cid`'s `Ord` impl
    /// (lexicographic-on-CID-bytes; deterministic across runs). Note: a
    /// `BTreeSet` is NOT insertion-ordered — it iterates in `Ord` order;
    /// the determinism guarantee here comes from that `Ord` ordering, not
    /// from FIFO insertion.
    entries: BTreeSet<Cid>,
    /// Stale flag — flipped when `mark_stale` fires.
    stale: bool,
}

impl GenericKernel {
    fn new(view_id: String, label_pattern: LabelPattern, projection: Projection) -> Self {
        Self {
            view_id,
            label_pattern,
            projection,
            entries: BTreeSet::new(),
            stale: false,
        }
    }

    /// Test the Node's FIRST label against this kernel's label pattern.
    ///
    /// Empty-label Nodes never match — a Node with `labels.is_empty()`
    /// has no "first label" to test, so `first()` returns `None` and the
    /// `is_some_and` arm short-circuits to `false`. The first-label-only
    /// convention is shared with the 5 Phase-1 hand-written canonical
    /// views (e.g. `ContentListingView::matches_label`); secondary labels
    /// are intentionally NOT consulted at the kernel boundary. Matchers
    /// that need multi-label semantics belong at a higher selector layer
    /// (named in `docs/future/phase-3-backlog.md` §5.1-followup-b for
    /// edge-traversal-keyed views).
    fn first_label_matches(&self, node: &Node) -> bool {
        node.labels
            .first()
            .is_some_and(|l| self.label_pattern.matches(l.as_str()))
    }
}

impl View for GenericKernel {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.stale {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                if let Some(node) = event.node.as_ref()
                    && self.first_label_matches(node)
                {
                    self.entries.insert(event.cid);
                }
            }
            ChangeKind::Deleted => {
                self.entries.remove(&event.cid);
            }
            // Edge events do not affect Node-keyed views.
            ChangeKind::EdgeCreated | ChangeKind::EdgeDeleted => {}
        }
        Ok(())
    }

    fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: self.view_id.clone(),
            });
        }
        Ok(ViewResult::Cids(self.entries.iter().copied().collect()))
    }

    fn read_allow_stale(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
        Ok(ViewResult::Cids(self.entries.iter().copied().collect()))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        // Rebuild in the generic kernel is a no-op: the kernel has no
        // external input source — it ingests events through `update` only.
        // Flipping fresh is the contract. Phase-3+ event-replay rebuild
        // wires the snapshot store; until then `rebuild` clears + resets
        // fresh so a previously stale-tripped view is observably re-armed.
        self.entries.clear();
        self.stale = false;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.view_id
    }

    fn is_stale(&self) -> bool {
        self.stale
    }

    fn mark_stale(&mut self) {
        self.stale = true;
    }

    fn strategy(&self) -> Strategy {
        Strategy::B
    }
}

/// Generalized Algorithm B view — single-kernel wrapper handling either
/// canonical or user-defined view ids.
///
/// - For canonical view ids the inner kernel is one of the 5 Phase-1
///   hand-written views (per `ivm-disagree-1` they are inner kernels of
///   Strategy::B, NOT Strategy::A baselines).
/// - For user-defined view ids the inner kernel is `GenericKernel`,
///   keyed on `(label_pattern, projection)`.
///
/// `View::strategy` returns [`Strategy::B`] for both — the wrapper itself
/// "is" Strategy::B per `D-PHASE-3-28 RESOLVED`. The Strategy::A
/// classification at [`dispatch_for`] is INTERNAL routing, not the
/// engine-boundary strategy of the resulting view.
pub struct AlgorithmBView {
    /// Stable view id.
    view_id: String,
    /// Content-addressed view definition used for registration. Stored on
    /// the wrapper for traceability.
    #[allow(
        dead_code,
        reason = "stored for traceability + future dispatch surface"
    )]
    definition: ViewDefinition,
    /// Inner kernel — either a canonical hand-written view or
    /// `GenericKernel` for user-defined ids.
    inner: Box<dyn View>,
}

impl core::fmt::Debug for AlgorithmBView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AlgorithmBView")
            .field("view_id", &self.view_id)
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl AlgorithmBView {
    /// Construct an Algorithm B view for one of the 5 canonical view ids.
    ///
    /// Phase-2b shipping shape — the inner kernel is one of the 5
    /// hand-written Phase-1 views. The Algorithm B wrapper "is"
    /// Strategy::B; the inner kernel is the canonical fast-path
    /// classified as [`Strategy::A`] by [`dispatch_for`] but invoked
    /// through Strategy::B's dispatch router (per `ivm-disagree-1`).
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::PatternMismatch`] when `view_id` is not one of
    /// the 5 canonical Phase-1 ids. Use [`AlgorithmBView::register`] for
    /// user-defined view ids.
    pub fn for_id(view_id: &str, mut definition: ViewDefinition) -> Result<Self, ViewError> {
        // Stamp the stored definition with `Strategy::B` for traceability —
        // the wrapper is Strategy::B at the engine boundary regardless of
        // which inner kernel the dispatch router selected.
        definition.strategy = Strategy::B;
        let inner: Box<dyn View> = match view_id {
            "capability_grants" => Box::new(CapabilityGrantsView::new()),
            "event_dispatch" => Box::new(EventDispatchView::new()),
            "content_listing" => {
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
                    "AlgorithmBView::for_id: unknown canonical view id `{unknown}` \
                     (canonical ids: {known:?}). Use AlgorithmBView::register for \
                     user-defined view ids.",
                    known = CANONICAL_VIEW_IDS
                )));
            }
        };
        Ok(Self {
            view_id: view_id.to_string(),
            definition,
            inner,
        })
    }

    /// Register an Algorithm B view for an arbitrary
    /// `(view_id, label_pattern, projection)` triple. Routes through
    /// [`dispatch_for`]:
    ///
    /// - canonical view ids → inner kernel is the matching hand-written
    ///   Phase-1 view (with `label_pattern` validated against the canonical
    ///   hardcoded label, fail-loud on mismatch).
    /// - user-defined view ids → inner kernel is `GenericKernel`.
    ///
    /// # Errors
    ///
    /// Returns [`AlgorithmError::ViewLabelMismatch`] when `view_id` is
    /// canonical and `label_pattern` does not select the canonical
    /// hardcoded label.
    pub fn register(
        view_id: &str,
        label_pattern: LabelPattern,
        projection: Projection,
    ) -> Result<Self, AlgorithmError> {
        // Fail-loud guard: canonical id + mismatched label_pattern. The
        // canonical hand-written view's dispatch arm ignores the supplied
        // label (uses its hardcoded value) — silently accepting a
        // mismatched pattern would yield a view filtered on the WRONG
        // label. Fail-loud at registration time per Phase-2b R6-R3
        // `r6-r3-ivm-1` precedent.
        //
        // **Bounded by `LabelPattern::matches`** — `AnchorPrefix("")`
        // prefix-matches every label (including the canonical hardcoded
        // label), so this guard does NOT fire for an empty-prefix
        // registration. The data-correctness implications are zero in
        // practice (the hand-written canonical kernel ignores the
        // supplied pattern entirely + uses its hardcoded label), but the
        // tightening — banning AnchorPrefix on canonical-id registration
        // outright — is named in `docs/future/phase-3-backlog.md` §5.1
        // as a follow-on per `g15a-mr-minor-4`.
        if let Some(hardcoded) = hardcoded_label_for_id(view_id)
            && !label_pattern.matches(hardcoded)
        {
            return Err(AlgorithmError::ViewLabelMismatch {
                view_id: view_id.to_string(),
                expected_label: hardcoded.to_string(),
                got_pattern: label_pattern,
            });
        }
        // `content_listing` is canonical but its arm honors the supplied
        // label. For non-canonical ids the same fail-loud principle does
        // NOT apply (any label_pattern is permitted).
        if is_canonical_view_id(view_id) {
            // Canonical lane: surface the hand-written inner kernel via
            // `for_id` — the input_pattern_label is the pattern's stable
            // string surface (used by `content_listing` for its label arg
            // and stored in the definition for the 4 hardcoded views).
            let definition = ViewDefinition {
                view_id: view_id.to_string(),
                input_pattern_label: Some(label_pattern.as_label_str().to_string()),
                output_label: "system:IVMView".to_string(),
                strategy: Strategy::B,
            };
            // SAFETY: canonical ids always succeed in `for_id`.
            let view = Self::for_id(view_id, definition).expect(
                "canonical view id resolved by is_canonical_view_id MUST succeed in for_id; \
                 dispatch table inconsistency is a programmer error",
            );
            return Ok(view);
        }
        // Non-canonical lane: instantiate the generic kernel.
        let definition = ViewDefinition {
            view_id: view_id.to_string(),
            input_pattern_label: Some(label_pattern.as_label_str().to_string()),
            output_label: "system:IVMView".to_string(),
            strategy: Strategy::B,
        };
        let inner = Box::new(GenericKernel::new(
            view_id.to_string(),
            label_pattern,
            projection,
        ));
        Ok(Self {
            view_id: view_id.to_string(),
            definition,
            inner,
        })
    }

    /// Try-shape of [`Self::register`]. Alias retained for symmetry with
    /// the test pin's `try_register` shape; behavior is identical.
    ///
    /// # Errors
    ///
    /// See [`Self::register`].
    pub fn try_register(
        view_id: &str,
        label_pattern: LabelPattern,
        projection: Projection,
    ) -> Result<Self, AlgorithmError> {
        Self::register(view_id, label_pattern, projection)
    }

    /// Materialize the kernel's current set of CIDs as a flat list.
    ///
    /// Phase-3 G15-A surface — the per-row READ gate composition lives at
    /// `crates/benten-engine/src/ivm_view_read_gate.rs`; this method is
    /// the unfiltered materialization the gate then row-filters.
    #[must_use]
    pub fn materialize_full(&self) -> Vec<Cid> {
        match self.inner.read(&ViewQuery::default()) {
            Ok(ViewResult::Cids(cids)) => cids,
            Ok(ViewResult::Current(Some(cid))) => vec![cid],
            Ok(ViewResult::Current(None) | ViewResult::Rules(_)) => Vec::new(),
            Err(_) => Vec::new(),
        }
    }
}

impl View for AlgorithmBView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.inner.update(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        self.inner.read(query)
    }

    fn read_allow_stale(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        self.inner.read_allow_stale(query)
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
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

/// Compatibility alias matching the test pin's `Algorithm` module-path
/// shape (e.g. `benten_ivm::algorithm_b::Algorithm::register(...)`).
pub type Algorithm = AlgorithmBView;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::{Cid, Node, Value};
    use benten_graph::{ChangeEvent, ChangeKind};

    fn dummy_cid_for(label: &str, idx: u64) -> Cid {
        let mut props = alloc::collections::BTreeMap::new();
        props.insert(String::from("seq"), Value::Int(idx as i64));
        let node = Node::new(vec![label.to_string()], props);
        node.cid().unwrap()
    }

    fn make_event(kind: ChangeKind, label: &str, idx: u64) -> ChangeEvent {
        let mut props = alloc::collections::BTreeMap::new();
        props.insert(String::from("seq"), Value::Int(idx as i64));
        let node = Node::new(vec![label.to_string()], props);
        let cid = node.cid().unwrap();
        ChangeEvent {
            cid,
            labels: vec![label.to_string()],
            kind,
            tx_id: idx,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: Some(node),
            edge_endpoints: None,
        }
    }

    #[test]
    fn dispatch_for_canonical_routes_to_strategy_a() {
        for id in CANONICAL_VIEW_IDS {
            assert_eq!(dispatch_for(id), Strategy::A, "canonical id {id} -> A");
        }
    }

    #[test]
    fn dispatch_for_user_defined_routes_to_strategy_b() {
        assert_eq!(dispatch_for("custom:posts_by_author"), Strategy::B);
        assert_eq!(dispatch_for("user:my_view"), Strategy::B);
        assert_eq!(dispatch_for(""), Strategy::B); // empty is non-canonical
    }

    #[test]
    fn label_pattern_exact_only_matches_equality() {
        let p = LabelPattern::exact("post");
        assert!(p.matches("post"));
        assert!(!p.matches("user"));
        assert!(!p.matches("posts"));
    }

    #[test]
    fn label_pattern_anchor_prefix_matches_prefix_not_equality() {
        let p = LabelPattern::anchor_prefix("crud:");
        assert!(p.matches("crud:post"));
        assert!(p.matches("crud:user"));
        assert!(p.matches("crud:"));
        assert!(!p.matches("post"));
    }

    #[test]
    fn register_user_defined_view_with_exact_label_succeeds() {
        let view = AlgorithmBView::register(
            "custom:posts",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .expect("user view + matching pattern succeeds");
        assert_eq!(view.id(), "custom:posts");
        assert_eq!(view.strategy(), Strategy::B);
    }

    #[test]
    fn register_canonical_view_with_mismatched_label_pattern_fails_loud() {
        let err = AlgorithmBView::register(
            "capability_grants",
            LabelPattern::exact("user"),
            Projection::all_props(),
        )
        .expect_err("canonical id + mismatched label MUST fail-loud");
        match err {
            AlgorithmError::ViewLabelMismatch {
                view_id,
                expected_label,
                ..
            } => {
                assert_eq!(view_id, "capability_grants");
                assert_eq!(expected_label, "system:CapabilityGrant");
            }
        }
    }

    #[test]
    fn register_canonical_view_with_matching_label_pattern_succeeds() {
        let view = AlgorithmBView::register(
            "capability_grants",
            LabelPattern::exact("system:CapabilityGrant"),
            Projection::all_props(),
        )
        .expect("canonical id + matching pattern succeeds");
        assert_eq!(view.id(), "capability_grants");
    }

    #[test]
    fn register_content_listing_with_arbitrary_label_succeeds() {
        // content_listing's arm honors the supplied label (no hardcoded
        // label) — any LabelPattern::Exact is permitted.
        let view = AlgorithmBView::register(
            "content_listing",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .expect("content_listing with arbitrary exact label succeeds");
        assert_eq!(view.id(), "content_listing");
    }

    #[test]
    fn generic_kernel_drops_silent_coerce_to_content_listing() {
        let mut view = AlgorithmBView::register(
            "custom:posts_by_author",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        view.update(&make_event(ChangeKind::Created, "post", 1))
            .unwrap();
        view.update(&make_event(ChangeKind::Created, "user", 2))
            .unwrap();
        view.update(&make_event(ChangeKind::Created, "post", 3))
            .unwrap();
        let result = view.read(&ViewQuery::default()).unwrap();
        match result {
            ViewResult::Cids(cids) => {
                assert_eq!(cids.len(), 2, "only post-labeled events admitted");
                let post1 = dummy_cid_for("post", 1);
                let post3 = dummy_cid_for("post", 3);
                assert!(cids.contains(&post1));
                assert!(cids.contains(&post3));
                let user2 = dummy_cid_for("user", 2);
                assert!(!cids.contains(&user2), "user-labeled event MUST NOT appear");
            }
            other => panic!("expected Cids, got {other:?}"),
        }
    }

    #[test]
    fn generic_kernel_anchor_prefix_pattern_drives_correct_subset() {
        let mut view = AlgorithmBView::register(
            "custom:by_prefix",
            LabelPattern::anchor_prefix("crud:"),
            Projection::all_props(),
        )
        .unwrap();
        view.update(&make_event(ChangeKind::Created, "crud:post", 1))
            .unwrap();
        view.update(&make_event(ChangeKind::Created, "crud:user", 2))
            .unwrap();
        view.update(&make_event(ChangeKind::Created, "post", 3))
            .unwrap();
        let result = view.read(&ViewQuery::default()).unwrap();
        match result {
            ViewResult::Cids(cids) => assert_eq!(cids.len(), 2, "crud:* matches both events"),
            other => panic!("expected Cids, got {other:?}"),
        }
    }

    #[test]
    fn generic_kernel_delete_removes_entry() {
        let mut view = AlgorithmBView::register(
            "custom:del",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        view.update(&make_event(ChangeKind::Created, "post", 1))
            .unwrap();
        view.update(&make_event(ChangeKind::Deleted, "post", 1))
            .unwrap();
        let result = view.read(&ViewQuery::default()).unwrap();
        match result {
            ViewResult::Cids(cids) => assert!(cids.is_empty(), "deleted CID removed"),
            other => panic!("expected Cids, got {other:?}"),
        }
    }

    #[test]
    fn generic_kernel_rebuild_resets_stale() {
        let mut view = AlgorithmBView::register(
            "custom:rebuild",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        view.mark_stale();
        assert!(view.is_stale());
        view.rebuild().unwrap();
        assert!(!view.is_stale());
    }

    #[test]
    fn for_id_canonical_succeeds_unknown_id_errors() {
        let def = ViewDefinition {
            view_id: "capability_grants".to_string(),
            input_pattern_label: Some("system:CapabilityGrant".to_string()),
            output_label: "system:IVMView".to_string(),
            strategy: Strategy::B,
        };
        let _ = AlgorithmBView::for_id("capability_grants", def).unwrap();
        let bad_def = ViewDefinition {
            view_id: "user:custom".to_string(),
            input_pattern_label: Some("post".to_string()),
            output_label: "system:IVMView".to_string(),
            strategy: Strategy::B,
        };
        let err = AlgorithmBView::for_id("user:custom", bad_def).unwrap_err();
        match err {
            ViewError::PatternMismatch(msg) => {
                assert!(msg.contains("user:custom"));
            }
            other => panic!("expected PatternMismatch, got {other:?}"),
        }
    }

    #[test]
    fn algorithm_b_view_strategy_is_b_for_both_lanes() {
        // Canonical lane.
        let canonical = AlgorithmBView::register(
            "content_listing",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        assert_eq!(canonical.strategy(), Strategy::B);
        // User-defined lane.
        let user = AlgorithmBView::register(
            "custom:foo",
            LabelPattern::exact("foo"),
            Projection::all_props(),
        )
        .unwrap();
        assert_eq!(user.strategy(), Strategy::B);
    }
}
