//! View 4 — Governance inheritance transitive closure (I6).
//!
//! Maintains `entity_cid → [rule_cids]` computed by walking `GovernedBy` edges
//! up the governance tree. Per ENGINE-SPEC §8, traversal is capped at
//! [`MAX_GOVERNANCE_DEPTH`] hops (5 for Phase 1): deeper chains truncate and
//! set the truncation flag so callers can detect the cap; cycles are detected
//! via a visited-set and reported through a dedicated `cycle_detected` flag
//! so the two truncation reasons remain distinguishable (R4 triage m5).
//!
//! ## Ingress paths
//!
//! After the G5 fix-pass, `ChangeEvent::edge_endpoints` carries
//! `(source, target, label)` for `EdgeCreated`/`EdgeDeleted` kinds, so the
//! trait `update` path wires `GovernedBy` edges directly into the parent
//! adjacency map. `add_edge(child, parent)` remains a direct-ingress
//! convenience used by unit tests and by engine-internal batches.
//!
//! A Node-event delete on a `GovernedBy`-labeled Node (rare — governance
//! is usually edge-shaped) invalidates any adjacency whose value equals the
//! deleted Cid as a best-effort cleanup (mini-review g5-cr-7).
//!
//! ## Budget
//!
//! See the analogous discussion on [`super::capability_grants`].

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use benten_core::Cid;
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{BudgetTracker, View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Depth cap for governance inheritance traversal per ENGINE-SPEC §8.
pub const MAX_GOVERNANCE_DEPTH: usize = 5;

const VIEW_ID: &str = "governance_inheritance";
const UNLIMITED_BUDGET: u64 = u64::MAX;

/// View 4 — governance inheritance transitive closure.
#[derive(Debug)]
pub struct GovernanceInheritanceView {
    /// Adjacency map: child → parent (single parent per child, matching the
    /// Phase-1 `GovernedBy` cardinality; Phase 2 extends to multi-parent).
    parent: BTreeMap<Cid, Cid>,
    /// Shared budget tracker — see `crate::budget` (r6-ref R-major-02).
    budget: BudgetTracker,
}

impl GovernanceInheritanceView {
    /// Construct a fresh view with an effectively-unbounded budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent: BTreeMap::new(),
            budget: BudgetTracker::new(UNLIMITED_BUDGET),
        }
    }

    /// Content-addressed definition for this view.
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.to_string(),
            input_pattern_label: Some("GovernedBy".to_string()),
            output_label: "system:IVMView".to_string(),
        }
    }

    /// Low-budget test constructor.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            parent: BTreeMap::new(),
            budget: BudgetTracker::new(budget),
        }
    }

    /// Ingest a node-level change. Consumes budget, flips to Stale when
    /// exhausted.
    pub fn on_change(&mut self, _node: benten_core::Node) {
        let _ = self.budget.try_consume(1, VIEW_ID);
        // Phase 1: `on_change` does not materialize edges (the `ChangeEvent`
        // / `Node` shape does not carry `GovernedBy` endpoint information).
        // Callers use `add_edge` to seed the adjacency.
    }

    /// Add a `GovernedBy` edge (child → parent). Test / engine surface for
    /// direct adjacency updates; the `ChangeEvent`-driven path defers this
    /// to Phase 2 when edge-endpoint events land on the change stream.
    pub fn add_edge(&mut self, child: &Cid, parent: &Cid) {
        if self.budget.is_stale() {
            return;
        }
        self.parent.insert(child.clone(), parent.clone());
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

    /// Compute effective rules for `entity` by walking the `GovernedBy`
    /// chain upward. Capped at [`MAX_GOVERNANCE_DEPTH`] hops. Sets
    /// `was_truncated` when the cap stops traversal, and `cycle_detected`
    /// when a visited node is revisited.
    ///
    /// Infallible: this method always returns an `EffectiveRules`. The
    /// fallible variant [`Self::read_effective_rules`] is the budget-aware
    /// wrapper used by the test harness that needs `ViewError::Stale`
    /// surfacing.
    #[must_use]
    pub fn effective_rules(&self, entity: &Cid) -> EffectiveRules {
        let mut rules: Vec<Cid> = Vec::new();
        let mut visited: BTreeSet<Cid> = BTreeSet::new();
        let mut cursor = entity.clone();
        visited.insert(cursor.clone());

        let mut depth = 0usize;
        let mut was_truncated = false;
        let mut cycle_detected = false;

        while let Some(parent) = self.parent.get(&cursor) {
            if depth >= MAX_GOVERNANCE_DEPTH {
                was_truncated = true;
                break;
            }
            depth += 1;
            if !visited.insert(parent.clone()) {
                // Revisiting a node we've already seen on this walk → cycle.
                cycle_detected = true;
                was_truncated = true;
                break;
            }
            rules.push(parent.clone());
            cursor = parent.clone();
        }

        EffectiveRules {
            depth,
            was_truncated,
            cycle_detected,
            rules,
        }
    }

    /// Fallible variant that surfaces `ViewError::Stale` when the view has
    /// tripped its budget. Shape used by the budget-exceeded test harness.
    pub fn read_effective_rules(&self, entity: &Cid) -> Result<EffectiveRules, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        Ok(self.effective_rules(entity))
    }

    fn apply_event(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.budget.try_consume(1, VIEW_ID)?;

        match event.kind {
            ChangeKind::EdgeCreated if event.has_label("GovernedBy") => {
                if let Some((source, target, _)) = &event.edge_endpoints {
                    // `GovernedBy` edge: source is the child (governed),
                    // target is the parent (governor).
                    self.parent.insert(source.clone(), target.clone());
                }
            }
            ChangeKind::EdgeDeleted if event.has_label("GovernedBy") => {
                if let Some((source, _target, _)) = &event.edge_endpoints {
                    self.parent.remove(source);
                }
            }
            ChangeKind::Deleted => {
                // A node-delete of a governance participant invalidates any
                // adjacency that pointed AT it; best-effort cleanup per
                // mini-review g5-cr-7.
                self.parent.retain(|_, v| v != &event.cid);
                self.parent.remove(&event.cid);
            }
            _ => {
                // Non-governance events are acknowledged but not acted on.
            }
        }
        Ok(())
    }
}

/// Result of a transitive-closure governance resolution.
///
/// Carries the resolved-rules chain, the depth reached during traversal, and
/// two distinguishable truncation flags: `was_truncated` fires for either
/// depth-cap or cycle-induced stops; `cycle_detected` fires only for
/// cycle-induced stops. R4 triage (m5) pinned the separation so a regression
/// that silently conflates the two reasons fails the cycle test.
#[derive(Debug, Clone)]
pub struct EffectiveRules {
    depth: usize,
    was_truncated: bool,
    cycle_detected: bool,
    rules: Vec<Cid>,
}

impl EffectiveRules {
    /// Depth of the ancestor walk that produced this rule set (0 == direct).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// `true` if the walk hit the configured max-depth cap before exhausting
    /// the ancestor chain.
    #[must_use]
    pub fn was_truncated(&self) -> bool {
        self.was_truncated
    }

    /// `true` if the walk detected a cycle in the GOVERNED_BY chain and
    /// short-circuited defensively.
    #[must_use]
    pub fn cycle_detected(&self) -> bool {
        self.cycle_detected
    }

    /// Effective rule CIDs in walk order (nearest ancestor first).
    #[must_use]
    pub fn rules(&self) -> &[Cid] {
        &self.rules
    }
}

impl Default for GovernanceInheritanceView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for GovernanceInheritanceView {
    fn update(&mut self, event: &ChangeEvent) -> Result<(), ViewError> {
        self.apply_event(event)
    }

    fn read(&self, query: &ViewQuery) -> Result<ViewResult, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        let rules_map: BTreeMap<String, benten_core::Value> = match &query.entity_cid {
            Some(entity) => {
                let resolved = self.effective_rules(entity);
                let mut m = BTreeMap::new();
                m.insert(
                    "depth".into(),
                    benten_core::Value::Int(resolved.depth as i64),
                );
                m.insert(
                    "rule_count".into(),
                    benten_core::Value::Int(resolved.rules.len() as i64),
                );
                m
            }
            None => BTreeMap::new(),
        };
        Ok(ViewResult::Rules(rules_map))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.parent.clear();
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
    use alloc::vec;
    use benten_core::{Node, Value};

    fn test_cid(n: i64) -> Cid {
        let mut props = BTreeMap::new();
        props.insert("n".into(), Value::Int(n));
        Node::new(vec!["Community".into()], props).cid().unwrap()
    }

    #[test]
    fn chain_of_length_3_resolves_to_depth_2() {
        let mut view = GovernanceInheritanceView::new();
        let c0 = test_cid(0);
        let c1 = test_cid(1);
        let c2 = test_cid(2);
        view.add_edge(&c0, &c1);
        view.add_edge(&c1, &c2);

        let resolved = view.effective_rules(&c0);
        assert_eq!(resolved.depth(), 2);
        assert_eq!(resolved.rules().len(), 2);
        assert!(!resolved.was_truncated());
        assert!(!resolved.cycle_detected());
    }

    #[test]
    fn orphan_has_zero_depth_and_empty_rules() {
        let view = GovernanceInheritanceView::new();
        let orphan = test_cid(99);
        let resolved = view.effective_rules(&orphan);
        assert_eq!(resolved.depth(), 0);
        assert!(resolved.rules().is_empty());
    }
}
