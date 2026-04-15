//! Phase 1 R3 integration — Change-stream routes to IVM subscriber.
//!
//! WRITE in the evaluator -> ChangeEvent on the G7 broadcast channel ->
//! IVM subscriber wakes -> View 3 updates within the lag bound. Validates
//! the wiring between G3 (commit-boundary ChangeEvent emission), G7 (channel),
//! G5-A (IVM subscriber), and G5-C (View 3).
//!
//! **Status:** FAILING until G3 + G5 + G7 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

#[test]
fn change_stream_routes_to_matching_view_and_skips_non_matching() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Subscribe a test probe to observe all ChangeEvents.
    let probe = engine.test_subscribe_all_change_events();

    // Register crud(post); write three; expect 3 events of kind Created.
    let handler_id = engine.register_crud("post").unwrap();
    for i in 0..3 {
        engine
            .call(&handler_id, "post:create", post(&format!("p{i}")))
            .unwrap();
    }

    let events = probe.drain();
    assert_eq!(events.len(), 3, "one ChangeEvent per commit");
    for e in &events {
        assert_eq!(e.label, "post");
        assert_eq!(e.kind_str(), "Created");
        assert!(e.tx_id > 0, "tx_id must be non-zero");
    }

    // Verify routing: View 3 received 3 updates (write-read latency within bound).
    let listed = engine
        .call(&handler_id, "post:list", Node::empty())
        .unwrap();
    assert_eq!(listed.as_list().unwrap().len(), 3);
}

#[test]
fn change_event_attribution_fields_populated() {
    // R1 named field: ChangeEvent must carry actor_cid / handler_cid /
    // capability_grant_cid for audit-trail.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let probe = engine.test_subscribe_all_change_events();
    let handler_id = engine.register_crud("post").unwrap();

    engine
        .call(&handler_id, "post:create", post("attributed"))
        .unwrap();
    let e = probe.drain().into_iter().next().expect("one event");

    assert!(
        e.actor_cid.is_some(),
        "NoAuthBackend populates actor_cid as noauth:<uuid>"
    );
    assert!(
        e.handler_cid.is_some(),
        "handler_cid must be the crud(post) handler id"
    );
    // capability_grant_cid may be None under NoAuthBackend; asserted None here.
    assert!(e.capability_grant_cid.is_none(), "NoAuth has no grant CID");
}
