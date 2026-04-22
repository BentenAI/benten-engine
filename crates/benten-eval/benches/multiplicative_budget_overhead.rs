//! Criterion benchmark: per-node overhead of the Inv-8 multiplicative
//! iteration-budget check fired at every ITERATE / CALL boundary.
//!
//! **Target source:** plan §4.4 derived — "< 1 µs per ITERATE boundary
//! check on dev hardware." The source is derived (not a §14.6 direct
//! headline number) from the §4.4 commitment that threading multiplicative
//! budget through the evaluator must not exceed ~10% overhead on the
//! `ten_node_handler_eval` number, and the §9.12 fairness constraint that
//! budget probes are hot-path on every CALL / ITERATE traversal edge.
//!
//! **Gate policy:** CI-GATED — regressions fail the `phase-2a-exit-criteria`
//! workflow. Threshold is <1 µs median on dev hardware (M-class Apple
//! silicon / recent x86 server cores). Noisy CI runners get a 3× cushion
//! applied via the `BENTEN_BENCH_GATE_MULTIPLIER` env var; the baseline
//! threshold is the plan number.
//!
//! **Threshold encoding (machine-readable):** the gate workflow reads the
//! `median_ns` field from Criterion's JSON output (`target/criterion/<id>/
//! new/estimates.json`) and fails if `median_ns > THRESHOLD_NS *
//! BENTEN_BENCH_GATE_MULTIPLIER`. The value here is the contract:
//!
//! ```text
//! BENCH_ID = multiplicative_budget_overhead/boundary_check_per_node
//! THRESHOLD_NS = 1000  // 1 µs per §4.4 derived
//! POLICY = fail-on-regression
//! ```
//!
//! ## Red-phase TDD
//!
//! At the time this bench is committed (R3, Phase 2a), the multiplicative
//! budget implementation in G4-A is NOT yet landed. The bench references
//! `benten_eval::testing::multiplicative_budget_probe` which returns
//! `todo!()` until G4-A closes. `cargo bench` will panic on first iteration
//! with the standard TDD red-phase signal; once G4-A lands the probe, the
//! measurement becomes real and the gate fires.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

/// Per-node overhead of the Inv-8 multiplicative boundary check.
///
/// The workload constructs a synthetic evaluator frame stack at a depth
/// representative of the `paper-prototype-handlers.md` average (3 nested
/// ITERATE / CALL boundaries) and measures one probe call. The measurement
/// deliberately excludes the frame-stack setup — each iteration reuses the
/// pre-built frames so the reported number is the per-call probe cost, not
/// the cost of building test scaffolding.
fn bench_boundary_check_per_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiplicative_budget_overhead");
    // Warmup + measurement aligned with `ten_node_handler.rs` so apples-
    // to-apples comparison stays valid across re-runs.
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // MACHINE-READABLE GATE: the exit-criteria workflow greps this comment
    // for THRESHOLD_NS and fails the job if the observed median exceeds it.
    // THRESHOLD_NS=1000 policy=fail-on-regression source=plan-§4.4-derived

    group.bench_function("boundary_check_per_node", |b| {
        // The probe handle is opaque at R3 — G4-A fills in the frame-stack
        // construction. `todo!()` here makes the TDD red phase explicit:
        // the bench compiles + links, and panics on first iteration until
        // G4-A lands the implementation.
        b.iter(|| {
            let probe = benten_eval::testing::multiplicative_budget_probe();
            black_box(probe);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_boundary_check_per_node);
criterion_main!(benches);
