//! Phase 2b R3-B — SANDBOX cold-start bench skeleton (G7-C).
//!
//! D22-RESOLVED tiered numeric targets per platform:
//!   - Linux x86_64:        ≤2ms p95 / ≤5ms p99
//!   - macOS arm64:         ≤5ms p95 / ≤10ms p99
//!   - Windows x86_64:      ≤5ms p95 / ≤10ms p99
//!
//! Per-platform thresholds in `bench_thresholds.toml` at workspace root.
//! Breach escalates to Ben (= D3-RESOLVED's "real-workload data" trigger
//! for opt-in pool reconsideration).
//!
//! R3-B writes the skeleton; R3-D owns the bench framework + criterion
//! harness wiring per dispatch brief.
//!
//! Pin sources: D22-RESOLVED, wsa-5.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

use criterion::{Criterion, criterion_group, criterion_main};

/// `bench_sandbox_cold_start_per_platform_thresholds` — measures
/// 1000 iterations of `engine.sandbox_call(echo_module_cid, manifest,
/// input)` where Engine + Module are warm-cached (singleton + content-CID
/// cached per wsa-20).
///
/// Reports p50/p95/p99 wall-time for the per-call sequence:
///   Store + Instance construction + cap-intersection + execution + teardown.
///
/// Bench gate FAILS CI if:
///   - p95 > target_p95_ms[platform]
///   - p99 > target_p99_ms[platform]
fn bench_sandbox_cold_start_per_platform_thresholds(c: &mut Criterion) {
    // R3-D scope: criterion harness + bench_thresholds.toml read +
    // platform detection + per-platform threshold extraction.
    //
    // R5 G7-C scope: actual `engine.sandbox_call(echo_cid, ...)`
    // invocation per bench iteration.
    //
    // Skeleton:
    //   c.bench_function("sandbox_cold_start_echo", |b| {
    //       let engine = setup_engine();
    //       let echo_cid = setup_echo_module(&engine);
    //       let manifest = ManifestRef::Named("compute-basic".into());
    //       let input = b"".to_vec();
    //       b.iter(|| {
    //           engine.sandbox_call(echo_cid, &manifest, &input).unwrap()
    //       });
    //   });
    let _ = c;
}

criterion_group!(benches, bench_sandbox_cold_start_per_platform_thresholds);
criterion_main!(benches);
