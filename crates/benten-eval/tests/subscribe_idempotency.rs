#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE handler idempotency replay-safe via Inv-13
//! (G6-A).
//!
//! Pin source: streaming-systems stream-d5-1 — handler with WRITE
//! side-effect; replay event 5×; assert WRITE happens once (Inv-13
//! immutability provides this naturally).
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::clone_on_copy)]

use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_register_idempotent_write_handler,
    testing_subscribe_inject_event, testing_subscribe_register,
};
use std::num::NonZeroUsize;

/// SUBSCRIBE handler that issues a WRITE side-effect: redelivering the same
/// event N times produces ONE write (Inv-13 immutability + handler-boundary
/// dedup combined). Replay-safe by construction.
#[test]
fn subscribe_handler_idempotency_replay_safe_via_inv_13() {
    let handler = testing_register_idempotent_write_handler();

    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/idem/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");
    sub.bind_handler(&handler).expect("bind");

    let anchor = benten_core::Cid::sample_for_test();
    let mut event =
        testing_make_change_event(anchor, ChangeKind::Created, serde_json::json!({"k": "v"}));
    event.seq = 7;

    // Replay 5×.
    for _ in 0..5 {
        testing_subscribe_inject_event(&sub, event.clone()).unwrap();
    }

    let observed_writes = handler.observed_write_count();
    assert_eq!(
        observed_writes, 1,
        "handler writes once across replays — handler-boundary dedup + Inv-13 collision-safety"
    );
}
