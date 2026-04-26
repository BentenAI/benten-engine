//! Phase 2b R3-B — SANDBOX output strictly-bounded property test (G7-A).
//!
//! Property: for any sequence of write batches, `consumed` never
//! exceeds `limit`. The CountedSink primary path traps before
//! crossing the boundary.
//!
//! Pin source: plan §4 + wsa-D17.
//! Iterations: 10k (per R2 §3).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// `prop_sandbox_output_size_strictly_bounded` — consumed <= limit
    /// for any write-batch schedule.
    ///
    /// Strategy:
    ///   - Random output_max_bytes limit in [1, 1_000_000].
    ///   - Random write-batch sizes (Vec<usize>) summing to potentially
    ///     more than the limit.
    ///   - Replay the writes through a real CountedSink; collect the
    ///     observed `consumed` value AT THE END.
    ///   - Assert: consumed <= limit always.
    ///
    /// The test also pins that EITHER:
    ///   (a) all writes fit (sum of batch sizes <= limit) → all
    ///       writes succeeded + consumed == sum;
    ///   (b) some write would have crossed the boundary → that write
    ///       trapped + consumed reflects only the writes that
    ///       succeeded BEFORE the trap (deterministic boundary).
    ///
    /// Defends against arithmetic-overflow attacks on the AtomicU64
    /// counter (e.g., a write of usize::MAX bytes that would
    /// silently wrap consumed to 0).
    #[test]
    #[ignore = "Phase 2b G7-A pending — output bounded property"]
    fn prop_sandbox_output_size_strictly_bounded(
        limit in 1u64..1_000_000u64,
        batches in proptest::collection::vec(0usize..200_000, 1..50),
    ) {
        // R5 G7-A pseudo:
        //   let mut sink = CountedSink::new(SandboxOutputBudget {
        //       consumed: AtomicU64::new(0),
        //       limit,
        //   });
        //   let mut consumed = 0u64;
        //   let mut tripped = false;
        //   for n in &batches {
        //       let bytes = vec![0u8; *n];
        //       match sink.write(&bytes) {
        //           Ok(()) => consumed = consumed.saturating_add(*n as u64),
        //           Err(Inv7Trap { .. }) => { tripped = true; break; }
        //       }
        //   }
        //   prop_assert!(consumed <= limit);
        //   if !tripped {
        //       prop_assert_eq!(consumed, batches.iter().sum::<usize>() as u64);
        //   }
        let _ = (limit, batches);
        // R4-FP-A — `prop_assume!(false)` DISCARDS the case (silent
        // vacuous-pass after un-ignore); `prop_assert!(false, ...)`
        // actually fails the case, preserving fail-fast intent
        // (rust-test-reviewer.json tq-2b-3).
        prop_assert!(
            false,
            "Phase 2b G7-A pending: write output-strictly-bounded property \
             body (replace this prop_assert!(false) with the consumed-vs-limit \
             assertion described in the file pseudo)."
        );
    }
}
