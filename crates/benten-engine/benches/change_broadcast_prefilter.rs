//! Fwd-2 #1038 (umbrella #1194): `ChangeBroadcast` pattern-prefilter
//! scaling bench.
//!
//! Pins the core forward-readiness claim: with N narrow-pattern
//! subscribers each watching a distinct label, publishing ONE event whose
//! label matches exactly ONE subscriber must cost ~O(1) (a hash probe +
//! the single relevant callback) rather than ~O(N) (invoke every
//! subscriber + rely on each to self-filter).
//!
//! Two comparison arms at N = {16, 64, 256, 1024} prefix subscribers:
//!
//! - `prefiltered` — subscribers registered via
//!   `subscribe_fn_with_prefix`; `publish` consults the prefix index and
//!   invokes only the 1 relevant callback. Latency must stay ~flat across
//!   N (the O(1) claim).
//! - `unfiltered_baseline` — the same N subscribers registered via
//!   `subscribe_fn` (the pre-#1038 all-fan-out shape) each guarded by an
//!   in-callback label check. Latency grows ~linearly in N (the O(N)
//!   baseline the prefilter eliminates).
//!
//! The visible runtime gap between the two arms at N=1024 is the
//! machine-readable evidence that the prefilter delivers the O(1)-vs-O(N)
//! win. Informational (not CI-gated); the assertion lives in the
//! `change.rs` unit tests (`non_matching_prefix_subscriber_is_skipped`)
//! which proves the *semantic* invariant deterministically.
//!
//! Threshold (machine-readable, mirrors `bench-threshold-drift.yml`):
//!
//! ```text
//! BENCH_ID = change_broadcast_prefilter/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = refinement-audit-2026-05-#1194-fwd-2-#1038
//! ```

// THRESHOLD_NS=informational policy=informational source=refinement-audit-2026-05-#1194-fwd-2-#1038

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::testing::canonical_test_node;
use benten_engine::change::ChangeBroadcast;
use benten_graph::{ChangeEvent, ChangeKind};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn event_with_label(label: &str) -> ChangeEvent {
    let cid = canonical_test_node().cid().unwrap();
    ChangeEvent {
        cid,
        labels: vec![label.to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    }
}

fn bench_prefilter(c: &mut Criterion) {
    let counts = [16usize, 64, 256, 1024];

    // Arm 1: prefix-indexed — `publish` skips the N-1 non-matching
    // subscribers. Expected ~flat latency across N.
    let mut g1 = c.benchmark_group("change_broadcast_prefilter/prefiltered");
    for &n in &counts {
        g1.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let bc = ChangeBroadcast::new();
            let hits = Arc::new(AtomicU64::new(0));
            for i in 0..n {
                let h = Arc::clone(&hits);
                bc.subscribe_fn_with_prefix(format!("Label{i}"), move |_| {
                    h.fetch_add(1, Ordering::Relaxed);
                });
            }
            // Event matches exactly ONE of the N subscribers.
            let ev = event_with_label("Label0");
            b.iter(|| {
                bc.publish(&ev);
            });
        });
    }
    g1.finish();

    // Arm 2: unfiltered baseline — the pre-#1038 shape: all N subscribers
    // invoked, each self-filtering. Expected ~linear latency in N.
    let mut g2 = c.benchmark_group("change_broadcast_prefilter/unfiltered_baseline");
    for &n in &counts {
        g2.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let bc = ChangeBroadcast::new();
            let hits = Arc::new(AtomicU64::new(0));
            for i in 0..n {
                let want = format!("Label{i}");
                let h = Arc::clone(&hits);
                bc.subscribe_fn(move |ev: &ChangeEvent| {
                    if ev.has_label(&want) {
                        h.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
            let ev = event_with_label("Label0");
            b.iter(|| {
                bc.publish(&ev);
            });
        });
    }
    g2.finish();
}

criterion_group!(change_broadcast_prefilter, bench_prefilter);
criterion_main!(change_broadcast_prefilter);
