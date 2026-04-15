//! Edge-case test: `engine.read_view(...)` on a stale view returns
//! `E_IVM_VIEW_STALE` (strict mode) or last-known-good (relaxed mode).
//!
//! This is the Engine-surface complement to the IVM crate's view-level
//! stale tests. Covers the full-stack path: engine -> ivm subscriber ->
//! view store -> read. Any link in that chain that hides the stale state
//! must be caught here.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError, ReadViewOptions};
use tempfile::tempdir;

fn engine_with_low_budget_view() -> (Engine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("stale.redb"))
        .with_test_ivm_budget(1) // tiny budget => trips quickly
        .build()
        .unwrap();
    (engine, dir)
}

fn make_post(title: &str, created_at: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    props.insert("createdAt".into(), Value::Int(created_at));
    Node::new(vec!["post".into()], props)
}

#[test]
fn read_view_strict_returns_stale_error() {
    let (engine, _dir) = engine_with_low_budget_view();

    // Push enough updates to trip the budget.
    for i in 0..5 {
        engine
            .create_node(&make_post(&format!("p{i}"), i as i64))
            .unwrap();
    }

    // Strict read: must surface E_IVM_VIEW_STALE through EngineError.
    let err = engine
        .read_view_with("system:ivm:content_listing", ReadViewOptions::strict())
        .expect_err("stale view strict-read must error");
    match err {
        EngineError::IvmViewStale { .. } => {}
        other => panic!("expected EngineError::IvmViewStale (E_IVM_VIEW_STALE), got {other:?}"),
    }
}

#[test]
fn read_view_relaxed_returns_last_known_good() {
    let (engine, _dir) = engine_with_low_budget_view();

    // Seed with one clean post so last-known-good is non-empty.
    engine.create_node(&make_post("seed", 1)).unwrap();
    let seed_snapshot = engine
        .read_view_with("system:ivm:content_listing", ReadViewOptions::strict())
        .expect("first read before budget trip must succeed");

    // Over-budget writes.
    for i in 2..10 {
        engine
            .create_node(&make_post(&format!("p{i}"), i as i64))
            .unwrap();
    }

    // Relaxed read: must return the last-known-good (seed only),
    // not error, not partial mid-update state.
    let relaxed = engine
        .read_view_with("system:ivm:content_listing", ReadViewOptions::allow_stale())
        .expect("relaxed read must succeed");
    assert_eq!(
        relaxed, seed_snapshot,
        "relaxed read on stale view must byte-equal last-known-good"
    );
}

#[test]
fn read_view_unknown_view_id_errors() {
    // Degenerate: a view id that no View ever registered for. Must error
    // cleanly, NOT return empty (which would be ambiguous with "view
    // exists, no entries").
    let (engine, _dir) = engine_with_low_budget_view();
    let err = engine
        .read_view_with("system:ivm:nonexistent", ReadViewOptions::strict())
        .expect_err("unknown view id must error");
    match err {
        EngineError::UnknownView { .. } => {}
        other => panic!("expected EngineError::UnknownView, got {other:?}"),
    }
}
