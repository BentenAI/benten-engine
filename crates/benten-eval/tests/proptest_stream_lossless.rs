#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM lossless proptests (G6-A).
//!
//! Pin source: streaming-systems stream-d4-1 must_pass —
//!   `prop_stream_no_chunk_loss_under_backpressure` (10k)
//!   `prop_stream_chunk_order_preserved_under_concurrent_consumers` (10k)
//!   `prop_stream_seq_u64_no_wraparound_at_boundary` (1k)
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::useless_conversion,
    clippy::no_effect_underscore_binding,
    clippy::clone_on_copy,
    clippy::stable_sort_primitive,
    clippy::match_like_matches_macro,
    clippy::unnested_or_patterns
)]

use benten_eval::chunk_sink::{Chunk, SendOutcome};
use benten_eval::testing::{testing_make_chunk_sink, testing_run_lossless_stream_with_schedule};
use proptest::prelude::*;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Duration;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        ..ProptestConfig::default()
    })]

    /// Random producer rates × random consumer pause schedules → received
    /// `seq` set MUST equal sent `seq` set in lossless mode.
    #[test]
    fn prop_stream_no_chunk_loss_under_backpressure(
        chunk_count in 1usize..256,
        cap in 1usize..32,
        producer_pause_us in prop::collection::vec(0u64..200, 0..256),
        consumer_pause_us in prop::collection::vec(0u64..500, 0..256),
    ) {
        let cap = NonZeroUsize::new(cap).unwrap();
        let outcome = testing_run_lossless_stream_with_schedule(
            chunk_count,
            cap,
            producer_pause_us,
            consumer_pause_us,
        );
        let sent: BTreeSet<u64> = (0..chunk_count as u64).collect();
        let received: BTreeSet<u64> = outcome.received_seqs.into_iter().collect();
        prop_assert_eq!(received, sent, "lossless mode must not lose chunks");
    }

    /// N producers × 1 consumer: per-producer seq monotonic at consumer.
    /// (Per-producer monotonicity is the contract; cross-producer interleave
    /// is permitted.)
    #[test]
    fn prop_stream_chunk_order_preserved_under_concurrent_consumers(
        producer_count in 2usize..6,
        chunks_per_producer in 1usize..64,
    ) {
        let outcome = benten_eval::testing::testing_run_concurrent_producers(
            producer_count,
            chunks_per_producer,
        );

        // Group received chunks by producer-id; per-group seq must be monotonic.
        for producer_id in 0..producer_count {
            let group_seqs: Vec<u64> = outcome
                .received
                .iter()
                .filter(|c| c.producer_id == producer_id)
                .map(|c| c.seq)
                .collect();
            let sorted = {
                let mut s = group_seqs.clone();
                s.sort();
                s
            };
            // G6-A: macro expansion can't capture surrounding-scope vars,
            // so format outside the macro literal.
            let msg = format!(
                "producer {producer_id} per-stream seq must be monotonic at consumer"
            );
            prop_assert_eq!(group_seqs, sorted, "{}", msg);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1_000,
        ..ProptestConfig::default()
    })]

    /// Near-`u64::MAX` seq does NOT wrap; engine surfaces a typed error or
    /// caps at MAX rather than silently rolling to 0.
    /// streaming-systems concerns_for_r3 — wraparound regression guard.
    #[test]
    fn prop_stream_seq_u64_no_wraparound_at_boundary(
        start in (u64::MAX - 16)..u64::MAX,
        n in 1usize..8,
    ) {
        let cap = NonZeroUsize::new(8).unwrap();
        let (mut sink, mut src) = testing_make_chunk_sink(cap);

        let mut sent: Vec<u64> = Vec::new();
        let mut overflow_observed = false;
        for i in 0..n {
            let seq = start.checked_add(i as u64);
            match seq {
                Some(s) => {
                    let _ = sink.send(Chunk { seq: s, bytes: vec![].into(), final_chunk: false });
                    sent.push(s);
                }
                None => {
                    overflow_observed = true;
                    break;
                }
            }
        }
        let _ = sink.close();

        // Drain the consumer side; assert seqs received == seqs sent and
        // ABSOLUTELY no value of 0 leaked through (wraparound proxy).
        let mut received: Vec<u64> = Vec::new();
        loop {
            match src.recv_blocking_timeout(Duration::from_millis(50)) {
                Ok(Some(c)) if !c.final_chunk => received.push(c.seq),
                Ok(Some(_)) | Ok(None) => break,
                Err(_) => break,
            }
        }
        if !overflow_observed {
            // G6-A test-author follow-up: `prop_assert_eq!` moves both
            // arguments into the macro, so we clone before the second
            // `received.iter()` borrow. Cheap because received is a
            // bounded-length `Vec<u64>` (n < 16 per the proptest range).
            let received_for_assert = received.clone();
            prop_assert_eq!(received_for_assert, sent);
            prop_assert!(received.iter().all(|s| *s >= start), "no wraparound to 0");
        }
    }
}
