//! G6-B integration: Engine STREAM surface (`call_stream` / `open_stream`)
//! against the dx-optimizer-corrected surface from plan §3 G6-B row.
//!
//! # Status by test
//!
//! - `engine_call_stream_surface_present_returns_handle_with_pending_error` —
//!   PASSES TODAY against the G6-B stub. Pins the surface shape +
//!   `E_PRIMITIVE_NOT_IMPLEMENTED` first-poll behavior.
//! - `engine_open_stream_surface_present_returns_handle_with_pending_error` —
//!   PASSES TODAY. Same shape contract as `call_stream`.
//! - `engine_call_stream_unregistered_handler_returns_not_found` — PASSES
//!   TODAY. Pins the early `E_NOT_FOUND` edge.
//! - `engine_stream_end_to_end` — `#[ignore]`d pending G6-A executor wiring
//!   (tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR). End-to-end
//!   chunk delivery requires the G6-A `tokio::sync::mpsc` executor body.
//! - `engine_callstream_returns_asynciterable` — `#[ignore]`d pending G6-A.
//!   The Rust-side `StreamHandle` is the locked-shape representation that
//!   the napi layer renders as JS `AsyncIterable`; the production iteration
//!   path requires G6-A's executor.
//! - `engine_openstream_explicit_close` — PASSES TODAY against the G6-B
//!   stub via the test-helper handle factory. Pins idempotent `close()`.
//! - `stream_close_propagates` — PASSES TODAY. Pins close() draining the
//!   pre-buffered chunks immediately.
//! - `stream_chunk_sequence_preserves_order` — PASSES TODAY against the
//!   test-helper handle. Pins `seq_so_far` bumping per delivered chunk.
//! - `stream_persist_true_materializes_aggregate_node` — `#[ignore]`d
//!   pending G6-A executor wiring (phil-r1-1 aggregate-Node materialization
//!   only fires once the executor body is live).
//! - `stream_backpressure_engages` — `#[ignore]`d pending G6-A executor
//!   wiring (D4-RESOLVED PULL-based bounded mpsc only exists once G6-A's
//!   executor is live).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::{Engine, ErrorCode, error::EngineError};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
fn engine_call_stream_surface_present_returns_handle_with_pending_error() {
    // Pin: G6-B's surface returns a `StreamHandle` whose first
    // `next_chunk()` surfaces `E_PRIMITIVE_NOT_IMPLEMENTED` until G6-A's
    // executor wires in. This test exercises the registered-handler
    // happy path; it does NOT require G6-A.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "post:list", Node::empty())
        .expect("call_stream surface present");
    let err = handle.next_chunk().unwrap_err();
    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(code, ErrorCode::PrimitiveNotImplemented);
        }
        other => panic!("expected PrimitiveNotImplemented, got {other:?}"),
    }
}

#[test]
fn engine_open_stream_surface_present_returns_handle_with_pending_error() {
    // Same shape contract as `call_stream`; the only public-API
    // difference is lifecycle (TS wrapper exposes explicit `dispose`/
    // `close` for `openStream` callers — `for await` auto-close is the
    // `callStream` path). Both share the inner dispatch.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .open_stream(&handler_id, "post:list", Node::empty())
        .expect("open_stream surface present");
    let err = handle.next_chunk().unwrap_err();
    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(code, ErrorCode::PrimitiveNotImplemented);
        }
        other => panic!("expected PrimitiveNotImplemented, got {other:?}"),
    }
}

#[test]
fn engine_call_stream_unregistered_handler_returns_not_found() {
    // Pin: pre-G6-A the engine still verifies the handler is registered
    // so callers get a useful E_NOT_FOUND early instead of an opaque
    // "stream did nothing" outcome.
    let (engine, _d) = open_engine();
    let err = engine
        .call_stream("nonexistent_handler", "act", Node::empty())
        .unwrap_err();
    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn engine_openstream_explicit_close() {
    // The `openStream` lifecycle contract: explicit `close()` is
    // idempotent + drains the handle. This pin exercises the TS-side
    // surface contract via the Rust `StreamHandle::close()` call. Uses
    // the G6-B test-helper to bypass the (G6-A-pending) executor.
    let (engine, _d) = open_engine();
    let mut handle = engine.testing_open_stream_for_test(vec![vec![1, 2], vec![3, 4]]);
    handle.close();
    handle.close(); // idempotent
    assert!(handle.is_drained(), "explicit close must drain the handle");
    assert!(
        handle.next_chunk().unwrap().is_none(),
        "drained handle yields end-of-stream"
    );
}

#[test]
fn stream_close_propagates() {
    // Pin: close() releases pre-buffered chunks immediately; subsequent
    // `next_chunk()` returns end-of-stream not the buffered chunk. This
    // models the dx-r1-2b-3 `for await ... break` semantic on the
    // Rust side.
    let (engine, _d) = open_engine();
    let mut handle = engine.testing_open_stream_for_test(vec![vec![10], vec![20], vec![30]]);
    // Pull one chunk then close.
    let first = handle.next_chunk().unwrap().expect("first chunk available");
    assert_eq!(first.bytes, vec![10]);
    handle.close();
    assert!(
        handle.next_chunk().unwrap().is_none(),
        "close must drop remaining chunks"
    );
}

#[test]
fn stream_chunk_sequence_preserves_order() {
    // Pin: chunks delivered in insertion order; `seq_so_far` bumps per
    // delivered chunk so the TS wrapper can expose `chunk.seq` for
    // replay/dedup symmetry with SUBSCRIBE.
    let (engine, _d) = open_engine();
    let chunks: Vec<Vec<u8>> = (0..8u8).map(|i| vec![i]).collect();
    let mut handle = engine.testing_open_stream_for_test(chunks);
    assert_eq!(handle.seq_so_far(), 0);
    let mut received = Vec::<u8>::new();
    while let Some(chunk) = handle.next_chunk().unwrap() {
        received.push(chunk.bytes[0]);
    }
    assert_eq!(received, (0..8u8).collect::<Vec<_>>());
    assert_eq!(handle.seq_so_far(), 8);
}

// ---------------------------------------------------------------------------
// Tests below this line require G6-A's `tokio::sync::mpsc` executor body
// + the real `benten_eval::primitives::stream` evaluator. Tracked in G6-A's
// `phase-2b/g6/a-stream-subscribe-core` PR.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn engine_stream_end_to_end() {
    // End-to-end: register a STREAM-emitting handler, invoke via
    // `call_stream`, drain N chunks, observe the terminal end-of-stream.
    // Requires G6-A's executor to actually drive the chunk_sink.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "post:stream", Node::empty())
        .expect("call_stream returns handle");
    let mut received = Vec::<Vec<u8>>::new();
    while let Some(chunk) = handle.next_chunk().expect("recv chunk (post-G6-A)") {
        received.push(chunk.bytes);
    }
    assert!(
        !received.is_empty(),
        "post-G6-A: at least one chunk delivered"
    );
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn engine_callstream_returns_asynciterable() {
    // The Rust-side `StreamHandle` IS the locked-shape representation
    // that the napi layer renders as a JS `AsyncIterable<Chunk>`. The
    // shape contract is already pinned by the type itself; this test
    // exercises the production iteration path which requires G6-A's
    // executor to deliver real chunks.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "post:stream", Node::empty())
        .expect("call_stream returns handle");
    let mut count = 0;
    while let Some(_chunk) = handle.next_chunk().expect("recv chunk (post-G6-A)") {
        count += 1;
    }
    assert!(count > 0, "post-G6-A: AsyncIterable yielded chunks");
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn stream_persist_true_materializes_aggregate_node() {
    // phil-r1-1 aggregate-Node materialization + canonical-CID
    // stability: `stream({ persist: true })` materializes the cumulative
    // chunk sequence as an aggregate Node whose CID is canonical-bytes
    // stable across invocations with identical input. Requires G6-A's
    // executor to actually emit + WRITE the aggregate.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "post:stream_persisted", Node::empty())
        .expect("call_stream returns handle");
    while handle.next_chunk().expect("recv (post-G6-A)").is_some() {}
    // Post-G6-A: assert that an aggregate Node materialized with the
    // expected canonical CID.
    let _ = engine; // placeholder until G6-A surface lands
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn stream_backpressure_engages() {
    // D4-RESOLVED: PULL-based bounded `tokio::sync::mpsc` (default
    // capacity 16). When the consumer is slower than the producer, the
    // producer's `send()` pends instead of buffering unboundedly. The
    // bounded mpsc only exists once G6-A's executor is live; pre-G6-A
    // the chunk-sink scaffold is empty.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "post:stream_high_volume", Node::empty())
        .expect("call_stream returns handle");
    // Post-G6-A: capacity_remaining drops to 0 under back-pressure;
    // total bytes consumed equals total bytes produced (lossless
    // default).
    while handle.next_chunk().expect("recv (post-G6-A)").is_some() {}
}
