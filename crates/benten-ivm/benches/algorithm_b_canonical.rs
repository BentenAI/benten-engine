//! G15-A canonical-view fast-path preservation bench.
//!
//! ## Purpose
//!
//! Measures the wallclock cost of incrementally maintaining the
//! `content_listing` canonical view under two register paths:
//!
//! - `Strategy_B_baseline` — the Phase-2b shipping shape: construct via
//!   [`benten_ivm::algorithm_b::AlgorithmBView::for_id`] (the canonical
//!   constructor that wraps the hand-written inner kernel directly).
//! - `post_g15a` — the post-G15-A generalized register path: construct
//!   via [`benten_ivm::Algorithm::register`] which routes through
//!   [`benten_ivm::dispatch_for`] (canonical id → canonical fast-path
//!   classification → same hand-written inner kernel).
//!
//! The companion test
//! `crates/benten-ivm/tests/algorithm_b_general.rs::algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline`
//! parses the criterion JSON output at the LOAD-BEARING paths
//! `target/criterion/algorithm_b_canonical_view_fast_path/Strategy_B_baseline/new/estimates.json`
//! + `.../post_g15a/new/estimates.json` and asserts ratio <= 1.20.
//!
//! At G15-A landing the gate is INFORMATIONAL (matches Phase-2b
//! precedent for new bench gates); promoted to required at R6 once
//! baseline measurements are stable.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::{
    Algorithm, AlgorithmBView, LabelPattern, Projection, Strategy, View, ViewDefinition,
};
use criterion::{Criterion, criterion_group, criterion_main};

const CORPUS_SIZE: usize = 64;

fn make_event(idx: u64) -> ChangeEvent {
    let mut props = BTreeMap::new();
    props.insert(String::from("seq"), Value::Int(idx as i64));
    props.insert(String::from("createdAt"), Value::Int(idx as i64));
    let node = Node::new(vec!["post".to_string()], props);
    let cid = node.cid().unwrap();
    ChangeEvent {
        cid,
        labels: vec!["post".to_string()],
        kind: ChangeKind::Created,
        tx_id: idx,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    }
}

fn corpus() -> Vec<ChangeEvent> {
    (0..CORPUS_SIZE as u64).map(make_event).collect()
}

fn algorithm_b_canonical_view_fast_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_b_canonical_view_fast_path");

    let events = corpus();

    // Strategy::B baseline — direct AlgorithmBView::for_id construction.
    group.bench_function("Strategy_B_baseline", |b| {
        b.iter(|| {
            let definition = ViewDefinition {
                view_id: "content_listing".to_string(),
                input_pattern_label: Some("post".to_string()),
                output_label: "system:IVMView".to_string(),
                strategy: Strategy::B,
            };
            let mut view = AlgorithmBView::for_id("content_listing", definition).unwrap();
            for e in &events {
                view.update(e).unwrap();
            }
            std::hint::black_box(&view);
        });
    });

    // post-G15-A generalized register path — same canonical fast-path
    // classification, routed through the new `Algorithm::register`
    // surface.
    group.bench_function("post_g15a", |b| {
        b.iter(|| {
            let mut view = Algorithm::register(
                "content_listing",
                LabelPattern::exact("post"),
                Projection::all_props(),
            )
            .unwrap();
            for e in &events {
                view.update(e).unwrap();
            }
            std::hint::black_box(&view);
        });
    });

    group.finish();
}

criterion_group!(benches, algorithm_b_canonical_view_fast_path);
criterion_main!(benches);
