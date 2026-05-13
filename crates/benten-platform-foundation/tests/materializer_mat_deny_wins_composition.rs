//! R3 Family E RED-PHASE pin: dual-gate composition — mat-layer deny wins
//! even when delivery-layer would admit (LOAD-BEARING).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 5.
//! - §3.Y materializer dual-gate inheritance commitment (sec-3.5-r1-1).
//! - cap-r4-3 composition: deny-from-either-layer wins.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    mat-deny composition arm doesn't exist at HEAD; G23-B wave-5 wires dual-gate \
    composition with deny-from-either-layer-wins semantics. Closes r2-test-landscape \
    §2.5 row 5 + sec-3.5-r1-1 dual-gate composition pin 2 of 4."]
fn materializer_mat_deny_wins_composition() {
    // G23-B implementer wires this:
    //
    //   // Setup: a row is admitted by an allow-all delivery layer but
    //   // DENIED by the per-row materialization gate. Composed outcome
    //   // MUST be deny (mat-layer wins per cap-r4-3).
    //
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //   use benten_engine::cap_recheck::CapRecheckFn;
    //   use benten_engine::ivm_view_read_gate::IvmViewReadGate;
    //   use std::sync::Arc;
    //
    //   let (admitted_node, denied_node) = materializer_fixtures::dual_gate_fixture_pair();
    //   // ... write both through engine ...
    //
    //   // Materialization-layer: deny `denied_cid`.
    //   let denied_cid = denied_node.cid().unwrap();
    //   let mat_recheck: CapRecheckFn = {
    //       Arc::new(move |_p, _zone, cid| *cid != denied_cid)
    //   };
    //   let mat_gate = IvmViewReadGate::new(alice, "post", mat_recheck);
    //
    //   // Delivery-layer: allow-all (admit everything).
    //   let delivery_recheck: CapRecheckFn = Arc::new(|_p, _zone, _cid| true);
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let out = mat
    //       .materialize_with_gate_and_delivery(/* spec */ .., &mat_gate, delivery_recheck)
    //       .unwrap();
    //   let cids = out.materialized_row_cids();
    //
    //   // mat-deny wins: denied_cid is suppressed despite delivery admitting.
    //   assert!(
    //       !cids.contains(&denied_cid),
    //       "mat-deny wins composition: row denied at materialization MUST be suppressed \
    //        regardless of delivery-layer allow-all"
    //   );
    //   assert_eq!(
    //       cids.len(), 1,
    //       "exactly 1 row admitted (the non-denied row); not 2 = mat-deny bypassed; \
    //        not 0 = both-layers-deny"
    //   );
    let _ = materializer_fixtures::dual_gate_fixture_pair();
    unimplemented!(
        "G23-B wave-5 wires mat-deny composition arm; deny-from-mat-layer wins per cap-r4-3"
    );
}
