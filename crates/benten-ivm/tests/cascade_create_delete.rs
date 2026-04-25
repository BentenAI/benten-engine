//! Phase 2a R6 fix-pass — ivm-r6-1 closure: cascade Create→Delete
//! correctness across multiple registered IVM views (RESULTS.md §3
//! contract).
//!
//! `crates/benten-ivm/src/lib.rs:53-56` carried a `TODO(phase-2-ivm-cascade)`
//! marker noting that the RESULTS.md §3 cascade contract was unwitnessed
//! after R5. This test closes that gap: it constructs a small multi-view
//! Subscriber, feeds Create events to populate every view, then a cascade
//! of Delete events (some bulk, some targeted), and asserts each
//! dependent view's state correctly reflects the cascade. No view leaks
//! deleted entries; no view rejects the cascade as a budget breach
//! (budgets are unbounded by `View::new()`); the fan-out routes every
//! event to every registered view (Phase-1 fan-out semantics).
//!
//! Methodology:
//!   1. Build a `Subscriber` with TWO live views — `CapabilityGrantsView`
//!      (View 1) and `ContentListingView` (View 3). Different label
//!      filters so the routing pattern-match is exercised end-to-end
//!      (View 1 matches `system:CapabilityGrant`; View 3 matches `post`).
//!   2. Feed N Create events: K grant events (entity-keyed) + M post
//!      events (label-keyed with `createdAt`).
//!   3. Verify both views report the populated state via direct reads.
//!   4. Feed the cascade of Delete events covering EVERY created CID.
//!   5. Assert each view converges to empty after the cascade.
//!
//! Why two views (not just one): the dispatch wording asks for cascade
//! "across 2+ views" so the routing path's per-view dispatch is also
//! exercised — a bug that broke the per-view application loop after a
//! delete burst would surface here even if a single-view replay test
//! looked clean.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::Subscriber;
use benten_ivm::views::{CapabilityGrantsView, ContentListingView};
use benten_ivm::{View, ViewQuery, ViewResult};

/// Build a `system:CapabilityGrant`-labeled change event keyed on the
/// canonical event CID (View 1's identity-only fallback path).
fn grant_event(seed: u8, kind: ChangeKind, tx_id: u64) -> (Cid, ChangeEvent) {
    // Synthesize a deterministic CID per (seed) so we can target this
    // exact entry on the cascade-delete pass.
    let cid = Cid::from_blake3_digest(*blake3::hash(&[seed]).as_bytes());
    let event = ChangeEvent {
        cid,
        labels: vec!["system:CapabilityGrant".to_string()],
        kind,
        tx_id,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    };
    (cid, event)
}

/// Build a `post`-labeled change event with a `createdAt` property so
/// View 3 picks it up. Returns the post's content-addressed CID alongside
/// the event so the cascade-delete pass can target it precisely.
fn post_event(idx: u64, kind: ChangeKind, tx_id: u64) -> (Cid, ChangeEvent) {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("post-{idx}")));
    props.insert(
        "createdAt".into(),
        Value::Int(i64::try_from(idx).unwrap_or(0)),
    );
    let node = Node::new(vec!["post".into()], props);
    let cid = node.cid().expect("post node CID");
    let event = ChangeEvent {
        cid,
        labels: node.labels.clone(),
        kind,
        tx_id,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    };
    (cid, event)
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "fixture-heavy multi-view integration test; splitting artificially would hurt readability"
)]
fn cascade_create_then_delete_converges_every_view_to_empty() {
    // Step 1: build a multi-view subscriber. Two distinct view types so
    // the per-view dispatch loop is exercised on every event, not just
    // the one whose label matches.
    let mut subscriber = Subscriber::new();
    subscriber.register_view(Box::new(CapabilityGrantsView::new()));
    subscriber.register_view(Box::new(ContentListingView::new("post")));
    assert_eq!(
        subscriber.view_count(),
        2,
        "fixture sanity: both views must register"
    );

    // Step 2: feed N Create events. Mix grants and posts so the routing
    // pattern-match has to discriminate per view per event.
    let mut grant_cids: Vec<Cid> = Vec::new();
    for seed in 1u8..=4 {
        let (cid, event) = grant_event(seed, ChangeKind::Created, u64::from(seed));
        grant_cids.push(cid);
        let applied = subscriber
            .route_change_event(&event)
            .expect("grant create routes cleanly");
        // Both views see every event. View 3 (content_listing) ignores
        // grant-labeled events but its budget still ticks; the apply
        // count is just View 1 having absorbed the grant.
        assert!(
            applied >= 1,
            "at least View 1 must apply the grant create event"
        );
    }

    let mut post_cids: Vec<Cid> = Vec::new();
    for idx in 1u64..=5 {
        let (cid, event) = post_event(idx, ChangeKind::Created, 100 + idx);
        post_cids.push(cid);
        let applied = subscriber
            .route_change_event(&event)
            .expect("post create routes cleanly");
        assert!(
            applied >= 1,
            "at least View 3 must apply the post create event"
        );
    }

    // Step 3: verify populated state via direct reads. View 1 keys grants
    // under the event CID (identity-only fallback path); View 3 reports
    // every entry inserted under the matching label.
    {
        let view1 = CapabilityGrantsView::new();
        // Re-read via the subscriber so we exercise the read_view path.
        for grant_cid in &grant_cids {
            let q = ViewQuery {
                entity_cid: Some(*grant_cid),
                ..ViewQuery::default()
            };
            match subscriber.read_view("capability_grants", &q) {
                Some(Ok(ViewResult::Cids(cids))) => {
                    assert_eq!(
                        cids,
                        vec![*grant_cid],
                        "View 1 must report the grant under its own CID after Create"
                    );
                }
                other => panic!("View 1 read returned unexpected payload: {other:?}"),
            }
        }
        // Silence the unused fixture (kept for type-symmetry with the
        // post-cascade view-state assertion below).
        let _ = view1.id();
    }
    {
        // View 3 paginates by createdAt; ask for the full page.
        let q = ViewQuery {
            label: Some("post".into()),
            limit: Some(100),
            ..ViewQuery::default()
        };
        match subscriber.read_view("content_listing", &q) {
            Some(Ok(ViewResult::Cids(cids))) => {
                assert_eq!(
                    cids.len(),
                    post_cids.len(),
                    "View 3 must report exactly the posts created"
                );
                let actual: std::collections::BTreeSet<Cid> = cids.into_iter().collect();
                let expected: std::collections::BTreeSet<Cid> = post_cids.iter().copied().collect();
                assert_eq!(
                    actual, expected,
                    "View 3 entry set must match the created CID set"
                );
            }
            other => panic!("View 3 read returned unexpected payload: {other:?}"),
        }
    }

    // Step 4: cascade of Delete events. Every CID inserted in step 2
    // gets a paired Delete; tx_id continues monotonically so the views
    // don't get confused by a clock rewind.
    let mut tx = 1_000u64;
    for (seed, _grant_cid) in (1u8..=4).zip(grant_cids.iter()) {
        let (_cid, event) = grant_event(seed, ChangeKind::Deleted, tx);
        tx += 1;
        let _ = subscriber
            .route_change_event(&event)
            .expect("grant delete routes cleanly");
    }
    for (idx, _post_cid) in (1u64..=5).zip(post_cids.iter()) {
        let (_cid, event) = post_event(idx, ChangeKind::Deleted, tx);
        tx += 1;
        let _ = subscriber
            .route_change_event(&event)
            .expect("post delete routes cleanly");
    }

    // Step 5: every view must converge to empty across the cascade.
    for grant_cid in &grant_cids {
        let q = ViewQuery {
            entity_cid: Some(*grant_cid),
            ..ViewQuery::default()
        };
        match subscriber.read_view("capability_grants", &q) {
            Some(Ok(ViewResult::Cids(cids))) => assert!(
                cids.is_empty(),
                "View 1 must drop the grant entry after cascade Delete; got {cids:?}"
            ),
            other => panic!("View 1 post-cascade read returned unexpected payload: {other:?}"),
        }
    }
    {
        let q = ViewQuery {
            label: Some("post".into()),
            limit: Some(100),
            ..ViewQuery::default()
        };
        match subscriber.read_view("content_listing", &q) {
            Some(Ok(ViewResult::Cids(cids))) => assert!(
                cids.is_empty(),
                "View 3 must drop every post after cascade Delete; got {} entries",
                cids.len()
            ),
            other => panic!("View 3 post-cascade read returned unexpected payload: {other:?}"),
        }
    }

    // Step 6 (cascade idempotency): replaying the entire delete cascade
    // a second time MUST be a no-op — every view stays empty, no view
    // panics, and no view flips stale on a delete-storm against an
    // already-empty index.
    for seed in 1u8..=4 {
        let (_cid, event) = grant_event(seed, ChangeKind::Deleted, tx);
        tx += 1;
        let _ = subscriber
            .route_change_event(&event)
            .expect("idempotent grant delete routes cleanly");
    }
    for idx in 1u64..=5 {
        let (_cid, event) = post_event(idx, ChangeKind::Deleted, tx);
        tx += 1;
        let _ = subscriber
            .route_change_event(&event)
            .expect("idempotent post delete routes cleanly");
    }
    assert_eq!(
        subscriber.stale_count_tally(),
        0,
        "cascade Delete (twice) must NOT flip any view stale — both views \
         carry unbounded budgets"
    );
}
