//! Criterion benchmark: evaluate a 10-node handler subgraph.
//!
//! **Target source:** §14.6 direct — "10-node handler evaluation:
//! 150–300µs for mixed handlers." The qualifier "mixed" matters: handlers
//! with 2+ WRITEs and IVM propagation sit at the upper end of the range;
//! pure TRANSFORM pipelines can reach <100µs (acknowledged in §14.6 but
//! not the headline target because pure-TRANSFORM handlers are rare in
//! real applications — the real handlers from
//! `docs/validation/paper-prototype-handlers.md` average 2.8 WRITEs).
//!
//! ## Handler shape (representative mixed)
//!
//! The bench constructs a canonical 10-node handler:
//!
//! 1. READ input
//! 2. TRANSFORM (validate shape)
//! 3. BRANCH on validation result
//! 4. WRITE (primary entity)
//! 5. WRITE (audit log entry)
//! 6. CALL (sub-handler for notifications)
//! 7. TRANSFORM (build response)
//! 8. EMIT (event notification)
//! 9. RESPOND (success terminal)
//! 10. RESPOND (error terminal, from BRANCH ON_INVALID edge)
//!
//! This matches `crud('post').create` after expansion — the
//! exit-criterion-load-bearing path.
//!
//! ## Gate policy
//!
//! - Median > 300µs: CI fails.
//! - Median < 150µs: CI warns (suspiciously fast for a mixed handler —
//!   verify the bench isn't cold-caching, that all 2 WRITEs actually
//!   happened, and that IVM propagation was measured).
//!
//! ## Stub-graceful
//!
//! `benten-eval` is STUB at spike end. The bench references placeholder
//! functions (`build_canonical_10_node_handler_stub`, `evaluate_stub`)
//! that `todo!()` with pointers to E2/E3/E5. Running the bench before
//! those land will panic; that is the correct R3 TDD signal.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Stubs — replace when E2/E3/E5 land.
// ---------------------------------------------------------------------------

/// Placeholder handler id. Real type is the evaluator's `HandlerId`.
type HandlerIdStub = u64;

fn build_canonical_10_node_handler_stub() -> HandlerIdStub {
    todo!(
        "E1/E5 — 10-node handler registration not yet implemented; \
         bench will pass once benten-eval exposes subgraph construction + \
         structural validation."
    )
}

fn evaluate_stub(_handler_id: HandlerIdStub) {
    todo!(
        "E2/E3 — iterative evaluator with the 8 executable primitives not yet \
         implemented; bench will pass once the evaluator walks 10-node subgraphs."
    )
}

// ---------------------------------------------------------------------------
// Bench
// ---------------------------------------------------------------------------

fn bench_ten_node_handler_eval(c: &mut Criterion) {
    let handler = build_canonical_10_node_handler_stub();

    let mut group = c.benchmark_group("10_node_handler_eval");
    // Mixed handlers need a longer warmup — the first call touches the
    // TRANSFORM parser cache, the IVM subscribers, and the capability
    // backend. We want steady-state numbers, not first-call numbers.
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.bench_function("mixed_2_writes_plus_ivm", |b| {
        b.iter(|| {
            evaluate_stub(black_box(handler));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_ten_node_handler_eval);
criterion_main!(benches);
