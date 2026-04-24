//! Criterion benchmark: `get_node_label_only` fast-path probe.
//!
//! R4 cov-10 naming note: previously known as `invariant_11_prefix_probe`
//! in `.addl/phase-2a/r2-test-landscape.md` §5 — renamed to
//! `get_node_label_only_sub_1us` to clarify the Phase-2a surface being
//! measured (the `KVBackend::get_node_label_only` probe, not Inv-11 as
//! an abstract concept). Naming drift, not a missing bench.
//!
//! **Target source:** plan §9.10 direct commitment — "< 1 µs per lookup
//! on dev hardware." This is the public cost-gate for the Inv-11 runtime
//! enforcement path: every user-subgraph READ resolving a `Value::Cid`
//! must call `get_node_label_only(cid)` to check the resolved Node's head
//! label against `SYSTEM_ZONE_PREFIXES` BEFORE returning the resolved
//! content. If the probe itself exceeds 1 µs, the evaluator's hot-path
//! READ budget (and by extension the §14.6 10-node handler target) is
//! blown.
//!
//! **Gate policy:** CI-GATED — fails `phase-2a-exit-criteria` on regression.
//! Threshold is <1 µs median measured against a warmed-up cache.
//!
//! **Threshold encoding (machine-readable):**
//!
//! ```text
//! BENCH_ID = get_node_label_only/hot_cache
//! THRESHOLD_NS = 1000  // 1 µs per plan §9.10
//! POLICY = fail-on-regression
//! ```
//!
//! The bench composes into `invariant_11_prefix_probe` (a derivative
//! measurement that adds the phf prefix-table check on top of this
//! probe). Keeping the two separate makes regressions attributable: if
//! `invariant_11_prefix_probe` regresses but `get_node_label_only` holds,
//! the regression is in the phf table or in the code surrounding the
//! probe — not in the storage fast-path.
//!
//! ## Red-phase → green-phase
//!
//! `get_node_label_only` was a G5-B-i deliverable; at R3 the function was
//! `todo!()`. G5-B-i landed the Phase-2a full-decode-then-drop impl
//! (`serde_ipld_dagcbor::from_slice::<Node>(bytes)` then keep only the first
//! label). The mini-review C2 fix additionally `create_node`s the fixture
//! before the probe so this bench measures the hot-cache HIT path (the real
//! evaluator scenario), not the miss-before-decode fast-out.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::Duration;

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;
use criterion::{Criterion, criterion_group, criterion_main};

/// Hot-cache probe: measure the `get_node_label_only` fast path under the
/// realistic scenario where the target Node is in the redb page cache.
/// This is the common case for runtime Inv-11 probes: a user subgraph
/// reads a CID it just materialised in an earlier TRANSFORM, so the Node
/// is warm.
fn bench_get_node_label_only_hot_cache(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Seed a non-system-zone Node and capture its CID for the probe.
    // The canonical test node has label "post" — not in SYSTEM_ZONE_PREFIXES,
    // so the probe returns `Ok(Some("post"))` ("not system-zone") which is
    // the hot path the evaluator exercises on every legitimate user READ.
    //
    // G5-B-i mini-review C2 fix: the earlier bench body computed `node.cid()`
    // but never persisted the Node, so `get_node_label_only(cid)` returned
    // `Ok(None)` at the first key-lookup step — measuring the MISS path, not
    // the hot-cache HIT. `engine.create_node` now writes the fixture into
    // redb so the probe reaches the `serde_ipld_dagcbor::from_slice::<Node>`
    // decode step (the full-decode-then-drop Phase-2a impl per
    // `NodeStore::get_node_label_only`).
    let node = canonical_test_node();
    let cid = engine
        .create_node(&node)
        .expect("seed fixture via the user-facing CRUD path");

    // Pre-warm: one call to populate the redb page cache line for this CID.
    let _ = engine.get_node_label_only(&cid);

    let mut group = c.benchmark_group("get_node_label_only");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // MACHINE-READABLE GATE: the exit-criteria workflow reads this comment.
    // THRESHOLD_NS=1000 policy=fail-on-regression source=plan-§9.10

    group.bench_function("hot_cache", |b| {
        b.iter(|| {
            let out = engine.get_node_label_only(black_box(&cid));
            let _ = black_box(out);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_get_node_label_only_hot_cache);
criterion_main!(benches);
