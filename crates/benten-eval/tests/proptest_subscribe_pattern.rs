#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE pattern + dedup proptests (G6-A).
//!
//! Pin source: streaming-systems must_pass —
//!   `prop_subscribe_pattern_no_false_positives` (10k)
//!   `prop_subscribe_seq_dedup_at_handler_idempotent_under_replay` (10k) (D5)
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

use benten_eval::testing::{testing_run_pattern_proptest, testing_run_replay_dedup_proptest};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    /// Random pattern × random WRITE; assert NO event delivered for
    /// non-matching anchor.
    #[test]
    fn prop_subscribe_pattern_no_false_positives(
        pattern_glob in "[a-z/]{1,12}",
        anchor_label in "[a-z/]{1,16}",
    ) {
        let outcome = testing_run_pattern_proptest(&pattern_glob, &anchor_label);
        if !outcome.expected_match {
            prop_assert_eq!(
                outcome.delivered_count, 0,
                "non-matching anchor must never deliver"
            );
        }
    }

    /// Replay event N times; handler observes deliver-once-then-drop
    /// semantics at the handler boundary. D5 + stream-d5-1.
    #[test]
    fn prop_subscribe_seq_dedup_at_handler_idempotent_under_replay(
        seq in 0u64..1024,
        replay_count in 1usize..16,
    ) {
        let outcome = testing_run_replay_dedup_proptest(seq, replay_count);
        prop_assert_eq!(
            outcome.handler_invocation_count, 1,
            "handler-boundary dedup: deliver-once regardless of replay count"
        );
    }
}
