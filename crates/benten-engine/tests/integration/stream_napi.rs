//! R3-A red-phase: STREAM napi async-iterator back-pressure (G6-B).
//!
//! Pin source: streaming-systems must_pass + dx-r1-2b-3.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_engine_with_streaming_handler, testing_napi_consumer_break, testing_napi_consumer_pause,
};
use std::time::Duration;

/// Native consumer's `for await` pause-via-explicit-await stalls native
/// producer (cross-boundary back-pressure).
#[test]
#[ignore = "Phase 2b G6-B pending — napi back-pressure cross-boundary"]
fn stream_napi_async_iterator_back_pressure_propagates_native() {
    let (mut engine, handler_id) = testing_engine_with_streaming_handler();
    let stream = engine
        .call_stream(&handler_id, "infinite", serde_json::json!({}))
        .expect("infinite-streaming handler present");

    // Pause the napi-side consumer for 100ms; native producer MUST observe
    // back-pressure (capacity_remaining drops to 0 or producer awaits).
    let report = testing_napi_consumer_pause(stream, Duration::from_millis(100));
    assert!(
        report.producer_observed_backpressure,
        "native producer must observe back-pressure when napi consumer pauses"
    );
    assert!(
        report.chunks_dropped == 0,
        "lossless default: no chunks dropped under back-pressure"
    );
}

/// `for await` early `break` on the napi side releases the producer (no
/// orphan task).
#[test]
#[ignore = "Phase 2b G6-B pending — for-await break releases producer"]
fn stream_for_await_break_releases_producer() {
    let (mut engine, handler_id) = testing_engine_with_streaming_handler();
    let stream = engine
        .call_stream(&handler_id, "infinite", serde_json::json!({}))
        .expect("infinite-streaming handler");

    let report = testing_napi_consumer_break(stream, /* chunks_before_break */ 4);

    assert_eq!(report.chunks_received, 4);
    assert!(
        report.producer_released,
        "for-await break must release the producer; no orphan task lingers"
    );
}
