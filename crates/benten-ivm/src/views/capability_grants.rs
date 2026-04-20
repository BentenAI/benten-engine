//! View 1 — Capability grants indexed by entity (I3).
//!
//! Maintains a map `entity_cid → {grant_cids}` so capability lookups for a
//! given entity are O(1) after a one-time hash-map hit.
//!
//! ## Ingress paths
//!
//! The widened `ChangeEvent` (post-G5 fix-pass) carries the full Node body
//! for `ChangeKind::{Created, Updated, Deleted}` events. When the event's
//! `node` carries a `grantee` property (Cid-valued), the view keys the grant
//! under that CID — the proper `entity → {grants}` mapping. When the event
//! only carries identity (no node, or node without a `grantee` property),
//! the view falls back to keying under `event.cid` for back-compat with the
//! original identity-only test harness.
//!
//! Edge events (`ChangeKind::EdgeCreated` with label `GRANTED_TO`) are
//! treated as the canonical grant-to-entity wiring: the edge's source is the
//! grant Cid, the target is the entity Cid. Phase 1 accepts either the Node
//! path or the edge path; which one production code prefers depends on how
//! the capability Node shape is finalized in `benten-caps` (see G4).
//!
//! ## Budget
//!
//! `with_budget_for_testing(N)` caps the number of `on_change` applications
//! to `N` successful updates; the `(N+1)`th flips the view to
//! `ViewState::Stale` and subsequent reads return `ViewError::Stale`.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::vec::Vec;

use benten_core::{Cid, Value};
use benten_graph::{ChangeEvent, ChangeKind};

use crate::{BudgetTracker, View, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState};

/// Stable view identifier used in error messages and `View::id`.
const VIEW_ID: &str = "capability_grants";

/// Default budget — large enough that unit tests using `new()` never trip.
const UNLIMITED_BUDGET: u64 = u64::MAX;

/// View 1 — capability grants indexed by entity.
#[derive(Debug)]
pub struct CapabilityGrantsView {
    /// entity CID → set of grant CIDs.
    by_entity: BTreeMap<Cid, BTreeSet<Cid>>,
    /// Shared `remaining/original/stale` state — see `crate::budget`
    /// (r6-ref R-major-02). Every `on_change` / `apply_event` call
    /// charges one unit; exhaustion flips the view stale and `rebuild`
    /// restores the original cap (g5-cr-3).
    budget: BudgetTracker,
}

/// Back-compat alias. Some tests / docs write `CapGrants` for brevity.
pub type CapGrants = CapabilityGrantsView;

impl CapabilityGrantsView {
    /// Construct a fresh view with an effectively unlimited budget.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_entity: BTreeMap::new(),
            budget: BudgetTracker::new(UNLIMITED_BUDGET),
        }
    }

    /// Content-addressed definition for this view. Written to the graph as a
    /// `system:IVMView` Node.
    ///
    /// The input pattern label is `"system:CapabilityGrant"` (namespaced),
    /// matching the `CAPABILITY_GRANT_LABEL` constant in `benten-caps` and
    /// the label that `Engine::grant_capability` writes via the privileged
    /// path. A bare `"CapabilityGrant"` label would silently skip every
    /// real grant event (r6b-ivm-2).
    pub fn definition() -> ViewDefinition {
        ViewDefinition {
            view_id: VIEW_ID.to_string(),
            input_pattern_label: Some("system:CapabilityGrant".to_string()),
            output_label: "system:IVMView".to_string(),
        }
    }

    /// Test-only constructor that caps update budget at `budget` successful
    /// applications. The `(budget + 1)`th update trips the view to `Stale`.
    #[must_use]
    pub fn with_budget_for_testing(budget: u64) -> Self {
        Self {
            by_entity: BTreeMap::new(),
            budget: BudgetTracker::new(budget),
        }
    }

    /// Ingest a node-level change directly, bypassing the trait. Used by the
    /// budget-exceeded test harness which exercises the `Node` shape rather
    /// than the `ChangeEvent` shape.
    pub fn on_change(&mut self, node: benten_core::Node) {
        if self.budget.try_consume(1, VIEW_ID).is_err() {
            return;
        }

        // Prefer the node's `grantee` property (Cid-valued) as entity key.
        // Fall back to the node's own CID when absent (identity-only path).
        let entity_key = match extract_grantee(&node) {
            Some(gcid) => gcid,
            None => match node.cid() {
                Ok(c) => c,
                Err(_) => return,
            },
        };
        let grant_cid = match node.cid() {
            Ok(c) => c,
            Err(_) => return,
        };
        self.by_entity
            .entry(entity_key)
            .or_default()
            .insert(grant_cid);
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

    /// Direct read (bypasses the trait). Returns the grant-CID set for
    /// `entity`, in sorted order. Refuses with `ViewError::Stale` when the
    /// view is stale.
    pub fn read_for_entity(&self, entity: &Cid) -> Result<Vec<Cid>, ViewError> {
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
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
        self.budget.try_consume(1, VIEW_ID)?;

        // Edge path: GRANTED_TO edge wiring — source is grant, target is entity.
        if matches!(
            event.kind,
            ChangeKind::EdgeCreated | ChangeKind::EdgeDeleted
        ) {
            if !event.has_label("GRANTED_TO") {
                return Ok(());
            }
            if let Some((source, target, _)) = &event.edge_endpoints {
                match event.kind {
                    ChangeKind::EdgeCreated => {
                        self.by_entity
                            .entry(target.clone())
                            .or_default()
                            .insert(source.clone());
                    }
                    ChangeKind::EdgeDeleted => {
                        if let Some(set) = self.by_entity.get_mut(target) {
                            set.remove(source);
                            if set.is_empty() {
                                self.by_entity.remove(target);
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            return Ok(());
        }

        // Node path: CapabilityGrant-labeled node events.
        //
        // Match on the namespaced `"system:CapabilityGrant"` label — grants
        // are engine-internal state written via the privileged path and carry
        // the system-zone prefix. See `benten_caps::grant::CAPABILITY_GRANT_LABEL`.
        // An older draft matched the unqualified `"CapabilityGrant"` label,
        // which meant View 1 silently skipped every real grant event
        // (r6b-ivm-2).
        if !event.has_label("system:CapabilityGrant") {
            return Ok(());
        }
        match event.kind {
            ChangeKind::Created | ChangeKind::Updated => {
                // Prefer the node's `grantee` property as entity key. When
                // absent (identity-only legacy path), fall back to event.cid
                // so the original single-entity test harness still works.
                let entity_key = event
                    .node
                    .as_ref()
                    .and_then(extract_grantee)
                    .unwrap_or_else(|| event.cid.clone());
                self.by_entity
                    .entry(entity_key)
                    .or_default()
                    .insert(event.cid.clone());
            }
            ChangeKind::Deleted => {
                // Prefer the pre-delete node's `grantee` property; fall back
                // to iterating every entity's set to clean up the grant CID.
                if let Some(entity) = event.node.as_ref().and_then(extract_grantee) {
                    if let Some(set) = self.by_entity.get_mut(&entity) {
                        set.remove(&event.cid);
                        if set.is_empty() {
                            self.by_entity.remove(&entity);
                        }
                    }
                } else {
                    // Legacy identity-only path: remove event.cid from any
                    // set that carries it, then drop empty sets.
                    for set in self.by_entity.values_mut() {
                        set.remove(&event.cid);
                    }
                    self.by_entity.retain(|_, v| !v.is_empty());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Extract the `grantee` property from a Node as a `Cid`. Returns `None`
/// when the property is absent or not CID-shaped.
///
/// Phase 1 encodes CID-valued properties as `Value::Bytes` carrying the raw
/// CID byte representation (`Cid::as_bytes`). A Phase-2 string-CID parse
/// path (`Cid::from_str`, currently deferred — see benten-core) will accept
/// `Value::Text` too; until then the bytes form is canonical.
fn extract_grantee(node: &benten_core::Node) -> Option<Cid> {
    match node.properties.get("grantee") {
        Some(Value::Bytes(b)) => Cid::from_bytes(b).ok(),
        _ => None,
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
        if self.budget.is_stale() {
            return Err(BudgetTracker::stale_error(VIEW_ID));
        }
        // This view maintains a single index: `entity_cid → Set<Cid>`. A
        // query that names no `entity_cid` has no partition to serve; fail
        // fast with `E_IVM_PATTERN_MISMATCH` rather than silently returning
        // an empty `Cids` set (r6b §5.5 audit: the catalog documents this
        // code and the TS binding raises a typed error — returning empty
        // silently makes every mismatched query look like an empty result).
        let Some(entity) = query.entity_cid.as_ref() else {
            return Err(ViewError::PatternMismatch(
                "capability_grants: query missing required `entity_cid`".to_string(),
            ));
        };
        let cids = self
            .by_entity
            .get(entity)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        Ok(ViewResult::Cids(cids))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        // Phase 1 rebuild clears state, resets the stale flag, and restores
        // the originally-configured budget. A view constructed with a finite
        // budget that's tripped then rebuilt must accept up to the same
        // number of events again — see mini-review g5-cr-3.
        self.by_entity.clear();
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

    fn grant_node(n: i64) -> Node {
        let mut props = BTreeMap::new();
        props.insert("n".into(), Value::Int(n));
        // Namespaced label — matches the `CAPABILITY_GRANT_LABEL` constant
        // in `benten-caps` so this view's in-crate tests exercise the same
        // label surface production grants carry (r6b-ivm-2).
        Node::new(vec!["system:CapabilityGrant".into()], props)
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

    #[test]
    fn rebuild_restores_original_budget() {
        // g5-cr-3: rebuild must restore the budget to the original cap so a
        // view that's rebuilt can accept the same number of events again.
        let mut v = CapabilityGrantsView::with_budget_for_testing(1);
        v.on_change(grant_node(1)); // consumes the 1-unit budget
        v.on_change(grant_node(2)); // trips the view stale
        assert_eq!(v.state(), ViewState::Stale);
        v.rebuild().unwrap();
        assert_eq!(v.state(), ViewState::Fresh);
        // Post-rebuild: budget is back to 1, so one more update is accepted.
        v.on_change(grant_node(3));
        assert_eq!(v.state(), ViewState::Fresh);
        v.on_change(grant_node(4));
        assert_eq!(v.state(), ViewState::Stale);
    }
}
