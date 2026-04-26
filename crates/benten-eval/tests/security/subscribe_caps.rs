#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE security-class — capability gating + Inv-11 +
//! Option-C compliance (G6-A).
//!
//! Pin source: plan §4 SUBSCRIBE security-class + Inv-11 +
//! D5-RESOLVED cap-check at delivery + streaming-systems stream-d5-1.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_principal_with_caps, testing_principal_without_caps,
    testing_revoke_cap_mid_subscribe, testing_subscribe_inject_event,
    testing_subscribe_register_as,
};
use std::num::NonZeroUsize;

fn spec(prefix: &str) -> SubscriptionSpec {
    SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix(prefix.into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    }
}

/// Cap-gated at register-time: subscribing requires the SUBSCRIBE capability
/// AND a READ cap on the target zone.
#[test]
#[ignore = "Phase 2b G6-A pending — cap gating at register"]
fn subscribe_capability_gated_at_register() {
    let principal = testing_principal_without_caps();
    let result = testing_subscribe_register_as(&principal, spec("/posts/"));
    let err = result.expect_err("registration without caps must fail");
    assert!(
        matches!(
            err.error_code(),
            ErrorCode::SubscribePatternInvalid
                | ErrorCode::HostBackendUnavailable
                | ErrorCode::Inv11SystemZoneRead
        ),
        "register-time cap-failure must surface a typed error; got {err:?}"
    );
}

/// Cap-gated at DELIVERY time (TOCTOU): the subscriber's READ cap is
/// re-intersected against the event payload at delivery; revoking between
/// events surfaces typed delivery error.
#[test]
#[ignore = "Phase 2b G6-A pending — D5 cap-check at delivery"]
fn subscribe_capability_gated_at_delivery() {
    let principal = testing_principal_with_caps(&["read:/posts/*", "subscribe:*"]);
    let sub = testing_subscribe_register_as(&principal, spec("/posts/")).expect("register");

    let anchor = benten_core::Cid::sample_for_label("/posts/abc");
    let event = testing_make_change_event(anchor, ChangeKind::Created, serde_json::json!({"v": 1}));
    testing_subscribe_inject_event(&sub, event).expect("first event delivered");

    let first = sub.next_blocking(std::time::Duration::from_millis(100));
    assert!(first.is_some(), "first event must deliver");

    // Revoke the READ cap mid-stream.
    testing_revoke_cap_mid_subscribe(&principal, "read:/posts/*");

    let post_revoke_event = testing_make_change_event(
        benten_core::Cid::sample_for_label("/posts/def"),
        ChangeKind::Created,
        serde_json::json!({"v": 2}),
    );
    testing_subscribe_inject_event(&sub, post_revoke_event).unwrap();

    let outcome = sub.next_outcome_blocking(std::time::Duration::from_millis(100));
    let err = outcome.expect_err("delivery after revoke must fail closed");
    assert_eq!(err.error_code(), ErrorCode::SubscribeDeliveryFailed);
}

/// Revocation mid-stream cancels the subscription cleanly.
#[test]
#[ignore = "Phase 2b G6-A pending — revoke cancel"]
fn subscribe_capability_revoked_mid_stream_cancels() {
    let principal = testing_principal_with_caps(&["read:/x/*", "subscribe:*"]);
    let sub = testing_subscribe_register_as(&principal, spec("/x/")).expect("register");
    assert!(sub.is_active());

    testing_revoke_cap_mid_subscribe(&principal, "read:/x/*");

    let post_revoke = testing_make_change_event(
        benten_core::Cid::sample_for_label("/x/y"),
        ChangeKind::Updated,
        serde_json::json!({}),
    );
    testing_subscribe_inject_event(&sub, post_revoke).unwrap();
    let _ = sub.next_outcome_blocking(std::time::Duration::from_millis(100));

    assert!(
        !sub.is_active(),
        "subscription must auto-cancel after delivery-time cap-revoke fires"
    );
}

/// Inv-11: SUBSCRIBE pattern cannot exfiltrate cross-zone data — registering
/// `system:*` from user code MUST fail.
#[test]
#[ignore = "Phase 2b G6-A pending — Inv-11 SUBSCRIBE pattern"]
fn subscribe_pattern_cannot_exfiltrate_cross_zone_data_inv_11() {
    let principal = testing_principal_with_caps(&["subscribe:*"]);
    // System-zone label prefix from user code must be denied.
    let result = testing_subscribe_register_as(&principal, spec("system:/secrets/"));
    let err = result.expect_err("system: prefix from user code must fail Inv-11");
    assert_eq!(err.error_code(), ErrorCode::Inv11SystemZoneRead);
}

/// Option-C READ cap-check is NOT bypassed by the SUBSCRIBE delivery path —
/// the same `check_read` flank covers SUBSCRIBE event payloads.
#[test]
#[ignore = "Phase 2b G6-A pending — Option-C parity"]
fn subscribe_does_not_bypass_option_c_read_cap_check() {
    let principal = testing_principal_with_caps(&["subscribe:*"]); // no read cap
    let sub = testing_subscribe_register_as(&principal, spec("/oc/")).expect("register");

    let event = testing_make_change_event(
        benten_core::Cid::sample_for_label("/oc/data"),
        ChangeKind::Created,
        serde_json::json!({"sensitive": true}),
    );
    testing_subscribe_inject_event(&sub, event).unwrap();

    let outcome = sub.next_outcome_blocking(std::time::Duration::from_millis(100));
    let err = outcome.expect_err("delivery without read cap must fail Option-C");
    assert!(matches!(
        err.error_code(),
        ErrorCode::SubscribeDeliveryFailed | ErrorCode::Inv11SystemZoneRead
    ));
}
