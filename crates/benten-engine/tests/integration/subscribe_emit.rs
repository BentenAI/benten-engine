//! G6-B: SUBSCRIBE composes with EMIT (plan §4 SUBSCRIBE integration).
//!
//! # Status
//!
//! `#[ignore]`d pending G6-A executor wiring; tracks G6-A's
//! `phase-2b/g6/a-stream-subscribe-core` PR. The composition requires
//! G6-A's change-stream port + the SUBSCRIBE executor body to actually
//! deliver change events to the subscribed handler so it can EMIT in
//! response. Pre-G6-A `is_active() == false` so no events would fire.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_engine::{Engine, OnChangeCallback};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn subscribe_composes_with_emit_subscriber_side_strategy() {
    // SUBSCRIBE → handler → EMIT: a subscribed handler that responds
    // to a change event by EMITting (subscriber-side strategy). Requires
    // the change-stream port + executor body to drive the chain.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_seq, _chunk| {});
    let _sub = engine
        .on_change("/orders/*", cb)
        .expect("on_change registers");
    // Post-G6-A: write to /orders/789; assert the EMIT bus observes the
    // subscribed handler's emitted event with the matching payload.
}
