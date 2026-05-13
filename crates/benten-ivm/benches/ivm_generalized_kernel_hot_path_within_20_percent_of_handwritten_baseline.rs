//! G23-0a IVM-subgraph generalization: kernel hot-path within 20% of
//! hand-written baseline.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.2 row 5
//! (mat-r1-2).
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a.
//!
//! ## What this measures
//!
//! Two register paths over the **same** canonical view + corpus:
//!
//! - `handwritten_baseline` — pre-G23-0a shape, hand-written
//!   `ContentListingView` driven directly by `View::update` per event.
//! - `subgraph_generalized` — post-G23-0a shape,
//!   `Algorithm::register_subgraph(SubgraphSpec::for_canonical_view(...))`
//!   walking the same event stream. The generalized kernel routes the
//!   canonical view id through the fast-path classification per
//!   `algorithm_b.rs::dispatch_for`, then walks the schema-shaped
//!   subgraph through the engine evaluator's primitive dispatch.
//!
//! Threshold (informational at G23-0a landing per pim-2 §3.6b
//! companion-bench precedent; promoted to required at R6 once
//! baseline measurements are stable):
//!
//! ```text
//! BENCH_ID = ivm_generalized_kernel_hot_path/*
//! THRESHOLD = subgraph_generalized <= 1.20 * handwritten_baseline
//! POLICY = informational at G23-0a; required at R6
//! SOURCE = .addl/phase-4-foundation/r2-test-landscape.md §2.2 + mat-r1-2
//! ```
//!
//! ## RED-PHASE skeleton
//!
//! The bench body is a stub at R3 RED-PHASE — the actual
//! `Algorithm::register_subgraph` + `SubgraphSpec::for_canonical_view`
//! production surface lands at R5 G23-0a. The companion gate test
//! (R5 implementer adds at landing) parses criterion JSON output
//! at `target/criterion/ivm_generalized_kernel_hot_path/<axis>/new/
//! estimates.json` per `benten_ivm::testing::criterion_estimates_mean_ns`
//! convention.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

mod inner {
    use std::collections::BTreeMap;
    use std::hint::black_box;

    use benten_core::{Node, Value};
    use benten_graph::{ChangeEvent, ChangeKind};
    use benten_ivm::View;
    use benten_ivm::views::ContentListingView;
    use criterion::{Criterion, criterion_group};

    /// Corpus size — matches the existing `algorithm_b_canonical` bench
    /// for cross-bench comparability.
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

    /// G23-0a kernel hot-path comparison bench.
    pub fn ivm_generalized_kernel_hot_path(c: &mut Criterion) {
        let mut group = c.benchmark_group("ivm_generalized_kernel_hot_path");

        let events = corpus();

        // Baseline: hand-written ContentListingView. Pre-G23-0a shape.
        group.bench_function("handwritten_baseline", |b| {
            b.iter(|| {
                let mut view = ContentListingView::new("post");
                for e in &events {
                    let _ = view.update(e);
                }
                black_box(&view);
            });
        });

        // Generalized: post-G23-0a `Algorithm::register_subgraph`. R5
        // implementer wires the production call here:
        //
        //   use benten_ivm::{Algorithm, SubgraphSpec};
        //   let spec = SubgraphSpec::for_canonical_view("content_listing")
        //       .with_label_pattern_exact("post");
        //   let mut view = Algorithm::register_subgraph(spec).unwrap();
        //   for e in &events { view.update(e).unwrap(); }
        //
        // RED-PHASE skeleton drives the hand-written view AGAIN so the
        // bench compiles + criterion output JSON shape is valid. The
        // ratio measurement is meaningful only after R5 landing —
        // until then both arms measure the same workload + the gate
        // is trivially within 20%. The gate test the R5 implementer
        // adds at landing parses these JSON outputs.
        group.bench_function("subgraph_generalized", |b| {
            b.iter(|| {
                let mut view = ContentListingView::new("post");
                for e in &events {
                    let _ = view.update(e);
                }
                black_box(&view);
            });
        });

        group.finish();
    }

    criterion_group!(benches, ivm_generalized_kernel_hot_path);
}

criterion::criterion_main!(inner::benches);
