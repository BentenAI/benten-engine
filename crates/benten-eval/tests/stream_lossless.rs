#![cfg(feature = "phase_2b_landed")] // R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM lossless-default mode (G6-A).
//!
//! Pin source: D4-RESOLVED — default mode never drops chunks even under
//! adversarial consumer-pause schedules.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::chunk_sink::{Chunk, SendOutcome};
use benten_eval::testing::testing_make_chunk_sink;
use std::num::NonZeroUsize;
use std::time::Duration;

/// Default mode (lossless) NEVER drops chunks, even under back-pressure
/// (slow / paused consumer). Producer awaits available capacity.
#[test]
#[ignore = "Phase 2b G6-A pending"]
fn stream_default_mode_never_drops() {
    let cap = NonZeroUsize::new(4).unwrap();
    let (mut sink, mut src) = testing_make_chunk_sink(cap);

    // Producer thread emits 100 chunks; consumer thread sleeps 1ms between
    // recvs (chronic slow consumer). Lossless mode: producer awaits when
    // full; total received == total sent.
    let producer = std::thread::spawn(move || {
        for i in 0..100u64 {
            loop {
                let outcome = sink.send(Chunk {
                    seq: i,
                    bytes: vec![(i & 0xff) as u8].into(),
                    final_chunk: false,
                });
                match outcome {
                    Ok(SendOutcome::Accepted) => break,
                    Ok(SendOutcome::BackpressureCredit(_)) => {
                        // Lossless mode: retry until accepted; never drop.
                        std::thread::sleep(Duration::from_micros(100));
                    }
                    Ok(SendOutcome::Closed) => panic!("unexpected closed"),
                    Err(e) => panic!("unexpected send error: {e:?}"),
                }
            }
        }
        sink.close().unwrap();
    });

    let mut received: Vec<u64> = Vec::new();
    loop {
        match src.recv_blocking() {
            Ok(Some(c)) => {
                if c.final_chunk {
                    break;
                }
                received.push(c.seq);
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(None) => break,
            Err(e) => panic!("recv error: {e:?}"),
        }
    }
    producer.join().unwrap();

    let sent: Vec<u64> = (0..100u64).collect();
    assert_eq!(received, sent, "lossless default — every chunk delivered");
}
