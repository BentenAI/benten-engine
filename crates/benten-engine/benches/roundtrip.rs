//! Criterion benchmark: Node round-trip through the engine.
//!
//! Measures:
//!
//! - `hash_only` — content-address a Node (BLAKE3 over DAG-CBOR canonical form).
//! - `create_node` — hash + persist through the redb backend (one transaction).
//! - `get_node` — fetch a Node by CID from redb (hot path, cached pages).
//! - `full_roundtrip` — create + get + verify the returned hash matches.
//!
//! Compared to `ENGINE-SPEC.md` Section 14.6 realistic targets:
//!
//! - Node lookup by ID: 1-50us
//! - Node creation + IVM update: 100-500us realistic
//!
//! The `full_roundtrip` benchmark is the headline number.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_hash_only(c: &mut Criterion) {
    let node = canonical_test_node();
    c.bench_function("hash_only", |b| {
        b.iter(|| {
            let cid = black_box(&node).cid().expect("hash");
            black_box(cid);
        });
    });
}

fn bench_create_node(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();
    c.bench_function("create_node", |b| {
        b.iter(|| {
            let cid = engine.create_node(black_box(&node)).expect("create");
            black_box(cid);
        });
    });
}

fn bench_get_node(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();
    let cid = engine.create_node(&node).expect("create");
    c.bench_function("get_node", |b| {
        b.iter(|| {
            let fetched = engine.get_node(black_box(&cid)).expect("get");
            black_box(fetched);
        });
    });
}

fn bench_full_roundtrip(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("benten.redb")).expect("open");
    let node = canonical_test_node();
    c.bench_function("full_roundtrip", |b| {
        b.iter(|| {
            let cid = engine.create_node(black_box(&node)).expect("create");
            let fetched = engine.get_node(&cid).expect("get").expect("present");
            let rehash = fetched.cid().expect("rehash");
            assert_eq!(rehash, cid);
            black_box(fetched);
        });
    });
}

criterion_group!(
    benches,
    bench_hash_only,
    bench_create_node,
    bench_get_node,
    bench_full_roundtrip,
);
criterion_main!(benches);
