//! G6-B: STREAM composition inside CALL + ITERATE (plan §4 STREAM integration).
//!
//! # Status by test
//!
//! Both tests `#[ignore]`d pending G6-A executor wiring; tracks G6-A's
//! `phase-2b/g6/a-stream-subscribe-core` PR. Composition fixtures
//! require the G6-A `tokio::sync::mpsc` executor body to actually emit
//! chunks; the G6-B stub returns `E_PRIMITIVE_NOT_IMPLEMENTED` on first
//! poll which short-circuits the composition observation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::Engine;

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn stream_composes_inside_call_handler_of_handler_streaming() {
    // `subgraph(...).stream(args)` composes inside a CALL handler. The
    // outer handler's call to a STREAM-emitting inner handler returns a
    // chunk-stream without flattening. Requires G6-A's executor.
    let (engine, _d) = open_engine();
    let outer_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&outer_id, "outer_stream", Node::empty())
        .expect("call_stream surface present");
    while handle.next_chunk().expect("recv (post-G6-A)").is_some() {}
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn stream_inside_iterate_bounded_chunked_output_per_iteration() {
    // STREAM inside ITERATE: bounded chunked output per iteration.
    // Requires G6-A's executor + ITERATE composition wired through.
    let (engine, _d) = open_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "iterate_chunked", Node::empty())
        .expect("call_stream surface present");
    while handle.next_chunk().expect("recv (post-G6-A)").is_some() {}
}
