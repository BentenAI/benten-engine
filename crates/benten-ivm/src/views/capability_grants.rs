//! View 1 — Capability grants indexed by entity (I3).
//!
//! Maintains a map `entity_cid → {grant_cids}` so capability lookups for a
//! given entity are O(1) after a one-time hash-map hit. Inputs:
//!
//! - `ChangeEvent` with label `"CapabilityGrant"` and kind `Created` / `Updated`
//!   adds the grant CID to the entity's grant set. Phase 1 uses the event's
//!   own CID as the entity key (the `ChangeEvent` shape does not yet carry
//!   property data — see the G5-B coordination note below).
//! - `ChangeEvent` with label `"CapabilityGrant"` and kind `Deleted` removes
//!   the grant from every entity's set.
//!
//! ## G5-B coordination note (ChangeEvent shape)
//!
//! The current `ChangeEvent` (post-G3) carries only `cid`, `labels`, `kind`,
//! `tx_id`, and optional attribution CIDs — not the Node's property body. The
//! Phase-1 "entity" key is therefore the event's own CID. When `ChangeEvent`
//! grows to carry the grant's `grantee` property (Phase 2 or a G3 fix-pass),
//! this view's `update` path extracts that property instead. The public
//! read shape (`entity_cid → {grant_cids}`) is stable across that refactor.
//!
//! ## Budget
//!
//! `with_budget_for_testing(N)` caps the number of `on_change` applications to
//! `N` successful updates; the `(N+1)`th flips the view to `ViewState::Stale`
//! and subsequent reads return `ViewError::Stale { view_id: "capability_grants" }`
//! (the stable error code is `E_IVM_VIEW_STALE`). The default constructor
//! `new()` installs an effectively-unlimited budget.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use benten_core::Cid;
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Stable view identifier used in error messages and `View::id`.
const VIEW_ID: &str = "capability_grants";

/// Default budget — large enough that unit tests using `new()` never trip.
const UNLIMITED_BUDGET: u64 = u64::MAX;

/// View 1 — capability grants indexed by entity.
#[derive(Debug)]
pub struct CapabilityGrantsView {
    /// entity CID → set of grant CIDs.
    by_entity: BTreeMap<Cid, BTreeSet<Cid>>,
    /// Remaining update budget. Decremented by each `on_change` / `update`
    /// call; when it would go negative, the view flips to `Stale`.
    remaining_budget: u64,
    /// Whether the view is currently stale.
    stale: bool,
}

/// Back-compat alias. Some tests / docs write `CapGrants` for brevity.
pub type CapGrants = CapabilityGrantsView;

impl CapabilityGrantsView {
    /// Construct a fresh view with an effectively unlimited budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_entity: BTreeMap::new(),
            remaining_budget: UNLIMITED_BUDGET,
            stale: false,
        }
    }

    /// Content-addressed definition for this view. Written to the graph as a
    /// `system:IVMView` Node.
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.to_string(),
            input_pattern_label: Some("CapabilityGrant".to_string()),
            output_label: "system:IVMView".to_string(),
        }
    }

    /// Test-only constructor that caps update budget at `budget` successful
    /// applications. The `(budget + 1)`th update trips the view to `Stale`.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            by_entity: BTreeMap::new(),
            remaining_budget: budget,
            stale: false,
        }
    }

    /// Ingest a node-level change directly, bypassing the trait. Used by the
    /// budget-exceeded test harness which exercises the `Node` shape rather
    /// than the `ChangeEvent` shape.
    pub fn on_change(&mut self, node: benten_core::Node) {
        if self.stale {
            return;
        }
        if self.remaining_budget == 0 {
            self.stale = true;
            return;
        }
        self.remaining_budget -= 1;

        // Phase 1: use the node CID as both the entity key and the grant CID.
        // See the crate-level coordination note; the mapping is stable when
        // ChangeEvent grows property attachment in Phase 2.
        if let Ok(cid) = node.cid() {
            self.by_entity.entry(cid.clone()).or_default().insert(cid);
        }
    }

    /// Current runtime state.
    #[must_use]
    pub fn state(&self) -> ViewState {
        if self.stale {
            ViewState::Stale
        } else {
            ViewState::Fresh
        }
    }

    /// Direct read (bypasses the trait). Returns the grant-CID set for
    /// `entity`, in sorted order. Refuses with `ViewError::Stale` when the
    /// view is stale.
    pub fn read_for_entity(&self, entity: &Cid) -> Result<Vec<Cid>, ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: VIEW_ID.to_string(),
            });
        }
        Ok(self
            .by_entity
            .get(entity)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// Internal: apply a `ChangeEvent`, decrementing budget. Returns `Ok` even
    /// when the event is a no-op (wrong label) so the subscriber's per-event
    /// accounting stays simple.
    fn apply_event(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: VIEW_ID.to_string(),
            });
        }
        if self.remaining_budget == 0 {
            self.stale = true;
            return Err(ViewError::BudgetExceeded(VIEW_ID.to_string()));
        }
        self.remaining_budget -= 1;

        // Filter: only `CapabilityGrant`-labeled events are relevant.
        if !event.has_label("CapabilityGrant") {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                self.by_entity
                    .entry(event.cid.clone())
                    .or_default()
                    .insert(event.cid.clone());
            }
            ChangeKind::Deleted => {
                // Remove the grant from every entity's set. The set is
                // insert-keyed by event.cid so only the entries whose entity
                // key equals event.cid can actually hold it under Phase-1
                // semantics, but iterating keeps the shape resilient if the
                // Phase-2 property-bearing path is added without a Deleted
                // fix-up.
                for set in self.by_entity.values_mut() {
                    set.remove(&event.cid);
                }
                // Drop empty sets so post-revocation reads return an empty
                // Vec via the `None` branch of `get`.
                self.by_entity.retain(|_, v| !v.is_empty());
            }
        }
        Ok(())
    }
}

impl Default for CapabilityGrantsView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for CapabilityGrantsView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.apply_event(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: VIEW_ID.to_string(),
            });
        }
        let cids = match &query.entity_cid {
            Some(entity) => self
                .by_entity
                .get(entity)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default(),
            None => Vec::new(),
        };
        Ok(ViewResult::Cids(cids))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        // Phase 1 rebuild clears state and resets the stale flag. A
        // full-history replay from the graph belongs to the subscriber
        // (G5-A) and the engine (G7); the view itself is the pure
        // maintainer, and `rebuild` is the reset hook.
        self.by_entity.clear();
        self.stale = false;
        Ok(())
    }

    fn id(&self) -> &str {
        VIEW_ID
    }

    fn is_stale(&self) -> bool {
        self.stale
    }

    fn mark_stale(&mut self) {
        self.stale = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use benten_core::{Node, Value};

    fn grant_node(n: i64) -> Node {
        let mut props = BTreeMap::new();
        props.insert("n".into(), Value::Int(n));
        Node::new(vec!["CapabilityGrant".into()], props)
    }

    #[test]
    fn fresh_view_has_fresh_state() {
        let v = CapabilityGrantsView::new();
        assert_eq!(v.state(), ViewState::Fresh);
        assert!(!v.is_stale());
    }

    #[test]
    fn budget_one_trips_on_second_update() {
        let mut v = CapabilityGrantsView::with_budget_for_testing(1);
        v.on_change(grant_node(1));
        assert_eq!(v.state(), ViewState::Fresh);
        v.on_change(grant_node(2));
        assert_eq!(v.state(), ViewState::Stale);
    }

    #[test]
    fn rebuild_clears_stale() {
        let mut v = CapabilityGrantsView::with_budget_for_testing(0);
        v.on_change(grant_node(1));
        assert_eq!(v.state(), ViewState::Stale);
        v.rebuild().unwrap();
        assert_eq!(v.state(), ViewState::Fresh);
    }
}
