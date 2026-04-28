#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! `prop_algorithm_b_incremental_equals_rebuild` (G8-A — 10k cases).
//!
//! For any sequence of `ChangeEvent`s, an Algorithm B view that has run a
//! `rebuild()` (which delegates to the inner hand-written view's
//! state-clearing rebuild — see `algorithm_b.rs::rebuild`) and then
//! re-ingested the events MUST yield the same observable state as a
//! straight incremental application of the same events. If the
//! rebuild+replay path diverges from the incremental path, B is silently
//! lossy and Phase-3 sync breaks.
//!
//! ## Why this shape (cr-g8a-mr-1 fix-pass)
//!
//! The original draft of this test fed identical events into two views and
//! called a no-op `rebuild()` on one — the assertion was vacuously true.
//! This rewrite makes `rebuild()` actually do something (delegate to the
//! inner view's state-clearing rebuild; cr-g8a-mr-2 fix) and then
//! genuinely exercises the round-trip:
//!
//! 1. `incremental` view: feed events once via `update()`.
//! 2. `rebuilt` view: feed events, call `rebuild()` (clears state via
//!    inner.rebuild), then feed the same events again via `update()`.
//! 3. Assert `incremental.read(q) == rebuilt.read(q)` on the actual
//!    observable content.
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §3 row
//! `prop_algorithm_b_incremental_equals_rebuild`.
//!
//! 10k cases per workspace convention (see `benten-eval/tests/proptest_*.rs`).
//! The R2 landscape entry called for 100k but the workspace-wide proptest
//! norm is 10k cases at 0..64 event vectors — at 100k we trip the nextest
//! 180s slow-test timeout. With the rebuild-then-replay path the per-case
//! cost roughly doubles vs the prior vacuous shape; cr-g8a-mr-9 flagged a
//! re-evaluation. 10k still fits the 180s slow-test budget for a 32-event
//! average vector (~640k discrete update applications per run).

#![allow(clippy::unwrap_used)]

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
// Renamed to avoid clash with `proptest::strategy::Strategy` trait imported
// via `proptest::prelude::*`.
use benten_ivm::Strategy as IvmStrategy;
use benten_ivm::algorithm_b::AlgorithmBView;
use benten_ivm::views::ContentListingView;
use benten_ivm::{View, ViewQuery, ViewResult};
use proptest::prelude::*;
use std::collections::BTreeMap;

fn arb_change_kind() -> impl Strategy<Value = ChangeKind> {
    prop_oneof![
        Just(ChangeKind::Created),
        Just(ChangeKind::Updated),
        Just(ChangeKind::Deleted),
    ]
}

fn arb_label() -> impl Strategy<Value = String> {
    // A small finite alphabet — biases pattern-match hits over random
    // strings the views will universally reject.
    prop_oneof![
        Just("post".to_string()),
        Just("comment".to_string()),
        Just("user".to_string()),
        Just("Handler".to_string()),
    ]
}

fn arb_node() -> impl Strategy<Value = Node> {
    (arb_label(), 0i64..10_000, "[a-z]{1,8}").prop_map(|(label, created_at, title)| {
        let mut props = BTreeMap::new();
        props.insert("title".into(), Value::Text(title));
        props.insert("createdAt".into(), Value::Int(created_at));
        Node::new(vec![label], props)
    })
}

fn arb_change_event() -> impl Strategy<Value = ChangeEvent> {
    (arb_node(), arb_change_kind(), 1u64..1_000_000).prop_map(|(node, kind, tx)| {
        let labels = node.labels.clone();
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            labels,
            kind,
            tx,
            Some(node),
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        // Source of variability is bounded by `arb_change_event`; default
        // shrinking is fine.
        ..ProptestConfig::default()
    })]

    /// For any change-event sequence, Algorithm B's incremental snapshot
    /// equals a rebuild-then-replay of the same sequence.
    #[test]
    fn prop_algorithm_b_incremental_equals_rebuild(
        events in proptest::collection::vec(arb_change_event(), 0..64)
    ) {
        // Path 1 — incremental: feed events once, no rebuild.
        let mut incremental: Box<dyn View> = Box::new(
            AlgorithmBView::for_id("content_listing", ContentListingView::definition())
                .expect("content_listing is a known view id"),
        );
        for ev in &events {
            let _ = incremental.update(ev);
        }

        // Path 2 — rebuild-then-replay. After cr-g8a-mr-2 the wrapper's
        // `rebuild()` delegates to the inner hand-written view's
        // `rebuild()`, which CLEARS state + resets the budget tracker
        // (see e.g. `content_listing.rs::rebuild_from_scratch`). The
        // post-rebuild view is empty; we re-feed the same events and assert
        // it converges to the same state as the incremental path. This is
        // the genuine `incremental_equals_rebuild` invariant — a vacuous
        // version of this test (no rebuild + identical inputs) was the
        // cr-g8a-mr-1 finding.
        let mut rebuilt: Box<dyn View> = Box::new(
            AlgorithmBView::for_id("content_listing", ContentListingView::definition())
                .expect("content_listing is a known view id"),
        );
        for ev in &events {
            let _ = rebuilt.update(ev);
        }
        rebuilt.rebuild().expect("rebuild on Algorithm B view must succeed");
        // Post-rebuild the inner is empty; replay the events. State after
        // this loop must match the incremental path (Path 1).
        for ev in &events {
            let _ = rebuilt.update(ev);
        }

        // R4-FP-A — Snapshots must match on ACTUAL CONTENT, not just id +
        // is_stale (the prior projection was tautological because both views
        // share the same id by construction; per rust-test-reviewer.json
        // tq-2b-1). Project via `read(&ViewQuery { label: "post", limit: 100 })`
        // — both views must report the same paginated CID set after the
        // event sequence, including identical Err(PatternMismatch) shapes
        // for queries the view can't serve.
        let q = ViewQuery {
            label: Some("post".into()),
            limit: Some(100),
            offset: Some(0),
            ..ViewQuery::default()
        };
        let inc_result: Result<ViewResult, _> = incremental.read(&q);
        let reb_result: Result<ViewResult, _> = rebuilt.read(&q);
        prop_assert_eq!(format!("{inc_result:?}"), format!("{reb_result:?}"));
        // Liveness pin: both views must report stale-state identically (an
        // Algorithm B that silently flips to stale on rebuild would diverge
        // here while the read snapshots could still match by coincidence).
        prop_assert_eq!(incremental.is_stale(), rebuilt.is_stale());
        prop_assert_eq!(incremental.strategy(), IvmStrategy::B);
        prop_assert_eq!(rebuilt.strategy(), IvmStrategy::B);
    }
}
