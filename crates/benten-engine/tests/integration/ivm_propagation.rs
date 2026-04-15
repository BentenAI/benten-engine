//! Phase 1 R3 integration — IVM write-propagation (Exit-criterion #2 foundation).
//!
//! Write 10 posts, list via View 3 (content listing, I5/G5-C), assert:
//!   (a) pagination returns all 10 in `createdAt` order
//!   (b) incremental view lag never exceeds one write
//!   (c) delete removes from the view
//!
//! **Status:** FAILING until G3 + G5 + G7 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post(n: u32) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("post-{n:02}")));
    props.insert("body".into(), Value::Text("lorem".into()));
    Node::new(vec!["post".into()], props)
}

#[test]
fn ivm_ten_writes_reflected_in_list_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    for i in 0..10u32 {
        engine.call(&handler_id, "post:create", post(i)).unwrap();
    }

    let listed = engine
        .call(&handler_id, "post:list", Node::empty())
        .unwrap();
    let items = listed.as_list().expect("post:list returns List");
    assert_eq!(items.len(), 10, "IVM View 3 must contain all 10 writes");

    let created_at: Vec<i64> = items
        .iter()
        .map(|n| match n.properties.get("createdAt") {
            Some(Value::Int(t)) => *t,
            other => panic!("every post has createdAt: Int; got {other:?}"),
        })
        .collect();
    let mut sorted = created_at.clone();
    sorted.sort_unstable();
    assert_eq!(
        created_at, sorted,
        "View 3 must return posts in createdAt order"
    );
}

#[test]
fn incremental_view_lag_never_exceeds_one_write() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    for i in 0..10u32 {
        engine.call(&handler_id, "post:create", post(i)).unwrap();
        let listed = engine
            .call(&handler_id, "post:list", Node::empty())
            .unwrap();
        let observed = listed.as_list().unwrap().len();
        let expected = usize::try_from(i + 1).unwrap();
        assert!(
            observed + 1 >= expected,
            "View 3 lag exceeded 1 write: observed={observed}, expected={expected}"
        );
    }
}

#[test]
fn delete_removes_from_view() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    let outcome = engine.call(&handler_id, "post:create", post(1)).unwrap();
    let cid = outcome.created_cid().unwrap();

    let mut del_input = BTreeMap::new();
    del_input.insert("cid".into(), Value::Text(cid.to_base32()));
    engine
        .call(
            &handler_id,
            "post:delete",
            Node::new(vec!["input".into()], del_input),
        )
        .unwrap();

    let listed = engine
        .call(&handler_id, "post:list", Node::empty())
        .unwrap();
    assert_eq!(
        listed.as_list().unwrap().len(),
        0,
        "deleted post must not appear in View 3"
    );
}
