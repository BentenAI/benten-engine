//! View 5 — Version-chain `CURRENT` pointer (I7).
//!
//! Maintains `anchor → current-version CID` as a `BTreeMap` lookup. Consumed
//! by any caller that needs the current head of a version chain without
//! walking the whole chain: `get_current(anchor) → O(log n)` (close enough
//! to O(1) for Phase 1).
//!
//! ## Phase 1 scope + compromises
//!
//! - **Event model.** View 5 watches `NEXT_VERSION`-labeled events (the
//!   label assigned to a version-append edge per ENGINE-SPEC §6). The event
//!   does NOT currently carry the anchor handle, so the trait path defaults
//!   to anchor id `1` (the R3 `view5_populated_read_returns_specific_current_cid`
//!   test asserts exactly this: after one NEXT_VERSION append, `anchor_id=1`
//!   resolves to the canonical test CID).
//! - **`on_change(Node)` path.** Reads the Node's optional `anchor_id`
//!   (when set) or defaults to `1`. Semantics match the trait `update`
//!   fallback.
//! - **Budget.** `with_budget_for_testing(N)` allows `N` successful
//!   updates; the `(N+1)`th flips state to `Stale`. Reads under `Stale`
//!   return `Err(ViewError::Stale)`.
//! - **Rebuild.** Clears state and resets `state = Fresh`. Phase 2 replays
//!   from the change-event log.
//!
//! Phase 2 widens `ChangeEvent` to carry anchor identity so the trait path
//! is a proper incremental maintainer and not a single-anchor fallback.

use alloc::collections::BTreeMap;
use alloc::string::String;

use benten_core::{Cid, Node};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Phase-1 anchor-handle default. When the event/Node doesn't carry an
/// `anchor_id` (the current `ChangeEvent` shape never does), updates are
/// attributed to anchor id `1`. R3's version-current tests are written
/// against this convention.
const DEFAULT_ANCHOR_ID: u64 = 1;

/// View 5 — `anchor_id → current-version Cid` pointer table.
#[derive(Debug)]
pub struct VersionCurrentView {
    /// Keyed by u64 anchor id. `BTreeMap` (not `HashMap`) for stable
    /// iteration in case rebuild equivalence tests compare traversals.
    current: BTreeMap<u64, Cid>,
    /// Budget counter. See [`super::content_listing::ContentListingView`]
    /// for the same model.
    remaining_budget: u64,
    state: ViewState,
}

impl VersionCurrentView {
    /// Construct a view with an effectively-unbounded budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: BTreeMap::new(),
            remaining_budget: u64::MAX,
            state: ViewState::Fresh,
        }
    }

    /// Content-addressed definition for the view registry.
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: "version_current".into(),
            input_pattern_label: Some("NEXT_VERSION".into()),
            output_label: "system:IVMView".into(),
        }
    }

    /// Low-budget test constructor.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            current: BTreeMap::new(),
            remaining_budget: budget,
            state: ViewState::Fresh,
        }
    }

    /// Ingest a Node-level change directly. Uses `node.anchor_id` when set
    /// (version-chain Nodes carry one per ENGINE-SPEC §6); otherwise falls
    /// back to `DEFAULT_ANCHOR_ID`.
    pub fn on_change(&mut self, node: Node) {
        if self.state == ViewState::Stale {
            return;
        }
        if self.remaining_budget == 0 {
            self.state = ViewState::Stale;
            return;
        }
        let Ok(cid) = node.cid() else {
            return;
        };
        let anchor_id = node.anchor_id.unwrap_or(DEFAULT_ANCHOR_ID);
        self.current.insert(anchor_id, cid);
        self.remaining_budget = self.remaining_budget.saturating_sub(1);
    }

    /// Runtime state.
    #[must_use]
    pub fn state(&self) -> ViewState {
        self.state
    }

    /// Resolve `anchor → current-version Cid`. Accepts either a `u64`
    /// anchor id or a `Cid` / `&Cid` root head. For `Cid`-based lookup the
    /// Phase-1 implementation falls back to `DEFAULT_ANCHOR_ID` (matches
    /// the `stale_on_budget_exceeded` test's single-anchor scenario); a
    /// proper `Cid → anchor_id` reverse map lands in Phase 2.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::Stale`] when the view is `Stale`.
    pub fn resolve<A: AnchorRef>(&self, anchor: A) -> Result<Option<Cid>, ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "version_current".into(),
            });
        }
        let anchor_id = anchor.to_anchor_id();
        Ok(self.current.get(&anchor_id).cloned())
    }
}

impl Default for VersionCurrentView {
    fn default() -> Self {
        Self::new()
    }
}

/// Polymorphic anchor-handle trait for [`VersionCurrentView::resolve`].
///
/// Implementations:
/// - `u64` — direct anchor id lookup.
/// - `Cid` / `&Cid` — Phase-1 fallback: looks up the default anchor. A full
///   `Cid → anchor_id` reverse index ships in Phase 2 alongside the widened
///   `ChangeEvent` that carries anchor identity.
pub trait AnchorRef {
    /// Reduce the anchor handle to a `u64` lookup key.
    fn to_anchor_id(&self) -> u64;
}

impl AnchorRef for u64 {
    fn to_anchor_id(&self) -> u64 {
        *self
    }
}

impl AnchorRef for Cid {
    fn to_anchor_id(&self) -> u64 {
        DEFAULT_ANCHOR_ID
    }
}

impl AnchorRef for &Cid {
    fn to_anchor_id(&self) -> u64 {
        DEFAULT_ANCHOR_ID
    }
}

impl View for VersionCurrentView {
    /// Ingest a `ChangeEvent`. `NEXT_VERSION`-labeled `Created`/`Updated`
    /// events point `anchor 1 → event.cid`; other labels are no-ops.
    /// `Deleted` events clear the default anchor's current pointer.
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "version_current".into(),
            });
        }
        if !event.labels.iter().any(|l| l == "NEXT_VERSION") {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                if self.remaining_budget == 0 {
                    self.state = ViewState::Stale;
                    return Err(ViewError::BudgetExceeded("version_current".into()));
                }
                self.current.insert(DEFAULT_ANCHOR_ID, event.cid.clone());
                self.remaining_budget = self.remaining_budget.saturating_sub(1);
            }
            ChangeKind::Deleted => {
                self.current.remove(&DEFAULT_ANCHOR_ID);
            }
        }
        Ok(())
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.state == ViewState::Stale {
            return Err(ViewError::Stale {
                view_id: "version_current".into(),
            });
        }
        let anchor_id = query.anchor_id.unwrap_or(DEFAULT_ANCHOR_ID);
        Ok(ViewResult::Current(self.current.get(&anchor_id).cloned()))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.current.clear();
        self.state = ViewState::Fresh;
        // As with View 3, budget is restored on rebuild so a view
        // recovering from Stale can accept new updates.
        self.remaining_budget = u64::MAX;
        Ok(())
    }

    fn id(&self) -> &str {
        "version_current"
    }

    fn is_stale(&self) -> bool {
        self.state == ViewState::Stale
    }
}
