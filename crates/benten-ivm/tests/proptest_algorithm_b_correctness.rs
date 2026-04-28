#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! `prop_algorithm_b_incremental_equals_rebuild` (G8-A — 100k cases).
//!
//! For any sequence of `ChangeEvent`s, Algorithm B's incremental snapshot
//! must equal a full rebuild from the same event sequence. This is the
//! core correctness invariant that justifies running B in production at
//! all — if incremental ≠ rebuild, B is silently lossy and Phase-3 sync
//! breaks.
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §3 row
//! `prop_algorithm_b_incremental_equals_rebuild`.
//!
//! 10k cases per workspace convention (see `benten-eval/tests/proptest_*.rs`).
//! The R2 landscape entry called for 100k but the workspace-wide proptest
//! norm is 10k cases at 0..64 event vectors — at 100k we trip the nextest
//! 180s slow-test timeout. Per-event work in `update` + `read` is bounded;
//! 10k iterations × ~32 average events still gives ~320k discrete event
//! applications per run, which exercises the cancellation-shape surface
//! the landscape called out.

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
    /// equals a full rebuild from the same sequence.
    #[test]
    fn prop_algorithm_b_incremental_equals_rebuild(
        events in proptest::collection::vec(arb_change_event(), 0..64)
    ) {
        // Path 1 — incremental: feed events one at a time, no rebuild.
        let mut incremental: Box<dyn View> = Box::new(AlgorithmBView::for_id(
            "content_listing",
            ContentListingView::definition(),
        ));
        for ev in &events {
            let _ = incremental.update(ev);
        }

        // Path 2 — full rebuild: feed events into a fresh view, then rebuild.
        let mut rebuilt: Box<dyn View> = Box::new(AlgorithmBView::for_id(
            "content_listing",
            ContentListingView::definition(),
        ));
        for ev in &events {
            let _ = rebuilt.update(ev);
        }
        rebuilt.rebuild().expect("rebuild on Algorithm B view must succeed");

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
