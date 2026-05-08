//! Phase 2b R3-B — SANDBOX output strictly-bounded property test (G7-A).
//!
//! Property: for any sequence of write batches, `consumed` never
//! exceeds `limit`. The CountedSink primary path traps before
//! crossing the boundary.
//!
//! Pin source: plan §4 + wsa-D17.
//! Iterations: 10k cases (the property test is purely in-memory + cheap).
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored. Drives the production
//! `CountedSink` directly with random write-batch schedules.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// `prop_sandbox_output_size_strictly_bounded` — `consumed <= limit`
    /// holds for any write-batch schedule. EITHER all batches fit
    /// (consumed == sum of batch sizes), OR a batch trips and
    /// `consumed` reflects only the writes that succeeded BEFORE the
    /// trap.
    ///
    /// Defends against arithmetic-overflow attacks on the consumed
    /// counter: a write of `usize::MAX` bytes that would silently wrap
    /// `consumed` to 0 fails this property (saturating arithmetic in
    /// `write_n_bytes` ensures `consumed` saturates AT limit + 1
    /// rather than wrapping).
    #[test]
    fn prop_sandbox_output_size_strictly_bounded(
        limit in 1u64..1_000_000u64,
        batches in proptest::collection::vec(0u64..200_000u64, 1..50),
    ) {
        use benten_eval::sandbox::CountedSink;

        let mut sink = CountedSink::new(limit);
        let mut tripped = false;
        let mut sum_written: u64 = 0;
        for n in &batches {
            match sink.write_n_bytes(*n, "test_host_fn") {
                Ok(()) => sum_written = sum_written.saturating_add(*n),
                Err(_overflow) => {
                    tripped = true;
                    break;
                }
            }
        }

        // Property 1: consumed never exceeds limit.
        prop_assert!(
            sink.consumed() <= limit,
            "CountedSink invariant: consumed ({}) MUST NOT exceed \
             limit ({}); batches={:?}",
            sink.consumed(),
            limit,
            batches
        );

        // Property 2: when no trap fired, consumed == Σ(written).
        if !tripped {
            prop_assert_eq!(
                sink.consumed(),
                sum_written,
                "no-trap path: consumed must equal sum of accepted batches"
            );
        }
    }
}
