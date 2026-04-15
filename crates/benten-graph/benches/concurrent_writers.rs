//! Criterion benchmark: concurrent writers against a single redb replica.
//!
//! **Target source:** §14.6 direct — "Concurrent writers (per community/
//! instance): 100–1000 writes/sec. redb single-writer serialization is a
//! hard ceiling."
//!
//! **Gate policy: informational, not gated.** This bench is explicitly
//! listed as informational in the implementation plan §4.4:
//!
//! > `concurrent_writers` — benchmark the 100–1000 writes/sec single-
//! > community ceiling. **(§14.6 direct — "Concurrent writers")** — surface
//! > in CI as **informational, not a gate**.
//!
//! Rationale: the ceiling is a hard property of redb's single-writer
//! serialization model. A regression here indicates either redb has
//! changed or our transaction shape has grown expensive — but the spec
//! acknowledges the range (100–1000) is wide because filesystem, CPU,
//! and contention all move the number within the decade. Gating on a
//! single point would create flakiness without catching real regression;
//! the *trend* across releases is the signal.
//!
//! The bench reports writes-per-second at several writer-thread counts
//! (1, 2, 4, 8) so we can see the contention curve rather than a single
//! aggregate number.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use benten_core::testing::canonical_test_node;
use benten_graph::RedbBackend;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::tempdir;

fn bench_concurrent_writers(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_writers");
    // Contention benchmarks need more samples to be meaningful; extend the
    // measurement window.
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(8));
    group.sample_size(20);

    for writer_count in [1usize, 2, 4, 8] {
        group.throughput(Throughput::Elements(writer_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(writer_count),
            &writer_count,
            |b, &writer_count| {
                b.iter_custom(|iters| {
                    let dir = tempdir().expect("tempdir");
                    let backend =
                        Arc::new(RedbBackend::open(dir.path().join("benten.redb")).expect("open"));
                    let node = canonical_test_node();
                    let per_thread = (iters as usize).div_ceil(writer_count).max(1);
                    let started = AtomicUsize::new(0);
                    let start = std::time::Instant::now();

                    thread::scope(|s| {
                        let handles: Vec<_> = (0..writer_count)
                            .map(|_| {
                                let backend = Arc::clone(&backend);
                                let node = node.clone();
                                let started = &started;
                                s.spawn(move || {
                                    // Cheap barrier: spin until every thread has
                                    // incremented `started`. Keeps all threads
                                    // contending from the first write.
                                    started.fetch_add(1, Ordering::SeqCst);
                                    while started.load(Ordering::SeqCst) < writer_count {
                                        std::hint::spin_loop();
                                    }
                                    for _ in 0..per_thread {
                                        let _ = backend.put_node(&node).expect("put");
                                    }
                                })
                            })
                            .collect();
                        for h in handles {
                            h.join().expect("join");
                        }
                    });

                    start.elapsed()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_concurrent_writers);
criterion_main!(benches);
