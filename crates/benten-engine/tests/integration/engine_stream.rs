//! R3-A red-phase: Engine STREAM end-to-end + DSL surfaces (G6-B).
//!
//! Pin source: exit-1 + plan §3 G6-B + dx-r1-2b STREAM DSL.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_engine_with_streaming_handler, testing_register_streaming_handler,
};

/// `Engine::stream_handler` end-to-end: register a streaming handler →
/// invoke → drain chunks.
#[test]
#[ignore = "Phase 2b G6-B pending — Engine::stream_handler"]
fn engine_stream_end_to_end() {
    let (mut engine, handler_id) = testing_engine_with_streaming_handler();
    let chunks_to_emit = 8;
    let mut iter = engine
        .stream_handler(
            &handler_id,
            "echo_stream",
            serde_json::json!({"n": chunks_to_emit}),
        )
        .expect("stream_handler returns AsyncIterable-shaped iterator");

    let mut received: Vec<u64> = Vec::new();
    while let Some(chunk) = iter.next_blocking().expect("recv chunk") {
        if chunk.final_chunk {
            break;
        }
        received.push(chunk.seq);
    }
    assert_eq!(received, (0..chunks_to_emit as u64).collect::<Vec<_>>());
}

/// `engine.callStream(handlerId, action, input)` returns `AsyncIterable<Chunk>`
/// (dx-r1-2b corrected DSL surface).
#[test]
#[ignore = "Phase 2b G6-B pending — engine.callStream shape"]
fn engine_callstream_returns_asynciterable() {
    let (mut engine, handler_id) = testing_engine_with_streaming_handler();
    let stream = engine
        .call_stream(&handler_id, "echo_stream", serde_json::json!({"n": 4}))
        .expect("callStream surface present");

    // Stream must be `AsyncIterable`-shaped: implements Iterator-of-Chunk
    // (Rust side; napi side wraps as JS AsyncIterable).
    fn assert_asynciter<T: benten_engine::stream::ChunkIter>(_: &T) {}
    assert_asynciter(&stream);
}

/// `engine.openStream` returns a handle with explicit `close()`; close is
/// idempotent.
#[test]
#[ignore = "Phase 2b G6-B pending — engine.openStream explicit close"]
fn engine_openstream_explicit_close() {
    let (mut engine, handler_id) = testing_engine_with_streaming_handler();
    let mut handle = engine
        .open_stream(&handler_id, "echo_stream", serde_json::json!({"n": 4}))
        .expect("openStream surface present");

    handle
        .close()
        .expect("close is infallible (or returns idempotent Ok)");
    handle.close().expect("close is idempotent on repeat");
}
