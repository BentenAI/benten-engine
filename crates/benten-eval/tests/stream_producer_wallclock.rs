#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
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
