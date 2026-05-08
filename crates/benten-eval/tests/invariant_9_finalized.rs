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

/// Phase-3 G21-T1 sec-major-2: Inv-9 MUST also fire when a CALL Node's
/// `target` names a typed-CALL op whose own determinism classification
/// is non-deterministic — e.g. `engine:typed:keypair_generate` (OS
/// CSPRNG). Bare `PrimitiveKind::Call.is_deterministic()` returns
/// `true`, so without this gate, OS CSPRNG would leak through into
/// handlers declared `deterministic: true`.
#[test]
fn invariant_9_fires_for_typed_call_keypair_generate_in_deterministic_handler() {
    use benten_core::{OperationNode, Subgraph, Value};
    use benten_eval::TypedCallOp;

    // Sanity pin: the typed-CALL op classifies non-deterministic.
    assert!(
        !TypedCallOp::KeypairGenerate.is_deterministic(),
        "keypair_generate MUST classify non-deterministic (OS CSPRNG)"
    );

    // Build a deterministic-declared Subgraph directly with a CALL
    // Node whose `target` names `engine:typed:keypair_generate`. The
    // typed-CALL fork keys off the `target` property; the SubgraphBuilder
    // helpers don't expose a typed-CALL DX yet, so we construct the
    // raw shape.
    let read_node = OperationNode::new("r", PrimitiveKind::Read);
    let typed_call_node = OperationNode::new("typed_call", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed:keypair_generate"));
    let sg = Subgraph {
        handler_id: "det_with_typed_keygen".into(),
        nodes: vec![read_node, typed_call_node],
        edges: vec![("r".into(), "typed_call".into(), "next".into())],
        deterministic: true,
    };

    let err = invariants::validate_subgraph(&sg, &InvariantConfig::default(), false)
        .expect_err("Inv-9 MUST reject non-deterministic typed-CALL in deterministic handler");
    assert!(
        matches!(err.kind(), InvariantViolation::Determinism),
        "expected Determinism violation; got {:?}",
        err.kind()
    );
}

/// Phase-3 G21-T1 sec-major-2 — counterpart to the above: a CALL Node
/// whose `target` names a deterministic typed-CALL op
/// (`engine:typed:blake3_hash`) MUST pass Inv-9.
#[test]
fn invariant_9_permits_deterministic_typed_call_op_in_deterministic_handler() {
    use benten_core::{OperationNode, Subgraph, Value};
    use benten_eval::TypedCallOp;

    assert!(
        TypedCallOp::Blake3Hash.is_deterministic(),
        "blake3_hash MUST classify deterministic (pure function of input)"
    );

    let read_node = OperationNode::new("r", PrimitiveKind::Read);
    let typed_call_node = OperationNode::new("typed_call", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed:blake3_hash"));
    let sg = Subgraph {
        handler_id: "det_with_typed_hash".into(),
        nodes: vec![read_node, typed_call_node],
        edges: vec![("r".into(), "typed_call".into(), "next".into())],
        deterministic: true,
    };

    invariants::validate_subgraph(&sg, &InvariantConfig::default(), false)
        .expect("deterministic typed-CALL op MUST pass Inv-9 in a deterministic handler");
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
