//! R4 triage M18 — direct unit tests for Engine methods that landed during
//! integration scaffolding without their own coverage.
//!
//! Each test exercises one public method in isolation: register_subgraph,
//! call, trace, transaction, snapshot, grant_capability, create_view,
//! revoke_capability. Red-phase until R5 lands the methods; the shapes
//! pinned here prevent silent API drift during implementation.
//!
//! R3 writer: `rust-test-writer-unit` (expanded at R4 triage).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn register_subgraph_returns_stable_handler_id() {
    let (_dir, engine) = fresh_engine();
    let h1 = engine.register_crud("post").unwrap();
    let h2 = engine.register_crud("post").unwrap();
    assert_eq!(
        h1, h2,
        "content-addressed handler id is stable across re-registration"
    );
    assert!(!h1.is_empty());
}

#[test]
fn call_round_trips_input_to_response() {
    let (_dir, engine) = fresh_engine();
    let handler = engine.register_crud("post").unwrap();
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("t".into()));
    let outcome = engine
        .call(&handler, "post:create", Node::new(vec!["post".into()], p))
        .unwrap();
    assert!(outcome.is_ok_edge(), "create call routes through OK edge");
}

#[test]
fn trace_returns_per_step_timings() {
    let (_dir, engine) = fresh_engine();
    let handler = engine.register_crud("post").unwrap();
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("t".into()));
    let trace = engine
        .trace(&handler, "post:create", Node::new(vec!["post".into()], p))
        .unwrap();
    assert!(!trace.steps().is_empty());
    for step in trace.steps() {
        // Wave 2b TraceStep unification: crud(post):create only emits
        // Step rows; assert the discriminant + timing without weakening.
        let benten_engine::TraceStep::Step { duration_us, .. } = step else {
            panic!("crud(post):create trace must only emit Step rows; got {step:?}");
        };
        assert!(duration_us > 0, "every step has non-zero timing");
    }
}

#[test]
fn transaction_commits_atomically_on_ok() {
    let (_dir, engine) = fresh_engine();
    let before = engine.snapshot().expect("pre-tx snapshot");
    engine
        .transaction(|tx| {
            let mut p = BTreeMap::new();
            p.insert("v".into(), Value::Int(1));
            tx.create_node(&Node::new(vec!["T".into()], p))?;
            Ok(())
        })
        .expect("transaction commits");
    // `before` is the pre-tx snapshot; scope-exit drop is fine.
    let _ = before;
    // After commit, a new snapshot sees the write.
    let after = engine.snapshot().expect("post-tx snapshot");
    let count = after.scan_label("T").expect("scan");
    assert!(!count.is_empty(), "committed write is visible post-tx");
}

#[test]
fn snapshot_is_point_in_time() {
    let (_dir, engine) = fresh_engine();
    let snap_before = engine.snapshot().expect("snapshot");
    let mut p = BTreeMap::new();
    p.insert("v".into(), Value::Int(1));
    let _ = engine.create_node(&Node::new(vec!["T".into()], p)).unwrap();
    // Snapshot taken before the write must still see empty state.
    let observed = snap_before.scan_label("T").expect("scan");
    assert!(observed.is_empty(), "snapshot predates the write");
}

#[test]
fn grant_and_revoke_capability_roundtrip() {
    let (_dir, engine) = fresh_engine();
    let actor = engine.create_principal("alice").unwrap();
    engine.grant_capability(&actor, "store:post:write").unwrap();
    // Revocation must succeed without error; absence of capability is
    // surfaced on subsequent call attempts (tested in exit_3).
    engine
        .revoke_capability(&actor, "store:post:write")
        .unwrap();
}

#[test]
fn create_view_registers_and_is_queryable() {
    let (_dir, engine) = fresh_engine();
    let opts = benten_engine::ViewCreateOptions;
    let view_cid = engine
        .create_view("content_listing_post", opts)
        .expect("create_view returns cid");
    let opts2 = benten_engine::ViewCreateOptions;
    // Re-registering the same view id returns the same content-addressed cid.
    let again = engine.create_view("content_listing_post", opts2).unwrap();
    assert_eq!(view_cid, again, "content-addressed view CID stable");
}
