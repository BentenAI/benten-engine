//! View 2 — Event handler dispatch table (I4).
//!
//! Maintains `event_name → {handler_cids}` so event dispatch is O(1). Inputs:
//!
//! - `ChangeEvent` with label `"SubscribesTo"` and kind `Created` / `Updated`
//!   adds the handler CID to the subscriber set.
//! - `ChangeEvent` with label `"SubscribesTo"` and kind `Deleted` removes the
//!   handler CID.
//!
//! ## G5-B coordination note (ChangeEvent shape)
//!
//! The `ChangeEvent` shape delivered by G3 does not carry the
//! `subscribesTo: [...]` property list of the originating edge; it carries
//! only identity fields (cid, labels, kind, tx_id). Phase 1 stores the
//! handler CIDs in a single global dispatch set — any query with a non-empty
//! `event_name` resolves to that set. This degrades cleanly: the total set of
//! handlers is correct; the per-event-name partitioning is the Phase-2
//! refinement once `ChangeEvent` grows property attachment.
//!
//! ## Budget
//!
//! See the analogous discussion on [`super::capability_grants`].

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use benten_core::Cid;
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

const VIEW_ID: &str = "event_dispatch";
const UNLIMITED_BUDGET: u64 = u64::MAX;

/// Back-compat alias used by some R3 tests.
pub type EventHandlerDispatchView = EventDispatchView;

/// View 2 — event handler dispatch table.
#[derive(Debug)]
pub struct EventDispatchView {
    /// Current handlers. A Phase-1 global set (see module-level note); a
    /// query for any event name resolves to this set in sorted order.
    handlers: BTreeSet<Cid>,
    remaining_budget: u64,
    stale: bool,
}

impl EventDispatchView {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: BTreeSet::new(),
            remaining_budget: UNLIMITED_BUDGET,
            stale: false,
        }
    }

    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.to_string(),
            input_pattern_label: Some("SubscribesTo".to_string()),
            output_label: "system:IVMView".to_string(),
        }
    }

    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            handlers: BTreeSet::new(),
            remaining_budget: budget,
            stale: false,
        }
    }

    /// Ingest a node-level change directly. Counts against the budget.
    pub fn on_change(&mut self, node: benten_core::Node) {
        if self.stale {
            return;
        }
        if self.remaining_budget == 0 {
            self.stale = true;
            return;
        }
        self.remaining_budget -= 1;
        if let Ok(cid) = node.cid() {
            self.handlers.insert(cid);
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

    /// Direct read (bypasses the trait). `_event` is currently ignored (see
    /// module-level coordination note); the Phase-1 dispatch set is global.
    pub fn read_handlers_for_event(&self, _event: &str) -> Result<Vec<Cid>, ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: VIEW_ID.to_string(),
            });
        }
        Ok(self.handlers.iter().cloned().collect())
    }

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

        if !event.has_label("SubscribesTo") {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                self.handlers.insert(event.cid.clone());
            }
            ChangeKind::Deleted => {
                self.handlers.remove(&event.cid);
            }
        }
        Ok(())
    }
}

impl Default for EventDispatchView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for EventDispatchView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.apply_event(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.stale {
            return Err(ViewError::Stale {
                view_id: VIEW_ID.to_string(),
            });
        }
        // Phase 1: any event_name query resolves to the global handler set.
        let cids: Vec<Cid> = if query.event_name.is_some() {
            self.handlers.iter().cloned().collect()
        } else {
            Vec::new()
        };
        Ok(ViewResult::Cids(cids))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.handlers.clear();
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

    #[test]
    fn new_view_is_fresh_and_empty() {
        let v = EventDispatchView::new();
        assert_eq!(v.state(), ViewState::Fresh);
        assert!(v.read_handlers_for_event("any").unwrap().is_empty());
    }

    #[test]
    fn budget_zero_trips_immediately_on_change() {
        let mut v = EventDispatchView::with_budget_for_testing(0);
        v.on_change(benten_core::Node::empty());
        assert_eq!(v.state(), ViewState::Stale);
    }
}
