#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE pattern matching + invalid-pattern typed
//! error (G6-A).
//!
//! Pin source: plan §3 G6-A error-catalog — `E_SUBSCRIBE_PATTERN_INVALID`.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_subscribe_inject_event, testing_subscribe_register,
};
use std::num::NonZeroUsize;

/// Pattern label-matches anchor glob: `/posts/*` matches `/posts/123` but
/// not `/comments/123`.
#[test]
#[ignore = "Phase 2b G6-A pending — pattern label glob"]
fn subscribe_pattern_label_matches_anchor_glob() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::LabelGlob("/posts/*".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");

    let posts_anchor = benten_core::Cid::sample_for_label("/posts/123");
    let comments_anchor = benten_core::Cid::sample_for_label("/comments/456");

    testing_subscribe_inject_event(
        &sub,
        testing_make_change_event(
            posts_anchor.clone(),
            ChangeKind::Created,
            serde_json::json!({}),
        ),
    )
    .unwrap();
    testing_subscribe_inject_event(
        &sub,
        testing_make_change_event(comments_anchor, ChangeKind::Created, serde_json::json!({})),
    )
    .unwrap();

    let received = sub.drain_events_blocking(std::time::Duration::from_millis(100));
    assert_eq!(received.len(), 1, "only matching anchors deliver");
    assert_eq!(received[0].anchor_cid, posts_anchor);
}

/// Invalid pattern (e.g. malformed glob, empty pattern) → typed error at
/// registration time.
#[test]
#[ignore = "Phase 2b G6-A pending — pattern invalid"]
fn subscribe_pattern_invalid_fires_e_subscribe_pattern_invalid() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::LabelGlob("[unclosed-bracket".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    };
    let result = testing_subscribe_register(spec);
    let err = result.expect_err("malformed pattern must fail registration");
    assert_eq!(err.error_code(), ErrorCode::SubscribePatternInvalid);
}
