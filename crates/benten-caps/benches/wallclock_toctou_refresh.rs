//! Criterion benchmark: wall-clock TOCTOU dual-source refresh overhead.
//!
//! **Target source:** plan §9.13 + §9.10 — "300s default refresh cadence
//! on `MonotonicSource::elapsed`; HLC consulted alongside for Phase-3
//! federation-correlation." No §14.6 direct number — this bench measures
//! what §9.13's TOCTOU refresh costs per ITERATE iteration so regressions
//! show up.
//!
//! **Gate policy:** INFORMATIONAL. The refresh fires at most once per
//! 300s window (default), so the per-iteration amortised cost is
//! sub-nanosecond on the happy path (the `MonotonicSource::elapsed`
//! check compared against the stored deadline is a plain integer
//! comparison). The bench exists to protect against a regression where
//! the check path accidentally pulls in the full refresh (e.g. someone
//! mis-wires the dual-source check and consults the HLC unconditionally).
//! Gating would require hardware-independent nanosecond thresholds that
//! don't survive the CI runner variance; informational is honest.
//!
//! **Threshold encoding (machine-readable):**
//!
//! ```text
//! BENCH_ID = wallclock_toctou_refresh/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = plan-§9.13-toctou-dual-source
//! ```
//!
//! ## What the two bench functions measure
//!
//! - `elapsed_check_no_refresh` — the common case: inside the 300s
//!   window, refresh is NOT due, the check returns "not yet" after one
//!   monotonic-clock read + one integer compare. This is what happens
//!   on every ITERATE iteration during normal execution.
//! - `refresh_fires_dual_source` — the uncommon case: the 300s boundary
//!   is crossed, refresh actually fires, MonotonicSource elapsed is
//!   re-anchored, and the HLC ride-along stamp is captured. This is
//!   what happens once per 300s of cumulative ITERATE wall-time.
//!
//! ## Red-phase TDD
//!
//! `WallclockRefreshProbe` is a G9-A deliverable; at R3 the probe
//! returns `todo!()`. The bench panics on first iteration until G9-A
//! lands. Once landed, the numbers become real.
//!
//! ## Phase-3 forward-compat
//!
//! The HLC ride-along exists specifically so Phase-3 federation receive
//! can correlate refresh events across peers. Phase-3 wiring must not
//! regress this bench — if it does, the Phase-3 layer broke the "HLC
//! is consulted, not authoritative" contract from the §9.13 resolution
//! of sec-r1-2 vs ucca-5.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

/// Happy path: inside the 300s window, the check returns "not yet"
/// without triggering a grant re-lookup. Measures the per-iteration
/// cost the evaluator pays on every ITERATE boundary.
fn bench_elapsed_check_no_refresh(c: &mut Criterion) {
    // G9-A testing helper: constructs a probe with a MonotonicSource
    // pinned to "just refreshed" so the elapsed check returns `false`.
    let probe = benten_caps::testing::wallclock_refresh_probe_fresh();

    let mut group = c.benchmark_group("wallclock_toctou_refresh");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // INFORMATIONAL — no gate
    // THRESHOLD_NS=informational policy=informational

    group.bench_function("elapsed_check_no_refresh", |b| {
        b.iter(|| {
            // G9-A NOT LANDED — returns todo!() at R3.
            let needs_refresh = probe.check_elapsed(black_box(Duration::from_mins(5)));
            black_box(needs_refresh);
        });
    });
    group.finish();
}

/// Uncommon path: 300s boundary crossed, refresh fires. Measures the
/// MonotonicSource re-anchor + HLC ride-along-stamp capture.
fn bench_refresh_fires_dual_source(c: &mut Criterion) {
    // Probe pre-configured to be "over the 300s deadline" — every
    // `check_elapsed` call returns true and fires a refresh. To avoid
    // measuring the repeated fire path, we use iter_batched with a
    // fresh-each-time setup so every iteration is the first refresh.
    let mut group = c.benchmark_group("wallclock_toctou_refresh");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(30);

    group.bench_function("refresh_fires_dual_source", |b| {
        b.iter_batched(
            || benten_caps::testing::wallclock_refresh_probe_expired(),
            |probe| {
                // Force the refresh: returns the new anchor pair
                // (MonotonicSource instant + HLC stamp).
                let anchors = probe.force_refresh();
                black_box(anchors);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_elapsed_check_no_refresh,
    bench_refresh_fires_dual_source
);
criterion_main!(benches);
