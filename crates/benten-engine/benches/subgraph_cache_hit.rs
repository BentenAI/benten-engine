//! Phase 2a G2-B / arch-r1-5 informational bench: AST cache hit/miss latency
//! for the `(handler_id, op, subgraph_cid)` key tuple.
//!
//! Three measurement cases:
//! 1. Cold path — no cache entry; full subgraph decode.
//! 2. Warm path — cache hit; O(1) lookup. Target <5µs per plan §4.4.
//! 3. Invalidation after re-registration under a different `subgraph_cid`
//!    (dx-r1-5 contract).
//!
//! R3 red-phase: the bench body routes through
//! `Engine::benchmark_helper_subgraph_cache_*` stubs that `todo!()` until
//! G2-B lands the cache wire-through. The bench compiles today; iteration
//! panics when the inner helper is invoked.
//!
//! Informational (not CI-gated); R5 G2-B flips it live once the cache
//! round-trip is wired.
//!
//! ```text
//! BENCH_ID = subgraph_cache_hit/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = plan-§4.4-G2-B-cache-warm-path
//! ```

// THRESHOLD_NS=informational policy=informational source=plan-§4.4-G2-B-cache-warm-path

use benten_engine::Engine;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_cold_path(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("cache_cold.redb")).unwrap();
    c.bench_function("subgraph_cache_cold_path", |b| {
        b.iter(|| {
            engine.benchmark_helper_subgraph_cache_cold(
                std::hint::black_box("h_cold"),
                std::hint::black_box("run"),
            );
        });
    });
}

fn bench_warm_path(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("cache_warm.redb")).unwrap();
    // Pre-warm via the helper so the measured iteration is a cache hit.
    let () = engine.benchmark_helper_subgraph_cache_prewarm("h_warm", "run");
    c.bench_function("subgraph_cache_warm_path_hit", |b| {
        b.iter(|| {
            engine.benchmark_helper_subgraph_cache_warm(
                std::hint::black_box("h_warm"),
                std::hint::black_box("run"),
            );
        });
    });
}

fn bench_invalidation(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("cache_inv.redb")).unwrap();
    let () = engine.benchmark_helper_subgraph_cache_prewarm("h_inv", "run");
    c.bench_function("subgraph_cache_invalidation_after_reregister", |b| {
        b.iter(|| {
            engine.benchmark_helper_subgraph_cache_reregister_and_miss(
                std::hint::black_box("h_inv"),
                std::hint::black_box("run"),
            );
        });
    });
}

criterion_group!(
    subgraph_cache_hit,
    bench_cold_path,
    bench_warm_path,
    bench_invalidation,
);
criterion_main!(subgraph_cache_hit);
