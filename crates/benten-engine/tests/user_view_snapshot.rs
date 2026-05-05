//! R3-E RED-PHASE pins for G19-C1 UserView.snapshot() + onUpdate()
//! (wave-7 parallel; §7.1.3).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C1 must-pass column):
//!
//! - `tests/user_view_snapshot_returns_current_materialized_rows` — §7.1.3
//! - `tests/user_view_on_update_yields_incremental_deltas` — §7.1.3
//!
//! ## What G19-C1 establishes (§7.1.3)
//!
//! `crates/benten-engine/src/engine_views.rs` adds:
//! - `user_view_snapshot(view_id)` returning the current materialized
//!   row set
//! - `user_view_on_update(view_id, callback)` yielding incremental
//!   deltas as the view materializes
//!
//! Per cross-wave-file-touch note (seq-minor-1): post-G15-B
//! `engine_views.rs` already carries `PrefixMatcher` selector type
//! landed; G19-C1's user_view_snapshot/onUpdate is additive.
//!
//! ## RED-PHASE discipline
//!
//! Methods don't exist yet. R5 implementer wires them + drops `#[ignore]`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C1 wave-7 wires Engine::user_view_snapshot in engine_views.rs"]
fn user_view_snapshot_returns_current_materialized_rows() {
    // §7.1.3 pin. G19-C1 implementer wires this:
    //
    //   use benten_engine::Engine;
    //   let engine = Engine::open_in_memory().unwrap();
    //
    //   // Register a user-defined view + a handler that writes rows
    //   // matching the view label pattern:
    //   let view_id = engine.register_user_view(/* spec */).unwrap();
    //   let post_sg = engine.register_subgraph(crud("post")).unwrap();
    //   engine.call(post_sg, "post:create", json!({"title": "first"})).unwrap();
    //   engine.call(post_sg, "post:create", json!({"title": "second"})).unwrap();
    //
    //   // user_view_snapshot returns the CURRENT materialized rows:
    //   let rows = engine.user_view_snapshot(view_id).unwrap();
    //   assert_eq!(rows.len(), 2,
    //       "user_view_snapshot must return all materialized rows");
    //
    // OBSERVABLE consequence: callers receive a point-in-time snapshot
    // of the materialized view without subscribing. Defends against
    // the missing-API failure mode (callers having to roll their own
    // materialization walk via low-level APIs).
    unimplemented!("G19-C1 wires Engine::user_view_snapshot returning current rows");
}

#[test]
#[ignore = "RED-PHASE: G19-C1 wave-7 wires Engine::user_view_on_update with incremental deltas"]
fn user_view_on_update_yields_incremental_deltas() {
    // §7.1.3 pin. G19-C1 implementer wires this:
    //
    //   let engine = Engine::open_in_memory().unwrap();
    //   let view_id = engine.register_user_view(/* spec */).unwrap();
    //   let post_sg = engine.register_subgraph(crud("post")).unwrap();
    //
    //   let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    //   let received_clone = received.clone();
    //   engine.user_view_on_update(view_id, move |delta| {
    //       received_clone.lock().unwrap().push(delta.clone());
    //   }).unwrap();
    //
    //   // Drive a write that materializes a new row:
    //   engine.call(post_sg, "post:create", json!({"title": "new"})).unwrap();
    //
    //   // OBSERVABLE consequence: the callback fires with an incremental
    //   // delta (NOT a full re-snapshot):
    //   std::thread::sleep(std::time::Duration::from_millis(50)); // settle
    //   let collected = received.lock().unwrap();
    //   assert_eq!(collected.len(), 1, "onUpdate must fire once per write");
    //   assert!(collected[0].is_incremental_delta(),
    //       "onUpdate must yield incremental deltas, not full snapshots");
    //
    // Defends against the failure shape where onUpdate is wired but
    // returns full snapshots (would inflate per-write cost from O(delta)
    // to O(view-size)). Pim-2 §3.6b end-to-end test pin.
    unimplemented!("G19-C1 wires Engine::user_view_on_update yielding incremental deltas");
}
