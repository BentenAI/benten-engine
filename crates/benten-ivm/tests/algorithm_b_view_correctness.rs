#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! Algorithm B vs hand-written equivalence tests — 5 views (G8-A).
//!
//! For each of the 5 Phase-1 hand-written views, run the same `ChangeEvent`
//! sequence through both the hand-written `Strategy::A` view AND the
//! generalized `Strategy::B` (Algorithm B) implementation, then assert
//! row-equivalence on every observable read.
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A (5-view
//! correctness must-pass).
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 rows 4-8.
//!
//! Future API per D8 + R2 §9 helper:
//! - `benten_ivm::testing::testing_construct_view_with_strategy(Strategy)
//!     -> Box<dyn View>` constructs a view of the named id under the chosen
//!   strategy. The id is supplied via a separate `with_id` argument shape
//!   per the testing helper signature documented in the R2 landscape (see
//!   §9 row "testing_construct_view_with_strategy").
//! - For tests that need a concrete shape, `algorithm_b::AlgorithmBView::for_id(view_id, def)`
//!   returns a `Box<dyn View>` that runs the generalized algorithm against
//!   the same `ViewDefinition` the hand-written view publishes.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::algorithm_b::AlgorithmBView;
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};
use benten_ivm::{View, ViewQuery, ViewResult};

// ---------------------------------------------------------------------------
// Content projection (R4-FP-A — replaces id-only tautology)
// ---------------------------------------------------------------------------
//
// `format!("{:?}", v.id())` was a tautological projection — both views ship
// with the SAME id by construction (the id is supplied as the first arg to
// `AlgorithmBView::for_id`). The reviewer (rust-test-reviewer.json tq-2b-1)
// flagged this as a vacuous-pass: any AlgorithmB divergence that doesn't
// rename the view passes, which is the entire class of bugs G8-A is supposed
// to catch.
//
// `project_view_content` reads the view's actual observable output via
// `view.read(&ViewQuery { ... })` and returns its Debug-rendered form. The
// `ViewResult` enum (Cids/Current/Rules) does not derive `Eq`, so we use
// the stable Debug-string projection — `Vec<Cid>` Debug ordering is
// deterministic; `BTreeMap<String, Value>` Debug iterates in key order.
// Two views diverging on actual content WILL produce different strings.
//
// `read()` may return `ViewError::PatternMismatch` for queries against
// indexes the view doesn't maintain — that's still useful as a
// content-equivalence projection (both views must pattern-mismatch
// identically). We capture both Ok/Err in the projection.
fn project_view_content(v: &dyn View, query: &ViewQuery) -> String {
    let result: Result<ViewResult, _> = v.read(query);
    format!("{result:?}")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn post_node(title: &str, created_at: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    props.insert("createdAt".into(), Value::Int(created_at));
    Node::new(vec!["post".into()], props)
}

fn cap_grant_node(grantee: Cid, scope: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("grantee".into(), Value::Bytes(grantee.as_bytes().to_vec()));
    props.insert("scope".into(), Value::Text(scope.into()));
    Node::new(vec!["CapabilityGrant".into()], props)
}

fn handler_node(subscribes_to: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("subscribes_to".into(), Value::Text(subscribes_to.into()));
    Node::new(vec!["Handler".into()], props)
}

fn governance_node(parent: Option<Cid>) -> Node {
    let mut props = BTreeMap::new();
    if let Some(p) = parent {
        props.insert("parent".into(), Value::Bytes(p.as_bytes().to_vec()));
    }
    Node::new(vec!["Governance".into()], props)
}

fn version_node(anchor: Cid, revision: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("anchor".into(), Value::Bytes(anchor.as_bytes().to_vec()));
    props.insert("revision".into(), Value::Int(revision));
    Node::new(vec!["Version".into()], props)
}

/// Replay one `ChangeEvent` sequence through both `a` and `b`. Used by every
/// per-view correctness test below. `query_fn` reads each side's snapshot and
/// returns a normalized projection that is `Eq` so both sides can be compared.
fn replay_and_compare<F, R>(
    mut a: Box<dyn View>,
    mut b: Box<dyn View>,
    events: &[ChangeEvent],
    query_fn: F,
) where
    F: Fn(&dyn View) -> R,
    R: Eq + std::fmt::Debug,
{
    for ev in events {
        let _ = a.update(ev);
        let _ = b.update(ev);
    }
    let result_a = query_fn(a.as_ref());
    let result_b = query_fn(b.as_ref());
    assert_eq!(
        result_a,
        result_b,
        "Algorithm B (Strategy::B) diverged from hand-written (Strategy::A) for view `{}`",
        a.id()
    );
}

// ---------------------------------------------------------------------------
// 1 — capability_grants  (LOW gate-risk per ivm-r1 perf table)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_correctness_against_capability_grants_view() {
    let a: Box<dyn View> = Box::new(CapabilityGrantsView::new());
    let b: Box<dyn View> = Box::new(AlgorithmBView::for_id(
        "capability_grants",
        CapabilityGrantsView::definition(),
    ));

    let actor = Cid::from_blake3_digest([0u8; 32]);
    let events = vec![
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["CapabilityGrant".into()],
            ChangeKind::Created,
            1,
            Some(cap_grant_node(actor, "write:post")),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["CapabilityGrant".into()],
            ChangeKind::Created,
            2,
            Some(cap_grant_node(actor, "read:post")),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["CapabilityGrant".into()],
            ChangeKind::Deleted,
            3,
            Some(cap_grant_node(actor, "read:post")),
        ),
    ];

    replay_and_compare(a, b, &events, |v| {
        // Project the actor's actual grant set from each view (CapabilityGrants
        // keys on `entity_cid`; both views must report the same Cids slice for
        // the actor or Algorithm B has diverged).
        let q = ViewQuery {
            entity_cid: Some(actor),
            ..ViewQuery::default()
        };
        project_view_content(v, &q)
    });
}

// ---------------------------------------------------------------------------
// 2 — event_handler_dispatch  (LOW gate-risk)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_correctness_against_event_handler_dispatch_view() {
    let a: Box<dyn View> = Box::new(EventDispatchView::new());
    let b: Box<dyn View> = Box::new(AlgorithmBView::for_id(
        "event_dispatch",
        EventDispatchView::definition(),
    ));

    let events = vec![
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Handler".into()],
            ChangeKind::Created,
            1,
            Some(handler_node("post.created")),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Handler".into()],
            ChangeKind::Created,
            2,
            Some(handler_node("post.deleted")),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Handler".into()],
            ChangeKind::Deleted,
            3,
            Some(handler_node("post.created")),
        ),
    ];

    replay_and_compare(a, b, &events, |v| {
        // Project the handler set for the `post.created` event name (the
        // EventDispatch view keys on `event_name`; both views must report the
        // same handler-CID set or Algorithm B has diverged).
        let q = ViewQuery {
            event_name: Some("post.created".into()),
            ..ViewQuery::default()
        };
        project_view_content(v, &q)
    });
}

// ---------------------------------------------------------------------------
// 3 — content_listing  (HIGH gate-risk per ivm-r1 perf table — ~25-35% B overhead)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_correctness_against_content_listing_view() {
    // R3-D ownership disambiguation per landscape §10 row
    // `algorithm_b_correctness_against_content_listing_view` — R3-D iterates
    // here first because this view is the highest gate-risk for the 20% bench
    // gate.
    let a: Box<dyn View> = Box::new(ContentListingView::new("post"));
    let b: Box<dyn View> = Box::new(AlgorithmBView::for_id(
        "content_listing",
        ContentListingView::definition(),
    ));

    let mut events = Vec::new();
    for i in 0u64..32 {
        events.push(ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["post".into()],
            ChangeKind::Created,
            i + 1,
            Some(post_node(&format!("post-{i}"), i as i64 * 100)),
        ));
    }
    // Sprinkle in deletes to exercise the cancellation path.
    for i in (0u64..32).step_by(7) {
        events.push(ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["post".into()],
            ChangeKind::Deleted,
            100 + i,
            Some(post_node(&format!("post-{i}"), i as i64 * 100)),
        ));
    }
    // Non-matching label — both sides must skip identically.
    events.push(ChangeEvent::new_node(
        Cid::from_blake3_digest([0u8; 32]),
        vec!["comment".into()],
        ChangeKind::Created,
        999,
        Some(post_node("not-a-post", 9999)),
    ));

    replay_and_compare(a, b, &events, |v| {
        // Project the full first page of the `post` listing (the
        // ContentListing view keys on `label`; both views must report the
        // same paginated CID set or Algorithm B has diverged on the
        // cancellation path).
        let q = ViewQuery {
            label: Some("post".into()),
            limit: Some(100),
            offset: Some(0),
            ..ViewQuery::default()
        };
        project_view_content(v, &q)
    });
}

// ---------------------------------------------------------------------------
// 4 — governance_inheritance  (MEDIUM-HIGH gate-risk per ivm-r1 — transitive closure)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_correctness_against_governance_inheritance_view() {
    let a: Box<dyn View> = Box::new(GovernanceInheritanceView::new());
    let b: Box<dyn View> = Box::new(AlgorithmBView::for_id(
        "governance_inheritance",
        GovernanceInheritanceView::definition(),
    ));

    let root = Cid::from_blake3_digest([0u8; 32]);
    let events = vec![
        ChangeEvent::new_node(
            root,
            vec!["Governance".into()],
            ChangeKind::Created,
            1,
            Some(governance_node(None)),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Governance".into()],
            ChangeKind::Created,
            2,
            Some(governance_node(Some(root))),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Governance".into()],
            ChangeKind::Created,
            3,
            Some(governance_node(Some(root))),
        ),
    ];

    replay_and_compare(a, b, &events, |v| {
        // Project the effective rules for the root governance entity (the
        // GovernanceInheritance view keys on `entity_cid`; both views must
        // report the same effective-rules map or Algorithm B has diverged on
        // the transitive-closure path).
        let q = ViewQuery {
            entity_cid: Some(root),
            ..ViewQuery::default()
        };
        project_view_content(v, &q)
    });
}

// ---------------------------------------------------------------------------
// 5 — version_current  (MEDIUM gate-risk per ivm-r1)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_correctness_against_version_current_view() {
    let a: Box<dyn View> = Box::new(VersionCurrentView::new());
    let b: Box<dyn View> = Box::new(AlgorithmBView::for_id(
        "version_current",
        VersionCurrentView::definition(),
    ));

    let anchor = Cid::from_blake3_digest([0u8; 32]);
    let events = vec![
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Version".into()],
            ChangeKind::Created,
            1,
            Some(version_node(anchor, 1)),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Version".into()],
            ChangeKind::Created,
            2,
            Some(version_node(anchor, 2)),
        ),
        ChangeEvent::new_node(
            Cid::from_blake3_digest([0u8; 32]),
            vec!["Version".into()],
            ChangeKind::Created,
            3,
            Some(version_node(anchor, 3)),
        ),
    ];

    replay_and_compare(a, b, &events, |v| {
        // Project the CURRENT version pointer for the anchor (the
        // VersionCurrent view keys on `anchor_id`; both views must report the
        // same Current(Cid) or Algorithm B has diverged on the
        // last-revision-wins path). Note: `anchor_id: u64` per ViewQuery
        // shape; the 0 sentinel is fine for this fixture because the test
        // uses a single anchor.
        let q = ViewQuery {
            anchor_id: Some(0),
            ..ViewQuery::default()
        };
        project_view_content(v, &q)
    });
}
