//! Phase 2a G11-A Wave 1 informational bench: 80/20 read/write mixed workload
//! against the subgraph AST cache.
//!
//! Closes the G2-B mini-review note on cold-path fidelity: the original
//! `subgraph_cache_hit` bench exercised the cold path via a synthetic probe
//! helper that skipped the real `SubgraphBuilder` + property-patch work a
//! live dispatch performs. This bench drives `engine.call(...)` directly so
//! every iteration runs the complete cache-miss → SubgraphBuilder →
//! build_unvalidated_for_test → cache-insert pipeline. The 80/20 mix
//! matches the "readers dominate" workload the plan names for a typical
//! CRUD front-end.
//!
//! Three measurement cases:
//!
//! 1. `subgraph_cache_cold_populate` — cold fresh handler; one dispatch per
//!    iteration; every call is a miss because the handler_id rotates. Fixes
//!    the G2-B fidelity gap.
//! 2. `subgraph_cache_80_20_mixed` — 80% reads / 20% writes against a
//!    stable handler. Reads hit the cache, writes go through the stamped-
//!    property WRITE path.
//! 3. `subgraph_cache_pure_read_hit` — all reads; pure hit path for an
//!    upper-bound measurement. The warm-path claim in plan §4.4 is pinned
//!    against this.
//!
//! Informational (not CI-gated for Phase-2a).

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::Engine;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("bench.redb")).unwrap();
    (dir, engine)
}

fn seed_post(engine: &Engine, title: &str) {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    let node = Node::new(vec!["post".into()], props);
    let _ = engine.create_node(&node);
}

fn bench_cold_populate(c: &mut Criterion) {
    c.bench_function("subgraph_cache_cold_populate", |b| {
        b.iter_batched(
            || {
                // Fresh engine per-iteration so the cache is cold. This is
                // what the G2-B mini-review meant by "real cold-path
                // fidelity" — the helper-based variant only fakes it.
                let (dir, engine) = fresh_engine();
                let handler = engine.register_crud("post").unwrap();
                (dir, engine, handler)
            },
            |(dir, engine, handler)| {
                // One list call exercises the full cache-miss pipeline.
                let _ = engine.call(&handler, "list", Node::empty());
                std::hint::black_box(dir);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_80_20_mixed(c: &mut Criterion) {
    let (_dir, engine) = fresh_engine();
    let handler = engine.register_crud("post").unwrap();
    // Seed a handful of posts so the list has non-trivial content.
    for i in 0..8 {
        seed_post(&engine, &format!("seed-{i}"));
    }

    // Counter so the 80/20 mix alternates deterministically.
    let counter = std::cell::Cell::new(0u32);
    c.bench_function("subgraph_cache_80_20_mixed", |b| {
        b.iter(|| {
            let n = counter.get();
            counter.set(n.wrapping_add(1));
            if n.is_multiple_of(5) {
                // 20% writes.
                let mut props: BTreeMap<String, Value> = BTreeMap::new();
                props.insert("title".into(), Value::Text(format!("bench-write-{n}")));
                let node = Node::new(vec!["post".into()], props);
                let _ = engine.call("crud:post", "create", node);
            } else {
                // 80% reads.
                let _ = engine.call("crud:post", "list", Node::empty());
            }
        });
    });
}

fn bench_pure_read_hit(c: &mut Criterion) {
    let (_dir, engine) = fresh_engine();
    let handler = engine.register_crud("post").unwrap();
    seed_post(&engine, "warm");
    // Warm the list cache so the first iteration is a hit.
    let _ = engine.call(&handler, "list", Node::empty());
    c.bench_function("subgraph_cache_pure_read_hit", |b| {
        b.iter(|| {
            let _ = engine.call("crud:post", "list", Node::empty());
        });
    });
}

criterion_group!(
    subgraph_cache_hit_80_20_mixed,
    bench_cold_populate,
    bench_80_20_mixed,
    bench_pure_read_hit,
);
criterion_main!(subgraph_cache_hit_80_20_mixed);
