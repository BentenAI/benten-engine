//! Criterion benchmark: Option C evaluator-path threading overhead.
//!
//! **Target source:** not a §14.6 direct or derived number — this bench
//! is the **measurement artifact** for a compromise surface. G4-A threads
//! `PrimitiveHost::check_read_capability` into every content-returning
//! PrimitiveHost method (`read_node`, `get_by_label`, `get_by_property`,
//! `read_view`) to close the sec-r1-5 flanking gap. The question the
//! bench answers is: "how expensive is the added indirection?"
//!
//! **Gate policy:** INFORMATIONAL. Phase 2a does not fail PRs on this
//! number. The bench produces two measurements per method — one with a
//! `NoAuthBackend` policy (check returns immediately) and one with a
//! realistic `GrantBackedPolicy` — so the delta attributable to the
//! threading (vs. the policy lookup itself) is attributable.
//!
//! **Threshold encoding (machine-readable):**
//!
//! ```text
//! BENCH_ID = option_c_evaluator_path_overhead/*
//! THRESHOLD_NS = informational  // no CI gate
//! POLICY = informational
//! ```
//!
//! ## Why not gate?
//!
//! Gating on relative overhead (e.g. "Option C threading must add
//! <10 ns per READ") is the wrong shape for CI: the baseline itself
//! moves commit-to-commit as the READ primitive evolves, so a relative
//! gate would fire spuriously. The bench is the tripwire for human
//! review; R4b decides whether to promote to CI-gated once the numbers
//! stabilise across several PRs.
//!
//! ## Red-phase TDD
//!
//! `Engine::read_node_with_policy` / `get_by_label_with_policy` etc. are
//! G4-A deliverables. At R3 they return `todo!()`; the bench panics on
//! first iteration until G4-A lands. Once landed, the measurement
//! becomes real.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::Duration;

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;
use criterion::{Criterion, criterion_group, criterion_main};

/// Baseline: READ primitive without any `check_read_capability` threading
/// (the Phase-1 pre-Option-C surface). Re-benched so the delta vs. the
/// threaded path is attributable.
fn bench_read_baseline_no_capability_check(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let node = canonical_test_node();
    let cid = engine.put_node(&node).expect("seed node");

    let mut group = c.benchmark_group("option_c_evaluator_path_overhead");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // INFORMATIONAL — no gate
    // THRESHOLD_NS=informational policy=informational

    group.bench_function("read_baseline_no_capability_check", |b| {
        b.iter(|| {
            let n = engine.get_node(black_box(&cid));
            let _ = black_box(n);
        });
    });
    group.finish();
}

/// Threaded path: READ primitive with `check_read_capability` threading
/// and a `NoAuthBackend` policy (fast-return permit). Isolates the cost
/// of the indirection from the cost of a real policy lookup.
fn bench_read_threaded_noauth(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let node = canonical_test_node();
    let cid = engine.put_node(&node).expect("seed node");

    let mut group = c.benchmark_group("option_c_evaluator_path_overhead");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("read_threaded_noauth", |b| {
        b.iter(|| {
            // G4-A deliverable — returns todo!() at R3.
            let n = engine.read_node_with_policy(black_box(&cid));
            let _ = black_box(n);
        });
    });
    group.finish();
}

/// Threaded path with a realistic `GrantBackedPolicy` that actually
/// consults the capability-grant subgraph. This is the number product
/// users will feel in production.
fn bench_read_threaded_grant_backed(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .with_grant_backed_policy()
        .build()
        .unwrap();
    let node = canonical_test_node();
    let cid = engine.put_node(&node).expect("seed node");

    // Seed a grant so the policy returns `permit` (not denied).
    engine
        .grant_read_capability_for_testing(&cid)
        .expect("seed grant");

    let mut group = c.benchmark_group("option_c_evaluator_path_overhead");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("read_threaded_grant_backed", |b| {
        b.iter(|| {
            let n = engine.read_node_with_policy(black_box(&cid));
            let _ = black_box(n);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_read_baseline_no_capability_check,
    bench_read_threaded_noauth,
    bench_read_threaded_grant_backed
);
criterion_main!(benches);
