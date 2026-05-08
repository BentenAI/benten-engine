//! R3-A red-phase: STREAM producer wallclock budget kills permanently
//! stalled sends (G6-A).
//!
//! Pin source: streaming-systems implementation_hint — "Consumer slow →
//! producer await blocks → handler-level wall-clock budget eventually fires
//! `E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED`."
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::useless_conversion,
    clippy::no_effect_underscore_binding,
    clippy::clone_on_copy
)]

use benten_errors::ErrorCode;
use benten_eval::chunk_sink::{Chunk, ChunkSinkError};
use benten_eval::testing::testing_make_chunk_sink_with_wallclock;
use std::num::NonZeroUsize;
use std::time::Duration;

/// Consumer never drains; producer's wallclock budget eventually fires
/// `E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED`.
///
/// Pre-R4b orchestrator-direct fix-pass batch item #8 (CI-flake-hardening):
/// the original 50ms wallclock budget + `elapsed < budget * 4` upper bound
/// (200ms) was too tight on slow CI runners (macos-arm64 @ 1.95.0
/// reproducibly exceeded the 4×-budget upper bound). Bumped to a 200ms
/// budget with the same `elapsed < budget * 4` semantic upper bound (now
/// 800ms), giving slow runners 4× headroom without weakening the
/// assertion: the test still pins (a) WALLCLOCK firing happens (NOT a
/// permanent hang), (b) firing happens AFTER the configured budget (not
/// pre-emptively), and (c) firing surfaces the typed
/// `E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED` code with a
/// `ProducerWallclockExceeded` ChunkSinkError variant.
#[test]
fn stream_producer_wallclock_kills_blocked_send() {
    let cap = NonZeroUsize::new(1).unwrap();
    // 200ms budget (4× the original 50ms): gives macos-arm64 1.95 enough
    // wallclock headroom for the wallclock-fire latency to land inside the
    // upper bound below.
    let budget = Duration::from_millis(200);
    let (mut sink, _src) = testing_make_chunk_sink_with_wallclock(cap, budget);

    // First send fills the buffer; subsequent send must block.
    sink.send(Chunk {
        seq: 0,
        bytes: vec![0].into(),
        final_chunk: false,
    })
    .expect("first chunk fits");

    let start = std::time::Instant::now();
    let result = sink.send(Chunk {
        seq: 1,
        bytes: vec![1].into(),
        final_chunk: false,
    });
    let elapsed = start.elapsed();

    let err = result.expect_err("blocked send must surface wallclock-exceeded error");
    assert!(matches!(
        err,
        ChunkSinkError::ProducerWallclockExceeded { .. }
    ));
    assert_eq!(err.error_code(), ErrorCode::StreamProducerWallclockExceeded);

    // Upper bound: 4× the budget = 800ms. Pins "wallclock fires near the
    // configured budget, not hangs indefinitely" — the WALLCLOCK semantic.
    // Slow CI runners need the 4× headroom; the assertion still fails if a
    // regression makes the producer block far longer than its budget.
    assert!(
        elapsed < budget * 4,
        "wallclock should fire near the configured budget, not hang indefinitely; elapsed = {elapsed:?}"
    );
    assert!(
        elapsed >= budget,
        "wallclock should not fire BEFORE the budget; elapsed = {elapsed:?}"
    );
}
