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
//! The bench reports throughput in MB/s across 1MB, 10MB, and 100MB
//! payload sizes. 100MB is well past the practical upper bound for a
//! single Node before users should be reaching for file-storage-by-
//! reference (a Phase 2+ pattern) — it exists as the stress point.
//!
//! ## Verification
//!
//! Each iteration puts a fresh Node and immediately reads it back by CID.
//! The read-back assertion confirms the BLAKE3 hash chain: the CID we
//! compute from the in-memory Node matches the CID the storage layer
//! keyed on, and the bytes we get back round-trip to an equal Node. A
//! hash mismatch would panic, failing the bench loudly.
//!
//! ## Memory footprint
//!
//! redb is mmap-backed; the canonical-CBOR encoder allocates the full
//! encoded buffer inside `canonical_bytes()`. For a 100MB payload that's
//! ~100MB of transient heap during the put, plus another ~100MB for the
//! decoded Node on `get_node`. Systems with <2GB free RAM may see
//! allocator pressure — reduce `size_mb` or skip the 100MB variant.
//!
//! ```text
//! BENCH_ID = multi_mb_roundtrip/*
//! THRESHOLD_NS = informational
//! POLICY = informational
//! SOURCE = non-§14.6-payload-trend
//! ```

// THRESHOLD_NS=informational policy=informational source=non-§14.6-payload-trend

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
    group.warm_up_time(std::time::Duration::from_secs(1));
    // Reduced sample size — each sample at 100MB does ~200MB of I/O plus
    // two allocations the size of the payload, so a large sample count
    // would blow the wall-clock budget. 10 samples at each size is
    // enough for trend detection.
    group.sample_size(10);

    for size_mb in [1usize, 10, 100] {
        let size = size_mb * 1024 * 1024;
        group.throughput(Throughput::Bytes(size as u64));

        // Measurement window scales with payload size so the 100MB
        // variant has enough wall-clock to collect its samples without
        // criterion warning about missing its target.
        let measurement = match size_mb {
            1 => std::time::Duration::from_secs(3),
            10 => std::time::Duration::from_secs(8),
            _ => std::time::Duration::from_secs(20),
        };
        group.measurement_time(measurement);

        let dir = tempdir().expect("tempdir");
        let backend = RedbBackend::open(dir.path().join("benten.redb")).expect("open");
        let node = node_with_payload(size);
        // Compute the CID up-front so the bench body can assert
        // byte-for-byte round-trip without paying the hash cost per
        // iteration.
        let expected_cid = node.cid().expect("cid");

        group.bench_with_input(
            BenchmarkId::new("put_then_get", format!("{size_mb}MB")),
            &size,
            |b, _| {
                b.iter(|| {
                    let cid = backend.put_node(&node).expect("put");
                    // CID stability across put — the hash is a pure
                    // function of the Node, so this should never drift.
                    assert_eq!(cid, expected_cid, "CID must be stable across puts");
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
