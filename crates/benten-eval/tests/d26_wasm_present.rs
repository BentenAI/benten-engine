//! Phase-3 G17-B GREEN-PHASE pins for D26 `.wasm` bytes per fixture
//! (phase-3-backlog §6.2 + r1-wsa-5 + r4-r1-wsa-9).
//!
//! ## D26 closure shape
//!
//! Phase-2b committed `.wat` source for SANDBOX fixtures only;
//! `.wasm` bytes were assembled at test time via shell `wat2wasm` (wabt
//! / Bytecode Alliance), which introduced two failure modes:
//!
//! 1. Cross-platform CID drift if wabt versions differed.
//! 2. CI runtime cost (~5-30s per assembled fixture).
//!
//! Phase-3 G17-B closes both:
//!
//! - `crates/benten-eval/build.rs` emits `cargo:rerun-if-changed=` for
//!   every `.wat` + `.wasm` so test binaries recompile when fixtures
//!   change.
//! - Pre-built `.wasm` bytes are committed alongside `.wat` source at
//!   `crates/benten-eval/tests/fixtures/sandbox/**/*.wasm`.
//! - The `wat` crate is the workspace-locked exact-version dep
//!   (`=1.248.0` per `[workspace.dependencies] wat`) per r4-r1-wsa-9
//!   recalibration — the pre-Phase-3 r1-wsa-5 RECOMMENDATION named
//!   `wasm-tools 1.227.x` but the actual ecosystem dep we already used
//!   at integration-test time was the sibling `wat` crate; r4-r1-wsa-9
//!   recalibrated to a single tool to close the producer/consumer
//!   drift.
//! - CID rebake protocol: `cargo bench-wat-rebake` (alias →
//!   `tools/bench-wat-rebake/`) uses the same exact-version `wat`
//!   crate as both the producer (regenerator) and the consumer
//!   (`benten_eval::test_fixtures::load_fixture`).
//!
//! ## Loader fallback shape
//!
//! Tests load fixtures with the helper:
//!
//! ```ignore
//! benten_eval::test_fixtures::load_fixture("escape/forged_cap_claim_section")
//! ```
//!
//! Loader strategy: prefer the committed `.wasm` if present + valid;
//! fall back to assembling the `.wat` only if the `.wasm` is missing
//! (e.g. fresh checkout before `cargo bench-wat-rebake` ran).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sandbox")
}

fn collect_wat(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_wat(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("wat") {
            out.push(path);
        }
    }
    out
}

#[test]
fn d26_wasm_bytes_committed_per_fixture_present() {
    // phase-3-backlog §6.2 pin. Every `.wat` MUST have a paired
    // committed `.wasm` sibling. OBSERVABLE consequence: every SANDBOX
    // fixture has BOTH `.wat` (source) and `.wasm` (committed bytes);
    // a fresh checkout has no missing committed bytes. Defends against
    // the failure shape "ESC test depends on a fixture whose .wasm
    // bytes weren't committed, breaking CI on fresh checkouts."
    let root = fixture_root();
    let wat_paths = collect_wat(&root);
    assert!(
        !wat_paths.is_empty(),
        "expected at least one .wat fixture under {}",
        root.display()
    );
    let mut missing = Vec::new();
    for wat in &wat_paths {
        let wasm = wat.with_extension("wasm");
        if !wasm.exists() {
            missing.push(wasm);
        }
    }
    assert!(
        missing.is_empty(),
        "{} fixture(s) missing committed .wasm bytes per phase-3-backlog §6.2 \
         (run `cargo bench-wat-rebake` per r4-r1-wsa-9 to regenerate):\n  {}",
        missing.len(),
        missing
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn d26_wasm_runtime_loader_prefers_wasm_falls_back_to_wat() {
    // phase-3-backlog §6.2 pin. The loader at
    // `benten_eval::test_fixtures::load_fixture` MUST:
    //
    //   1. Return the committed `.wasm` bytes if present.
    //   2. Fall back to assembling `.wat` via the same workspace-locked
    //      `wat` crate if the `.wasm` is missing.
    //   3. Both branches return `\0asm`-prefixed valid wasm bytes.
    //
    // OBSERVABLE consequence: loader works whether `.wasm` is present
    // (fast path; CI baseline) or absent (fresh-checkout fallback).
    use benten_eval::test_fixtures::{load_fixture, load_fixture_wat_only};

    // Prefer-wasm path: depth_nest_1.wasm IS committed (Phase-2b G7-B).
    let bytes = load_fixture("depth_nest_1").expect("committed .wasm must load");
    assert_eq!(
        &bytes[..4],
        b"\0asm",
        "prefer-wasm path must return valid wasm bytes"
    );

    // Fallback-to-wat path: force the .wat-only branch via the
    // dedicated helper. Round-trips to valid wasm bytes via the same
    // `wat` crate as the producer.
    let bytes_from_wat = load_fixture_wat_only("depth_nest_1").expect("wat-fallback must compile");
    assert_eq!(
        &bytes_from_wat[..4],
        b"\0asm",
        "wat-fallback path must return valid wasm bytes"
    );

    // Subdir fixture (escape/) — committed .wasm exists post-G17-B
    // rebake; loader resolves cleanly via the prefer-wasm branch.
    let escape = load_fixture("escape/infinite_loop").expect("subdir fixture must load");
    assert_eq!(&escape[..4], b"\0asm");
}

#[test]
fn d26_cross_platform_fixture_cid_stable() {
    // phase-3-backlog §6.2 + r1-wsa-5 + r4-r1-wsa-9 pin. Cross-
    // platform CID stability is defended by:
    //
    //   1. Workspace `wat = "=..."` exact-version pin (no soft
    //      matchers).
    //   2. Committed `.wasm` bytes (loader prefers these).
    //   3. Single-tool commitment (no parallel `wasm-tools` dep).
    //
    // OBSERVABLE consequence: a CI run on Linux x86_64 + macOS arm64
    // produces the same fixture CIDs because all three defenses hold.
    // Defends r1-wsa-5 determinism contract + r4-r1-wsa-9
    // single-tool + exact-version recalibration.
    //
    // Pairs with the AArch64 CI cell (also G17-B) which exercises
    // this assertion on the `macos-14` runner.
    use benten_eval::test_fixtures::load_fixture;

    let workspace_cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.toml");
    let workspace_cargo = fs::read_to_string(&workspace_cargo_path).unwrap();

    // [workspace.dependencies] section names `wat` (NOT wasm-tools):
    assert!(
        workspace_cargo.contains("\nwat = ") || workspace_cargo.contains("\nwat="),
        "workspace Cargo.toml [workspace.dependencies] MUST declare `wat` per \
         r4-r1-wsa-9 recalibration (the `wat` crate is the wabt-ecosystem sibling \
         already used at test time by sandbox_*.rs callers — NOT `wasm-tools` which is a \
         different Bytecode Alliance CLI tool that emits slightly different bytes; \
         mixing tools at build vs test time would break determinism + introduce p/c drift)"
    );

    // Exact-version pin (= prefix) — implementer locks the
    // bench-wat-rebake target version. Reject `^`, `~`, or bare
    // patterns that permit minor bumps:
    assert!(
        workspace_cargo.contains("wat = \"=") || workspace_cargo.contains("wat=\"="),
        "workspace `wat` dep MUST use `= ` exact-version pin per r4-r1-wsa-9 \
         (`^x.y.z`, `~x.y.z`, or bare `\"x.y.z\"` permit silent minor bumps that \
         may change emitted .wasm bytes + break cross-platform CID stability)"
    );

    // Forbid `wasm-tools` from sneaking in as a parallel dep — the
    // r4-r1-wsa-9 recalibration explicitly chose `wat` as the single
    // tool; presence of both would invite producer/consumer drift:
    assert!(
        !workspace_cargo.contains("\nwasm-tools = "),
        "workspace Cargo.toml MUST NOT declare a parallel `wasm-tools` dep \
         per r4-r1-wsa-9 single-tool commitment (would split build-time vs \
         test-time .wat-compilation paths)"
    );

    // CID-stability check: load the canonical fixture + assert its
    // BLAKE3 matches the pinned hash (which is itself enforced by
    // `tests/fixture_wasm_hashes_stable`). The bytes loaded here must
    // be byte-identical to what the drift-detector pinned.
    let bytes = load_fixture("depth_nest_1").expect("canonical fixture");
    let cid = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(
        cid, "1ca2cf216d0dc5dd6b1bfc4e49748fee72d4b508ce5087d6e04d009f8e17c77b",
        "fixture BLAKE3 drifted from pinned value; if `wat` crate version \
         was bumped, run `cargo bench-wat-rebake` to regenerate fixtures + \
         update PINNED_FIXTURES in tests/fixture_wasm_hashes_stable.rs"
    );
}
