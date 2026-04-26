#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE ordering — within-key strict, cross-key
//! unordered (G6-A).
//!
//! Pin source: D5-RESOLVED — within-key strict ordering, cross-key
//! unordered (matches actual graph-write concurrency model).
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_subscribe_inject_event, testing_subscribe_register,
};
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Within-key (per anchor CID) ordering is STRICT. Concurrent writes to the
/// same anchor MUST be delivered in commit order at every subscriber.
#[test]
#[ignore = "Phase 2b G6-A pending — D5 within-key strict"]
fn subscribe_within_key_ordering_strict() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/ordering/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(64).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");

    let anchor = benten_core::Cid::sample_for_test();
    for i in 0..32u64 {
        let mut e = testing_make_change_event(
            anchor.clone(),
            ChangeKind::Updated,
            serde_json::json!({"v": i}),
        );
        e.seq = i;
        testing_subscribe_inject_event(&sub, e).unwrap();
    }

    let received: Vec<u64> = sub.drain_blocking(std::time::Duration::from_millis(100));
    let same_anchor_seqs: Vec<u64> = received.into_iter().collect();
    assert_eq!(
        same_anchor_seqs,
        (0..32u64).collect::<Vec<_>>(),
        "within-key delivery must be strict commit-order"
    );
}

/// Cross-key ordering is UNORDERED — events on different anchors may
/// interleave arbitrarily. Test only asserts that ALL events arrive,
/// NOT that they arrive in any particular cross-anchor order.
#[test]
#[ignore = "Phase 2b G6-A pending — D5 cross-key unordered"]
fn subscribe_cross_key_ordering_unordered_documented() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/cross/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(128).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");

    let anchors: Vec<_> = (0..4)
        .map(|_| benten_core::Cid::sample_for_test())
        .collect();
    let mut expected: HashMap<benten_core::Cid, Vec<u64>> = HashMap::new();
    let mut next_seq: u64 = 0;
    for round in 0..16u64 {
        for anchor in &anchors {
            let mut e = testing_make_change_event(
                anchor.clone(),
                ChangeKind::Updated,
                serde_json::json!({"round": round}),
            );
            e.seq = next_seq;
            next_seq += 1;
            expected.entry(anchor.clone()).or_default().push(e.seq);
            testing_subscribe_inject_event(&sub, e).unwrap();
        }
    }

    let events = sub.drain_events_blocking(std::time::Duration::from_millis(200));
    assert_eq!(
        events.len(),
        expected.values().map(|v| v.len()).sum::<usize>()
    );

    // Per-anchor ordering preserved (within-key); we DO NOT assert any
    // particular cross-anchor ordering. The point is that the documented
    // unordered semantics are honored — no spurious cross-anchor ordering
    // assertion creeps in.
    let mut per_anchor: HashMap<benten_core::Cid, Vec<u64>> = HashMap::new();
    for ev in &events {
        per_anchor
            .entry(ev.anchor_cid.clone())
            .or_default()
            .push(ev.seq);
    }
    for (anchor, seqs) in &per_anchor {
        let exp = expected.get(anchor).unwrap();
        assert_eq!(seqs, exp, "within-key still strict for anchor {anchor:?}");
    }
}
