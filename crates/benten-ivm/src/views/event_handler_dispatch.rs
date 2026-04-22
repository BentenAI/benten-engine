//! View 2 — Event handler dispatch table (I4).
//!
//! Maintains `event_name → {handler_cids}` so event dispatch is O(1) per
//! event name.
//!
//! ## Ingress paths
//!
//! With the widened `ChangeEvent`, node-shaped handler events carry the
//! originating handler's `subscribes_to` property — a `Value::List` of
//! `Value::Text` event names. The view partitions the dispatch table by
//! event name using that list when present. When the event does not carry
//! a node (identity-only legacy harness) the view falls back to a single
//! global set keyed under the empty-string event name for back-compat.
//!
//! Edge-shaped `SubscribesTo` events are ALSO accepted: the edge's source
//! is the handler CID and the edge carries an `event_name` — Phase 1
//! doesn't see properties on edges, so edge-event routing is bucketed into
//! the global set for now (the edge-endpoint widening still dominates over
//! the previous identity-only degenerate path).
//!
//! ## Budget
//!
//! See the analogous discussion on [`super::capability_grants`].

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use benten_core::{Cid, Value};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{BudgetTracker, View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

const VIEW_ID: &str = "event_dispatch";
const UNLIMITED_BUDGET: u64 = u64::MAX;

/// Event-name bucket used when the event doesn't carry an explicit
/// `subscribes_to` list. Keeps the back-compat single-set path for the
/// identity-only test harness on a clearly-labeled key.
const GLOBAL_BUCKET: &str = "";

/// Back-compat alias used by some R3 tests.
pub type EventHandlerDispatchView = EventDispatchView;

/// View 2 — event handler dispatch table.
#[derive(Debug)]
pub struct EventDispatchView {
    /// Per-event-name dispatch set. The `""` key is the global bucket for
    /// identity-only legacy events (see module doc).
    by_event: BTreeMap<String, BTreeSet<Cid>>,
    /// Shared budget tracker — see `crate::budget` (r6-ref R-major-02).
    budget: BudgetTracker,
}

impl EventDispatchView {
    /// Construct a fresh view with an effectively-unbounded budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_event: BTreeMap::new(),
            budget: BudgetTracker::new(UNLIMITED_BUDGET),
        }
    }

    /// Content-addressed definition for this view.
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.to_string(),
            input_pattern_label: Some("SubscribesTo".to_string()),
            output_label: "system:IVMView".to_string(),
        }
    }

    /// Low-budget test constructor.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            by_event: BTreeMap::new(),
            budget: BudgetTracker::new(budget),
        }
    }

    /// Ingest a node-level change directly. Counts against the budget.
    pub fn on_change(&mut self, node: benten_core::Node) {
        if self.budget.try_consume(1, VIEW_ID).is_err() {
            return;
        }
        let Ok(cid) = node.cid() else {
            return;
        };
        for bucket in extract_event_names(&node) {
            self.by_event.entry(bucket).or_default().insert(cid.clone());
        }
    }

    /// Current runtime state.
    #[must_use]
    pub fn state(&self) -> ViewState {
        if self.budget.is_stale() {
            ViewState::Stale
        } else {
            ViewState::Fresh
        }
    }

    /// Direct read (bypasses the trait). Returns handlers subscribed to
    /// `event`. When the event isn't partitioned (legacy harness), returns
    /// the global bucket.
    pub fn read_handlers_for_event(&self, event: &str) -> Result<Vec<Cid>, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        let mut out: BTreeSet<Cid> = BTreeSet::new();
        if let Some(set) = self.by_event.get(event) {
            out.extend(set.iter().cloned());
        }
        if let Some(set) = self.by_event.get(GLOBAL_BUCKET) {
            out.extend(set.iter().cloned());
        }
        Ok(out.into_iter().collect())
    }

    fn apply_event(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.budget.try_consume(1, VIEW_ID)?;

        // Edge path: SubscribesTo edges route into the global bucket for
        // Phase 1 — edge events don't carry a property payload.
        if matches!(
            event.kind,
            ChangeKind::EdgeCreated | ChangeKind::EdgeDeleted
        ) {
            if !event.has_label("SubscribesTo") {
                return Ok(());
            }
            if let Some((source, _target, _label)) = &event.edge_endpoints {
                match event.kind {
                    ChangeKind::EdgeCreated => {
                        self.by_event
                            .entry(GLOBAL_BUCKET.to_string())
                            .or_default()
                            .insert(source.clone());
                    }
                    ChangeKind::EdgeDeleted => {
                        if let Some(set) = self.by_event.get_mut(GLOBAL_BUCKET) {
                            set.remove(source);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            return Ok(());
        }

        if !event.has_label("SubscribesTo") {
            return Ok(());
        }
        // Determine bucket set: prefer the node's subscribes_to list; fall
        // back to the global bucket.
        let buckets: Vec<String> = event
            .node
            .as_ref()
            .map(extract_event_names)
            .unwrap_or_default();
        let buckets = if buckets.is_empty() {
            vec![GLOBAL_BUCKET.to_string()]
        } else {
            buckets
        };
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                for b in buckets {
                    self.by_event
                        .entry(b)
                        .or_default()
                        .insert(event.cid.clone());
                }
            }
            ChangeKind::Deleted => {
                // g5-p2-ivm-2: match cost to work done. The per-event base
                // charge (line ~149) covers one probe; additional charge
                // here is proportional to the number of bucket removals
                // performed, so a delete storm against many buckets
                // consumes budget at the same rate as the incremental
                // rebuild would. `retain` is O(n_buckets) on the map.
                let mut extra_cost: u64 = 0;
                for b in &buckets {
                    if let Some(set) = self.by_event.get_mut(b)
                        && set.remove(&event.cid)
                    {
                        extra_cost = extra_cost.saturating_add(1);
                    }
                }
                // Drop empty buckets.
                self.by_event.retain(|_, v| !v.is_empty());
                if extra_cost > 0 {
                    // Best-effort: charge additional cost. If this trips the
                    // tracker to stale, the caller path has already applied
                    // the delete — the next update will surface the stale.
                    let _ = self.budget.try_consume(extra_cost, VIEW_ID);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Extract the handler's `subscribes_to` property as a list of event names.
/// Returns empty when the property is absent or not a list-of-strings.
fn extract_event_names(node: &benten_core::Node) -> Vec<String> {
    match node.properties.get("subscribes_to") {
        Some(Value::List(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::Text(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
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
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        // This view maintains a single index: `event_name → Set<Cid>`. A
        // query that names no `event_name` has no partition to serve; fail
        // fast with `E_IVM_PATTERN_MISMATCH` rather than silently returning
        // an empty `Cids` set (r6b §5.5).
        let Some(name) = query.event_name.as_ref() else {
            return Err(ViewError::PatternMismatch(
                "event_dispatch: query missing required `event_name`".to_string(),
            ));
        };
        let mut out: BTreeSet<Cid> = BTreeSet::new();
        if let Some(set) = self.by_event.get(name) {
            out.extend(set.iter().cloned());
        }
        if let Some(set) = self.by_event.get(GLOBAL_BUCKET) {
            out.extend(set.iter().cloned());
        }
        let cids: Vec<Cid> = out.into_iter().collect();
        Ok(ViewResult::Cids(cids))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.by_event.clear();
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
