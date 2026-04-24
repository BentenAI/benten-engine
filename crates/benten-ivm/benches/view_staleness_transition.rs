//! Criterion benchmark: IVM view staleness-transition cost.
//!
//! **Target source:** plan §G11-A bench backfill — characterise the cost
//! of transitioning a View from healthy to Stale (budget-exceeded).
//!
//! **Gate policy: informational, not gated.** The transition cost is
//! dominated by the View's per-update bookkeeping; the bench exists to
//! surface a regression if the mark-stale path grows expensive (e.g. a
//! future implementation that re-scans all pending work on trip-over).
//!
//! ## Pattern
//!
//! A ContentListingView is constructed with a tiny budget (1 unit per
//! update) and fed events one at a time. The first event trips the budget
//! and flips the view to `Stale`; subsequent events should short-circuit
//! (`apply_event` skips stale views). The bench measures the
//! `route_change_event` latency across the transition so the delta between
//! "healthy update" and "already stale" is observable.
//!
//! Separately the bench also measures a single clean update on a healthy
//! view (the baseline) so regressions in the common path show up.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::collections::BTreeMap;
use std::hint::black_box;

use benten_core::{Cid, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::Subscriber;
use benten_ivm::views::ContentListingView;
use criterion::{Criterion, criterion_group, criterion_main};

fn post_event(i: u64) -> ChangeEvent {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("p{i}")));
    props.insert("createdAt".into(), Value::Int(i64::try_from(i).unwrap_or(0)));
    let node = Node::new(vec!["post".into()], props);
    let cid = node
        .cid()
        .unwrap_or_else(|_| Cid::from_blake3_digest([0; 32]));
    ChangeEvent {
        cid,
        labels: node.labels.clone(),
        kind: ChangeKind::Created,
        tx_id: i,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    }
}

fn bench_healthy_update(c: &mut Criterion) {
    c.bench_function("view_staleness_transition_healthy_update", |b| {
        let mut subscriber = Subscriber::new();
        subscriber.register_view(Box::new(ContentListingView::new("post")));
        let event = post_event(1);
        b.iter(|| {
            let applied = subscriber.route_change_event(black_box(&event)).unwrap();
            black_box(applied);
        });
    });
}

fn bench_transition_to_stale(c: &mut Criterion) {
    c.bench_function("view_staleness_transition_to_stale", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            for i in 0..iters {
                // Fresh subscriber + budget-1 view per iteration so each
                // measurement captures the actual trip-over cost.
                let mut subscriber = Subscriber::new();
                subscriber.register_view(Box::new(ContentListingView::with_budget_for_testing(1)));
                let event = post_event(i);
                let start = std::time::Instant::now();
                let _ = subscriber.route_change_event(&event).unwrap();
                total += start.elapsed();
            }
            total
        });
    });
}

fn bench_stale_short_circuit(c: &mut Criterion) {
    c.bench_function("view_staleness_transition_already_stale", |b| {
        let mut subscriber = Subscriber::new();
        subscriber.register_view(Box::new(ContentListingView::with_budget_for_testing(1)));
        // Trip the view to stale before entering the measured loop.
        let _ = subscriber.route_change_event(&post_event(0)).unwrap();
        let event = post_event(1);
        b.iter(|| {
            let applied = subscriber.route_change_event(black_box(&event)).unwrap();
            black_box(applied);
        });
    });
}

criterion_group!(
    benches,
    bench_healthy_update,
    bench_transition_to_stale,
    bench_stale_short_circuit,
);
criterion_main!(benches);
