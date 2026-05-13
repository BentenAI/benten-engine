//! G23-B GREEN: dual-gate composition — mat-layer deny wins even when
//! delivery-layer would admit.
//!
//! **Scope (per G23-B mr-3 amendment):** this pin exercises the
//! `Materializer::dual_gate_admits(..)` HELPER-FUNCTION contract over
//! two closures — NOT the production walk path
//! `Materializer::materialize_with_gate(..)`, which threads only the
//! mat-layer gate. The production composition (mat × delivery at
//! WRITE + SUBSCRIBE seams) is asserted by
//! `materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op.rs`
//! (the LOAD-BEARING end-to-end pin) + lands fully at G24-A admin-UI
//! integration when `Engine::on_change_as_with_cursor` is wired.
//!
//! Closes r2-test-landscape §2.5 row 5 + sec-3.5-r1-1 dual-gate
//! composition pin 2 of 4 (helper-function level).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_platform_foundation::{
    HtmlJsonMaterializer, Materializer, MaterializerCapRecheck, allow_all_cap_recheck,
};
use std::sync::Arc;

#[test]
fn materializer_mat_deny_wins_composition() {
    let (admitted_node, denied_node) = materializer_fixtures::dual_gate_fixture_pair();
    let admitted_cid = admitted_node.cid().unwrap();
    let denied_cid = denied_node.cid().unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Materialization-layer: deny `denied_cid`; permit `admitted_cid`.
    let denied_cid_for_closure = denied_cid;
    let mat_gate: MaterializerCapRecheck =
        Arc::new(move |_p, _zone, cid| *cid != denied_cid_for_closure);

    // Delivery-layer: allow-all.
    let delivery_gate: MaterializerCapRecheck = allow_all_cap_recheck();

    let mat = HtmlJsonMaterializer;
    let admitted_observable =
        mat.dual_gate_admits(&admitted_cid, &alice, "post", &mat_gate, &delivery_gate);
    let denied_observable =
        mat.dual_gate_admits(&denied_cid, &alice, "post", &mat_gate, &delivery_gate);

    // mat-deny wins: denied_cid is suppressed despite delivery admitting.
    assert!(
        admitted_observable,
        "non-denied row admitted at both layers"
    );
    assert!(
        !denied_observable,
        "mat-deny wins composition: row denied at mat-layer MUST be suppressed \
         regardless of delivery-layer allow-all"
    );
}
