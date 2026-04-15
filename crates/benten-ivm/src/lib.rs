//! # benten-ivm — Incremental View Maintenance (Phase 1 stubs)
//!
//! Phase 1 ships five hand-written IVM views:
//!
//! 1. Capability grants per entity
//! 2. Event handler dispatch
//! 3. Content listing (paginated, sorted by `createdAt`) — load-bearing for
//!    the crud('post') exit criterion
//! 4. Governance inheritance
//! 5. Version-chain CURRENT pointer resolution
//!
//! All views subscribe to the graph change stream from `benten-graph` and
//! maintain their state incrementally. The evaluator is deliberately ignorant
//! of IVM; IVM is a subscriber, not an engine-internal feature.
//!
//! This file is the R3 stub scaffold — every public item is a `todo!()`
//! placeholder that the R3 unit test suite references. R5 implementation
//! lands in Phase 1 proper.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]

use benten_core::{Cid, Value};
use benten_graph::ChangeEvent;

/// Marker for the current stub phase. Removed when real IVM lands.
pub const STUB_MARKER: &str = "benten-ivm::stub";

/// IVM error type (Phase 1 stub).
///
/// Exposed under two names (`IvmError` and `ViewError`) because different
/// R3 writers named it differently; the aliases let both test files compile.
#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    #[error("view stale: {view_id}")]
    Stale { view_id: String },

    #[error("pattern match failed: {0}")]
    PatternMismatch(String),

    #[error("budget exceeded for view {0}")]
    BudgetExceeded(String),
}

/// Back-compat alias for the error type. Some test files name it `IvmError`.
pub type IvmError = ViewError;

/// Runtime state of a view. `Fresh` means incremental maintenance is caught
/// up; `Stale` means budget exceeded mid-update and reads refuse (strict
/// mode) or return last-known-good (relaxed mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    Fresh,
    Stale,
}

/// The result of a view read — polymorphic; each view defines its own shape.
#[derive(Debug, Clone)]
pub enum ViewResult {
    /// Ordered list of Node CIDs (used by Views 1, 2, 3).
    Cids(Vec<Cid>),
    /// Single Cid pointer (used by View 5).
    Current(Option<Cid>),
    /// Governance rules map (used by View 4).
    Rules(std::collections::BTreeMap<String, Value>),
}

/// The shared trait every IVM view implements.
///
/// **Phase 1 G5 stub.**
pub trait View: Send + Sync {
    /// Ingest a single change event. Implementations update incrementally.
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError>;

    /// Read the view under a per-view query shape.
    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError>;

    /// Rebuild the view from scratch. Used for bootstrap + equivalence tests.
    fn rebuild(&mut self) -> Result<(), ViewError>;

    /// Stable identifier (`"capability_grants"`, `"content_listing"`, ...).
    fn id(&self) -> &str;

    /// True if the view's incremental state is stale (budget exceeded or
    /// corrupted). Read paths return `E_IVM_VIEW_STALE` when true unless
    /// `allow_stale` is set.
    fn is_stale(&self) -> bool;
}

/// Per-view query shape. Phase 1 is untyped JSON-like; a typed variant per
/// view lands in Phase 2.
#[derive(Debug, Clone, Default)]
pub struct ViewQuery {
    pub label: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub anchor_id: Option<u64>,
    pub entity_cid: Option<Cid>,
    pub event_name: Option<String>,
}

/// Content-addressed view definition. `system:IVMView` label.
///
/// **Phase 1 G5 stub.**
#[derive(Debug, Clone, PartialEq)]
pub struct ViewDefinition {
    pub view_id: String,
    pub input_pattern_label: Option<String>,
    pub output_label: String,
}

impl ViewDefinition {
    /// Serialize the definition as a Node suitable for storage.
    pub fn as_node(&self) -> benten_core::Node {
        todo!("ViewDefinition::as_node — G5 (Phase 1)")
    }

    /// Compute the CID of this view definition.
    pub fn cid(&self) -> Result<Cid, benten_core::CoreError> {
        todo!("ViewDefinition::cid — G5 (Phase 1)")
    }
}

// ---------------------------------------------------------------------------
// The five Phase 1 view constructors — each in its own submodule so the
// per-view test file can do `use benten_ivm::views::content_listing::{...}`.
// ---------------------------------------------------------------------------

pub mod views {
    //! Five hand-written Phase 1 IVM views. Each lives in its own submodule
    //! and is re-exported from `views::*` so tests can reach it either way.

    pub use capability_grants::CapabilityGrantsView;
    pub use content_listing::ContentListingView;
    pub use event_handler_dispatch::{
        EventDispatchView, EventDispatchView as EventHandlerDispatchView,
    };
    pub use governance_inheritance::GovernanceInheritanceView;
    pub use version_current::VersionCurrentView;

    pub mod capability_grants {
        use super::super::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};

        /// View 1 — capability grants indexed by entity.
        pub struct CapabilityGrantsView;

        impl CapabilityGrantsView {
            #[must_use]
            pub fn new() -> Self {
                Self
            }
            pub fn definition() -> ViewDefinition {
                ViewDefinition {
                    view_id: "capability_grants".into(),
                    input_pattern_label: Some("CapabilityGrant".into()),
                    output_label: "system:IVMView".into(),
                }
            }
            /// Low-budget test constructor (used by `stale_on_budget_exceeded`).
            #[must_use]
            pub fn with_budget_for_testing(_budget: u64) -> Self {
                Self
            }

            /// Ingest a change directly (test-surface shortcut for edge-case tests).
            pub fn on_change(&mut self, _node: benten_core::Node) {
                todo!("CapabilityGrantsView::on_change — G5 (Phase 1)")
            }

            /// Current view state.
            #[must_use]
            pub fn state(&self) -> super::super::ViewState {
                todo!("CapabilityGrantsView::state — G5 (Phase 1)")
            }

            /// Read grants for an entity.
            pub fn read_for_entity(
                &self,
                _entity: &benten_core::Cid,
            ) -> Result<Vec<benten_core::Cid>, super::super::ViewError> {
                todo!("CapabilityGrantsView::read_for_entity — G5 (Phase 1)")
            }
        }

        impl Default for CapabilityGrantsView {
            fn default() -> Self {
                Self::new()
            }
        }

        /// Alias name used by some tests.
        pub type CapGrants = CapabilityGrantsView;

        impl View for CapabilityGrantsView {
            fn update(&mut self, _event: &benten_graph::ChangeEvent) -> Result<(), ViewError> {
                todo!()
            }
            fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
                todo!()
            }
            fn rebuild(&mut self) -> Result<(), ViewError> {
                todo!()
            }
            fn id(&self) -> &str {
                "capability_grants"
            }
            fn is_stale(&self) -> bool {
                todo!()
            }
        }
    }

    pub mod event_handler_dispatch {
        use super::super::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};

        /// Alias name some R3 tests use; identical type.
        pub type EventHandlerDispatchView = EventDispatchView;

        /// View 2 — event handler dispatch table (event_name → handler_cids).
        pub struct EventDispatchView;

        impl EventDispatchView {
            #[must_use]
            pub fn new() -> Self {
                Self
            }
            pub fn definition() -> ViewDefinition {
                ViewDefinition {
                    view_id: "event_dispatch".into(),
                    input_pattern_label: Some("SubscribesTo".into()),
                    output_label: "system:IVMView".into(),
                }
            }
            #[must_use]
            pub fn with_budget_for_testing(_budget: u64) -> Self {
                Self
            }

            /// Ingest a node-level change directly (test shortcut).
            pub fn on_change(&mut self, _node: benten_core::Node) {
                todo!("EventDispatchView::on_change — G5 (Phase 1)")
            }

            /// Current view state.
            #[must_use]
            pub fn state(&self) -> super::super::ViewState {
                todo!("EventDispatchView::state — G5 (Phase 1)")
            }

            /// Read handler CIDs that subscribe to a given event name.
            pub fn read_handlers_for_event(
                &self,
                _event: &str,
            ) -> Result<Vec<benten_core::Cid>, ViewError> {
                todo!("EventDispatchView::read_handlers_for_event — G5 (Phase 1)")
            }
        }

        impl Default for EventDispatchView {
            fn default() -> Self {
                Self::new()
            }
        }

        impl View for EventDispatchView {
            fn update(&mut self, _event: &benten_graph::ChangeEvent) -> Result<(), ViewError> {
                todo!()
            }
            fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
                todo!()
            }
            fn rebuild(&mut self) -> Result<(), ViewError> {
                todo!()
            }
            fn id(&self) -> &str {
                "event_dispatch"
            }
            fn is_stale(&self) -> bool {
                todo!()
            }
        }
    }

    pub mod content_listing {
        use super::super::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};
        use benten_core::Node;

        /// View 3 — paginated content listing, sorted by createdAt.
        pub struct ContentListingView;

        impl ContentListingView {
            #[must_use]
            pub fn new(_label: impl Into<String>) -> Self {
                Self
            }
            pub fn definition() -> ViewDefinition {
                ViewDefinition {
                    view_id: "content_listing".into(),
                    input_pattern_label: Some("post".into()),
                    output_label: "system:IVMView".into(),
                }
            }
            /// Low-budget test constructor — any non-trivial update trips the
            /// budget and flips state to `Stale`.
            #[must_use]
            pub fn with_budget_for_testing(_budget: u64) -> Self {
                Self
            }

            /// Fallible variant of [`Self::with_budget_for_testing`], used by
            /// tests that pass an invalid budget.
            pub fn try_with_budget(_budget: u64) -> Result<Self, super::super::ViewError> {
                todo!("ContentListingView::try_with_budget — G5 (Phase 1)")
            }

            /// Force a clean rebuild from scratch.
            pub fn rebuild_from_scratch(&mut self) -> Result<(), super::super::ViewError> {
                todo!("ContentListingView::rebuild_from_scratch — G5 (Phase 1)")
            }

            /// Current runtime state (Fresh / Stale).
            #[must_use]
            pub fn state(&self) -> super::super::ViewState {
                todo!("ContentListingView::state — G5 (Phase 1)")
            }

            /// Ingest a change without going through the trait's `update`;
            /// test-surface shortcut for `view_read_allow_stale`.
            pub fn on_change(&mut self, _node: Node) {
                todo!("ContentListingView::on_change — G5 (Phase 1)")
            }

            /// Strict paginated read. Returns `Err(ViewError::Stale)` when
            /// the view's state is stale.
            pub fn read_page(
                &self,
                _offset: usize,
                _limit: usize,
            ) -> Result<Vec<benten_core::Cid>, ViewError> {
                todo!("ContentListingView::read_page — G5 (Phase 1)")
            }

            /// Relaxed paginated read that returns last-known-good on stale.
            pub fn read_page_allow_stale(
                &self,
                _offset: usize,
                _limit: usize,
            ) -> Result<Vec<benten_core::Cid>, ViewError> {
                todo!("ContentListingView::read_page_allow_stale — G5 (Phase 1)")
            }
        }

        impl View for ContentListingView {
            fn update(&mut self, _event: &benten_graph::ChangeEvent) -> Result<(), ViewError> {
                todo!()
            }
            fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
                todo!()
            }
            fn rebuild(&mut self) -> Result<(), ViewError> {
                todo!()
            }
            fn id(&self) -> &str {
                "content_listing"
            }
            fn is_stale(&self) -> bool {
                todo!()
            }
        }
    }

    pub mod governance_inheritance {
        use super::super::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};

        /// Depth cap for governance inheritance traversal (Phase 1).
        pub const MAX_GOVERNANCE_DEPTH: usize = 5;

        /// View 4 — governance inheritance transitive closure.
        pub struct GovernanceInheritanceView;

        impl GovernanceInheritanceView {
            #[must_use]
            pub fn new() -> Self {
                Self
            }
            pub fn definition() -> ViewDefinition {
                ViewDefinition {
                    view_id: "governance_inheritance".into(),
                    input_pattern_label: Some("GovernedBy".into()),
                    output_label: "system:IVMView".into(),
                }
            }
            #[must_use]
            pub fn with_budget_for_testing(_budget: u64) -> Self {
                Self
            }

            /// Ingest a node-level change directly (test shortcut).
            pub fn on_change(&mut self, _node: benten_core::Node) {
                todo!("GovernanceInheritanceView::on_change — G5 (Phase 1)")
            }

            /// Add a GovernedBy edge between two community CIDs.
            pub fn add_edge(
                &mut self,
                _child: &benten_core::Cid,
                _parent: &benten_core::Cid,
            ) -> Result<(), ViewError> {
                todo!("GovernanceInheritanceView::add_edge — G5 (Phase 1)")
            }

            /// Current view state.
            #[must_use]
            pub fn state(&self) -> super::super::ViewState {
                todo!("GovernanceInheritanceView::state — G5 (Phase 1)")
            }

            /// Compute effective rules for an entity (transitive closure).
            #[must_use]
            pub fn effective_rules(&self, _entity: &benten_core::Cid) -> EffectiveRules {
                todo!("GovernanceInheritanceView::effective_rules — G5 (Phase 1)")
            }

            /// Read effective rules — alternative method name some tests use.
            /// Returns a `Result` so the budget-exceeded path can surface
            /// `ViewError::Stale` (different from the in-budget `effective_rules`
            /// which always succeeds).
            pub fn read_effective_rules(
                &self,
                _entity: &benten_core::Cid,
            ) -> Result<EffectiveRules, ViewError> {
                todo!("GovernanceInheritanceView::read_effective_rules — G5 (Phase 1)")
            }
        }

        /// Result of a transitive-closure governance resolution. Carries the
        /// resolved-rules list, the traversal depth reached, and a truncation
        /// flag set when the depth cap stopped traversal.
        #[derive(Debug, Clone)]
        pub struct EffectiveRules {
            depth: usize,
            was_truncated: bool,
            cycle_detected: bool,
            rules: Vec<benten_core::Cid>,
        }

        impl EffectiveRules {
            #[must_use]
            pub fn depth(&self) -> usize {
                self.depth
            }
            #[must_use]
            pub fn was_truncated(&self) -> bool {
                self.was_truncated
            }
            /// Distinguishes a cycle-induced truncation from a depth-cap-induced
            /// one. Added at R4 triage (m5): the two reasons must be separable
            /// for callers to reason about graph shape.
            #[must_use]
            pub fn cycle_detected(&self) -> bool {
                self.cycle_detected
            }
            #[must_use]
            pub fn rules(&self) -> &[benten_core::Cid] {
                &self.rules
            }
        }

        impl Default for GovernanceInheritanceView {
            fn default() -> Self {
                Self::new()
            }
        }

        impl View for GovernanceInheritanceView {
            fn update(&mut self, _event: &benten_graph::ChangeEvent) -> Result<(), ViewError> {
                todo!()
            }
            fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
                todo!()
            }
            fn rebuild(&mut self) -> Result<(), ViewError> {
                todo!()
            }
            fn id(&self) -> &str {
                "governance_inheritance"
            }
            fn is_stale(&self) -> bool {
                todo!()
            }
        }
    }

    pub mod version_current {
        use super::super::{View, ViewDefinition, ViewError, ViewQuery, ViewResult};

        /// View 5 — Anchor → current-version CID pointer.
        pub struct VersionCurrentView;

        impl VersionCurrentView {
            #[must_use]
            pub fn new() -> Self {
                Self
            }
            pub fn definition() -> ViewDefinition {
                ViewDefinition {
                    view_id: "version_current".into(),
                    input_pattern_label: Some("NEXT_VERSION".into()),
                    output_label: "system:IVMView".into(),
                }
            }
            #[must_use]
            pub fn with_budget_for_testing(_budget: u64) -> Self {
                Self
            }

            /// Ingest a node-level change directly (test shortcut).
            pub fn on_change(&mut self, _node: benten_core::Node) {
                todo!("VersionCurrentView::on_change — G5 (Phase 1)")
            }

            /// Current view state.
            #[must_use]
            pub fn state(&self) -> super::super::ViewState {
                todo!("VersionCurrentView::state — G5 (Phase 1)")
            }

            /// Resolve anchor → current-version CID. Accepts either a u64
            /// anchor id or a `&Cid`; the trait-driven overload keeps both
            /// R3 test surfaces compiling.
            pub fn resolve<A: AnchorRef>(
                &self,
                _anchor: A,
            ) -> Result<Option<benten_core::Cid>, ViewError> {
                todo!("VersionCurrentView::resolve — G5 (Phase 1)")
            }
        }

        /// Anchor-reference overload trait — see `VersionCurrentView::resolve`.
        pub trait AnchorRef {}
        impl AnchorRef for u64 {}
        impl AnchorRef for &benten_core::Cid {}
        impl AnchorRef for benten_core::Cid {}

        impl Default for VersionCurrentView {
            fn default() -> Self {
                Self::new()
            }
        }

        impl View for VersionCurrentView {
            fn update(&mut self, _event: &benten_graph::ChangeEvent) -> Result<(), ViewError> {
                todo!()
            }
            fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
                todo!()
            }
            fn rebuild(&mut self) -> Result<(), ViewError> {
                todo!()
            }
            fn id(&self) -> &str {
                "version_current"
            }
            fn is_stale(&self) -> bool {
                todo!()
            }
        }
    }
}

/// The IVM subscriber. Routes change events to views whose input pattern matches.
///
/// **Phase 1 G5-A stub.**
pub struct Subscriber {
    views: Vec<Box<dyn View>>,
}

impl Subscriber {
    #[must_use]
    pub fn new() -> Self {
        Self { views: Vec::new() }
    }

    #[must_use]
    pub fn with_view(mut self, view: Box<dyn View>) -> Self {
        self.views.push(view);
        self
    }

    /// Number of registered views.
    #[must_use]
    pub fn view_count(&self) -> usize {
        self.views.len()
    }

    /// Route a single change event to every matching view.
    /// Returns the number of views that were updated.
    pub fn route_change_event(&mut self, _event: &ChangeEvent) -> Result<usize, ViewError> {
        todo!("Subscriber::route_change_event — G5-A (Phase 1)")
    }
}

impl Default for Subscriber {
    fn default() -> Self {
        Self::new()
    }
}
