//! Edge-case tests: `engine.register_subgraph(...)` failures surface the
//! catalog's `E_INV_*` codes with proper context.
//!
//! The happy-path "registration succeeds" test is owned by rust-test-writer-unit.
//! This file pins the NEGATIVE contract: every registration failure maps
//! to a specific code AND (in multi-fault cases) the catch-all populates
//! the violated list.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, EngineError, ErrorCode};
use benten_eval::SubgraphBuilder;
use tempfile::tempdir;

fn fresh_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("reg.redb"))
        .build()
        .unwrap();
    (engine, dir)
}

#[test]
fn register_cycle_fails_with_inv_cycle() {
    let (engine, _dir) = fresh_engine();

    let mut sb = SubgraphBuilder::new("bad_cycle");
    let r = sb.read("r");
    sb.add_edge(r, r);
    let sg = sb.build_unvalidated_for_test(); // bypass local validation to exercise the engine's path

    let err = engine
        .register_subgraph(sg)
        .expect_err("cycle must fail at registration");
    match err {
        EngineError::Invariant(e) => assert_eq!(e.code(), ErrorCode::InvCycle),
        other => panic!("expected EngineError::Invariant(E_INV_CYCLE), got {other:?}"),
    }
}

#[test]
fn register_depth_exceeded_fails_with_inv_depth() {
    let (engine, _dir) = fresh_engine();

    let cap = benten_eval::limits::DEFAULT_MAX_DEPTH;
    let mut sb = SubgraphBuilder::new("too_deep");
    let r = sb.read("r");
    let mut prev = r;
    for _ in 0..(cap + 2) {
        prev = sb.call(prev, "inner");
    }
    sb.respond(prev);

    let sg = sb.build_unvalidated_for_test();

    let err = engine
        .register_subgraph(sg)
        .expect_err("depth over cap must fail");
    match err {
        EngineError::Invariant(e) => assert_eq!(e.code(), ErrorCode::InvDepthExceeded),
        other => panic!("expected E_INV_DEPTH_EXCEEDED, got {other:?}"),
    }
}

#[test]
fn register_returns_inv_registration_on_multiple_violations() {
    let (engine, _dir) = fresh_engine();

    let mut sb = SubgraphBuilder::new("multi");
    let r = sb.read("r");
    // Violation 1: cycle
    sb.add_edge(r, r);
    // Violation 2: fan-out over cap
    for _ in 0..(benten_eval::limits::DEFAULT_MAX_FANOUT + 1) {
        let _ = sb.transform(r, "$input");
    }

    let sg = sb.build_unvalidated_for_test();

    // Engine registers in aggregate mode so the response names every
    // violation; otherwise a single-invariant specific code is returned.
    let err = engine
        .register_subgraph_aggregate(sg)
        .expect_err("multi-violation aggregate must fail");
    match err {
        EngineError::Invariant(e) => {
            assert_eq!(e.code(), ErrorCode::InvRegistration);
            let list = e
                .violated_invariants()
                .expect("aggregate mode populates violated list");
            assert!(list.contains(&1));
            assert!(list.contains(&3));
        }
        other => panic!("expected E_INV_REGISTRATION, got {other:?}"),
    }
}

#[test]
fn register_duplicate_handler_id_errors() {
    // Boundary: two subgraphs registered under the same handler id.
    // Phase 1 contract: second registration replaces first only if the
    // content hash matches (idempotent re-registration). Different
    // content under same id must error.
    let (engine, _dir) = fresh_engine();

    let mut sb1 = SubgraphBuilder::new("handler_x");
    let r1 = sb1.read("a");
    sb1.respond(r1);
    let sg1 = sb1.build_validated().unwrap();

    let mut sb2 = SubgraphBuilder::new("handler_x");
    let r2 = sb2.read("b"); // different payload -> different CID
    sb2.respond(r2);
    let sg2 = sb2.build_validated().unwrap();

    engine.register_subgraph(sg1).unwrap();

    let err = engine
        .register_subgraph(sg2)
        .expect_err("re-registration under same id with different content must fail");
    match err {
        EngineError::DuplicateHandler { .. } => {}
        other => panic!("expected EngineError::DuplicateHandler, got {other:?}"),
    }
}

#[test]
fn idempotent_re_registration_of_identical_subgraph_succeeds() {
    // Positive boundary pair: registering the SAME subgraph (same content,
    // same id) twice must succeed (idempotent), NOT error.
    let (engine, _dir) = fresh_engine();
    let mut sb = SubgraphBuilder::new("handler_y");
    let r = sb.read("a");
    sb.respond(r);
    let sg = sb.build_validated().unwrap();

    engine.register_subgraph(sg.clone()).unwrap();
    engine
        .register_subgraph(sg)
        .expect("re-registering identical subgraph must succeed (idempotent)");
}
