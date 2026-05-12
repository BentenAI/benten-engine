//! R3 Family E RED-PHASE pin: dual-gate end-to-end LOAD-BEARING pim-2 §3.6b
//! pin — would-FAIL-if-no-op'd (LOAD-BEARING; mirrors `ivm_read_gate.rs::materialize_view_with_gate_filters_rows_per_actor_cap_set_at_engine_entry_point_e2e`).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 7.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3.Y.
//! - pim-2 §3.6b end-to-end pin discipline (would-FAIL-if-no-op'd).
//! - sec-3.5-r1-1 dual-gate composition pin 4 of 4.
//!
//! ## Why LOAD-BEARING
//!
//! End-to-end pim-2 substance — drives the production
//! `Materializer::materialize_with_gate` entry point with Nodes WRITTEN
//! through the engine's normal transaction surface + a `CapRecheckFn` that
//! admits some CIDs and denies others. Asserts row-level filtering: the
//! result contains EXACTLY the admitted CIDs — not all of them (would fail
//! if the gate were silently bypassed) and not none of them (would fail if
//! the arm returned `Ok(Some(Vec::new()))` unconditionally).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer dual-gate end-to-end entry point does not exist at HEAD; G23-B wave-5 \
    wires Materializer::materialize_with_gate with the pim-2 §3.6b would-FAIL-if-no-op'd \
    shape. Closes r2-test-landscape §2.5 row 7 + sec-3.5-r1-1 composition pin 4 of 4."]
fn materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op() {
    // G23-B implementer wires this:
    //
    //   use benten_core::Cid;
    //   use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId};
    //   use benten_engine::ivm_view_read_gate::IvmViewReadGate;
    //   use benten_engine::Engine;
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //   use std::collections::BTreeSet;
    //   use std::sync::Arc;
    //
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    //
    //   // Write 2 `post`-labeled Nodes through the engine's transaction surface.
    //   let (admitted_node, denied_node) = materializer_fixtures::dual_gate_fixture_pair();
    //   let admitted_cid = admitted_node.cid().unwrap();
    //   let denied_cid = denied_node.cid().unwrap();
    //   engine.transaction(|tx| {
    //       tx.put_node(&admitted_node).unwrap();
    //       tx.put_node(&denied_node).unwrap();
    //       Ok(())
    //   }).unwrap();
    //
    //   // Construct a gate that admits ONLY admitted_cid.
    //   let admitted_set: BTreeSet<Cid> = std::iter::once(admitted_cid).collect();
    //   let admitted_arc = Arc::new(admitted_set);
    //   let cap_recheck: CapRecheckFn = {
    //       let set = Arc::clone(&admitted_arc);
    //       Arc::new(move |_p, _zone, cid| set.contains(cid))
    //   };
    //   let alice = PrincipalId::from_actor_cid(materializer_fixtures::actor_principal_alice_cid());
    //   let gate = IvmViewReadGate::new(alice, "post", cap_recheck);
    //
    //   // Drive PRODUCTION entry point Materializer::materialize_with_gate.
    //   let mat = HtmlJsonMaterializer::default();
    //   let out = mat
    //       .materialize_with_gate(&engine, /*spec=*/ .., &gate)
    //       .expect("materialize_with_gate succeeds");
    //   let cids = out.materialized_row_cids();
    //
    //   // pim-2 §3.6b would-FAIL-if-no-op'd: EXACTLY one row admitted.
    //   assert_eq!(
    //       cids.len(), 1,
    //       "exactly one row admitted (not 2 = gate-bypass; not 0 = arm-no-op); \
    //        pim-2 §3.6b end-to-end behavior. cids = {cids:?}"
    //   );
    //   assert_eq!(cids[0], admitted_cid, "admitted CID == cap-permit");
    //   assert!(!cids.contains(&denied_cid), "denied CID suppressed at materialization");
    //
    //   // Smoke-check: allow-all gate sees BOTH rows (proves 1-row count is
    //   // gate-driven, not view-empty).
    //   let allow_gate = IvmViewReadGate::allow_all_for(alice, "post");
    //   let allowed = mat.materialize_with_gate(&engine, /*spec=*/ .., &allow_gate).unwrap();
    //   assert_eq!(
    //       allowed.materialized_row_cids().len(), 2,
    //       "allow-all sees BOTH rows; the 1-row outcome above is gate-driven"
    //   );
    let _ = materializer_fixtures::dual_gate_fixture_pair();
    unimplemented!(
        "G23-B wave-5 wires PRODUCTION Materializer::materialize_with_gate end-to-end with \
         pim-2 §3.6b would-FAIL-if-no-op'd substance: 1-not-2-not-0 row admission"
    );
}
