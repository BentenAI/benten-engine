//! G6-B: STREAM napi cross-boundary back-pressure (Rust-side integration).
//!
//! The actual napi back-pressure surface tests live on the JS side at
//! `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`
//! (vitest, runs against the cdylib built with `--features test-helpers`).
//! This Rust-side file pins the `StreamHandle` shape that the napi
//! `Engine::callStream` / `Engine::openStream` / `Engine::testingOpenStreamForTest`
//! adapters wrap so a regression in the Rust-side surface fails the
//! workspace test run before the napi build is even invoked.
//!
//! # Status by test
//!
//! - `stream_handle_drains_pre_buffered_chunks_in_order_for_napi_bridge` —
//!   PASSES TODAY. Mirrors the contract the napi `next_chunk_adapter`
//!   relies on.
//! - `stream_handle_close_idempotent_for_napi_bridge` — PASSES TODAY.
//! - `stream_napi_async_iterator_back_pressure_propagates_native` —
//!   `#[ignore]`d pending G6-A executor wiring (D4-RESOLVED bounded
//!   `tokio::sync::mpsc` only exists once the executor is live).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Chunk, Engine, StreamHandle};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
fn stream_handle_drains_pre_buffered_chunks_in_order_for_napi_bridge() {
    // Pin: the contract `bindings/napi/src/stream.rs::next_chunk_adapter`
    // relies on. Each `next_chunk()` returns the next pre-buffered
    // chunk in insertion order; end-of-stream surfaces as `Ok(None)`
    // (the napi adapter renders this as `{ done: true }` to JS).
    let (engine, _d) = open_engine();
    let mut handle: StreamHandle =
        engine.testing_open_stream_for_test(vec![vec![1], vec![2], vec![3]]);
    let c1: Chunk = handle.next_chunk().unwrap().unwrap();
    assert_eq!(c1.bytes, vec![1]);
    let c2: Chunk = handle.next_chunk().unwrap().unwrap();
    assert_eq!(c2.bytes, vec![2]);
    let c3: Chunk = handle.next_chunk().unwrap().unwrap();
    assert_eq!(c3.bytes, vec![3]);
    assert!(handle.next_chunk().unwrap().is_none());
}

#[test]
fn stream_handle_close_idempotent_for_napi_bridge() {
    // Pin: the contract `bindings/napi/src/stream.rs::close_handle_adapter`
    // relies on. `close()` is infallible + idempotent.
    let (engine, _d) = open_engine();
    let mut handle = engine.testing_open_stream_for_test(vec![vec![1], vec![2]]);
    handle.close();
    handle.close();
    assert!(handle.is_drained());
}

#[test]
fn stream_napi_async_iterator_back_pressure_propagates_native() {
    // Phase-3 G20-A2 (D12 wave-8a): the Rust-side StreamHandle contract
    // backing the napi async-iterator surface. The actual bounded
    // tokio::sync::mpsc back-pressure semantics are tested at the JS
    // layer (`bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`);
    // this fixture pins the Rust-side observable: the
    // `testing_open_stream_for_test` factory produces a handle whose
    // pre-buffered chunks drain cleanly + close() is idempotent
    // — precisely the contract napi's `next_chunk_adapter` /
    // `close_handle_adapter` rely on.
    let (engine, _d) = open_engine();
    let mut handle = engine.testing_open_stream_for_test(vec![vec![10], vec![20], vec![30]]);
    let mut received = Vec::new();
    while let Some(chunk) = handle.next_chunk().expect("recv chunk") {
        received.push(chunk.bytes);
    }
    assert_eq!(received, vec![vec![10], vec![20], vec![30]]);
    handle.close();
    handle.close();
    assert!(handle.is_drained());
}
