//! Phase 2a R4 cov-6: `engine_diagnostics::view_stale_count()` tallies
//! correctly after view mutations.
//!
//! Traces to plan §3 G11-A + the R3 consolidation `todo!()` body on
//! `metrics_snapshot` where `benten.ivm.view_stale_count` was previously
//! hardcoded to `0.0`. G11-A Wave 1 replaced the hardcode with the real
//! subscriber-sourced tally (`Subscriber::stale_count_tally`); Wave 3a
//! lands this test green by driving it against a small-budget
//! ContentListingView so the mutation burst actually trips the view's
//! freshness bound. Without `.with_test_ivm_budget(small)` the default
//! view is constructed with `u64::MAX` budget and no realistic burst
//! would push it past.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// Engine with a deliberately tiny IVM budget so a modest write burst
/// saturates the ContentListingView and flips it stale. 4 is arbitrary —
/// chosen to be small enough that 128 inserts vastly overshoot, and
/// large enough that the first `testing_insert_privileged_fixture` does
/// not trip the view on the first event.
fn fresh_engine_with_small_ivm_budget() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .with_test_ivm_budget(4)
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn view_stale_count_tallies() {
    let (_dir, engine) = fresh_engine_with_small_ivm_budget();
    let snapshot_before = engine.metrics_snapshot();
    let before = snapshot_before
        .get("benten.ivm.view_stale_count")
        .copied()
        .unwrap_or(0.0);

    // Drive enough view-relevant writes to push at least one view past its
    // freshness bound. With `with_test_ivm_budget(4)` the ContentListingView
    // accepts 4 `post` inserts before the next update trips it stale; 128
    // inserts vastly overshoots so the stale tally MUST advance.
    for _ in 0..128_u32 {
        let _ = engine.testing_insert_privileged_fixture();
    }

    let snapshot_after = engine.metrics_snapshot();
    let after = snapshot_after
        .get("benten.ivm.view_stale_count")
        .copied()
        .unwrap_or(0.0);

    // Tighten the delta assertion (R4b COVERAGE M3 — the prior `after >=
    // before` is vacuous because `Subscriber::stale_count_tally` is
    // monotonic by construction). The realistic delta is exactly 1: the
    // builder auto-registers a single ContentListingView for "post"
    // (`builder.rs` ~line 321), and `stale_count_tally` returns the count
    // of currently-stale views — bounded above by the number of registered
    // views. After 128 inserts with `with_test_ivm_budget(4)` overshoots
    // the freshness bound, the single view MUST be stale, so
    // `after - before == 1` (or `after == 1` when the run starts fresh).
    // We assert `after - before >= 1` so the test is robust to a future
    // multi-view subscriber registration without weakening the firing
    // contract: the burst MUST flip at least one view stale.
    let delta = after - before;
    assert!(
        delta >= 1.0,
        "view_stale_count must advance by at least 1 after a mutation burst \
         big enough to push the auto-registered ContentListingView past its \
         freshness bound (with_test_ivm_budget(4), 128 inserts). Got \
         before={before}, after={after}, delta={delta}. \
         Either the subscriber is not registered (builder.rs ~line 316), \
         the view is not auto-registered (builder.rs ~line 323), or the \
         per-view budget tracker is not flipping is_stale on overshoot."
    );
    assert!(
        after > 0.0,
        "view_stale_count must be positive after a mutation burst big \
         enough to push a view past its freshness bound (G11-A Wave 1 \
         wired the real tally via Subscriber::stale_count_tally; Wave 3a \
         drives it with `.with_test_ivm_budget(small)` so a modest burst \
         actually trips the view). Got before={before} after={after}"
    );
}
