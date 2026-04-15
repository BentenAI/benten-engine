//! Criterion benchmark: content-address a Node (hash-only, no storage).
//!
//! This is the **baseline** benchmark and is **informational, not gated**.
//! It protects against regression from the spike's measured 892ns (SPIKE
//! results) on the canonical test Node.
//!
//! **Target source:** non-§14.6 — baseline protection (per implementation
//! plan §4.4). ENGINE-SPEC §14.6 tabulates engine-level targets (lookup,
//! create, view read, handler eval); raw hash throughput is not a §14.6
//! target because the engine-level numbers already amortize hashing into
//! their ranges. Regression here would manifest in every gated benchmark.
//!
//! ## Why informational
//!
//! If BLAKE3 or `serde_ipld_dagcbor` changes performance dramatically
//! (upstream regression, SIMD removal on a given target, etc.) we want to
//! see the delta — but we do NOT want CI to fail on a 200ns swing in a
//! hot path that is not itself in the §14.6 budget.
//!
//! ## Run
//!
//! ```ignore
//! cargo bench -p benten-core --bench hash_only
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;

use benten_core::testing::canonical_test_node;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_hash_only(c: &mut Criterion) {
    let node = canonical_test_node();
    let mut group = c.benchmark_group("hash_only");
    // Keep warmup + measurement aligned with the roundtrip bench in
    // benten-engine so cross-bench comparisons are meaningful.
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));
    group.bench_function("canonical_test_node", |b| {
        b.iter(|| {
            let cid = black_box(&node).cid().expect("hash");
            black_box(cid);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_hash_only);
criterion_main!(benches);
