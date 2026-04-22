//! Phase 2a arch-r1-1 (gate-5 descope witness) informational bench.
//!
//! Phase 2a **descoped exit gate 5** per `.addl/phase-2a/r1-triage.md`
//! rationale arch-r1-1: `DurabilityMode::Group` is a no-op in redb v4 so
//! changing the default to `Group` would not alter the fsync floor on
//! macOS APFS. This bench **exists as the measurement witness** for that
//! descope — it demonstrates the Immediate-vs-Group collapse on the
//! available redb version and documents the Phase-2b / Phase-3 re-entry
//! point (see named Compromise #N+3 in `docs/SECURITY-POSTURE.md`).
//!
//! Informational (not CI-gated). The delta between `DurabilityMode::Group`
//! and `DurabilityMode::Immediate` is the measurement of interest; when the
//! delta becomes non-zero (redb exposes real grouped-commit OR Benten adds
//! its own write-batching layer), gate 5 re-enters scope.
//!
//! Phase 2a R3 red-phase: the bench routes through a Phase-2a helper that
//! `todo!()`s until G2-A wires the durability-mode-aware put path. Today's
//! iteration panics; the bench compiles.

use benten_graph::{DurabilityMode, RedbBackend};
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_group_durability(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("durability_group.redb")).unwrap();

    c.bench_function("crud_post_create_dispatch_group_durability", |b| {
        b.iter(|| {
            backend.benchmark_helper_crud_post_create_dispatch(std::hint::black_box(
                DurabilityMode::Group,
            ));
        });
    });
}

fn bench_immediate_durability(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let backend =
        RedbBackend::open_or_create(dir.path().join("durability_immediate.redb")).unwrap();

    c.bench_function("crud_post_create_dispatch_immediate_durability", |b| {
        b.iter(|| {
            backend.benchmark_helper_crud_post_create_dispatch(std::hint::black_box(
                DurabilityMode::Immediate,
            ));
        });
    });
}

criterion_group!(
    crud_post_create_dispatch_group_durability,
    bench_group_durability,
    bench_immediate_durability,
);
criterion_main!(crud_post_create_dispatch_group_durability);
