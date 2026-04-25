//! Criterion benchmark: MVCC read latency during contended writes.
//!
//! **Target source:** ENGINE-SPEC §14.6 — "reads stay fast under
//! contention". redb's MVCC model allows multiple concurrent readers
//! alongside a single writer; this bench characterises the read latency
//! envelope as background writer threads compete for the write lock.
//!
//! **Gate policy: informational, not gated.** The absolute read latency
//! depends on filesystem, CPU, and redb version; the shape of the curve
//! (roughly flat across 0 / 1 / 4 / 16 contending writers, per redb's MVCC
//! guarantee) is the signal. A regression that shows reads SLOWING as
//! writer count grows would indicate the MVCC contract has been broken —
//! that's the trend the bench exists to catch over releases.
//!
//! ## Pattern
//!
//! - One reader thread issues `get_node(cid)` calls on a pre-populated CID
//!   in a tight loop; criterion measures the per-read wall-clock.
//! - N background writer threads (parameterised: 0, 1, 4, 16) spin-loop
//!   writing unique nodes to generate write-lock contention.
//! - Writer threads terminate after the measured interval.
//!
//! Under redb's MVCC, the reader should see a flat latency line regardless
//! of N — readers go through `begin_read` which opens a snapshot, not the
//! single-writer lock.
//!
//! ```text
//! BENCH_ID = mvcc_read_latency/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = §14.6-mvcc-reads-stay-fast-trend
//! ```

// THRESHOLD_NS=informational policy=informational source=§14.6-mvcc-reads-stay-fast-trend

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use benten_core::{Node, Value, testing::canonical_test_node};
use benten_graph::RedbBackend;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

/// Build a uniquely-keyed Node so concurrent writers don't collide on the
/// same CID (content-addressed storage dedupes identical content).
fn unique_node(seed: u64) -> Node {
    let mut props = BTreeMap::new();
    props.insert(
        "seed".to_string(),
        Value::Int(i64::try_from(seed).unwrap_or(0)),
    );
    Node::new(vec!["bench-writer".to_string()], props)
}

fn bench_mvcc_read_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("mvcc_read_latency");
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(6));
    group.sample_size(30);

    for writer_count in [0usize, 1, 4, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(writer_count),
            &writer_count,
            |b, &writer_count| {
                let dir = tempdir().expect("tempdir");
                let backend =
                    Arc::new(RedbBackend::open(dir.path().join("benten.redb")).expect("open"));
                // Pre-populate a single Node to read repeatedly.
                let target = canonical_test_node();
                let target_cid = backend.put_node(&target).expect("put_node");

                // Start background writers that keep the write lock busy.
                let stop = Arc::new(AtomicBool::new(false));
                let mut writer_handles = Vec::with_capacity(writer_count);
                for w in 0..writer_count {
                    let backend = Arc::clone(&backend);
                    let stop = Arc::clone(&stop);
                    writer_handles.push(thread::spawn(move || {
                        let mut seed: u64 = (w as u64) * 1_000_000;
                        while !stop.load(Ordering::Relaxed) {
                            let node = unique_node(seed);
                            let _ = backend.put_node(&node);
                            seed = seed.wrapping_add(1);
                        }
                    }));
                }

                b.iter(|| {
                    let node = backend.get_node(&target_cid).expect("get_node");
                    black_box(node);
                });

                // Tear down writers.
                stop.store(true, Ordering::Relaxed);
                for h in writer_handles {
                    let _ = h.join();
                }
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_mvcc_read_latency);
criterion_main!(benches);
