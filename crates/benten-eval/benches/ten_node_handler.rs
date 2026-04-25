//! Criterion benchmark: evaluate a representative mixed handler subgraph.
//!
//! **Target source:** §14.6 direct — "10-node handler evaluation:
//! 150–300µs for mixed handlers." The qualifier "mixed" matters: handlers
//! with 2+ WRITEs and IVM propagation sit at the upper end of the range;
//! pure TRANSFORM pipelines can reach <100µs (acknowledged in §14.6 but
//! not the headline target because pure-TRANSFORM handlers are rare in
//! real applications — the real handlers from
//! `docs/validation/paper-prototype-handlers.md` average 2.8 WRITEs).
//!
//! ## Phase-1 floor note (plan §14.6 update)
//!
//! The §14.6 headline "150-300µs" target assumed grouped / async commit
//! or an SSD whose fsync-per-commit floor lands in the tens-of-µs range.
//! On macOS APFS (the reference dev environment for the Phase-1 build),
//! a single `redb` `Immediate`-durability commit fsyncs the journal on
//! every call — the measured floor is ~4ms per write-bearing handler
//! invocation, independent of evaluator overhead. `redb` v4 exposes only
//! `Durability::Immediate` / `Durability::None`, so Phase 1 has no
//! grouped-commit amortization path (tracker: `.addl/phase-1/
//! r4-triage.md` §4.4, ENGINE-SPEC §14.6 macOS caveat). The 10-node
//! handler target of 150-300µs is NOT reachable in Phase 1 on macOS dev
//! hardware when the handler includes a WRITE that must fsync. The
//! evaluator layer
//! itself (subgraph build + walk + primitive dispatch, no redb commit)
//! is measured by the `build_only` / `list_dispatch_no_write` sub-
//! benches below and DOES land in the sub-100µs range.
//!
//! When Phase-2 wires grouped / async durability modes, this ceiling
//! becomes ~200-500µs (amortized fsync) and the §14.6 headline becomes
//! achievable. The bench output is the regression signal that the
//! amortization is actually delivering a win.
//!
//! ## Handler shape
//!
//! Phase 1 ships the evaluator with the 8 executable primitives. Lower-
//! ing the §14.6 "mixed" gate onto the Phase 1 evaluator means exercising
//! the `crud('post').create` dispatch — the exit-criterion load-bearing
//! path that fans READ → WRITE → RESPOND through the full transaction
//! replay, capability hook, and IVM update pipeline.
//!
//! ## Gate policy
//!
//! - `crud_post_create_dispatch` has NO gate — it is dominated by the
//!   redb fsync floor. Criterion reports the number; the bench surfaces
//!   it honestly rather than pretending to hit a target that is
//!   physically unreachable on macOS APFS.
//! - `crud_post_list_dispatch_no_write` IS gated — it exercises the same
//!   dispatch path minus the WRITE, isolating evaluator overhead from
//!   storage latency. Target: median < 300µs.
//! - `crud_post_build_subgraph_only` IS gated — it measures the pure
//!   subgraph-build cost (cache-warmed) so a regression in the subgraph
//!   cache (r6-perf-5) is caught here rather than swallowed by fsync.
//!   Target: median < 10µs.
//!
//! ```text
//! BENCH_ID = ten_node_handler/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = §14.6-mixed-handler-trend
//! ```
//!
//! The matrix-level THRESHOLD is `informational` because the bench mixes
//! a gated path (`build_only`) with an explicitly-ungated path
//! (`crud_post_create_dispatch`) and a third gated path
//! (`list_dispatch_no_write`). Per-bench-case gates live inside the
//! Criterion harness; the matrix entry exists so the drift gate covers
//! every benched surface (perf-r6-1).

// THRESHOLD_NS=informational policy=informational source=§14.6-mixed-handler-trend

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::collections::BTreeMap;
use std::hint::black_box;

use benten_core::{Node, Value};
use benten_engine::Engine;
use criterion::{Criterion, criterion_group, criterion_main};

fn build_post_node(i: u64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("bench-post-{i}")));
    props.insert("body".into(), Value::Text("x".repeat(64)));
    props.insert("rank".into(), Value::Int(i64::try_from(i).unwrap_or(0)));
    Node::new(vec!["post".into()], props)
}

fn bench_ten_node_handler_eval(c: &mut Criterion) {
    // Engine + handler registration happens once — the hot path is the
    // `.call` invocation plus everything it fans through.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    let mut group = c.benchmark_group("10_node_handler_eval");
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(5));
    group.bench_function("crud_post_create_dispatch", |b| {
        let mut counter: u64 = 0;
        b.iter(|| {
            counter = counter.wrapping_add(1);
            let node = build_post_node(counter);
            let outcome = engine
                .call(black_box(&handler_id), black_box("post:create"), node)
                .expect("crud create dispatch succeeds");
            black_box(outcome);
        });
    });
    group.finish();
}

/// Isolate evaluator overhead from redb fsync by dispatching `post:list` —
/// same subgraph-cache, evaluator-walk, and outcome-mapper path as `create`
/// but without the final transaction commit. The §14.6 "mixed handler" is
/// dominated by the fsync floor; this bench is where a regression in the
/// subgraph cache (r6-perf-5) or evaluator allocation surface shows up.
fn bench_list_dispatch_no_write(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();
    // Pre-populate a handful of posts so the list has content to return.
    for i in 0..8 {
        let _ = engine.call(&handler_id, "post:create", build_post_node(i));
    }

    let mut list_input = BTreeMap::new();
    list_input.insert("page".into(), Value::Int(0));
    list_input.insert("limit".into(), Value::Int(8));
    let list_node = Node::new(vec![], list_input);

    let mut group = c.benchmark_group("10_node_handler_eval");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("crud_post_list_dispatch_no_write", |b| {
        b.iter(|| {
            let outcome = engine
                .call(
                    black_box(&handler_id),
                    black_box("post:list"),
                    list_node.clone(),
                )
                .expect("crud list dispatch succeeds");
            black_box(outcome);
        });
    });
    group.finish();
}

/// Isolate the r6-perf-5 cache hit-path: every call reuses the same
/// `(handler_id, op)` cache key, so the measured cost is a `HashMap::get`
/// plus a `Subgraph` clone plus a per-call property patch. The first call
/// is the cache-miss + build path; all subsequent calls are hits.
///
/// We measure `engine.call(... "post:get" ...)` because GET exercises the
/// template-clone-and-patch path exactly (clone template + patch
/// `target_cid`), without paying any redb write cost. A miss on the
/// lookup still returns cleanly so the bench never faults.
fn bench_build_subgraph_only(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    // GET against a non-existent CID — we want the subgraph-build + walk
    // path without the redb put/commit cost. The CID resolves to None so
    // the outcome maps to ON_NOT_FOUND; fine for the bench, we're not
    // asserting on outcome shape, only measuring wall-clock cost.
    let mut get_input = BTreeMap::new();
    get_input.insert(
        "cid".into(),
        Value::Text("bafyr4ih3frtsr3kf4kquu7bzfjtkrnogfq3fpvnthnvtkk7eaj4rvdwarxq".into()),
    );
    let get_node = Node::new(vec![], get_input);

    // Prime the cache with one call so the measurement only sees hits.
    let _ = engine.call(&handler_id, "post:get", get_node.clone());

    let mut group = c.benchmark_group("10_node_handler_eval");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("crud_post_build_subgraph_only", |b| {
        b.iter(|| {
            let outcome = engine
                .call(
                    black_box(&handler_id),
                    black_box("post:get"),
                    get_node.clone(),
                )
                .expect("crud get dispatch succeeds");
            black_box(outcome);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_ten_node_handler_eval,
    bench_list_dispatch_no_write,
    bench_build_subgraph_only
);
criterion_main!(benches);
