//! R3-A red-phase: SUBSCRIBE composes with EMIT (G6-B).
//!
//! Pin source: plan §4 SUBSCRIBE — subscriber-side strategy where a
//! subscribed handler issues EMIT in response to change events.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_engine_with_subscribe_emit_chain, testing_subscribe_simulate_write,
};

/// SUBSCRIBE → handler → EMIT: a subscribed handler that responds to a
/// change event by EMITting (subscriber-side strategy). End-to-end the EMIT
/// must be observable through the engine's emit-bus.
#[test]
#[ignore = "Phase 2b G6-B pending — SUBSCRIBE + EMIT composition"]
fn subscribe_composes_with_emit_subscriber_side_strategy() {
    let (mut engine, handler_id, emit_bus) =
        testing_engine_with_subscribe_emit_chain("/orders/*", "order_event");

    testing_subscribe_simulate_write(&mut engine, "/orders/789", serde_json::json!({"total": 42}));

    let emitted = emit_bus
        .next_blocking(std::time::Duration::from_millis(200))
        .expect("subscribed handler must emit in response to matching write");
    assert_eq!(emitted.event_name, "order_event");
    assert_eq!(emitted.payload["total"], 42);
    let _ = handler_id;
}
