//! Phase 2a R3 integration — Wall-clock TOCTOU revokes mid-ITERATE.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G9-A
//! (`caps_wallclock_bound_refreshes_at_300s_default`,
//! `long_running_transform_honors_wallclock`,
//! `wallclock_toctou_monotonic`, `wallclock_refresh_ntp_slew_doesnt_skip`)
//! + §9.13 TOCTOU refresh-point #5 (wall-clock boundary every 300s
//! default). Closes Compromise #1 Phase-2 item.
//!
//! Dual-source: `MonotonicSource` drives the 300s cadence (drift-exploit
//! hard); `HLC` consulted alongside for federation correlation. Owned by
//! `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: gated under `phase_2a_pending_apis` until G9-A lands the
// `call_with_ticking_mono_clock` + `monotonic_source` builder surface.
// See `wait_inside_wait_serializes_correctly.rs` header for the rationale.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test `MonotonicSource` impl that returns a caller-controlled elapsed
/// duration. Enables the 300s-bound test to run in <1s wall-time.
#[derive(Clone, Default)]
struct MockMonotonic {
    elapsed: Arc<Mutex<Duration>>,
}

impl MockMonotonic {
    fn tick(&self, by: Duration) {
        let mut slot = self.elapsed.lock().unwrap();
        *slot += by;
    }
}

impl benten_eval::MonotonicSource for MockMonotonic {
    fn elapsed(&self) -> Duration {
        *self.elapsed.lock().unwrap()
    }
}

fn long_iterate_handler() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("wallclock:long_iterate")
        .iterate(|i| i.over("$input.items").max(1000))
        .write(|w| {
            w.label("audit_entry")
                .property("iter", Value::Text("$iter_index".into()))
                .requires("store:audit_entry:write")
        })
        .respond(|r| r.body("$result"))
        .build()
}

/// Wall-clock boundary at 300s default: a cap revoked before the boundary
/// but after CALL entry surfaces E_CAP_REVOKED_MID_EVAL at the boundary
/// crossing.
#[test]
fn wallclock_toctou_revokes_mid_iterate() {
    let dir = tempfile::tempdir().unwrap();
    let mono = MockMonotonic::default();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .monotonic_source(Box::new(mono.clone()))
        .build()
        .unwrap();

    let handler_id = engine.register_subgraph(long_iterate_handler()).unwrap();
    let alice = engine.create_principal("alice").unwrap();
    engine
        .grant_capability(&alice, "store:audit_entry:write")
        .unwrap();

    let mut input = BTreeMap::new();
    input.insert(
        "items".into(),
        Value::List((0..1000).map(Value::Int).collect()),
    );

    let outcome = engine.call_with_ticking_mono_clock(
        &handler_id,
        "wallclock:iterate_run",
        Node::new(vec!["input".into()], input),
        &alice,
        |ticker| {
            ticker.tick_batches(1);
            mono.tick(Duration::from_secs(100));
            engine
                .revoke_capability(&alice, "store:audit_entry:write")
                .unwrap();
            mono.tick(Duration::from_secs(250));
            ticker.tick_batches(2);
        },
    );

    assert_eq!(
        outcome.error_code(),
        Some("E_CAP_REVOKED_MID_EVAL"),
        "revocation observed at the 300s wall-clock boundary must surface \
         E_CAP_REVOKED_MID_EVAL; got {outcome:?}"
    );
    assert!(outcome.routed_through_edge("ON_DENIED"));
}

/// NTP slew doesn't skip refresh: wall-clock jumping backward must not
/// satisfy the 300s elapsed-bound (sec-r1-2 / ucca-5).
#[test]
fn wallclock_refresh_ntp_slew_doesnt_skip() {
    let dir = tempfile::tempdir().unwrap();
    let mono = MockMonotonic::default();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .monotonic_source(Box::new(mono.clone()))
        .testing_wall_clock_ntp_slew_simulation(true)
        .build()
        .unwrap();

    let handler_id = engine.register_subgraph(long_iterate_handler()).unwrap();
    let alice = engine.create_principal("alice").unwrap();
    engine
        .grant_capability(&alice, "store:audit_entry:write")
        .unwrap();

    let mut input = BTreeMap::new();
    input.insert(
        "items".into(),
        Value::List((0..500).map(Value::Int).collect()),
    );

    mono.tick(Duration::from_secs(100));
    engine.testing_simulate_wallclock_jump(Duration::from_secs(300), -1);

    let refresh_count_before = engine.testing_wallclock_refresh_count();
    let _ = engine.call_as(
        &handler_id,
        "wallclock:iterate_run",
        Node::new(vec!["input".into()], input),
        &alice,
    );
    let refresh_count_after = engine.testing_wallclock_refresh_count();

    assert_eq!(
        refresh_count_after - refresh_count_before,
        0,
        "NTP wall-clock slew must NOT trigger a refresh when monotonic \
         elapsed < 300s; refreshed {} times",
        refresh_count_after - refresh_count_before
    );
}
