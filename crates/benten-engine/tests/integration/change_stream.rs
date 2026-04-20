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
        assert!(
            e.has_label("post"),
            "event must carry the post label; got labels={:?}",
            e.labels
        );
        assert_eq!(e.kind_str(), "Created");
        assert!(e.tx_id > 0, "tx_id must be non-zero");
    }

    // Verify routing: View 3 received 3 updates (write-read latency within bound).
    let listed = engine
        .call(&handler_id, "post:list", Node::empty())
        .unwrap();
    assert_eq!(listed.as_list().unwrap().len(), 3);
}

/// r6-sec-5 regression — an unbounded `observed_events` buffer is a
/// memory-exhaustion DoS against a long-running engine: an attacker who can
/// drive the write path faster than any subscriber drains will grow the
/// buffer without limit. The engine now bounds the buffer with a
/// drop-oldest policy and surfaces drops via the
/// `benten.change_stream.dropped_events` metric.
///
/// The test writes past the configured capacity WITHOUT draining and
/// asserts both that the buffer stays bounded and that the drop-count
/// metric reflects the overflow.
#[test]
fn change_stream_bounded_under_probe_without_drain() {
    let dir = tempfile::tempdir().unwrap();
    let cap: usize = 8;
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .change_stream_capacity(cap)
        .build()
        .unwrap();

    assert_eq!(engine.change_stream_capacity(), cap);

    let handler_id = engine.register_crud("post").unwrap();

    // Write 3x the capacity with no probe drain — an attacker-driven write
    // loop that races a stalled subscriber.
    let writes: usize = cap * 3;
    for i in 0..writes {
        engine
            .call(&handler_id, "post:create", post(&format!("p{i}")))
            .unwrap();
    }

    // The cumulative event counter reflects every commit.
    assert_eq!(
        engine.change_event_count(),
        writes as u64,
        "every commit must increment the cumulative counter regardless of buffer cap"
    );

    // The drop counter is the overflow past capacity.
    let metrics = engine.metrics_snapshot();
    let dropped = metrics
        .get("benten.change_stream.dropped_events")
        .copied()
        .expect("dropped-events metric must be surfaced");
    let expected_drops = writes - cap;
    #[allow(
        clippy::cast_precision_loss,
        reason = "expected_drops is small (24 in this test); lossy cast to f64 is exact here"
    )]
    let expected_drops_f64 = expected_drops as f64;
    assert!(
        (dropped - expected_drops_f64).abs() < f64::EPSILON,
        "dropped count must equal overflow past capacity; got {dropped}, expected {expected_drops}"
    );

    // A late-attached probe sees only the bounded tail — never more than
    // `cap` events.
    let probe = engine.test_subscribe_all_change_events();
    // The probe's start_offset is the current event_count; no NEW writes
    // have happened since, so drain() returns empty. The important
    // assertion is that the buffer itself stayed bounded: a fresh write
    // under the same saturated state must succeed (no OOM) and the probe
    // then sees it.
    engine
        .call(&handler_id, "post:create", post("after-probe"))
        .unwrap();
    let drained = probe.drain();
    assert_eq!(
        drained.len(),
        1,
        "probe attached after saturation sees only events observed after attachment"
    );
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
