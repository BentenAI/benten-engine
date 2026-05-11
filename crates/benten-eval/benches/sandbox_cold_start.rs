// THRESHOLD_NS=informational policy=informational source=docs/SANDBOX-LIMITS.md
//
//! SANDBOX cold-start bench (D22-RESOLVED).
//!
//! D22-RESOLVED tiered numeric targets per platform (sourced from
//! workspace-root `bench_thresholds.toml`):
//!   - Linux x86_64:        ≤2ms p95 / ≤5ms p99
//!   - macOS arm64:         ≤5ms p95 / ≤10ms p99
//!   - Windows x86_64:      ≤5ms p95 / ≤10ms p99
//!
//! The bench drives the cold-start path the SANDBOX surface owns
//! (Engine singleton lookup + Module compile + cap-intersection
//! scaffold + Store + Instance construction + module invocation +
//! teardown). Per-platform thresholds load from `bench_thresholds.toml`;
//! the threshold-drift gate wires through
//! `.github/workflows/bench-threshold-drift.yml`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stderr)]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, execute, module_for_bytes,
    shared_engine,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::path::PathBuf;

/// Per-platform p95 + p99 thresholds (ms). Loaded from workspace-root
/// `bench_thresholds.toml`. Keys are conventional `(target_os,
/// target_arch)` tuples joined with `-`.
#[derive(Debug)]
struct PlatformThresholds {
    p95_ms: f64,
    p99_ms: f64,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn load_thresholds() -> Option<PlatformThresholds> {
    let path = workspace_root().join("bench_thresholds.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    let parsed: toml::Value = toml::from_str(&text).ok()?;
    let section = parsed.get("sandbox_cold_start")?.as_table()?;

    let key = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x86_64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-aarch64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x86_64"
    } else {
        return None;
    };

    let row = section.get(key)?.as_table()?;
    Some(PlatformThresholds {
        p95_ms: row.get("p95_ms")?.as_float()?,
        p99_ms: row.get("p99_ms")?.as_float()?,
    })
}

fn empty_module_wasm() -> Vec<u8> {
    wat::parse_str("(module)").expect("empty module compiles")
}

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
        // Phase-3 G16-B sync-boundary fields default to None/0 for
        // purely-local benches — keeps the canonical Node encoding at the
        // Phase-2a 3/4-key shape so the schema-fixture CID stays stable.
        ..Default::default()
    }
}

/// `bench_sandbox_cold_start_per_platform_thresholds` — measures the
/// SANDBOX cold-start surface (Engine singleton + Module cache +
/// cap-intersection scaffold + Store + Instance + module invocation).
fn bench_sandbox_cold_start_per_platform_thresholds(c: &mut Criterion) {
    let _ = shared_engine();
    let bytes = empty_module_wasm();
    let _ = module_for_bytes(&bytes).expect("module compiles");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();

    c.bench_function("sandbox_cold_start_scaffold_path", |b| {
        b.iter(|| {
            let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
            execute(
                &bytes,
                ManifestRef::Inline(inline),
                &registry,
                SandboxConfig::default(),
                &["host:compute:time".to_string()],
                &attribution,
            )
            .expect("scaffold succeeds")
        });
    });

    // Load + log per-platform thresholds so the operator can correlate
    // measured numbers with the documented bound. The assertion is
    // informational; the gate flips to enforced when the matrix row in
    // `.github/workflows/bench-threshold-drift.yml` is promoted from
    // `informational` to a numeric value.
    if let Some(t) = load_thresholds() {
        eprintln!(
            "sandbox_cold_start thresholds for current platform: \
             p95 ≤ {:.1}ms / p99 ≤ {:.1}ms (informational)",
            t.p95_ms, t.p99_ms,
        );
    } else {
        eprintln!(
            "sandbox_cold_start: bench_thresholds.toml unavailable or current \
             platform unmapped; thresholds suppressed."
        );
    }
}

criterion_group!(benches, bench_sandbox_cold_start_per_platform_thresholds);
criterion_main!(benches);
