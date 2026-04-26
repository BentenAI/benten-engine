//! Criterion bench: Algorithm B (`Strategy::B`) vs hand-written (`Strategy::A`),
//! per view (G8-A).
//!
//! ## R3-consolidation gating
//!
//! When `phase_2b_landed` is OFF (CI required-check default during R3), this
//! bench is a no-op `main()`. The `[[bench]] harness = false` row in
//! Cargo.toml needs the file to provide a `main`, so we cannot use a
//! crate-level `#![cfg]` to make the file empty. R5 G8-A flips the feature
//! to enable the real body. See `.addl/phase-2b/r3-consolidation.md` §4.
//!
//! ## Gate
//!
//! Algorithm B must run within **20%** of the corresponding hand-written
//! baseline for each of the 5 Phase-1 views. Threshold values live in the
//! per-bench `bench_thresholds.toml` (per Phase-2a bench precedent — numeric
//! thresholds belong in `.toml` config, not in code).
//!
//! ## Bench order (perf-risk descending per `r1-ivm-algorithm.json`)
//!
//! 1. `content_listing` — HIGH (~25-35% B overhead expected; first to bench
//!    because it is the gate-breaker if B can't be made competitive).
//! 2. `governance_inheritance` — MEDIUM-HIGH (transitive closure).
//! 3. `version_current` — MEDIUM.
//! 4. `capability_grants` — LOW.
//! 5. `event_handler_dispatch` — LOW.
//!
//! ```text
//! BENCH_ID = algorithm_b_vs_handwritten/*
//! THRESHOLD = b_within_20pct_of_a (per-view; ratio gate, not ns ceiling)
//! POLICY = gated
//! SOURCE = .addl/phase-2b/00-implementation-plan.md §3 G8-A
//! ```

// THRESHOLD_NS=informational policy=ratio_gate source=§G8-A-b-within-20pct-of-a

#[cfg(not(feature = "phase_2b_landed"))]
fn main() {
    // R3-consolidation no-op: bench body lives in `landed` module below,
    // gated on `phase_2b_landed` feature. R5 G8-A enables.
}

#[cfg(feature = "phase_2b_landed")]
mod landed {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "benches may use unwrap/expect per workspace policy"
    )]

    use std::collections::BTreeMap;
    use std::hint::black_box;

    use benten_core::{Cid, Node, Value};
    use benten_graph::{ChangeEvent, ChangeKind};
    use benten_ivm::View;
    use benten_ivm::algorithm_b::AlgorithmBView;
    use benten_ivm::views::{
        CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
        VersionCurrentView,
    };
    use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

    const SEED_COUNT: u64 = 256;

    fn post_node(title: &str, created_at: i64) -> Node {
        let mut props = BTreeMap::new();
        props.insert("title".into(), Value::Text(title.into()));
        props.insert("createdAt".into(), Value::Int(created_at));
        Node::new(vec!["post".into()], props)
    }

    fn cap_grant_node(scope: &str) -> Node {
        let mut props = BTreeMap::new();
        props.insert(
            "grantee".into(),
            Value::Bytes(Cid::from_blake3_digest([0u8; 32]).as_bytes().to_vec()),
        );
        props.insert("scope".into(), Value::Text(scope.into()));
        Node::new(vec!["CapabilityGrant".into()], props)
    }

    fn handler_node(subscribes_to: &str) -> Node {
        let mut props = BTreeMap::new();
        props.insert("subscribes_to".into(), Value::Text(subscribes_to.into()));
        Node::new(vec!["Handler".into()], props)
    }

    fn governance_node(parent: Option<Cid>) -> Node {
        let mut props = BTreeMap::new();
        if let Some(p) = parent {
            props.insert("parent".into(), Value::Bytes(p.as_bytes().to_vec()));
        }
        Node::new(vec!["Governance".into()], props)
    }

    fn version_node(anchor: Cid, revision: i64) -> Node {
        let mut props = BTreeMap::new();
        props.insert("anchor".into(), Value::Bytes(anchor.as_bytes().to_vec()));
        props.insert("revision".into(), Value::Int(revision));
        Node::new(vec!["Version".into()], props)
    }

    fn bench_pair<MA, MB>(
        c: &mut Criterion,
        group_name: &str,
        events: &[ChangeEvent],
        mut make_a: MA,
        mut make_b: MB,
    ) where
        MA: FnMut() -> Box<dyn View>,
        MB: FnMut() -> Box<dyn View>,
    {
        let mut group = c.benchmark_group(group_name);
        group.bench_with_input(BenchmarkId::new("strategy", "A"), &events, |b, evs| {
            b.iter(|| {
                let mut v = make_a();
                for ev in evs.iter() {
                    let _ = v.update(black_box(ev));
                }
                black_box(v);
            });
        });
        group.bench_with_input(BenchmarkId::new("strategy", "B"), &events, |b, evs| {
            b.iter(|| {
                let mut v = make_b();
                for ev in evs.iter() {
                    let _ = v.update(black_box(ev));
                }
                black_box(v);
            });
        });
        group.finish();
    }

    fn bench_content_listing(c: &mut Criterion) {
        let events: Vec<ChangeEvent> = (0..SEED_COUNT)
            .map(|i| {
                ChangeEvent::new_node(
                    Cid::from_blake3_digest([0u8; 32]),
                    vec!["post".into()],
                    ChangeKind::Created,
                    i + 1,
                    Some(post_node(&format!("post-{i}"), i as i64 * 100)),
                )
            })
            .collect();

        bench_pair(
            c,
            "algorithm_b_vs_handwritten/content_listing",
            &events,
            || Box::new(ContentListingView::new("post")),
            || {
                Box::new(AlgorithmBView::for_id(
                    "content_listing",
                    ContentListingView::definition(),
                ))
            },
        );
    }

    fn bench_governance_inheritance(c: &mut Criterion) {
        let root = Cid::from_blake3_digest([0u8; 32]);
        let events: Vec<ChangeEvent> = (0..SEED_COUNT)
            .map(|i| {
                let parent = if i == 0 { None } else { Some(root) };
                ChangeEvent::new_node(
                    Cid::from_blake3_digest([0u8; 32]),
                    vec!["Governance".into()],
                    ChangeKind::Created,
                    i + 1,
                    Some(governance_node(parent)),
                )
            })
            .collect();

        bench_pair(
            c,
            "algorithm_b_vs_handwritten/governance_inheritance",
            &events,
            || Box::new(GovernanceInheritanceView::new()),
            || {
                Box::new(AlgorithmBView::for_id(
                    "governance_inheritance",
                    GovernanceInheritanceView::definition(),
                ))
            },
        );
    }

    fn bench_version_current(c: &mut Criterion) {
        let anchor = Cid::from_blake3_digest([0u8; 32]);
        let events: Vec<ChangeEvent> = (0..SEED_COUNT)
            .map(|i| {
                ChangeEvent::new_node(
                    Cid::from_blake3_digest([0u8; 32]),
                    vec!["Version".into()],
                    ChangeKind::Created,
                    i + 1,
                    Some(version_node(anchor, i as i64 + 1)),
                )
            })
            .collect();

        bench_pair(
            c,
            "algorithm_b_vs_handwritten/version_current",
            &events,
            || Box::new(VersionCurrentView::new()),
            || {
                Box::new(AlgorithmBView::for_id(
                    "version_current",
                    VersionCurrentView::definition(),
                ))
            },
        );
    }

    fn bench_capability_grants(c: &mut Criterion) {
        let events: Vec<ChangeEvent> = (0..SEED_COUNT)
            .map(|i| {
                ChangeEvent::new_node(
                    Cid::from_blake3_digest([0u8; 32]),
                    vec!["CapabilityGrant".into()],
                    ChangeKind::Created,
                    i + 1,
                    Some(cap_grant_node(if i % 2 == 0 {
                        "read:post"
                    } else {
                        "write:post"
                    })),
                )
            })
            .collect();

        bench_pair(
            c,
            "algorithm_b_vs_handwritten/capability_grants",
            &events,
            || Box::new(CapabilityGrantsView::new()),
            || {
                Box::new(AlgorithmBView::for_id(
                    "capability_grants",
                    CapabilityGrantsView::definition(),
                ))
            },
        );
    }

    fn bench_event_handler_dispatch(c: &mut Criterion) {
        let events: Vec<ChangeEvent> = (0..SEED_COUNT)
            .map(|i| {
                ChangeEvent::new_node(
                    Cid::from_blake3_digest([0u8; 32]),
                    vec!["Handler".into()],
                    ChangeKind::Created,
                    i + 1,
                    Some(handler_node(if i % 2 == 0 {
                        "post.created"
                    } else {
                        "post.deleted"
                    })),
                )
            })
            .collect();

        bench_pair(
            c,
            "algorithm_b_vs_handwritten/event_handler_dispatch",
            &events,
            || Box::new(EventDispatchView::new()),
            || {
                Box::new(AlgorithmBView::for_id(
                    "event_dispatch",
                    EventDispatchView::definition(),
                ))
            },
        );
    }

    criterion_group!(
        benches,
        bench_content_listing,
        bench_governance_inheritance,
        bench_version_current,
        bench_capability_grants,
        bench_event_handler_dispatch,
    );
    // criterion_main expands to a `pub fn main()` inside this mod.
    criterion_main!(benches);
}

#[cfg(feature = "phase_2b_landed")]
fn main() {
    landed::main();
}
