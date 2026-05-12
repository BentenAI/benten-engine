//! R3 Family E RED-PHASE pin: dual-gate per-row check independent of delivery
//! (LOAD-BEARING; mirrors `ivm_read_gate.rs::ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 4.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3.Y materializer
//!   dual-gate inheritance commitment (sec-3.5-r1-1).
//! - D-4F-NEW-MATERIALIZER-READ-GATE = SHARE `IvmViewReadGate` machinery
//!   (mat-r1-5 resolution: materializer view IS IVM view per D-4F-2).
//!
//! ## Composition shape
//!
//! Dual-gate layers:
//! - **Materialization-layer** (per-row READ at fanout) — fires at the
//!   materializer's READ-fanout boundary, using `read_node_as(walk_principal, cid)`.
//! - **Delivery-layer** (G14-D SUBSCRIBE delivery gate) — fires at the
//!   `on_change_as_with_cursor` boundary.
//!
//! This pin asserts the materialization-layer gate fires INDEPENDENTLY of
//! delivery — without invoking any SUBSCRIBE path. Mirror shape to the
//! `ivm_read_gate.rs` per-row-independent pin.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer per-row gate doesn't exist at HEAD; G23-B wave-5 wires shared \
    IvmViewReadGate consumption at the materializer's READ-fanout boundary. Closes \
    r2-test-landscape §2.5 row 4 + sec-3.5-r1-1 dual-gate composition pin 1 of 4."]
fn materializer_per_row_gate_independent_of_delivery() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{
    //       HtmlJsonMaterializer, Materializer,
    //   };
    //   use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId};
    //   use benten_engine::ivm_view_read_gate::IvmViewReadGate;
    //   use std::sync::Arc;
    //   use std::collections::BTreeSet;
    //
    //   // 10 rows: 5 public + 5 private. Per-row gate admits ONLY public.
    //   let mut public = Vec::new();
    //   let mut private = Vec::new();
    //   for i in 0..5 {
    //       public.push(materializer_fixtures::make_post_row_node("public", i).cid().unwrap());
    //       private.push(materializer_fixtures::make_post_row_node("private", i).cid().unwrap());
    //   }
    //   let admitted_set: BTreeSet<_> = public.iter().copied().collect();
    //   let admitted_arc = Arc::new(admitted_set);
    //   let cap_recheck: CapRecheckFn = {
    //       let set = Arc::clone(&admitted_arc);
    //       Arc::new(move |_p, _zone, cid| set.contains(cid))
    //   };
    //   let alice = PrincipalId::from_actor_cid(
    //       materializer_fixtures::actor_principal_alice_cid(),
    //   );
    //   let gate = IvmViewReadGate::new(alice, "post", cap_recheck);
    //
    //   // Drive materializer through the per-row gate WITHOUT a SUBSCRIBE path.
    //   let mat = HtmlJsonMaterializer::default();
    //   let admitted = mat.filter_rows_at_materialization(
    //       /* spec */ ..,
    //       public.iter().chain(private.iter()).copied(),
    //       &gate,
    //   );
    //   assert_eq!(
    //       admitted.len(), 5,
    //       "materializer per-row gate admits exactly 5 of 10 (matches ivm_read_gate.rs shape); \
    //        independence: no SUBSCRIBE channel involved"
    //   );
    //   for cid in &admitted {
    //       assert!(admitted_arc.contains(cid), "every admitted row in public set");
    //   }
    let _ = materializer_fixtures::actor_principal_alice_cid();
    unimplemented!(
        "G23-B wave-5 wires materializer per-row gate via shared IvmViewReadGate; \
         this pin asserts independence-from-delivery"
    );
}
