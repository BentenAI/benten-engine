//! R3 Family E RED-PHASE pin: dual-gate composition — delivery-layer deny
//! wins even when mat-layer would admit (LOAD-BEARING).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 6.
//! - §3.Y materializer dual-gate inheritance commitment (sec-3.5-r1-1).
//! - cap-r4-3 composition: deny-from-either-layer wins; symmetric to the
//!   mat-deny-wins pin.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    delivery-deny composition arm doesn't exist at HEAD; G23-B wave-5 wires symmetric \
    delivery-layer-deny-wins enforcement. Closes r2-test-landscape §2.5 row 6 + \
    sec-3.5-r1-1 dual-gate composition pin 3 of 4."]
fn materializer_delivery_deny_wins_composition() {
    // G23-B implementer wires this:
    //
    //   // Setup symmetric to mat-deny-wins: row admitted by per-row
    //   // materialization gate but DENIED by delivery layer. Composed
    //   // outcome MUST be deny (delivery-layer wins per cap-r4-3).
    //
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //   use benten_engine::cap_recheck::CapRecheckFn;
    //   use benten_engine::ivm_view_read_gate::IvmViewReadGate;
    //   use std::sync::Arc;
    //
    //   let (admitted_node, _other) = materializer_fixtures::dual_gate_fixture_pair();
    //   let admitted_cid = admitted_node.cid().unwrap();
    //
    //   // Materialization-layer: allow-all.
    //   let mat_gate = IvmViewReadGate::allow_all_for(alice, "post");
    //
    //   // Delivery-layer: deny admitted_cid.
    //   let delivery_recheck: CapRecheckFn = Arc::new({
    //       let denied = admitted_cid;
    //       move |_p, _zone, cid| *cid != denied
    //   });
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let out = mat
    //       .materialize_with_gate_and_delivery(/* spec */ .., &mat_gate, delivery_recheck)
    //       .unwrap();
    //   let cids = out.delivered_row_cids();
    //
    //   // delivery-deny wins: admitted_cid is suppressed at delivery boundary.
    //   assert!(
    //       !cids.contains(&admitted_cid),
    //       "delivery-deny wins composition: row admitted at materialization but denied \
    //        at delivery MUST NOT reach the consumer"
    //   );
    let _ = materializer_fixtures::dual_gate_fixture_pair();
    unimplemented!("G23-B wave-5 wires delivery-deny composition arm; symmetric to mat-deny-wins");
}
