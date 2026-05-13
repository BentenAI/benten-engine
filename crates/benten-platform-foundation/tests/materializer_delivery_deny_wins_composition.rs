//! G23-B GREEN: dual-gate composition — delivery-layer deny wins even
//! when mat-layer would admit.
//!
//! **Scope (per G23-B mr-3 amendment):** this pin exercises the
//! `Materializer::dual_gate_admits(..)` HELPER-FUNCTION contract over
//! two closures — NOT the production walk path
//! `Materializer::materialize_with_gate(..)`, which threads only the
//! mat-layer gate. The production composition lands at G24-A admin-UI
//! integration when `Engine::on_change_as_with_cursor` is wired; the
//! end-to-end LOAD-BEARING pin is
//! `materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op.rs`.
//!
//! Closes r2-test-landscape §2.5 row 6 + sec-3.5-r1-1 dual-gate
//! composition pin 3 of 4 (helper-function level).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_platform_foundation::{
    HtmlJsonMaterializer, Materializer, MaterializerCapRecheck, allow_all_cap_recheck,
};
use std::sync::Arc;

#[test]
fn materializer_delivery_deny_wins_composition() {
    let (admitted_node, _other) = materializer_fixtures::dual_gate_fixture_pair();
    let admitted_cid = admitted_node.cid().unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Materialization-layer: allow-all.
    let mat_gate: MaterializerCapRecheck = allow_all_cap_recheck();

    // Delivery-layer: deny admitted_cid.
    let delivery_gate: MaterializerCapRecheck = {
        let denied = admitted_cid;
        Arc::new(move |_p, _zone, cid| *cid != denied)
    };

    let mat = HtmlJsonMaterializer;
    let observed = mat.dual_gate_admits(&admitted_cid, &alice, "post", &mat_gate, &delivery_gate);

    // delivery-deny wins: admitted_cid is suppressed at delivery boundary
    // despite mat-layer admitting.
    assert!(
        !observed,
        "delivery-deny wins composition: row admitted at materialization but denied \
         at delivery MUST NOT reach the consumer"
    );
}
