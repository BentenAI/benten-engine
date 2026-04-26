//! R3-A red-phase: Engine SUBSCRIBE end-to-end + ad-hoc onChange (G6-B).
//!
//! Pin source: exit-1 + plan §3 G6-B + dx-r1-2b SUBSCRIBE rename.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_engine_with_subscribe_handler, testing_subscribe_simulate_write,
};

/// `Engine::subscribe` end-to-end: register a SUBSCRIBE handler →
/// simulate matching write → handler observes the change event.
#[test]
#[ignore = "Phase 2b G6-B pending — Engine::subscribe"]
fn engine_subscribe_end_to_end() {
    let (mut engine, handler_id) = testing_engine_with_subscribe_handler("/posts/*");
    let observation_handle = engine
        .install_subscribe_observation(&handler_id)
        .expect("install observation");

    testing_subscribe_simulate_write(&mut engine, "/posts/123", serde_json::json!({"title": "x"}));

    let observed = observation_handle
        .next_blocking(std::time::Duration::from_millis(200))
        .expect("matching write must reach handler");
    assert_eq!(observed.label, "/posts/123");
}

/// `engine.onChange(pattern, callback) -> Subscription` — ad-hoc consumer
/// pattern (renamed from `engine.subscribe` to avoid name-collision with the
/// DSL builder `subgraph(...).subscribe(args)`).
#[test]
#[ignore = "Phase 2b G6-B pending — engine.onChange ad-hoc"]
fn engine_onchange_ad_hoc_consumer_pattern() {
    let mut engine = benten_engine::testing::testing_engine_default();
    let mut received: Vec<String> = Vec::new();
    let sub = engine
        .on_change("/users/*", |event| {
            // callback observed: anchor label → pushed
            received.push(event.label.clone());
        })
        .expect("onChange registers");

    testing_subscribe_simulate_write(&mut engine, "/users/alice", serde_json::json!({}));
    testing_subscribe_simulate_write(&mut engine, "/posts/123", serde_json::json!({}));

    sub.flush_blocking(std::time::Duration::from_millis(200));

    assert_eq!(received, vec!["/users/alice".to_string()]);
    sub.cancel().expect("cancel ok");
}
