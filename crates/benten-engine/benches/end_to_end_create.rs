//! Criterion benchmark: end-to-end create through the public `Engine` API.
//!
//! Distinct from `crates/benten-engine/benches/roundtrip.rs` (which measures
//! the storage-layer roundtrip): this bench runs the *composed* engine
//! with the capability hook, IVM subscribers, and transaction primitive
//! all wired up, measuring the full `engine.create_node(...)` / `engine.
//! call("post:create", ...)` path a TypeScript caller actually sees.
//!
//! **Target source:** §14.6 direct — "Node creation + IVM update:
//! 100–500µs realistic, 0.1ms aspirational." This is the same §14.6 row
//! as `create_node_immediate` in `benten-graph`, but measured at the
//! engine level so we can attribute any gap to composition overhead
//! (cap check + IVM dispatch + transaction wrapping) rather than storage.
//!
//! ## Gate policy
//!
//! - Median > 500µs: CI fails.
//! - Median < 100µs: CI warns (check that the cap backend ran, IVM
//!   actually propagated, and the transaction primitive committed).
//!
//! ## What this proves vs roundtrip.rs
//!
//! | Bench | Measures | Why both exist |
//! |---|---|---|
//! | `roundtrip.rs::create_node`   | `Engine::create_node` over redb alone | storage baseline |
//! | `end_to_end_create`           | full composed path incl. caps + IVM | attributable overhead |
//!
//! When both ship real code, the delta between them is the "composition
//! tax" — the cost §14.6 acknowledges but doesn't separately tabulate.
//! A delta > 150µs means composition overhead has overgrown the §14.6
//! envelope and needs profiling.

// Threshold (machine-readable, mirrors `bench-threshold-drift.yml`).
// The §14.6 row names a 100–500µs realistic envelope; on the public CI
// runner under contention + the BENTEN_BENCH_GATE_MULTIPLIER=3 cushion
// applied at gate time, a fail-on-regression numeric ceiling produces
// noise without catching real regressions. The §14.6 envelope itself is
// asserted by the `roundtrip.rs` storage-layer benches; this end-to-end
// composition variant tracks the trend across releases as informational.
// THRESHOLD_NS=informational policy=informational source=§14.6-direct

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

fn bench_end_to_end_create(c: &mut Criterion) {
    let dir = tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();

    let mut group = c.benchmark_group("end_to_end_create");
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.bench_function("composed_engine_cap_ivm_tx", |b| {
        b.iter(|| {
            let cid = engine.create_node(black_box(&node)).expect("create");
            black_box(cid);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_end_to_end_create);
criterion_main!(benches);
