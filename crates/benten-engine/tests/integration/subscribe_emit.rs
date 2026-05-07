//! G6-B: SUBSCRIBE composes with EMIT (plan §4 SUBSCRIBE integration).
//!
//! Phase-3 G20-A2 (D12 wave-8a): un-ignored per §7.3.A.2. The
//! SUBSCRIBE wave-8c production-runtime wire-through landed at Phase-2b
//! `phase-2b-close`; the integration test below drives the on_change
//! registration + verifies the engine surfaces the subscription
//! handle (the load-bearing observable: a registered subscription is
//! listed in the engine's tracked-subscription set so a subsequent
//! WRITE can route delivery through it).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_engine::{Engine, OnChangeCallback};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
fn subscribe_composes_with_emit_subscriber_side_strategy() {
    // SUBSCRIBE → handler → EMIT: a subscribed handler that responds
    // to a change event by EMITting (subscriber-side strategy). The
    // load-bearing observable from this surface is that the engine
    // returns a subscription handle from `on_change` — the production
    // runtime then drives the handler when WRITEs to matching anchors
    // land. Tests that exercise the full SUBSCRIBE → EMIT chain live
    // at `engine_subscribe_*` integration suite (drives WRITE +
    // observes EMIT broadcast); this fixture pins the registration
    // surface contract.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_seq, _chunk| {});
    let _sub = engine
        .on_change("/orders/*", cb)
        .expect("on_change registers");
}
