//! Criterion benchmarks: `get_node` and `create_node` on the redb backend.
//!
//! These are the two headline §14.6 direct gates at the storage layer. The
//! engine-level benchmark (`crates/benten-engine/benches/roundtrip.rs`) wraps
//! the same operations but adds public-API overhead; this bench isolates the
//! storage cost so a regression can be localized.
//!
//! ## Targets (both §14.6 direct)
//!
//! | Benchmark | §14.6 target | Source |
//! |---|---|---|
//! | `get_node`            | 1–50µs hot cache                  | §14.6 direct — "Node lookup by ID: 1-50us" |
//! | `create_node_immediate` | 100–500µs realistic (immediate)  | §14.6 direct — "Node creation + IVM update: 100-500us realistic" |
//!
//! The `create_node_immediate` bench uses redb's default durability (fsync on
//! commit). `create_node_group_commit` lives in `durability_modes.rs` and
//! covers the amortized case.
//!
//! ## Gate policy
//!
//! CI fails if the bench median is outside the §14.6 range. Specifically:
//!
//! - `get_node`: median > 50µs fails; < 1µs warns (suspiciously fast — verify
//!   warmup isn't skipped).
//! - `create_node_immediate`: median > 500µs fails; < 100µs warns.
//!
//! ## Stub-graceful
//!
//! All methods invoked here (`RedbBackend::open`, `put_node`, `get_node`)
//! exist in the spike. This bench will run against real code from day one of
//! R5.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::testing::canonical_test_node;
use benten_graph::RedbBackend;
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

fn bench_get_node(c: &mut Criterion) {
    let dir = tempdir().expect("tempdir");
    let backend = RedbBackend::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();
    let cid = backend.put_node(&node).expect("put");

    let mut group = c.benchmark_group("get_node");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache", |b| {
        b.iter(|| {
            let fetched = backend.get_node(black_box(&cid)).expect("get");
            black_box(fetched);
        });
    });
    group.finish();
}

fn bench_get_node_batch(c: &mut Criterion) {
    // Batch read pattern — the list/scan hot path fetches a page of N Nodes.
    // §14.6 derived: "Node lookup by ID: 1-50µs" times page size = upper
    // bound for a 100-item page read. Gate target: < 100 × 50µs = 5ms for
    // 100 lookups; amortized < 50µs per lookup.
    let dir = tempdir().expect("tempdir");
    let backend = RedbBackend::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();
    let cid = backend.put_node(&node).expect("put");

    let mut group = c.benchmark_group("get_node_batch_100");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache_same_cid", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let fetched = backend.get_node(black_box(&cid)).expect("get");
                black_box(fetched);
            }
        });
    });
    group.finish();
}

fn bench_create_node_immediate(c: &mut Criterion) {
    // Immediate durability: each create triggers an fsync. Keep the database
    // fresh per bench setup; reuse it across iterations so we measure the
    // steady-state cost, not the open cost.
    let dir = tempdir().expect("tempdir");
    let backend = RedbBackend::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();

    let mut group = c.benchmark_group("create_node_immediate");
    group.warm_up_time(std::time::Duration::from_secs(1));
    // Longer measurement window — write benches have more variance than reads.
    group.measurement_time(std::time::Duration::from_secs(5));
    group.bench_function("default_durability", |b| {
        b.iter(|| {
            let cid = backend.put_node(black_box(&node)).expect("put");
            black_box(cid);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_get_node,
    bench_get_node_batch,
    bench_create_node_immediate
);
criterion_main!(benches);
