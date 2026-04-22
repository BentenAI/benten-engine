//! Phase 2a R4 cov-6: `engine_diagnostics::view_stale_count()` tallies
//! correctly after view mutations.
//!
//! Traces to plan §3 G11-A + the R3 consolidation `todo!()` body on
//! `metrics_snapshot` where `benten.ivm.view_stale_count` is currently
//! hardcoded to `0.0` — G11-A replaces the hardcode with an actual tally
//! sourced from the IVM subscriber. This test asserts the tally increments
//! as views go stale.
//!
//! TDD red-phase: until G11-A lands, the stale counter stays at 0 even
//! after mutations go past the view's freshness bound, so this test
//! fails at the second assertion.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn view_stale_count_tallies() {
    let (_dir, engine) = fresh_engine();
    let snapshot_before = engine.metrics_snapshot();
    let before = snapshot_before
        .get("benten.ivm.view_stale_count")
        .copied()
        .unwrap_or(0.0);

    // Drive enough view-relevant writes to push at least one view past its
    // freshness bound. In Phase-2a G11-A, the subscriber marks a view stale
    // when its maintenance queue exceeds the per-view budget; the metric
    // snapshot reflects the count of stale views.
    for _ in 0..128_u32 {
        let _ = engine.testing_insert_privileged_fixture();
    }

    let snapshot_after = engine.metrics_snapshot();
    let after = snapshot_after
        .get("benten.ivm.view_stale_count")
        .copied()
        .unwrap_or(0.0);

    // The exact number depends on the per-view budget config; the contract
    // is "not zero after a non-trivial mutation burst". G11-A owns the
    // tally plumbing; R3 consolidation's `0.0` hardcode makes this test
    // fail until the wire-up lands.
    assert!(
        after >= before,
        "view_stale_count must be monotonically non-decreasing; before={before} after={after}"
    );
    assert!(
        after > 0.0,
        "view_stale_count must be positive after a mutation burst big \
         enough to push a view past its freshness bound (G11-A wires the \
         real tally; R3 stub hardcodes 0.0 — this assertion red-phases until \
         G11-A lands). Got before={before} after={after}"
    );
}
