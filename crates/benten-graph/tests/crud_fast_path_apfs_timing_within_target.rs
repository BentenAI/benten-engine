//! G13-E pin (LIVE since Phase-3 R5 wave-3): CRUD fast-path APFS timing
//! within the §14.6 target (150–300 µs on dev hardware) when the engine
//! defaults to [`DurabilityMode::Group`].
//!
//! Pin source: r2-test-landscape §2.1 G13-E row
//! `crud_fast_path_apfs_timing_within_target`; plan §3 G13-E.
//!
//! ## Posture: informational gate (Phase-3 G13-E) — required gate (later)
//!
//! Per plan §3 G13-E ("informational then required gate"), this test
//! ships at G13-E as an INFORMATIONAL timing assertion against a
//! generous upper bound (100 ms per CRUD-post-create commit on the
//! `cargo test` runner). The 150–300 µs §14.6 target is the
//! aspirational end-state; until either:
//!
//! 1. redb v4+ exposes native batched-fsync (so `DurabilityMode::Group`
//!    stops collapsing to `Durability::Immediate`), OR
//! 2. Benten adds its own write-batching layer above redb, OR
//! 3. CI moves to dedicated runners (no shared-runner ±30% variance),
//!
//! the timing on shared GitHub Actions runners cannot reliably gate
//! against the 150–300 µs target without false-positive churn. The
//! authoritative perf signal lives at the criterion bench
//! [`crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`]
//! and at [`crates/benten-graph/benches/durability_modes.rs`]; the bench
//! workflow `.github/workflows/bench.yml` is promoted from
//! informational to required at G13-E so a regression in the bench
//! signal is gating even when this test's wall-clock budget is loose.
//!
//! ## OBSERVABLE consequence (per pim-2 §3.6b)
//!
//! The test drives an actual `RedbBackend::put_node` through the
//! production-grade entry point under `DurabilityMode::default()`
//! (which is now `Group` post-G13-E). It measures wall-clock per
//! commit and asserts the median is within the (generous)
//! INFORMATIONAL upper bound. If a regression flips the default
//! back to `Immediate` AND a pathological mapping change lands that
//! makes the per-commit cost balloon, the assertion fires.
//!
//! The test is companion to (not substitute for):
//! - [`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`]
//!   — pins the default value itself.
//! - [`crates/benten-graph/tests/security_posture_compromise_12_marked_closed`]
//!   — pins the SECURITY-POSTURE.md narrative carries the closure.
//! - [`crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`]
//!   — bench-grade measurement (gated by `.github/workflows/bench.yml`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_graph::{DurabilityMode, RedbBackend};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Generous upper bound for the per-commit wall-clock on the `cargo test`
/// runner. The §14.6 production target is 150–300 µs; this test runs
/// under shared-runner variance + tempfile creation overhead + redb's
/// in-process lock setup + APFS fsync floor on the macOS runners, so
/// 100 ms gives headroom against false positives while still catching
/// pathological regressions (e.g. a 100×-slower mapping that balloons
/// the per-commit cost into the seconds range).
///
/// Empirical signal that motivated the 10 ms → 100 ms calibration
/// (G13-E mini-review BLOCKER 2): macos-x86_64-stable shared-runner
/// median measured 13.25 ms with samples ranging 8.1–58.6 ms — APFS
/// fsync floor + shared-runner variance dominate the wall-clock here.
/// Linux runners stay well under 10 ms. Bench-grade measurement at
/// `crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`
/// remains the authoritative perf signal; this test catches gross
/// regressions only.
const INFORMATIONAL_PER_COMMIT_BUDGET: Duration = Duration::from_millis(100);

/// Number of CRUD-post-create iterations measured. We take the median
/// to defang outliers (filesystem flushes, runner contention, GC
/// pauses on the test runner). 11 keeps the test under ~150 ms even
/// at the budget ceiling.
const ITER_COUNT: usize = 11;

fn fresh_post_node(idx: usize) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("post-{idx}")));
    props.insert("body".into(), Value::Text("hello".into()));
    Node::new(vec!["Post".into()], props)
}

#[test]
fn crud_fast_path_apfs_timing_within_target() {
    // Sanity: the engine surface defaults to Group post-G13-E. If this
    // assertion fails the default has regressed before the timing
    // measurement can be meaningful.
    assert_eq!(
        DurabilityMode::default(),
        DurabilityMode::Group,
        "G13-E default-flip pre-condition for the timing assertion"
    );

    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("crud_fast_path.redb")).unwrap();

    let mut samples: Vec<Duration> = Vec::with_capacity(ITER_COUNT);
    for i in 0..ITER_COUNT {
        let node = fresh_post_node(i);
        let t0 = Instant::now();
        backend
            .put_node(&node)
            .expect("CRUD post-create put_node must succeed under default durability");
        samples.push(t0.elapsed());
    }

    samples.sort();
    let median = samples[ITER_COUNT / 2];

    assert!(
        median < INFORMATIONAL_PER_COMMIT_BUDGET,
        "CRUD fast-path post-create commit median {median:?} exceeds the \
         informational upper bound {INFORMATIONAL_PER_COMMIT_BUDGET:?}; \
         the §14.6 production target is 150–300 µs but the cargo-test \
         runner budget is intentionally generous. Authoritative perf \
         signal is the bench workflow .github/workflows/bench.yml + \
         crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs. \
         A failure here suggests a pathological regression (e.g. a \
         100×-slower mapping or a missing default-flip) — investigate \
         before relaxing the budget. samples: {samples:?}"
    );
}
