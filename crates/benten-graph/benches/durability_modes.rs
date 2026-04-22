//! Criterion benchmark: `RedbBackend` writes under each `DurabilityMode`.
//!
//! Phase 1 exposes three durability modes on `RedbBackend`
//! (SPIKE Next Actions #4 and implementation plan §2.2 G2 /
//! `P1.graph.durability`):
//!
//! - [`DurabilityMode::Immediate`] — fsync on every commit. Safest.
//! - [`DurabilityMode::Group`]     — *intended* to batch fsyncs across
//!   commits. Phase 1 note: redb v4 exposes only `Durability::Immediate`
//!   and `Durability::None`, so `Group` collapses to `Immediate` and
//!   emits a one-shot warning on construction. This bench **demonstrates
//!   the collapse**: numbers for `Group` should be statistically
//!   indistinguishable from `Immediate`.
//! - [`DurabilityMode::Async`]     — commit returns before the durable
//!   fsync. Test-only / ephemeral.
//!
//! ## Measurement matrix
//!
//! For every mode the bench produces three numbers:
//!
//! | Bench id                                  | What it measures                          |
//! |-------------------------------------------|-------------------------------------------|
//! | `durability_modes/single_write/<mode>`    | Latency to put 1 Node + commit            |
//! | `durability_modes/batch_100/<mode>`       | Latency to put 100 Nodes in one commit    |
//! | `durability_modes/throughput/<mode>`      | Sustained writes/sec (commits/sec) window |
//!
//! `single_write` uses the [`KVBackend::put`] surface (one commit per call)
//! which is what audit-trail / capability-grant workloads hit. `batch_100`
//! uses [`KVBackend::put_batch`] (one commit covering 100 key/value pairs)
//! which is what bulk imports and the Phase-1 test landscape exercises.
//! `throughput` calls `put` in a tight loop and divides by elapsed time —
//! the measurement is wall-clock over a `measurement_time` window, so the
//! number criterion reports as "time/iter" divided by 1 is writes/sec.
//!
//! ## Targets
//!
//! Plan §3 G3 row targets "Group bench < 500µs per write". Because Group
//! collapses to Immediate in Phase 1, this target is driven by the same
//! write-commit latency as the Immediate mode. On a reasonable SSD
//! (NVMe with filesystem fsync paths working normally) 500µs is hit; on
//! slow filesystems (cold disks, FUSE, CI runners with disk-backed
//! overlay FS) the target may not hold. The bench surfaces the measured
//! number honestly — it does not tweak the window to pass.
//!
//! `Async` mode should produce meaningfully faster single-write latency
//! than `Immediate` because `redb::Durability::None` skips the fsync.
//! Typical SSDs see 5–10x throughput delta.
//!
//! ## Interpretation
//!
//! If `Group` measurably beats `Immediate` on your hardware, something
//! changed — the collapse warning may have been bypassed, or redb grew
//! a grouped-commit variant we haven't wired. Either signal is useful.
//!
//! If `Async` does NOT beat `Immediate` by a clear margin, the
//! filesystem is ignoring fsync (tmpfs, `/dev/shm`, certain CI
//! overlays), or the workload is CPU-bound in serialization rather
//! than I/O-bound in fsync. Both are worth investigating.
//!
//! ## Phase-1 compromise note (named compromise #7 adjacent)
//!
//! The Group-collapses-to-Immediate gap is an explicit Phase-1
//! compromise. The bench proves the collapse is observable — numbers
//! for Group and Immediate fall inside each other's confidence
//! intervals. When Phase 2 revisits (if redb grows grouped-commit
//! support), this bench becomes the regression signal that the
//! amortization is actually delivering a win.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::{Duration, Instant};

use benten_core::testing::canonical_test_node;
use benten_graph::{DurabilityMode, GraphError, KVBackend, RedbBackend};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::TempDir;

/// Fresh backend in a fresh tempdir at the requested durability mode.
///
/// Returning the `TempDir` alongside the backend keeps the database file
/// alive for the duration of the bench iteration — dropping the `TempDir`
/// too early would unlink the file under an open redb handle.
fn fresh_backend(mode: DurabilityMode) -> Result<(RedbBackend, TempDir), GraphError> {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("benten.redb");
    let backend = RedbBackend::open_or_create_with_durability(path, mode)?;
    Ok((backend, dir))
}

/// Produce deterministic-but-distinct key/value pairs for a batch. Varying
/// the key avoids redb's same-key write optimization masking per-commit
/// overhead with a single in-place update.
fn make_batch(n: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
    // A canonical Node serialized once; we reuse the bytes as the "value"
    // payload so the serializer cost is amortized out of the measurement
    // (durability is what we're measuring, not canonical-CBOR encoding).
    let node = canonical_test_node();
    let value = node.canonical_bytes().expect("canonical bytes");
    (0..n)
        .map(|i| {
            let mut key = Vec::with_capacity(12);
            key.extend_from_slice(b"bench:");
            key.extend_from_slice(&(i as u32).to_be_bytes());
            (key, value.clone())
        })
        .collect()
}

/// Short label for a mode; used in BenchmarkId so `cargo bench` output is
/// grep-friendly.
fn mode_label(mode: DurabilityMode) -> &'static str {
    match mode {
        DurabilityMode::Immediate => "immediate",
        DurabilityMode::Group => "group",
        DurabilityMode::Async => "async",
    }
}

const MODES: [DurabilityMode; 3] = [
    DurabilityMode::Immediate,
    DurabilityMode::Group,
    DurabilityMode::Async,
];

/// Single-write latency: one `put` → one commit.
fn bench_single_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("durability_modes/single_write");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(50);

    for mode in MODES {
        group.bench_with_input(
            BenchmarkId::from_parameter(mode_label(mode)),
            &mode,
            |b, &mode| {
                let (backend, _dir) = fresh_backend(mode).expect("backend");
                let value = canonical_test_node()
                    .canonical_bytes()
                    .expect("canonical bytes");
                // Counter in a Cell would work; using an AtomicUsize
                // avoids interior-mut dances and still monomorphizes to a
                // plain integer increment in release.
                let counter = std::sync::atomic::AtomicUsize::new(0);

                b.iter(|| {
                    // Unique key per iteration so every call performs a
                    // real insert (not an overwrite on the same page).
                    let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let mut key = [0u8; 12];
                    key[..6].copy_from_slice(b"bench:");
                    key[6..10].copy_from_slice(&(i as u32).to_be_bytes());
                    backend
                        .put(black_box(&key), black_box(&value))
                        .expect("put");
                });
            },
        );
    }
    group.finish();
}

/// Batch-100 latency: 100 puts in one commit via `put_batch`.
fn bench_batch_100(c: &mut Criterion) {
    let mut group = c.benchmark_group("durability_modes/batch_100");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(30);
    // Throughput as elements/sec so criterion reports the derived rate.
    group.throughput(Throughput::Elements(100));

    for mode in MODES {
        group.bench_with_input(
            BenchmarkId::from_parameter(mode_label(mode)),
            &mode,
            |b, &mode| {
                // Rebuild the backend for each sample via iter_batched so
                // successive 100-key batches don't collide on the same
                // key range (every iteration writes keys 0..100 into a
                // FRESH database, modeling cold-batch cost honestly).
                b.iter_batched(
                    || {
                        let (backend, dir) = fresh_backend(mode).expect("backend");
                        let batch = make_batch(100);
                        (backend, dir, batch)
                    },
                    |(backend, _dir, batch)| {
                        backend.put_batch(black_box(&batch)).expect("put_batch");
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Sustained throughput: how many single-key commits land in a 2-second
/// window. We report the derived writes/sec rather than per-op latency so
/// the metric is comparable across modes even when per-op variance is
/// high.
///
/// The plan calls for a 5-second window. CI uses `--measurement-time 2`
/// to keep wall-clock bounded; locally, pass `--measurement-time 5` for
/// the full-spec measurement. The window is taken from criterion's own
/// `measurement_time`, so the `-- --measurement-time N` CLI flag
/// controls it uniformly.
fn bench_sustained_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("durability_modes/throughput");
    group.warm_up_time(Duration::from_millis(500));
    // Default to a 2-second window; `cargo bench -- --measurement-time 5`
    // overrides this for an on-spec measurement.
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    // Mark each iteration as "1 element" so criterion reports
    // elements/sec — i.e., writes/sec.
    group.throughput(Throughput::Elements(1));

    for mode in MODES {
        group.bench_with_input(
            BenchmarkId::from_parameter(mode_label(mode)),
            &mode,
            |b, &mode| {
                let (backend, _dir) = fresh_backend(mode).expect("backend");
                let value = canonical_test_node()
                    .canonical_bytes()
                    .expect("canonical bytes");
                // Monotonic counter so every commit targets a fresh key.
                // Using a plain `usize` in a Cell would require
                // borrow-mut; atomic is trivial and has the same codegen.
                let counter = std::sync::atomic::AtomicUsize::new(0);

                // iter_custom gives us the iters count so we can report
                // wall-clock elapsed without criterion's per-iter
                // inference skewing the throughput number.
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let mut key = [0u8; 12];
                        key[..6].copy_from_slice(b"bench:");
                        key[6..10].copy_from_slice(&(i as u32).to_be_bytes());
                        backend.put(&key, &value).expect("put");
                    }
                    start.elapsed()
                });
            },
        );
    }
    group.finish();
}

/// Gate-5 descope proof: `DurabilityMode::Group` collapses to
/// `DurabilityMode::Immediate` because redb v4 only exposes
/// `Durability::Immediate` and `Durability::None` — no grouped-commit
/// primitive. This bench is an **informational** sibling of the gated
/// `single_write` cases: it measures the two modes back-to-back within
/// one bench function so the "collapse" is visible as overlapping
/// confidence intervals in Criterion's HTML output.
///
/// **Target source:** no §14.6 number — this is arch-r1-1 compromise
/// tracking. Gate 5 (P1.graph.durability) descoped from "Group beats
/// Immediate by an amortisation factor" to "Group variant preserved as
/// forward-compat shape only."
///
/// **Gate policy:** INFORMATIONAL. Recording the collapse is the point;
/// gating would require the collapse to *persist*, which is the
/// opposite of what Phase 2 wants.
///
/// ```text
/// BENCH_ID = durability_modes/gate5_descope_proof/*
/// THRESHOLD_NS = informational
/// POLICY = informational
/// SOURCE = arch-r1-1 / plan §2.2 G1
/// ```
///
/// Phase-2 forward signal: when redb grows grouped-commit support (or
/// the benten-graph layer wires a write batcher), this bench becomes
/// the regression detector that the amortization is actually
/// delivering a win. Promotion from informational to CI-gated happens
/// in the same PR that lands the grouped path.
fn bench_gate5_descope_proof(c: &mut Criterion) {
    let mut group = c.benchmark_group("durability_modes/gate5_descope_proof");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(50);
    // INFORMATIONAL — no gate. The presence of this bench is the gate-5
    // descope surface: we measure the collapse rather than hiding it.

    for mode in [DurabilityMode::Immediate, DurabilityMode::Group] {
        group.bench_with_input(
            BenchmarkId::from_parameter(mode_label(mode)),
            &mode,
            |b, &mode| {
                let (backend, _dir) = fresh_backend(mode).expect("backend");
                let value = canonical_test_node()
                    .canonical_bytes()
                    .expect("canonical bytes");
                let counter = std::sync::atomic::AtomicUsize::new(0);

                b.iter(|| {
                    let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let mut key = [0u8; 12];
                    key[..6].copy_from_slice(b"bench:");
                    key[6..10].copy_from_slice(&(i as u32).to_be_bytes());
                    backend
                        .put(black_box(&key), black_box(&value))
                        .expect("put");
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_single_write,
    bench_batch_100,
    bench_sustained_throughput,
    bench_gate5_descope_proof
);
criterion_main!(benches);
