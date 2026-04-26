#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM lossy opt-in mode (G6-A).
//!
//! Pin source: D4-RESOLVED — `try_send` + `lossy_mode = true` emits
//! `E_STREAM_BACKPRESSURE_DROPPED` in the trace per dropped chunk.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::chunk_sink::{Chunk, ChunkSinkError, SendOutcome};
use benten_eval::testing::testing_make_chunk_sink_lossy;
use std::num::NonZeroUsize;

/// Lossy mode + saturated sink → `try_send` returns `BackpressureDropped`
/// AND the trace records `E_STREAM_BACKPRESSURE_DROPPED` per dropped chunk
/// (loud-by-default per benten-philosophy concern).
#[test]
#[ignore = "Phase 2b G6-A pending"]
fn stream_lossy_mode_try_send_emits_e_stream_backpressure_dropped_in_trace() {
    let cap = NonZeroUsize::new(2).unwrap();
    let (mut sink, _src) = testing_make_chunk_sink_lossy(cap);

    // Saturate the buffer.
    for i in 0..2u64 {
        let outcome = sink
            .try_send(Chunk {
                seq: i,
                bytes: vec![i as u8].into(),
                final_chunk: false,
            })
            .expect("first chunks accepted");
        assert!(matches!(
            outcome,
            SendOutcome::Accepted | SendOutcome::BackpressureCredit(_)
        ));
    }

    // Next try_send hits a full buffer in lossy mode → typed dropped error.
    let dropped = sink
        .try_send(Chunk {
            seq: 2,
            bytes: vec![2].into(),
            final_chunk: false,
        })
        .expect_err("lossy mode: try_send on full buffer returns typed-dropped error");
    assert!(matches!(
        dropped,
        ChunkSinkError::BackpressureDropped { .. }
    ));
    assert_eq!(dropped.error_code(), ErrorCode::StreamBackpressureDropped);

    // Trace must surface the drop loudly — observable to the consumer's
    // tracing pipeline. The exact mechanism (TraceStep::StreamChunk with
    // dropped=true vs a sibling variant) is owned by G12-F; this test
    // pins that drops are NOT silent at the trace surface.
    let trace_entries = sink.drain_trace();
    let dropped_count = trace_entries
        .iter()
        .filter(|e| e.is_backpressure_dropped())
        .count();
    assert!(
        dropped_count >= 1,
        "lossy drop must surface as a trace entry, not silently disappear"
    );
}
