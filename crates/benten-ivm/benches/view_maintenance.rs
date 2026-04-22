//! Criterion benchmarks: IVM view read + incremental maintenance.
//!
//! Two §14.6 direct/derived gates at the IVM layer:
//!
//! | Benchmark | Target | Source |
//! |---|---|---|
//! | `view_read_content_listing`     | < 1µs hot cache       | **§14.6 direct** — "IVM view read (clean): 0.04–1µs" |
//! | `view_incremental_maintenance`  | < 50µs per write      | **§14.6 derived** — ENGINE-SPEC §14.6 puts the full "Node creation + IVM update" envelope at 100–500µs. Decomposition: reserve ~50µs for the IVM slice and the balance for storage (redb put) + hashing (DAG-CBOR + BLAKE3). |
//!
//! ## Pattern
//!
//! - `view_read_content_listing` exercises View 3 (content listing paginated
//!   by `createdAt`). The view is pre-seeded with a representative entry
//!   count so the read measures the hot-cache path.
//! - `view_incremental_maintenance` measures one `ChangeEvent` fanning out
//!   to the Subscriber's registered views (all 5 Phase-1 hand-written views
//!   attached). Events alternate matching / non-matching labels so the
//!   dispatch pattern-match gets exercised as well as the apply path.

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
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};
use criterion::{Criterion, criterion_group, criterion_main};

// Number of entries to seed the content-listing view with before measuring
// the read hot-cache path. Chosen to be representative of a "has been
// running for a while" state without blowing up bench warm-up time.
const SEED_COUNT: u64 = 512;

fn post_node(title: &str, created_at: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    props.insert("createdAt".into(), Value::Int(created_at));
    Node::new(vec!["post".into()], props)
}

fn other_node(label: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("filler".into(), Value::Text("x".into()));
    Node::new(vec![label.into()], props)
}

fn build_seeded_content_view() -> ContentListingView {
    let mut view = ContentListingView::new("post");
    for i in 0..SEED_COUNT {
        view.on_change(post_node(&format!("p{i}"), i64::try_from(i).unwrap_or(0)));
    }
    view
}

fn build_populated_subscriber() -> Subscriber {
    // Attach all five Phase-1 views so one ChangeEvent fans to every
    // dispatch path.
    Subscriber::new()
        .with_view(Box::new(CapabilityGrantsView::new()))
        .with_view(Box::new(EventDispatchView::new()))
        .with_view(Box::new(build_seeded_content_view()))
        .with_view(Box::new(GovernanceInheritanceView::new()))
        .with_view(Box::new(VersionCurrentView::new()))
}

fn change_event_for(node: Node, tx_id: u64) -> ChangeEvent {
    // Re-derive the Node's CID so the event carries a real identifier;
    // IVM views that read `event.cid` for keying see a stable value.
    let cid = node
        .cid()
        .unwrap_or_else(|_| Cid::from_blake3_digest([0; 32]));
    ChangeEvent::new_node(
        cid,
        node.labels.clone(),
        ChangeKind::Created,
        tx_id,
        Some(node),
    )
}

fn bench_view_read_content_listing(c: &mut Criterion) {
    let view = build_seeded_content_view();
    let mut group = c.benchmark_group("view_read_content_listing");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache_page_20", |b| {
        b.iter(|| {
            let page = view
                .read_page(black_box(0), black_box(20))
                .expect("view fresh");
            black_box(page);
        });
    });
    group.finish();
}

fn bench_view_read_governance_inheritance(c: &mut Criterion) {
    // View 4 — governance inheritance. §14.6 direct: "IVM view read (clean):
    // 0.04–1µs" applies to HashMap/sorted-list strategies. View 4 walks the
    // effective-rules transitive closure, which is maintained at write time
    // so the read remains O(1). Gate target: < 1µs median hot cache.
    let view = GovernanceInheritanceView::new();
    // A stable entity CID for the lookup — same CID across iterations so
    // the bench measures the hot-cache miss path (entity not in index),
    // which is O(1) per Phase 1 contract.
    let entity = Cid::from_blake3_digest(*blake3::hash(b"bench-entity").as_bytes());
    let mut group = c.benchmark_group("view_read_governance_inheritance");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("hot_cache_empty_closure", |b| {
        b.iter(|| {
            let result = view.read_effective_rules(black_box(&entity));
            black_box(result).ok();
        });
    });
    group.finish();
}

fn bench_view_incremental_maintenance(c: &mut Criterion) {
    let mut subscriber = build_populated_subscriber();
    let mut group = c.benchmark_group("view_incremental_maintenance");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("all_five_views_per_write", |b| {
        let mut counter: u64 = 0;
        b.iter(|| {
            counter = counter.wrapping_add(1);
            // Alternate matching/non-matching labels so the dispatch
            // pattern-match path gets exercised alongside the apply path.
            let node = if counter.is_multiple_of(2) {
                post_node(&format!("bp{counter}"), i64::try_from(counter).unwrap_or(0))
            } else {
                other_node("unrelated")
            };
            let event = change_event_for(node, counter);
            let applied = subscriber
                .route_change_event(black_box(&event))
                .expect("route_change_event returns Ok");
            black_box(applied);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_view_read_content_listing,
    bench_view_read_governance_inheritance,
    bench_view_incremental_maintenance
);
criterion_main!(benches);
