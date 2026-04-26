//! R3-A red-phase: STREAM composition inside CALL + ITERATE (G6-B).
//!
//! Pin source: plan §4 STREAM integration.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_engine_with_call_handler_streaming, testing_engine_with_iterate_handler_chunked,
};

/// `subgraph(...).stream(args)` composes inside a CALL handler. Outer
/// handler's call to a STREAM-emitting inner handler returns a chunk-stream
/// without flattening.
#[test]
#[ignore = "Phase 2b G6-B pending — STREAM inside CALL"]
fn stream_composes_inside_call_handler_of_handler_streaming() {
    let (mut engine, outer_id) = testing_engine_with_call_handler_streaming();
    let stream = engine
        .call_stream(&outer_id, "outer_stream", serde_json::json!({"n": 5}))
        .expect("call_stream from outer composing handler");

    let mut chunks: Vec<u64> = Vec::new();
    while let Some(chunk) = stream.next_blocking().expect("recv") {
        if chunk.final_chunk {
            break;
        }
        chunks.push(chunk.seq);
    }
    assert_eq!(chunks, (0..5u64).collect::<Vec<_>>());
}

/// STREAM inside ITERATE: bounded chunked output per iteration.
#[test]
#[ignore = "Phase 2b G6-B pending — STREAM inside ITERATE"]
fn stream_inside_iterate_bounded_chunked_output_per_iteration() {
    let (mut engine, handler_id) = testing_engine_with_iterate_handler_chunked(
        /* iterations */ 3, /* chunks_per_iter */ 4,
    );
    let stream = engine
        .call_stream(&handler_id, "iterate_chunked", serde_json::json!({}))
        .expect("call_stream returns AsyncIterable");

    let mut received_count = 0;
    while let Some(chunk) = stream.next_blocking().expect("recv") {
        if chunk.final_chunk {
            break;
        }
        received_count += 1;
    }
    assert_eq!(received_count, 3 * 4, "3 iterations × 4 chunks each");
}
