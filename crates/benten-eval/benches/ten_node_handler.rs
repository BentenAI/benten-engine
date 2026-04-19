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
//! ## Handler shape
//!
//! Phase 1 ships the evaluator with the 8 executable primitives. Lower-
//! ing the §14.6 "mixed" gate onto the Phase 1 evaluator means exercising
//! the `crud('post').create` dispatch — the exit-criterion load-bearing
//! path that fans READ → WRITE → RESPOND through the full transaction
//! replay, capability hook, and IVM update pipeline. The subgraph the
//! engine synthesizes carries the WRITE node plus the RESPOND terminal;
//! the timings below include registration-time structural validation on
//! every call (no cache), host-side CID projection, transaction replay,
//! and ChangeEvent fan-out to subscribers.
//!
//! ## Gate policy
//!
//! - Median > 300µs: CI fails.
//! - Median < 100µs: CI warns (suspiciously fast — may indicate no IVM
//!   or the cold-cache path short-circuited).

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

criterion_group!(benches, bench_ten_node_handler_eval);
criterion_main!(benches);
