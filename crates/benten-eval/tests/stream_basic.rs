#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM basic chunk-sequence + close-propagation tests
//! (G6-A).
//!
//! Pin source: streaming-systems stream-d4-1 must_pass list.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::useless_conversion,
    clippy::no_effect_underscore_binding,
    clippy::clone_on_copy
)]

use benten_errors::ErrorCode;
use benten_eval::chunk_sink::{Chunk, ChunkSinkError, SendOutcome};
use benten_eval::testing::testing_make_chunk_sink;
use std::num::NonZeroUsize;

/// Producer emits N chunks in order; consumer drains in identical seq order.
#[test]
fn stream_chunk_sequence_preserves_order() {
    let cap = NonZeroUsize::new(8).unwrap();
    let (mut sink, mut src) = testing_make_chunk_sink(cap);

    // Producer thread so the lossless `send` can block on capacity without
    // deadlocking the test (consumer also runs on the test thread, drains
    // below). With cap=8 and 16 sends, a single-thread sequence would
    // dead-stop at the 9th send; the dedicated producer thread lets the
    // consumer drain in lock-step.
    let producer = std::thread::spawn(move || {
        for i in 0..16u64 {
            let outcome = sink.send(Chunk {
                seq: i,
                bytes: vec![i as u8].into(),
                final_chunk: false,
            });
            assert!(matches!(
                outcome,
                Ok(SendOutcome::Accepted | SendOutcome::BackpressureCredit(_))
            ));
        }
        let _ = sink.close();
    });

    let mut received: Vec<u64> = Vec::new();
    while let Some(chunk) = src.recv_blocking().expect("recv") {
        if chunk.final_chunk {
            break;
        }
        received.push(chunk.seq);
        if received.len() == 16 {
            break;
        }
    }
    producer.join().expect("producer panicked");
    assert_eq!(received, (0..16u64).collect::<Vec<_>>());
}

/// Producer-side close propagates to consumer as `final_chunk: true` then
/// EOF.
#[test]
fn stream_close_propagates() {
    let cap = NonZeroUsize::new(4).unwrap();
    let (mut sink, mut src) = testing_make_chunk_sink(cap);

    sink.send(Chunk {
        seq: 0,
        bytes: vec![0xAA].into(),
        final_chunk: false,
    })
    .unwrap();
    sink.close().expect("close idempotent");

    let first = src.try_recv().expect("recv").expect("first chunk");
    assert_eq!(first.seq, 0);
    let last = src.try_recv().expect("recv").expect("close-marker chunk");
    assert!(last.final_chunk, "close emits final_chunk: true marker");
    let eof = src.try_recv().expect("recv");
    assert!(eof.is_none(), "after close-marker, recv returns EOF");
}

/// Consumer drops mid-stream; producer's next send surfaces typed error.
#[test]
fn stream_consumer_drop_surfaces_e_stream_closed_by_peer() {
    let cap = NonZeroUsize::new(4).unwrap();
    let (mut sink, src) = testing_make_chunk_sink(cap);
    drop(src); // simulate napi consumer disconnect

    let result = sink.send(Chunk {
        seq: 0,
        bytes: vec![0xFF].into(),
        final_chunk: false,
    });

    let err = result.expect_err("producer must surface typed error after consumer drop");
    assert!(matches!(err, ChunkSinkError::ClosedByPeer { .. }));
    let code: ErrorCode = err.error_code();
    assert_eq!(code, ErrorCode::StreamClosedByPeer);
}
