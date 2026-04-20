//! View trait, error type, query/result shapes, budget tracking, and
//! [`ViewDefinition`] content-addressing.
//!
//! **G5-A deliverable (Phase 1).**
//!
//! Every IVM view (the five Phase 1 hand-written views in [`crate::views`])
//! implements [`View`]. Views carry their own state and, when constructed with
//! a budget, trip [`ViewError::BudgetExceeded`] when incremental maintenance
//! would walk more than `max_work_per_update` units — at which point the
//! subscriber marks the view [`ViewState::Stale`] and strict reads return
//! [`ErrorCode::IvmViewStale`].
//!
//! [`ErrorCode::IvmViewStale`]: benten_core::ErrorCode::IvmViewStale

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use benten_core::{Cid, CoreError, Node, Value};
use benten_graph::ChangeEvent;

extern crate alloc;

// ---------------------------------------------------------------------------
// ViewError
// ---------------------------------------------------------------------------

/// Errors that a [`View`] surface can emit.
///
/// Mapped to stable codes in the [`ErrorCode`] catalog so cross-language
/// consumers (TS bindings, CLI) see the same string at every boundary.
///
/// [`ErrorCode`]: benten_core::ErrorCode
#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    /// The view's incremental state is stale — a prior update tripped its
    /// budget and the view has not been rebuilt since. Strict reads refuse;
    /// relaxed reads (`allow_stale = true`) return the last-known-good
    /// snapshot. Maps to [`ErrorCode::IvmViewStale`](benten_core::ErrorCode::IvmViewStale).
    #[error("view stale: {view_id}")]
    Stale {
        /// Stable view identifier (e.g. `"content_listing"`).
        view_id: String,
    },

    /// The query shape does not match any maintained pattern on the view
    /// (e.g. a `ViewQuery` with no `label` or `entity_cid` against a view
    /// that keys on them). Distinct from `Stale`: the view is healthy but
    /// the query is malformed.
    #[error("pattern match failed: {0}")]
    PatternMismatch(String),

    /// The view's per-update budget (`max_work_per_update`) was exhausted
    /// before the update completed. Signals the subscriber to flip the
    /// view to [`ViewState::Stale`]. Carries the view id.
    #[error("budget exceeded for view {0}")]
    BudgetExceeded(String),
}

impl ViewError {
    /// Stable error-catalog code for this error. Lets cross-language bindings
    /// surface the same string every time.
    #[must_use]
    pub fn code(&self) -> benten_core::ErrorCode {
        match self {
            // Stale and BudgetExceeded both map to E_IVM_VIEW_STALE — the
            // budget trip IS the reason the view is stale, so they share a
            // single stable catalog code.
            ViewError::Stale { .. } | ViewError::BudgetExceeded(_) => {
                benten_core::ErrorCode::IvmViewStale
            }
            // PatternMismatch: the caller asked the view for an index
            // partition it doesn't maintain (query shape invalid). r6-err-5
            // introduced `E_IVM_PATTERN_MISMATCH` so this runtime-query
            // shape error no longer shares a code with the registration-
            // time `E_INV_REGISTRATION` catch-all.
            ViewError::PatternMismatch(_) => benten_core::ErrorCode::IvmPatternMismatch,
        }
    }
}

/// Back-compat alias for [`ViewError`]. Some R3 test files name the type
/// `IvmError`; the alias keeps both compile paths working.
pub type IvmError = ViewError;

// ---------------------------------------------------------------------------
// ViewState
// ---------------------------------------------------------------------------

/// Runtime state of a view.
///
/// - `Fresh` — incremental maintenance is caught up; reads return live data.
/// - `Stale` — the budget was exceeded mid-update. Strict reads return
///   [`ViewError::Stale`]; relaxed reads return last-known-good. Phase 1 is
///   terminal until an explicit [`View::rebuild`]; Phase 2 adds async
///   background recompute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    /// Incremental maintenance is caught up; live reads succeed.
    Fresh,
    /// Budget exhausted; view refuses strict reads until rebuilt.
    Stale,
}

// ---------------------------------------------------------------------------
// ViewBudget
// ---------------------------------------------------------------------------

/// Per-view work budget for a single incremental update.
///
/// An update that walks more than `max_work_per_update` units (nodes, edges,
/// or edges visited along a transitive closure — whichever the view uses as
/// its work metric) must return [`ViewError::BudgetExceeded`] so the
/// subscriber flips the view to [`ViewState::Stale`]. Each view may choose
/// its own default.
///
/// `max_work_per_update` must be non-zero; the [`Self::new`] constructor
/// rejects a zero budget rather than silently producing a view that is stale
/// before any data arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewBudget {
    /// Maximum number of work units the view may consume per update before
    /// tripping.
    pub max_work_per_update: u64,
}

impl ViewBudget {
    /// Default per-update budget (1000 units). See ENGINE-SPEC §8.
    pub const DEFAULT: Self = Self {
        max_work_per_update: 1000,
    };

    /// Construct a budget. Rejects `0` because a zero-budget view is stale
    /// before any data arrives — that is a misconfiguration, not a valid
    /// runtime state.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::BudgetExceeded`] (carrying the string `"budget"`)
    /// when `max_work_per_update == 0`.
    pub fn new(max_work_per_update: u64) -> Result<Self, ViewError> {
        if max_work_per_update == 0 {
            return Err(ViewError::BudgetExceeded(String::from("budget")));
        }
        Ok(Self {
            max_work_per_update,
        })
    }
}

impl Default for ViewBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ---------------------------------------------------------------------------
// ViewQuery / ViewResult
// ---------------------------------------------------------------------------

/// Per-view query shape. Phase 1 is a single un-typed record carrying every
/// field any view needs; a typed-per-view variant lands in Phase 2 once the
/// views themselves stabilize.
#[derive(Debug, Clone, Default)]
pub struct ViewQuery {
    /// Label filter (used by [`crate::views::ContentListingView`]).
    pub label: Option<String>,
    /// Page size (used by pagination-aware views).
    pub limit: Option<usize>,
    /// Page offset (used by pagination-aware views).
    pub offset: Option<usize>,
    /// Version-chain anchor id (used by [`crate::views::VersionCurrentView`]).
    pub anchor_id: Option<u64>,
    /// Entity CID filter (used by [`crate::views::CapabilityGrantsView`]
    /// and [`crate::views::GovernanceInheritanceView`]).
    pub entity_cid: Option<Cid>,
    /// Event name filter (used by [`crate::views::EventDispatchView`]).
    pub event_name: Option<String>,
}

/// Polymorphic read result. Each view picks the variant whose shape matches
/// its answer.
#[derive(Debug, Clone)]
pub enum ViewResult {
    /// Ordered list of Node CIDs (views 1, 2, 3).
    Cids(Vec<Cid>),
    /// Single Cid pointer (view 5 — version CURRENT).
    Current(Option<Cid>),
    /// Governance rules map (view 4).
    Rules(BTreeMap<String, Value>),
}

// ---------------------------------------------------------------------------
// View trait
// ---------------------------------------------------------------------------

/// The shared trait every IVM view implements.
///
/// The five Phase 1 views (capability-grants, event-dispatch, content-listing,
/// governance-inheritance, version-current) each implement this trait. The
/// subscriber in [`crate::subscriber`] stores views as `Box<dyn View>` and
/// fans every [`ChangeEvent`] to every view; views filter internally.
///
/// Object-safety: the trait is object-safe so heterogeneous views can coexist
/// inside one subscriber. No generic methods, no `Self: Sized` bounds on
/// anything except helper constructors.
pub trait View: Send + Sync {
    /// Ingest a single change event. Implementations update incrementally.
    ///
    /// Return [`ViewError::BudgetExceeded`] when the per-update budget trips;
    /// the caller (subscriber) converts that into a [`ViewState::Stale`]
    /// transition via [`View::mark_stale`].
    ///
    /// # Errors
    ///
    /// Views may also return [`ViewError::Stale`] if they are already stale
    /// (idempotent: stale stays stale) and want to short-circuit the update.
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError>;

    /// Read the view under a per-view query shape.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::Stale`] when the view is stale (unless the
    /// caller used a relaxed read path). Returns
    /// [`ViewError::PatternMismatch`] for queries that don't name any
    /// index this view maintains.
    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError>;

    /// Rebuild the view from scratch. Used for bootstrap and for recovery
    /// after a stale trip. On success, the view is [`ViewState::Fresh`].
    ///
    /// # Errors
    ///
    /// Rebuild may fail if the view's input source (the graph change stream)
    /// is unavailable; Phase 1 implementations return `Ok(())` unconditionally
    /// because the inputs are kept in-memory alongside the view.
    fn rebuild(&mut self) -> Result<(), ViewError>;

    /// Stable identifier (`"capability_grants"`, `"content_listing"`, ...).
    /// Used for error messages and for stable identification across runs.
    fn id(&self) -> &str;

    /// True if the view's incremental state is stale (budget exceeded).
    /// Strict read paths return [`ViewError::Stale`] when true.
    fn is_stale(&self) -> bool;

    /// Mark the view stale. Called by the subscriber on a
    /// [`ViewError::BudgetExceeded`] trip. Idempotent.
    ///
    /// Default implementation is a no-op so existing views that already
    /// manage state internally compile unchanged; views that want the
    /// subscriber to flip the flag for them should override.
    fn mark_stale(&mut self) {
        // Default: views that track their own state override this.
    }
}

// ---------------------------------------------------------------------------
// ViewDefinition
// ---------------------------------------------------------------------------

/// Content-addressed view definition. Stored as a Node with label
/// `system:IVMView` so the definition itself is content-addressed and can
/// be stably referenced by CID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDefinition {
    /// Stable view id (`"content_listing"`, etc.).
    pub view_id: String,
    /// The Node label the view keys on (e.g. `"post"`, `"CapabilityGrant"`).
    /// `None` for views that key on structural patterns rather than a single
    /// label.
    pub input_pattern_label: Option<String>,
    /// The output label — always `"system:IVMView"` for Phase 1 (every view
    /// definition surfaces as a `system:IVMView` Node).
    pub output_label: String,
}

impl ViewDefinition {
    /// Serialize the definition as a Node suitable for storage.
    ///
    /// The Node carries the `output_label` (`system:IVMView`) as its sole
    /// label and the `view_id` / `input_pattern_label` as properties. The
    /// properties are written into a [`BTreeMap`] for deterministic
    /// iteration so the Node's CID is stable across calls.
    #[must_use]
    pub fn as_node(&self) -> Node {
        let mut props = BTreeMap::new();
        props.insert(String::from("view_id"), Value::text(self.view_id.as_str()));
        if let Some(label) = &self.input_pattern_label {
            props.insert(
                String::from("input_pattern_label"),
                Value::text(label.as_str()),
            );
        }
        Node::new(vec![self.output_label.clone()], props)
    }

    /// Compute the CID of this view definition.
    ///
    /// # Errors
    ///
    /// Propagates `CoreError::Serialize` from the underlying [`Node::cid`]
    /// call. The only way this errors in Phase 1 is if the caller has
    /// stashed a `Value::Float` containing NaN or non-finite in
    /// `input_pattern_label` — which cannot happen because the field is
    /// a `String`. Practically: infallible in all current call-sites, but
    /// the `Result` shape is preserved for parity with [`Node::cid`].
    pub fn cid(&self) -> Result<Cid, CoreError> {
        self.as_node().cid()
    }
}

// ---------------------------------------------------------------------------
// Object-safety compile check
// ---------------------------------------------------------------------------

/// Compile-time assertion that [`View`] is object-safe. If this fails to
/// compile, some method signature on the trait has broken object-safety and
/// the subscriber (which stores `Box<dyn View>`) will stop building.
#[allow(dead_code, reason = "compile-time assertion only")]
fn _assert_view_object_safe() {
    fn _takes(_: Box<dyn View>) {}
}
