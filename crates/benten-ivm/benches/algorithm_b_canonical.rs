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
//! ## What this is at R3-C landing time
//!
//! The bench file exists as a SKELETON so `cargo bench
//! --bench algorithm_b_canonical -p benten-ivm -- --help` resolves
//! the bench name, but the bench body itself stays trivial (a
//! `criterion_group` that benches a no-op). G15-A wave-5a fills
//! the bench with the real Strategy::B baseline + canonical view
//! comparison; the companion test
//! `tests/algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline`
//! parses the criterion output to enforce the 20% ceiling.
//!
//! ## RED-PHASE behavior
//!
//! At R3-C landing time `cargo bench -p benten-ivm` runs the no-op
//! bench in O(ms); the gate test is `#[ignore]`'d. G15-A implementer
//! replaces the bench body + un-ignores the gate test.

#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};

fn algorithm_b_canonical_view_fast_path(c: &mut Criterion) {
    c.bench_function(
        "algorithm_b_canonical_view_fast_path_RED_PHASE_NO_OP",
        |b| {
            // R3-C RED-PHASE: G15-A wave-5a replaces this body with the
            // real bench — Strategy::B baseline against the canonical-view
            // fast-path under the same fixture corpus, asserting that the
            // ratio post-generalization stays within 1.20x of the
            // pre-generalization Strategy::B baseline.
            b.iter(|| {
                // No-op. G15-A implementer wires the real bench.
                std::hint::black_box(0_u64)
            });
        },
    );
}

criterion_group!(benches, algorithm_b_canonical_view_fast_path);
criterion_main!(benches);
