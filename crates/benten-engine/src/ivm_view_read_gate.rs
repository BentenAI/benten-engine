//! Materialization-time per-row READ gate for IVM-materialized views
//! (Phase-3 G15-A; closes Compromise #11 in coordination with G14-D
//! delivery-time gate).
//!
//! ## Why a separate gate from G14-D
//!
//! G14-D's per-event read-cap recheck at SUBSCRIBE delivery boundary
//! (`engine_subscribe.rs`) gates which `ChangeEvent`s flow to a
//! subscriber. It runs at delivery time — i.e. once an event has reached
//! the subscriber's fan-out path — and consults [`crate::cap_recheck::CapRecheckFn`]
//! against the live grant store so a partial revoke cancels in-flight
//! events.
//!
//! G15-A's gate runs at MATERIALIZATION time per `ivm-major-2`: when the
//! engine reads a view (`Engine::read_view*`), each row in the
//! materialised list is re-checked against the actor's cap-set BEFORE
//! the row is yielded. This prevents an actor from materializing a view
//! whose backing rows they cannot READ — even if the view itself is
//! cap-grantable at the per-zone level (the Phase-2a coarse gate per
//! Compromise #11 §"Shape").
//!
//! Both layers run independently:
//! - **deny-from-either-layer wins** (per `cap-r4-3` composition): a row
//!   suppressed at materialization never reaches delivery; a row admitted
//!   at materialization is still subject to delivery-time recheck.
//! - **Materialization gate is structural** — it can fire on a row Alice
//!   cannot READ even before the SUBSCRIBE event flows.
//! - **Delivery gate is live** — it can fire after a partial-revoke even
//!   on rows that previously passed the materialization gate.
//!
//! ## Composition shape (per `ds-r4r2-7`)
//!
//! [`IvmViewReadGate`] composes:
//! 1. **Label-hint extraction** — derives the row's label (and optionally
//!    the row's CID) for the cap-recheck closure. Phase-3 G15-A keeps the
//!    hint extraction simple: the gate is constructed with an explicit
//!    label hint per view, sourced from the registered view's
//!    `input_pattern_label` (NOT from the view-id-prefix heuristic that
//!    Compromise #11's R6-Round-3 disclosure called out as bounded).
//! 2. **Actor-cap-set check** — invokes [`crate::cap_recheck::CapRecheckFn`]
//!    for each row, threading `(principal, label_hint, row_cid)`. The
//!    closure consults whatever cap store the engine was constructed
//!    with (defaults to `allow_all` matching `NoAuthBackend`; production
//!    deployments construct a closure closing over their durable grant
//!    store at G14-B).
//!
//! The shared scaffold at [`crate::cap_recheck`] is the load-bearing
//! single-signature surface BOTH G14-D AND G15-A consume per
//! `ds-r4r2-7`. Any signature change to [`crate::cap_recheck::CapRecheckFn`]
//! lands HERE first (the cap_recheck module rustdoc names this contract
//! explicitly).

use benten_core::Cid;

use crate::cap_recheck::{CapRecheckFn, PrincipalId, allow_all};

/// Per-row READ gate fired at IVM-view materialization time.
///
/// Construct one [`IvmViewReadGate`] per (actor, view-read) pair (or
/// reuse across rows if the actor + view label-hint are stable). The
/// gate consults the supplied [`CapRecheckFn`] for each row before the
/// row is yielded from the materialised list.
///
/// **Compromise #11 closure surface.** Phase-3 G15-A composes this gate
/// with the view's `input_pattern_label` to drive per-row READ checks
/// against the actor's cap-set. The Phase-2b coarse-grained gate (per-view
/// only) is supplemented — NOT replaced — by the per-row gate; the
/// per-view gate at `read_view_with` is upstream of materialization and
/// remains a fast-fail boundary.
///
/// **Composition with G14-D delivery gate.** The same [`CapRecheckFn`]
/// shape is consumed by `engine_subscribe.rs::Subscription` per
/// `ds-r4r2-7`; both layers must permit a row before the actor observes
/// it. Tested end-to-end by
/// `crates/benten-engine/tests/ivm_view_subscribe_compose.rs`.
#[derive(Clone)]
pub struct IvmViewReadGate {
    principal: PrincipalId,
    label_hint: String,
    cap_recheck: CapRecheckFn,
}

impl core::fmt::Debug for IvmViewReadGate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IvmViewReadGate")
            .field("principal", &self.principal)
            .field("label_hint", &self.label_hint)
            .finish_non_exhaustive()
    }
}

impl IvmViewReadGate {
    /// Construct a per-row READ gate for `principal` + view `label_hint`.
    ///
    /// `cap_recheck` MUST be constructed by the engine to close over the
    /// active grant store (or [`allow_all`] for the default
    /// `NoAuthBackend` posture). The gate stores it as a clonable `Arc`
    /// so multiple per-row checks share a single closure allocation.
    #[must_use]
    pub fn new(
        principal: PrincipalId,
        label_hint: impl Into<String>,
        cap_recheck: CapRecheckFn,
    ) -> Self {
        Self {
            principal,
            label_hint: label_hint.into(),
            cap_recheck,
        }
    }

    /// Convenience constructor for the `NoAuthBackend` default — gate
    /// permits every row. Used by tests that explicitly opt out of cap
    /// enforcement and by the engine when no policy is configured.
    #[must_use]
    pub fn allow_all_for(principal: PrincipalId, label_hint: impl Into<String>) -> Self {
        Self::new(principal, label_hint, allow_all())
    }

    /// Permit or deny `row_cid` based on the actor-cap-set check.
    ///
    /// Returns `true` to admit the row into the materialised list,
    /// `false` to suppress it. The gate is consulted per-row inside
    /// [`Self::filter_rows`]; callers wiring the gate into a custom
    /// materialisation path may invoke this directly.
    #[must_use]
    pub fn admits(&self, row_cid: &Cid) -> bool {
        (self.cap_recheck)(&self.principal, self.label_hint.as_str(), row_cid)
    }

    /// Filter `rows` per the per-row READ gate, returning only the CIDs
    /// the actor can READ. Order is preserved.
    ///
    /// This is the load-bearing materialization-time gate: a row whose
    /// underlying Node the actor cannot READ does NOT appear in the
    /// returned list. Per `ivm-major-2` this fires SEPARATELY from
    /// G14-D's delivery-time gate at SUBSCRIBE.
    #[must_use]
    pub fn filter_rows<I>(&self, rows: I) -> Vec<Cid>
    where
        I: IntoIterator<Item = Cid>,
    {
        rows.into_iter().filter(|cid| self.admits(cid)).collect()
    }

    /// Borrow the actor principal this gate filters for.
    #[must_use]
    pub fn principal(&self) -> &PrincipalId {
        &self.principal
    }

    /// Borrow the label-hint this gate composes against.
    #[must_use]
    pub fn label_hint(&self) -> &str {
        self.label_hint.as_str()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use crate::cap_recheck::deny_all;
    use benten_core::testing::canonical_test_node;
    use std::sync::Arc;

    fn principal_for(label: &str) -> PrincipalId {
        let mut node = canonical_test_node();
        node.labels = vec![label.to_string()];
        PrincipalId::from_actor_cid(node.cid().unwrap())
    }

    fn cid_for_label(label: &str, idx: u64) -> Cid {
        let mut props = std::collections::BTreeMap::new();
        props.insert(String::from("seq"), benten_core::Value::Int(idx as i64));
        let node = benten_core::Node::new(vec![label.to_string()], props);
        node.cid().unwrap()
    }

    #[test]
    fn allow_all_gate_admits_every_row() {
        let gate = IvmViewReadGate::allow_all_for(principal_for("alice"), "post");
        let rows = vec![
            cid_for_label("post", 1),
            cid_for_label("post", 2),
            cid_for_label("post", 3),
        ];
        let admitted = gate.filter_rows(rows.clone());
        assert_eq!(admitted, rows, "allow_all admits every row");
    }

    #[test]
    fn deny_all_gate_admits_no_rows() {
        let gate = IvmViewReadGate::new(principal_for("alice"), "post", deny_all());
        let rows = vec![cid_for_label("post", 1), cid_for_label("post", 2)];
        let admitted = gate.filter_rows(rows);
        assert!(admitted.is_empty(), "deny_all admits no rows");
    }

    #[test]
    fn per_row_gate_fires_against_actor_cap_set_compromise_11_closure() {
        // Compromise #11 closure scenario: 50 public + 50 private rows;
        // an actor with READ caps only on the public partition sees
        // EXACTLY 50 rows in the materialised view (NOT 0, NOT 100). The
        // pre-G15-A coarse gate would have answered 0 (deny entire view)
        // or 100 (admit entire view) — never 50.
        let public_rows: Vec<Cid> = (0..50).map(|i| cid_for_label("post:public", i)).collect();
        let private_rows: Vec<Cid> = (0..50).map(|i| cid_for_label("post:private", i)).collect();
        let public_set: std::collections::BTreeSet<Cid> = public_rows.iter().copied().collect();

        // Per-row check: admit iff cid is in the public set.
        let public_set_arc = Arc::new(public_set);
        let cap_recheck: CapRecheckFn = {
            let set = Arc::clone(&public_set_arc);
            Arc::new(move |_p: &PrincipalId, _zone: &str, cid: &Cid| set.contains(cid))
        };

        let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);
        let mut all_rows = Vec::with_capacity(100);
        all_rows.extend(public_rows.iter().copied());
        all_rows.extend(private_rows.iter().copied());

        let admitted = gate.filter_rows(all_rows);
        assert_eq!(
            admitted.len(),
            50,
            "per-row gate yields 50 (not 0, not 100) per Compromise #11 closure"
        );
        for cid in &admitted {
            assert!(
                public_set_arc.contains(cid),
                "every admitted row is in the public set"
            );
        }
    }

    #[test]
    fn admits_consults_cap_recheck_with_label_hint_and_principal() {
        let observed: Arc<std::sync::Mutex<Vec<(Cid, String)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed_clone = Arc::clone(&observed);
        let cap_recheck: CapRecheckFn = Arc::new(move |_p: &PrincipalId, zone: &str, cid: &Cid| {
            observed_clone
                .lock()
                .unwrap()
                .push((*cid, zone.to_string()));
            true
        });

        let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);
        let rows = vec![cid_for_label("post", 1), cid_for_label("post", 2)];
        let _ = gate.filter_rows(rows.clone());

        let log = observed.lock().unwrap();
        assert_eq!(log.len(), 2, "closure called once per row");
        for (_, zone) in log.iter() {
            assert_eq!(zone, "post", "label_hint threaded into cap-recheck");
        }
    }
}
