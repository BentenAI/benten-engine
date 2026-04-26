#![cfg(feature = "phase_2b_landed")] // R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE ordering + no-event-loss proptests (G6-A).
//!
//! Pin source: streaming-systems stream-d5-1 must_pass —
//!   `prop_subscribe_event_ordering` (10k)
//!   `prop_subscribe_no_event_loss_under_concurrent_writes` (10k)
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::testing::{
    testing_run_concurrent_subscribe_event_ordering,
    testing_run_concurrent_subscribe_no_event_loss,
};
use proptest::prelude::*;
use std::collections::BTreeSet;

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    /// Random concurrent writes across N anchors × M subscribers → per-anchor
    /// ordering at every subscriber. D5 within-key strict.
    #[test]
    #[ignore = "Phase 2b G6-A pending — D5 per-anchor ordering"]
    fn prop_subscribe_event_ordering(
        anchor_count in 1usize..6,
        subscriber_count in 1usize..4,
        writes_per_anchor in 1usize..32,
    ) {
        let outcome = testing_run_concurrent_subscribe_event_ordering(
            anchor_count,
            subscriber_count,
            writes_per_anchor,
        );

        for sub in &outcome.subscribers {
            for anchor_idx in 0..anchor_count {
                let per_anchor: Vec<u64> = sub.received
                    .iter()
                    .filter(|e| e.anchor_index == anchor_idx)
                    .map(|e| e.seq)
                    .collect();
                let mut sorted = per_anchor.clone();
                sorted.sort();
                prop_assert_eq!(
                    per_anchor, sorted,
                    "subscriber {} anchor {} per-anchor ordering must be commit-order",
                    sub.id, anchor_idx
                );
            }
        }
    }

    /// N concurrent writers × 1 subscriber → received ⊇ committed (duplicates
    /// allowed, since handler-boundary dedup is internal). At-least-once
    /// internal (D5).
    #[test]
    #[ignore = "Phase 2b G6-A pending — D5 at-least-once internal"]
    fn prop_subscribe_no_event_loss_under_concurrent_writes(
        writer_count in 2usize..6,
        writes_per_writer in 1usize..32,
    ) {
        let outcome = testing_run_concurrent_subscribe_no_event_loss(
            writer_count,
            writes_per_writer,
        );
        let committed: BTreeSet<u64> = outcome.committed_seqs.into_iter().collect();
        let received: BTreeSet<u64> = outcome.received_seqs.into_iter().collect();
        prop_assert!(
            received.is_superset(&committed),
            "subscriber received MUST include every committed write (at-least-once internal)"
        );
    }
}
