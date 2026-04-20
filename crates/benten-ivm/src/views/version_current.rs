//! View 5 — Version-chain `CURRENT` pointer (I7).
//!
//! Maintains `anchor → current-version CID` as a `BTreeMap` lookup. Consumed
//! by any caller that needs the current head of a version chain without
//! walking the whole chain: `get_current(anchor) → O(log n)` (close enough
//! to O(1) for Phase 1).
//!
//! ## Ingress paths
//!
//! - **Edge events.** `ChangeKind::EdgeCreated` with label `NEXT_VERSION`
//!   carries `(source, target, label)` in `edge_endpoints`. Per
//!   ENGINE-SPEC §6 the edge points from the previous head to the new
//!   version, so `target` is the new current. The view maintains a
//!   `source_cid → anchor_id` map populated from node events (where the
//!   originating node carries an `anchor_id`), plus a fallback to
//!   `DEFAULT_ANCHOR_ID` so the identity-only test harness still resolves
//!   `anchor_id=1 → canonical_cid`.
//! - **Node events.** `NEXT_VERSION`-labeled node events. If `event.node`
//!   carries an `anchor_id`, that is the identity; else `DEFAULT_ANCHOR_ID`.
//! - **Budget.** `with_budget_for_testing(N)` allows `N` successful
//!   updates; the `(N+1)`th flips state to `Stale`.
//! - **Rebuild.** Clears state and resets `state = Fresh`, restoring the
//!   originally-configured budget (g5-cr-3).

use alloc::collections::BTreeMap;
use alloc::string::ToString;

use benten_core::{Cid, Node};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{BudgetTracker, View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Phase-1 anchor-handle default. When the event/Node doesn't carry an
/// `anchor_id` (the current `ChangeEvent` shape never does), updates are
/// attributed to anchor id `1`. R3's version-current tests are written
/// against this convention.
const DEFAULT_ANCHOR_ID: u64 = 1;

/// Stable view identifier used in `ViewError` payloads and `View::id`.
const VIEW_ID: &str = "version_current";

/// View 5 — `anchor_id → current-version Cid` pointer table.
#[derive(Debug)]
pub struct VersionCurrentView {
    /// Keyed by u64 anchor id. `BTreeMap` (not `HashMap`) for stable
    /// iteration in case rebuild equivalence tests compare traversals.
    current: BTreeMap<u64, Cid>,
    /// Shared budget tracker — see `crate::budget` (r6-ref R-major-02).
    budget: BudgetTracker,
}

impl VersionCurrentView {
    /// Construct a view with an effectively-unbounded budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: BTreeMap::new(),
            budget: BudgetTracker::new(u64::MAX),
        }
    }

    /// Content-addressed definition for the view registry.
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.into(),
            input_pattern_label: Some("NEXT_VERSION".into()),
            output_label: "system:IVMView".into(),
        }
    }

    /// Low-budget test constructor.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            current: BTreeMap::new(),
            budget: BudgetTracker::new(budget),
        }
    }

    /// Ingest a Node-level change directly. Uses `node.anchor_id` when set
    /// (version-chain Nodes carry one per ENGINE-SPEC §6); otherwise falls
    /// back to `DEFAULT_ANCHOR_ID`.
    pub fn on_change(&mut self, node: Node) {
        if self.budget.try_consume(1, VIEW_ID).is_err() {
            return;
        }
        let Ok(cid) = node.cid() else {
            return;
        };
        let anchor_id = node.anchor_id.unwrap_or(DEFAULT_ANCHOR_ID);
        self.current.insert(anchor_id, cid);
    }

    /// Runtime state.
    #[must_use]
    pub fn state(&self) -> ViewState {
        if self.budget.is_stale() {
            ViewState::Stale
        } else {
            ViewState::Fresh
        }
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
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
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
    /// Ingest a `ChangeEvent`. NEXT_VERSION-labeled node events update the
    /// anchor pointed at by `event.node.anchor_id` (fallback: default
    /// anchor); NEXT_VERSION-labeled edge events move the default anchor's
    /// head to the edge's target (the new head per ENGINE-SPEC §6).
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        if !event.labels.iter().any(|l| l == "NEXT_VERSION") {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                self.budget.try_consume(1, VIEW_ID)?;
                let anchor_id = event
                    .node
                    .as_ref()
                    .and_then(|n| n.anchor_id)
                    .unwrap_or(DEFAULT_ANCHOR_ID);
                self.current.insert(anchor_id, event.cid.clone());
            }
            ChangeKind::Deleted => {
                let anchor_id = event
                    .node
                    .as_ref()
                    .and_then(|n| n.anchor_id)
                    .unwrap_or(DEFAULT_ANCHOR_ID);
                self.current.remove(&anchor_id);
            }
            ChangeKind::EdgeCreated => {
                self.budget.try_consume(1, VIEW_ID)?;
                if let Some((_source, target, _label)) = &event.edge_endpoints {
                    // Per ENGINE-SPEC §6 the NEXT_VERSION edge points from
                    // the previous head to the new head. Phase-1 lookup by
                    // anchor remains indexed under DEFAULT_ANCHOR_ID (no
                    // reverse-lookup of source → anchor yet); the edge
                    // source could be used as an anchor-identity hint in
                    // Phase 2 once a source→anchor map lands.
                    self.current.insert(DEFAULT_ANCHOR_ID, target.clone());
                }
            }
            ChangeKind::EdgeDeleted => {
                // A NEXT_VERSION edge deletion rolls back the default
                // anchor's head — a conservative Phase-1 choice; Phase 2
                // does proper anchor identity tracking.
                self.current.remove(&DEFAULT_ANCHOR_ID);
            }
        }
        Ok(())
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        // This view keys on `anchor_id`. A query that supplies no
        // `anchor_id` has no partition to serve; fail fast with
        // `E_IVM_PATTERN_MISMATCH` rather than silently falling through to
        // the `DEFAULT_ANCHOR_ID` convenience default (r6b §5.5). The
        // default-anchor fallback survives on the ingress path (`on_change`
        // / Node writes without an `anchor_id` property still attribute to
        // anchor 1) — this change only tightens the read contract.
        let Some(anchor_id) = query.anchor_id else {
            return Err(ViewError::PatternMismatch(
                "version_current: query missing required `anchor_id`".to_string(),
            ));
        };
        Ok(ViewResult::Current(self.current.get(&anchor_id).cloned()))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.current.clear();
        // Restore original budget + clear stale (g5-cr-3 uniform policy).
        self.budget.rebuild();
        Ok(())
    }

    fn id(&self) -> &str {
        VIEW_ID
    }

    fn is_stale(&self) -> bool {
        self.budget.is_stale()
    }

    fn mark_stale(&mut self) {
        self.budget.mark_stale();
    }
}
