//! View 3 — Content listing (I5, exit-criterion load-bearing — R2 landscape
//! §2.3 rows 6-7).
//!
//! On `post`-labeled node create/update/delete, maintain sorted-by-`createdAt`
//! list; paginated reads are O(log n + page_size). This is the view
//! `crud('post').list` consumes for Exit Criterion #2.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::ContentListingView;
use benten_ivm::{View, ViewQuery, ViewResult};
use proptest::prelude::*;

fn post_created() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        label: "Post".to_string(),
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
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
