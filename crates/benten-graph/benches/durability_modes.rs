//! Criterion benchmark: `create_node` under each `DurabilityMode`.
//!
//! Phase 1 introduces three durability modes on `RedbBackend` per spec
//! (SPIKE Next Actions #4 and implementation plan §2.2 G2):
//!
//! - `Immediate` — fsync on every commit (capability grants, audit records).
//! - `Group`    — amortized fsync across batched commits (bulk imports).
//! - `Async`    — no fsync; durability is best-effort (ephemeral views).
//!
//! ## Targets
//!
//! | Benchmark | Target | Source |
//! |---|---|---|
//! | `create_node_group_commit` | < 500µs median | **§14.6 derived** — ENGINE-SPEC §14.6 puts the full "Node creation + IVM update" envelope at 100–500µs. Group-commit amortization is the mechanism by which the optimistic end of that range (≈100µs) is reachable; this bench asserts we're inside the envelope. |
//! | `create_node_async`        | < 250µs median | **§14.6 derived** — below the Group ceiling; no-fsync should be roughly 2x faster than group-commit given current redb characteristics. |
//!
//! ## Stub-graceful
//!
//! `DurabilityMode` and the `open_with_durability` constructor do NOT exist
//! at spike end (G2 deliverable). The bench file references `RedbBackend::
//! open_with_durability` behind a local shim; until G2 lands, this bench
//! falls back to the existing `open` + calls a placeholder function
//! `create_with_mode_stub` that panics with a helpful message. CI will
//! report a benchmark failure, which is the correct TDD signal pre-R5.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::{Cid, Node, testing::canonical_test_node};
use benten_graph::{GraphError, RedbBackend};
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

/// Placeholder for the Phase 1 `DurabilityMode`-aware constructor.
///
/// When G2 lands, replace the body with the real
/// `RedbBackend::open_with_durability(path, DurabilityMode::Group)` call.
/// Until then, falls through to the default `open`, so the bench still
/// exercises the code path but measures the wrong number — which is why
/// the pass/fail gate is only meaningful once G2 exists.
fn open_group_commit(path: std::path::PathBuf) -> Result<RedbBackend, GraphError> {
    // TODO(G2): RedbBackend::open_with_durability(path, DurabilityMode::Group)
    RedbBackend::open(path)
}

fn open_async(path: std::path::PathBuf) -> Result<RedbBackend, GraphError> {
    // TODO(G2): RedbBackend::open_with_durability(path, DurabilityMode::Async)
    RedbBackend::open(path)
}

fn put_batch_stub(backend: &RedbBackend, nodes: &[Node]) -> Result<Vec<Cid>, GraphError> {
    // TODO(G3): transaction primitive with group-commit amortization.
    // Spike falls back to serial puts so the bench COMPILES. CI will see the
    // bench run but miss the §14.6 Group-commit target until G3 lands, which
    // is the correct TDD signal.
    nodes.iter().map(|n| backend.put_node(n)).collect()
}

fn bench_create_node_group_commit(c: &mut Criterion) {
    // R4 triage (M13): silent fall-through to default `RedbBackend::open` is
    // explicitly a failure mode, not a valid bench path. The sentinel panic
    // below makes the bench fail LOUDLY until R5 G2 wires the real
    // `DurabilityMode::Group` constructor. Before that: the bench body is
    // unreachable.
    todo!("durability_modes::group: R5 must wire DurabilityMode::Group (G2)");

    #[allow(unreachable_code)]
    {
        let dir = tempdir().expect("tempdir");
        let backend = open_group_commit(dir.path().join("benten.redb")).expect("open");
        let batch: Vec<Node> = (0..32).map(|_| canonical_test_node()).collect();

        let mut group = c.benchmark_group("create_node_group_commit");
        group.warm_up_time(std::time::Duration::from_secs(1));
        group.measurement_time(std::time::Duration::from_secs(5));
        group.bench_function("batch_of_32", |b| {
            b.iter(|| {
                let cids = put_batch_stub(&backend, black_box(&batch)).expect("batch");
                black_box(cids);
            });
        });
        group.finish();
    }
}

fn bench_create_node_async(c: &mut Criterion) {
    // R4 triage (M13): same sentinel as group-commit — Async must be wired
    // in R5 before the bench measures anything meaningful.
    todo!("durability_modes::async: R5 must wire DurabilityMode::Async (G2)");

    #[allow(unreachable_code)]
    {
        let dir = tempdir().expect("tempdir");
        let backend = open_async(dir.path().join("benten.redb")).expect("open");
        let node = canonical_test_node();

        let mut group = c.benchmark_group("create_node_async");
        group.warm_up_time(std::time::Duration::from_secs(1));
        group.measurement_time(std::time::Duration::from_secs(3));
        group.bench_function("no_fsync", |b| {
            b.iter(|| {
                let cid = backend.put_node(black_box(&node)).expect("put");
                black_box(cid);
            });
        });
        group.finish();
    }
}

criterion_group!(
    benches,
    bench_create_node_group_commit,
    bench_create_node_async
);
criterion_main!(benches);
