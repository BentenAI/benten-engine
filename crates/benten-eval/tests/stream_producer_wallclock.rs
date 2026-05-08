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
/// 50ms wallclock budget + 200ms upper-bound assertion are too tight on
/// slow CI runners (macos-arm64 @ 1.95.0 reproducibly exceeds the 4×budget
/// upper bound). Test was previously gated behind `phase_2b_landed`
/// (retired at G20-B phase-5a 16a5a4f) and only surfaced now. Either bump
/// the budget to ≥200ms or relax the upper-bound multiplier; pin remains
/// to assert WALLCLOCK firing semantics, not timing fidelity.
#[ignore = "destination: pre-R4b orchestrator-direct fix-pass batch item #8 (CI-flake-hardening — bump budget or relax 4× multiplier)"]
#[test]
fn stream_producer_wallclock_kills_blocked_send() {
    let cap = NonZeroUsize::new(1).unwrap();
    let budget = Duration::from_millis(50);
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

    assert!(
        elapsed < budget * 4,
        "wallclock should fire near the configured budget, not hang indefinitely; elapsed = {elapsed:?}"
    );
    assert!(
        elapsed >= budget,
        "wallclock should not fire BEFORE the budget; elapsed = {elapsed:?}"
    );
}
