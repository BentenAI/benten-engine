//! R3-C RED-PHASE bench skeleton for the canonical-view fast-path
//! preservation gate (G15-A wave-5a; ivm-minor-6 + ivm-disagree-1).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.3 G15-A row
//!   `algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline`.
//! - plan §3 G15-A row line "preserves canonical-view fast-path
//!   within 20% gate".
//! - `ivm-minor-6` + `ivm-disagree-1` (gate measures canonical-view
//!   wallclock against Strategy::B baseline; the pre-G15-A
//!   `algorithm_b_vs_handwritten` bench measured against
//!   hand-written Phase-1 baselines, which `ivm-disagree-1`
//!   re-categorised as inner kernels of Strategy::B rather than
//!   Strategy::A baselines).
//!
//! ## What this is at R3-C landing time (R4-FP-revised per ivm-r4-2)
//!
//! The bench file exists as a SKELETON so `cargo bench
//! --bench algorithm_b_canonical -p benten-ivm -- --help` resolves
//! the bench name, but the bench body itself stays trivial (a
//! `benchmark_group` that benches no-op stubs). G15-A wave-5a fills
//! the bench with the real Strategy::B baseline + canonical view
//! comparison; the companion test
//! `tests/algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline`
//! parses the criterion output at the LOAD-BEARING paths
//! `target/criterion/algorithm_b_canonical_view_fast_path/Strategy_B_baseline/estimates.json`
//! and `.../post_g15a/estimates.json` to enforce the 20% ceiling.
//!
//! ## ivm-r4-2 BLOCKER closure (R4 large-council Round 1)
//!
//! The original R3-C skeleton used `c.bench_function("..._RED_PHASE_NO_OP")`
//! WITHOUT a benchmark_group. Criterion writes outputs to
//! `target/criterion/<bench-fn-name>/<sub-id>/estimates.json` for that
//! shape — which DID NOT line up with the parser path the gate-test
//! consumes (`target/criterion/algorithm_b_canonical_view_fast_path/Strategy_B_baseline/...`).
//! ivm-r4-2 caught this as a BLOCKER: the gate would FileNotFound
//! rather than assert the 20% ratio. R4-FP closes the producer/
//! consumer pair: bench skeleton uses `benchmark_group("algorithm_b_canonical_view_fast_path")`
//! with member names `Strategy_B_baseline` + `post_g15a` matching the
//! gate-test parser path constants.
//!
//! ## RED-PHASE behavior
//!
//! At R3-C landing time `cargo bench -p benten-ivm` runs the no-op
//! benchmark group in O(ms); the gate test is `#[ignore]`'d. G15-A
//! implementer replaces each member's body with the real bench
//! (Strategy::B baseline + post-G15-A canonical fast-path) + un-ignores
//! the gate test.

#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};

fn algorithm_b_canonical_view_fast_path(c: &mut Criterion) {
    // ivm-r4-2 BLOCKER closure: benchmark_group + member names are
    // LOAD-BEARING for the gate-test parser path. The group name MUST
    // match the gate-test's
    // `target/criterion/algorithm_b_canonical_view_fast_path/...`
    // path; member names `Strategy_B_baseline` + `post_g15a` MUST
    // match the gate-test's `Strategy_B_baseline/estimates.json` +
    // `post_g15a/estimates.json` parses.
    let mut group = c.benchmark_group("algorithm_b_canonical_view_fast_path");

    group.bench_function("Strategy_B_baseline", |b| {
        // R3-C RED-PHASE / R4-FP'd: G15-A wave-5a replaces this body
        // with the Strategy::B baseline measurement against the
        // canonical-view fast-path under the same fixture corpus
        // (pre-generalization baseline).
        b.iter(|| {
            // No-op. G15-A implementer wires the real bench.
            std::hint::black_box(0_u64)
        });
    });

    group.bench_function("post_g15a", |b| {
        // R3-C RED-PHASE / R4-FP'd: G15-A wave-5a replaces this body
        // with the post-G15-A generalized-kernel measurement against
        // the same canonical-view fast-path; the gate-test asserts
        // the ratio (post_g15a / Strategy_B_baseline) stays within
        // 1.20x.
        b.iter(|| {
            // No-op. G15-A implementer wires the real bench.
            std::hint::black_box(0_u64)
        });
    });

    group.finish();
}

criterion_group!(benches, algorithm_b_canonical_view_fast_path);
criterion_main!(benches);
