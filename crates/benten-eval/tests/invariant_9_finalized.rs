//! 5d-J workstream 4 — Invariant 9 on the finalized Subgraph path.
//!
//! The builder snapshot path has always enforced Invariant 9 (a
//! handler declared deterministic rejects any non-deterministic
//! primitive). Before this workstream, the finalized `Subgraph`
//! struct dropped the flag during the builder→finalized projection
//! and `Subgraph::validate` silently skipped Invariant 9 — a gap
//! flagged in the invariants-module TODO.
//!
//! Now the flag is preserved through `build_unvalidated_for_test` /
//! `build_validated*`, and `Subgraph::set_deterministic` lets a
//! finalized Subgraph opt into the check explicitly.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{
    InvariantConfig, InvariantViolation, PrimitiveKind, SubgraphBuilder, invariants,
};

#[test]
fn invariant_9_enforced_on_finalized_subgraph_path() {
    // Build a deterministic-declared handler containing EMIT (a
    // non-deterministic primitive per `PrimitiveKind::is_deterministic`).
    // The builder path already rejects this; we reach the finalized
    // validator via `build_unvalidated_for_test` to prove the flag
    // survives the projection and the finalized-path check fires too.
    assert!(!PrimitiveKind::Emit.is_deterministic());

    let mut sb = SubgraphBuilder::new("det_handler");
    sb.declare_deterministic(true);
    let r = sb.read("r");
    sb.emit(r, "event_name");
    let sg = sb.build_unvalidated_for_test();

    assert!(
        sg.is_declared_deterministic(),
        "the deterministic flag must survive the builder→finalized projection (5d-J workstream 4)"
    );

    let err = invariants::validate_subgraph(&sg, &InvariantConfig::default(), false)
        .expect_err("Invariant 9 must fire on the finalized Subgraph path");
    assert!(matches!(err.kind(), InvariantViolation::Determinism));
}

#[test]
fn invariant_9_permits_deterministic_handler_on_finalized_path() {
    // A handler declared deterministic that only contains deterministic
    // primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL,
    // RESPOND) must pass both the builder and finalized-path checks.
    let mut sb = SubgraphBuilder::new("det_ok");
    sb.declare_deterministic(true);
    let r = sb.read("r");
    sb.respond(r);
    let sg = sb.build_unvalidated_for_test();

    assert!(sg.is_declared_deterministic());
    invariants::validate_subgraph(&sg, &InvariantConfig::default(), false)
        .expect("purely-deterministic handler must pass Invariant 9");
}

#[test]
fn subgraph_set_deterministic_retro_enables_invariant_9() {
    // A Subgraph constructed without the flag (legacy path) can opt
    // into Invariant 9 after the fact via Subgraph::set_deterministic.
    let mut sb = SubgraphBuilder::new("late_opt_in");
    let r = sb.read("r");
    sb.emit(r, "event");
    let mut sg = sb.build_unvalidated_for_test();
    assert!(
        !sg.is_declared_deterministic(),
        "builder did not declare deterministic — flag should be false"
    );

    // First validation passes because the flag is off.
    invariants::validate_subgraph(&sg, &InvariantConfig::default(), false)
        .expect("undeclared handler must pass (Invariant 9 is opt-in)");

    // Flip the flag on the finalized Subgraph; validation now fires.
    sg.set_deterministic(true);
    let err = invariants::validate_subgraph(&sg, &InvariantConfig::default(), false).unwrap_err();
    assert!(matches!(err.kind(), InvariantViolation::Determinism));
}
