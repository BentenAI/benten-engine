//! View 3 — Content listing (I5, exit-criterion load-bearing — R2 landscape
//! §2.3 rows 6-7).
//!
//! On `post`-labeled node create/update/delete, maintain sorted-by-`createdAt`
//! list; paginated reads are O(log n + page_size). This is the view
//! `crud('post').list` consumes for Exit Criterion #2.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::testing::canonical_test_node;
use benten_core::{Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::ContentListingView;
use benten_ivm::{View, ViewQuery, ViewResult};
use proptest::prelude::*;

fn post_created() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        labels: vec!["Post".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    }
}

#[test]
fn content_listing_all_returned_after_three_writes() {
    let mut v = ContentListingView::new("Post");
    v.update(&post_created()).unwrap();
    v.update(&post_created()).unwrap();
    v.update(&post_created()).unwrap();
    let q = ViewQuery {
        label: Some("Post".to_string()),
        limit: Some(100),
        offset: Some(0),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    match r {
        ViewResult::Cids(cids) => {
            // Three creates of the same canonical fixture → same CID. View
            // MUST still list 3 entries (list semantics, not set semantics).
            assert_eq!(cids.len(), 3);
        }
        other => panic!("expected Cids, got {other:?}"),
    }
}

#[test]
fn content_listing_delete_removes_entry() {
    let mut v = ContentListingView::new("Post");
    v.update(&post_created()).unwrap();
    let mut delete_ev = post_created();
    delete_ev.kind = ChangeKind::Deleted;
    v.update(&delete_ev).unwrap();
    let q = ViewQuery {
        label: Some("Post".to_string()),
        limit: Some(100),
        offset: Some(0),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    assert!(matches!(r, ViewResult::Cids(ref c) if c.is_empty()));
}

#[test]
fn content_listing_pagination_respects_limit_and_offset() {
    let mut v = ContentListingView::new("Post");
    for _ in 0..10 {
        v.update(&post_created()).unwrap();
    }
    let q = ViewQuery {
        label: Some("Post".to_string()),
        limit: Some(3),
        offset: Some(2),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    match r {
        ViewResult::Cids(cids) => assert_eq!(cids.len(), 3),
        other => panic!("expected Cids, got {other:?}"),
    }
}

#[test]
fn content_listing_id_is_content_listing() {
    let v = ContentListingView::new("Post");
    assert_eq!(v.id(), "content_listing");
}

/// Regression for g5-p2-ivm-1: mixed streams of Node-bearing events (with
/// `createdAt`) and legacy identity-only events (falling back to `tx_id`)
/// must interleave in the SAME order-preserving `u64` sort-key space. Prior
/// to the fix the `tx_id` fallback skipped the `bias_i64_to_u64` pass,
/// putting small `tx_id` values in the `[0, 2^63)` half while non-negative
/// `createdAt` values biased into `[2^63, u64::MAX)` — a `tx_id=100` event
/// would sort BEFORE a `createdAt=0` event, reversing chronology.
#[test]
fn content_listing_mixed_sort_key_ordering() {
    fn post_with_created_at(ts: i64) -> ChangeEvent {
        let mut props = BTreeMap::new();
        props.insert("createdAt".to_string(), Value::Int(ts));
        // Give each node a distinct property so CIDs don't collide —
        // otherwise BTreeMap de-duplication on the value side hides the
        // ordering behaviour we're trying to observe.
        props.insert("ts_key".to_string(), Value::Int(ts));
        let node = Node::new(vec!["Post".to_string()], props);
        let cid = node.cid().unwrap();
        ChangeEvent {
            cid,
            labels: vec!["Post".to_string()],
            kind: ChangeKind::Created,
            // tx_id deliberately large so it out-ranks the tx_id fallback
            // used on the identity-only events below — if the bug comes
            // back we want the test to still trip.
            tx_id: 1_000_000 + ts as u64,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: Some(node),
            edge_endpoints: None,
        }
    }

    fn post_identity_only(tx_id: u64) -> ChangeEvent {
        ChangeEvent {
            cid: canonical_test_node().cid().unwrap(),
            labels: vec!["Post".to_string()],
            kind: ChangeKind::Created,
            tx_id,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: None,
            edge_endpoints: None,
        }
    }

    let mut v = ContentListingView::new("Post");
    // Interleave: an identity-only event with a SMALL tx_id, then a
    // Node-bearing event with createdAt=0. Under the pre-fix code the
    // tx_id=100 raw key (100) would sort before the createdAt=0 biased
    // key (2^63), reversing chronology. After the fix both go through
    // `bias_i64_to_u64`: createdAt=0 → 2^63, tx_id=100 (as i64) → 2^63+100,
    // so createdAt=0 sorts first.
    let ev_identity = post_identity_only(100);
    let ev_created_zero = post_with_created_at(0);
    let ev_created_later = post_with_created_at(500);

    v.update(&ev_created_zero).unwrap();
    v.update(&ev_identity).unwrap();
    v.update(&ev_created_later).unwrap();

    let q = ViewQuery {
        label: Some("Post".to_string()),
        limit: Some(100),
        offset: Some(0),
        ..ViewQuery::default()
    };
    let cids = match v.read(&q).unwrap() {
        ViewResult::Cids(c) => c,
        other => panic!("expected Cids, got {other:?}"),
    };
    assert_eq!(cids.len(), 3, "all three inserts must be present");
    // Order: createdAt=0 (biased 2^63) < tx_id=100 (biased 2^63+100)
    //        < createdAt=500 (biased 2^63+500).
    assert_eq!(
        cids[0], ev_created_zero.cid,
        "createdAt=0 must sort before identity-only tx_id=100"
    );
    assert_eq!(
        cids[1], ev_identity.cid,
        "identity-only tx_id=100 must sort between createdAt=0 and createdAt=500"
    );
    assert_eq!(
        cids[2], ev_created_later.cid,
        "createdAt=500 must sort last"
    );
}

proptest! {
    /// `prop_content_listing_incremental_equivalence` (R2 landscape §3 row 7)
    /// — after N random creates, the incremental view state equals a
    /// full rebuild. Widened at R4 triage (M7) to 0..256 cases and to assert
    /// payload equality (actual `Vec<Cid>`), not just variant discriminant.
    #[test]
    fn prop_content_listing_incremental_equivalence(n in 0usize..256) {
        let mut incremental = ContentListingView::new("Post");
        for _ in 0..n {
            incremental.update(&post_created()).unwrap();
        }
        let mut rebuilt = ContentListingView::new("Post");
        rebuilt.rebuild().unwrap();

        let q = ViewQuery {
            label: Some("Post".to_string()),
            limit: Some(1024),
            offset: Some(0),
            ..ViewQuery::default()
        };
        let r_inc = incremental.read(&q).unwrap();
        let r_reb = rebuilt.read(&q).unwrap();
        match (r_inc, r_reb) {
            (ViewResult::Cids(a), ViewResult::Cids(b)) => {
                prop_assert_eq!(a, b, "incremental and rebuilt payloads must match");
            }
            (a, b) => prop_assert!(false, "expected Cids/Cids, got {:?} and {:?}", a, b),
        }
    }
}
