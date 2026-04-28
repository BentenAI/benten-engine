//! Phase 2b Wave-8f: `Engine::register_subgraph_replace` semantic + version-
//! chain bookkeeping.
//!
//! Pins the positive contract that `register_subgraph_replace`:
//! - admits a new CID under the same handler_id (no `DuplicateHandler`)
//! - reports the previous CID + bumped chain depth
//! - is idempotent for identical content (no chain growth, no error)
//! - re-runs the full G6 invariant battery on the replacement body
//! - leaves the legacy `register_subgraph` rejection contract untouched
//!
//! The legacy `register_subgraph` happy path + duplicate-rejection tests
//! live in `register_subgraph_failures.rs` + the wider integration
//! suite — this file targets the new replace surface only.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, EngineError, ErrorCode};
use benten_eval::SubgraphBuilder;
use benten_eval::{SubgraphBuilderExt, SubgraphExt};
use tempfile::tempdir;

fn fresh_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("replace.redb"))
        .build()
        .unwrap();
    (engine, dir)
}

fn build_handler(handler_id: &str, label: &str) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new(handler_id);
    let r = sb.read(label);
    sb.respond(r);
    sb.build_validated().expect("must build")
}

#[test]
fn register_subgraph_replace_first_call_seeds_chain_no_predecessor() {
    let (engine, _dir) = fresh_engine();
    let sg = build_handler("h-replace-first", "post");
    let expected_cid = sg.cid().unwrap();

    let outcome = engine
        .register_subgraph_replace(sg)
        .expect("first registration via replace must succeed");
    assert_eq!(outcome.handler_id, "h-replace-first");
    assert_eq!(outcome.cid, expected_cid);
    assert!(outcome.previous_cid.is_none());
    assert_eq!(outcome.chain_depth, 1);
    assert!(!outcome.replaced(), "first registration is not a replace");
    assert_eq!(outcome.version_tag(), "v1");

    let chain = engine.handler_version_chain("h-replace-first");
    assert_eq!(chain, vec![expected_cid]);
}

#[test]
fn register_subgraph_replace_distinct_content_bumps_chain() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-bump";
    let v1 = build_handler(h, "post");
    let v1_cid = v1.cid().unwrap();
    let v2 = build_handler(h, "comment"); // different body — different CID
    let v2_cid = v2.cid().unwrap();
    assert_ne!(v1_cid, v2_cid);

    let _ = engine.register_subgraph_replace(v1).unwrap();
    let outcome = engine
        .register_subgraph_replace(v2)
        .expect("second registration with different body must succeed");
    assert_eq!(outcome.cid, v2_cid);
    assert_eq!(outcome.previous_cid, Some(v1_cid));
    assert_eq!(outcome.chain_depth, 2);
    assert!(outcome.replaced());
    assert_eq!(outcome.version_tag(), "v2");

    let chain = engine.handler_version_chain(h);
    assert_eq!(chain, vec![v2_cid, v1_cid], "newest-first ordering");
}

#[test]
fn register_subgraph_replace_identical_content_is_idempotent_no_chain_growth() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-idem";
    let _ = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .unwrap();
    let outcome = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .expect("identical re-register must succeed idempotently");
    assert_eq!(
        outcome.chain_depth, 1,
        "chain must not grow on identical body"
    );
    assert!(!outcome.replaced(), "identical body is not a replace");
    assert_eq!(outcome.version_tag(), "v1");
}

#[test]
fn register_subgraph_replace_runs_full_invariant_battery_on_new_body() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-bad";
    let _ = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .unwrap();

    // Build a cyclic subgraph under the same handler_id; replace must
    // reject with the same Inv-1 cycle code register_subgraph would.
    let mut sb = SubgraphBuilder::new(h);
    let r = sb.read("post");
    sb.add_edge(r, r);
    let cyclic = sb.build_unvalidated_for_test();

    let err = engine
        .register_subgraph_replace(cyclic)
        .expect_err("cycle must fail at replace registration too");
    match err {
        EngineError::Invariant(e) => assert_eq!(e.code(), ErrorCode::InvCycle),
        other => panic!("expected EngineError::Invariant(InvCycle), got {other:?}"),
    }
}

#[test]
fn register_subgraph_replace_dispatches_new_body_on_next_call() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-dispatch";

    let v1 = build_handler(h, "post");
    let _ = engine.register_subgraph_replace(v1).unwrap();
    let live_after_v1 = engine.handler_version_chain(h);
    assert_eq!(live_after_v1.len(), 1);

    // Replace with a structurally-different body (different read label →
    // different node CID → different subgraph CID).
    let v2 = build_handler(h, "comment");
    let v2_cid = v2.cid().unwrap();
    let outcome = engine.register_subgraph_replace(v2).unwrap();

    // The handlers map's new live entry MUST equal v2's CID, not v1's.
    // The version-chain accessor exposes this directly without exposing
    // the handlers Mutex.
    let chain = engine.handler_version_chain(h);
    assert_eq!(chain[0], v2_cid, "live target must be v2 after replace");
    assert_eq!(outcome.chain_depth, 2);
}

#[test]
fn legacy_register_subgraph_still_rejects_duplicate_with_different_content() {
    // Wave-8f introduces register_subgraph_replace WITHOUT changing
    // legacy register_subgraph's rejection contract. This test pins
    // that the legacy method continues to reject a duplicate-with-
    // different-content under the same handler_id.
    let (engine, _dir) = fresh_engine();
    let h = "h-legacy";
    engine.register_subgraph(build_handler(h, "post")).unwrap();
    let err = engine
        .register_subgraph(build_handler(h, "comment"))
        .expect_err("legacy register must still reject a content-mismatched dup");
    assert!(matches!(err, EngineError::DuplicateHandler { .. }));
}

#[test]
fn legacy_register_subgraph_seeds_version_chain_too() {
    // Pin: register_subgraph (the legacy path) ALSO seeds the version
    // chain on first registration so a later register_subgraph_replace
    // can name the predecessor cleanly.
    let (engine, _dir) = fresh_engine();
    let h = "h-legacy-seed";
    let v1 = build_handler(h, "post");
    let v1_cid = v1.cid().unwrap();
    engine.register_subgraph(v1).unwrap();
    let chain = engine.handler_version_chain(h);
    assert_eq!(chain, vec![v1_cid]);

    // Now hot-replace via the new API — predecessor must be the
    // legacy-registered CID.
    let v2 = build_handler(h, "comment");
    let outcome = engine.register_subgraph_replace(v2).unwrap();
    assert_eq!(outcome.previous_cid, Some(v1_cid));
}
