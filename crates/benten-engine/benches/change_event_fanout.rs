//! Phase 2a G11-A Wave 1 informational bench: change-event fan-out latency
//! at N subscribers (1, 4, 16, 64).
//!
//! Per plan §G11-A bench backfill — pins the per-event CPU overhead the
//! `ChangeBroadcast` imposes as the subscriber count scales. A live-
//! subscriber fan-out that's non-linear in N is a performance signal for
//! the IVM sync path that Phase-2b wires for WASM + SANDBOX observers.
//!
//! Methodology:
//!
//! - Build an engine with `.without_ivm()` so the only live subscriber is
//!   the `ChangeBroadcast`'s internal tap — this isolates the fan-out cost
//!   from the IVM view-maintenance cost.
//! - Attach N additional probe-style subscribers via
//!   `Engine::subscribe_change_events`.
//! - Fire a single CRUD create per iteration; criterion measures the
//!   end-to-end latency including the fan-out loop.
//!
//! Informational (not CI-gated for Phase-2a).
//!
//! Threshold (machine-readable, mirrors `bench-threshold-drift.yml`):
//!
//! ```text
//! BENCH_ID = change_event_fanout/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = plan-§G11-A-bench-backfill
//! ```

// THRESHOLD_NS=informational policy=informational source=plan-§G11-A-bench-backfill

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::Engine;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("fanout.redb"))
        .without_ivm()
        .build()
        .unwrap();
    (dir, engine)
}

fn post_node(title: &str) -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

fn bench_fanout(c: &mut Criterion) {
    let mut group = c.benchmark_group("change_event_fanout");
    for &n_subs in &[1usize, 4, 16, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(n_subs), &n_subs, |b, &n| {
            let (_dir, engine) = fresh_engine();
            // Attach N probes. Each is a cheap drain-only observer;
            // the ChangeBroadcast's fan-out loop visits each one per
            // event.
            let probes: Vec<_> = (0..n).map(|_| engine.subscribe_change_events()).collect();
            let counter = std::cell::Cell::new(0u64);

            b.iter(|| {
                let n_call = counter.get();
                counter.set(n_call.wrapping_add(1));
                let _ = engine.create_node(&post_node(&format!("fanout-{n_call}")));
                // Drain probe 0 so the observed-events buffer doesn't
                // accumulate across iterations (bounds bench memory).
                if let Some(first) = probes.first() {
                    let _ = first.drain();
                }
            });
        });
    }
    group.finish();
}

criterion_group!(change_event_fanout, bench_fanout);
criterion_main!(change_event_fanout);
