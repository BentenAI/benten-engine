//! Phase 2b R3-B — STREAM-into-SANDBOX integration test (G7-A).
//!
//! Pin sources: wsa-18, arch-pre-r1-9. R2 §10 owner: R3-B (SANDBOX-side
//! composition is the point; ChunkSink is the consumed contract).
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored.
//!
//! The full STREAM-into-SANDBOX harness (a wasm guest reading chunks
//! from an upstream STREAM via a `chunk_emit` host-fn) requires a
//! host-fn that's NOT in the D1 surface (D1 ships time / log / kv:read
//! / random); D-PHASE-3-X may add `chunk_emit`. In place of the full
//! harness, this test exercises the load-bearing back-pressure
//! invariant via the eval-side ChunkSink directly: a producer fills a
//! bounded sink + a slower consumer drains; lossless mode applies
//! back-pressure (no chunks dropped). The shape extends naturally to
//! a full STREAM-into-SANDBOX wiring once the host-fn lands.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::num::NonZeroUsize;

use benten_eval::testing::{testing_make_chunk_sink, testing_run_lossless_stream_with_schedule};

#[test]
fn stream_into_sandbox_via_chunk_sink_back_pressure() {
    // wsa-18 + arch-pre-r1-9 — back-pressure invariant: in lossless
    // mode (the default), upstream producer blocks when downstream
    // consumer is slow; no chunks are dropped.
    //
    // Drive the lossless stream helper at a known schedule: producer
    // pauses very briefly per chunk; consumer pauses longer per chunk.
    // The producer hits the bounded-mpsc full state and applies
    // back-pressure (waits) instead of dropping. Total chunks
    // consumed equals total chunks produced — zero loss.
    let chunk_count = 50;
    let cap = NonZeroUsize::new(4).unwrap();
    let producer_pause: Vec<u64> = vec![0; chunk_count];
    let consumer_pause: Vec<u64> = vec![100; chunk_count]; // microseconds

    let outcome =
        testing_run_lossless_stream_with_schedule(chunk_count, cap, producer_pause, consumer_pause);

    // Lossless invariant: every produced seq appears in the
    // received_seqs trace; ZERO drops under back-pressure.
    assert_eq!(
        outcome.received_seqs.len(),
        chunk_count,
        "lossless stream MUST deliver every produced chunk; \
         received_seqs.len = {} expected = {}",
        outcome.received_seqs.len(),
        chunk_count
    );
    let expected: Vec<u64> = (0..chunk_count as u64).collect();
    assert_eq!(
        outcome.received_seqs, expected,
        "lossless stream MUST preserve seq order + zero drops"
    );

    // Companion smoke check: a fresh ChunkSink construction still
    // succeeds (the sink shape is the consumed contract a SANDBOX-
    // side `chunk_emit` host-fn would write into).
    let (_sink, _src) = testing_make_chunk_sink(cap);
}
