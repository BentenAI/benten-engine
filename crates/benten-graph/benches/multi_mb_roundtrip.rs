//! Criterion benchmark: multi-MB Node round-trip through redb.
//!
//! **Target source:** non-§14.6 — informational. Large-payload handling is
//! not tabulated in §14.6 (which focuses on the 10-node handler hot path).
//! This bench exists to catch linear-time regressions in the DAG-CBOR
//! canonical serializer or redb's page-store when a single Node's property
//! bytes grow into the megabyte range (file blobs inlined as `Value::Bytes`,
//! large JSON payloads pasted verbatim as `Value::Text`, etc.).
//!
//! **Informational reason:** no §14.6 entry; reporting a trend line across
//! releases is the signal. Flag if `create` or `get` grows super-linearly
//! with payload size (a bug) or if the constant factor changes by >2x
//! (likely an upstream redb regression worth investigating).
//!
//! The bench reports throughput in MB/s across 1MB, 4MB, and 16MB payload
//! sizes. 16MB is the practical upper bound for a single Node before users
//! should be reaching for file-storage-by-reference (a Phase 2+ pattern).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_graph::RedbBackend;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::tempdir;

fn node_with_payload(bytes: usize) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::text("large-blob"));
    // `Value::Bytes` is the right shape for binary payloads. `vec![0u8; n]`
    // is deterministic across runs so the CID stays stable across bench
    // repetitions.
    props.insert("blob".to_string(), Value::Bytes(vec![0u8; bytes]));
    Node::new(vec!["Asset".to_string()], props)
}

fn bench_multi_mb_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_mb_roundtrip");
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(20);

    for size_mb in [1usize, 4, 16] {
        let size = size_mb * 1024 * 1024;
        group.throughput(Throughput::Bytes(size as u64));

        let dir = tempdir().expect("tempdir");
        let backend = RedbBackend::open(dir.path().join("benten.redb")).expect("open");
        let node = node_with_payload(size);

        group.bench_with_input(
            BenchmarkId::new("put_then_get", format!("{size_mb}MB")),
            &size,
            |b, _| {
                b.iter(|| {
                    let cid = backend.put_node(&node).expect("put");
                    let fetched = backend.get_node(&cid).expect("get").expect("present");
                    std::hint::black_box(fetched);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_multi_mb_roundtrip);
criterion_main!(benches);
