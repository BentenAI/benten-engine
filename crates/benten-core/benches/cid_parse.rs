//! Criterion benchmark: parse a base32 CIDv1 string back into a `Cid`.
//!
//! **Target source:** non-§14.6 — informational. `Cid::from_str` is on the
//! napi-rs boundary path (TypeScript passes CIDs as strings, Rust parses)
//! and on any sync-protocol path where CIDs arrive over the wire; a slow
//! parser shows up on both hot paths. ENGINE-SPEC §14.6 does not tabulate
//! a parse target, so this bench reports a trend line rather than a gate.
//!
//! **Informational reason:** §14.6 was written around engine-level
//! throughput (lookup, create, view read, handler eval); parse is a
//! substep of lookup-by-CID and its cost is already amortized into the
//! "Node lookup by ID: 1-50µs" direct gate. A dedicated parse gate would
//! double-count. Keeping this bench informational lets us watch the trend
//! without gating on a number that isn't in the spec.
//!
//! ## Stub-graceful
//!
//! `Cid::from_str` is currently `todo!("Cid::from_str — G1 (Phase 1)")`.
//! Running this benchmark before G1 lands will panic at bench time. That
//! is expected and correct per the R3 TDD contract: benches land in R3,
//! implementations land in R5, benches pass in R6.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::Cid;
use benten_core::testing::canonical_test_node;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_cid_parse(c: &mut Criterion) {
    // Pre-compute the canonical CID string so the measurement isolates
    // the parse step (not the hash step).
    let cid_str: String = canonical_test_node().cid().expect("hash").to_base32();

    let mut group = c.benchmark_group("cid_parse");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("base32_roundtrip", |b| {
        b.iter(|| {
            // Will panic until `Cid::from_str` lands (G1). Expected pre-R5.
            let parsed = Cid::from_str(black_box(cid_str.as_str())).expect("parse");
            black_box(parsed);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_cid_parse);
criterion_main!(benches);
