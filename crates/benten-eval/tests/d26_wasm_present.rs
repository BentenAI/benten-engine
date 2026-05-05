//! R3-D RED-PHASE pins for D26 `.wasm` bytes per fixture (G17-B
//! wave 5b; phase-3-backlog §6.2 + r1-wsa-5).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-B):
//!
//! - `tests/d26_wasm_bytes_committed_per_fixture_present` — §6.2
//! - `tests/d26_wasm_runtime_loader_prefers_wasm_falls_back_to_wat` — §6.2
//! - `tests/d26_cross_platform_fixture_cid_stable` — §6.2
//!
//! ## D26 closure shape (phase-3-backlog §6.2 + r1-wsa-5)
//!
//! Phase-2b committed `.wat` source for SANDBOX fixtures only;
//! `.wasm` bytes were assembled at test time via wasm-tools, which
//! introduced two failure modes:
//!
//! 1. Cross-platform CID drift if wasm-tools versions differed.
//! 2. CI runtime cost (~5-30s per assembled fixture).
//!
//! Phase-3 G17-B closes both:
//!
//! - `crates/benten-eval/build.rs` compiles each `.wat` to `.wasm`
//!   at build time.
//! - Pre-built `.wasm` bytes are committed alongside `.wat` source at
//!   `crates/benten-eval/tests/fixtures/sandbox/*.wasm`.
//! - **`wat` crate is the workspace-locked dep at exact version per
//!   r1-wsa-5 RECOMMENDATION** (recalibrated R4-FP per r4-r1-wsa-9 — the
//!   existing dev-dep is `wat = "1"` per workspace `Cargo.toml:309`,
//!   used at runtime by `crates/benten-eval/tests/sandbox_*.rs`. The
//!   prior R3-D pin asserted `wasm-tools = ` substring which was a
//!   different crate (Bytecode Alliance CLI) — would have introduced a
//!   producer/consumer drift between the build.rs tool choice and the
//!   existing test-time `wat::parse_str` callers).
//! - CID rebake protocol via `cargo bench-wat-rebake` tooling subcommand
//!   driven by the `wat` crate (NOT `wasm-tools`).
//!
//! ## Loader fallback shape
//!
//! Tests load fixtures with the helper function (G17-B wires the
//! signature):
//!
//! ```ignore
//! benten_eval::tests::fixtures::load_fixture("esc_07_fuel_refill_via_host_fn_re_entry")
//! ```
//!
//! Loader strategy: prefer the committed `.wasm` if present + valid;
//! fall back to assembling the `.wat` only if the `.wasm` is missing
//! (e.g. fresh checkout before `build.rs` ran).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-B wave 5b authors build.rs that compiles .wat → .wasm + commits the bytes"]
fn d26_wasm_bytes_committed_per_fixture_present() {
    // phase-3-backlog §6.2 pin. G17-B implementer wires this:
    //
    //   let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("tests").join("fixtures").join("sandbox");
    //
    //   // Enumerate every .wat in the directory:
    //   let wat_fixtures: Vec<_> = std::fs::read_dir(&fixture_dir).unwrap()
    //       .filter_map(|e| e.ok())
    //       .filter(|e| e.path().extension().map_or(false, |ext| ext == "wat"))
    //       .collect();
    //
    //   assert!(!wat_fixtures.is_empty(), "expected at least one .wat fixture");
    //
    //   // Each .wat MUST have a paired .wasm:
    //   for wat in &wat_fixtures {
    //       let wasm_path = wat.path().with_extension("wasm");
    //       assert!(wasm_path.exists(),
    //           "fixture {} is missing committed .wasm bytes per phase-3-backlog §6.2 \
    //            (run `cargo bench-wat-rebake` per r1-wsa-5 to regenerate)",
    //           wat.path().display());
    //   }
    //
    // OBSERVABLE consequence: every SANDBOX fixture has BOTH .wat
    // (source) and .wasm (committed bytes). Defends against the
    // failure shape "ESC test depends on a fixture whose .wasm bytes
    // weren't committed, breaking CI on fresh checkouts."
    unimplemented!("G17-B wires .wat-/.wasm-pairing assertion against fixtures/sandbox/");
}

#[test]
#[ignore = "RED-PHASE: G17-B wave 5b authors fixture loader with .wasm-prefer + .wat-fallback"]
fn d26_wasm_runtime_loader_prefers_wasm_falls_back_to_wat() {
    // phase-3-backlog §6.2 pin. G17-B implementer wires this:
    //
    //   // Loader prefers .wasm bytes when present:
    //   let bytes = benten_eval::tests::fixtures::load_fixture(
    //       "sandbox_basic"
    //   );
    //
    //   // First 4 bytes are the WASM magic number `\0asm`:
    //   assert_eq!(&bytes[..4], b"\0asm",
    //       "loaded fixture must be valid WASM bytes (loader prefers committed .wasm per §6.2)");
    //
    //   // Fallback path test: temporarily move the committed .wasm,
    //   // verify the loader produces equivalent bytes from the .wat:
    //   //   (implementer wires this with care to restore the .wasm
    //   //    file even on test failure)
    //   //
    //   //   let tmp_path = move_wasm_aside("sandbox_basic");
    //   //   let bytes_from_wat = load_fixture("sandbox_basic");
    //   //   restore_wasm(tmp_path);
    //   //   assert_eq!(&bytes_from_wat[..4], b"\0asm");
    //
    // OBSERVABLE consequence: loader works whether .wasm is present
    // (fast path; CI baseline) or absent (fresh-checkout fallback).
    // Defends §6.2 loader strategy.
    unimplemented!("G17-B wires fixture-loader .wasm-prefer + .wat-fallback assertion");
}

#[test]
#[ignore = "RED-PHASE: G17-B wave 5b ensures cross-platform fixture-CID stability per §6.2 + r1-wsa-5"]
fn d26_cross_platform_fixture_cid_stable() {
    // phase-3-backlog §6.2 + r1-wsa-5 pin. G17-B implementer:
    //
    //   // CID of canonical `sandbox_basic` fixture is well-known:
    //   //   bafyr4i...  (implementer pins exact value at G17-B)
    //
    //   const KNOWN_FIXTURE_CID: &str = "bafyr4i..."; // G17-B implementer pins
    //
    //   let bytes = benten_eval::tests::fixtures::load_fixture("sandbox_basic");
    //   let cid = compute_cid_blake3_dagcbor(&bytes);
    //   assert_eq!(cid.to_string(), KNOWN_FIXTURE_CID,
    //       "fixture CID drifted; if `wat` crate version was bumped, run \
    //        `cargo bench-wat-rebake` to regenerate fixtures + bump KNOWN_FIXTURE_CID");
    //
    //   // The `wat` dev-dep is pinned at workspace level per r1-wsa-5
    //   // RECOMMENDATION + r4-r1-wsa-9 recalibration. EXACT-version
    //   // pin (`=` prefix); soft caret/tilde matchers (`^`, `~`, bare
    //   // version) defeat the determinism contract by permitting silent
    //   // minor bumps that may change emitted bytes.
    //   let workspace_cargo = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("Cargo.toml")
    //   ).unwrap();
    //
    //   // [workspace.dependencies] section names `wat` (NOT wasm-tools):
    //   assert!(workspace_cargo.contains("\nwat = ") || workspace_cargo.contains("\nwat="),
    //       "workspace Cargo.toml [workspace.dependencies] MUST declare `wat` per \
    //        r1-wsa-5 RECOMMENDATION + r4-r1-wsa-9 recalibration (the wabt-ecosystem \
    //        crate already used at test time by sandbox_*.rs callers — NOT `wasm-tools` \
    //        which is a different Bytecode Alliance CLI tool that emits slightly \
    //        different bytes; mixing tools at build vs test time would break \
    //        determinism + introduce p/c drift)");
    //
    //   // The version is EXACT-pinned (= prefix) — implementer locks
    //   // the bench-wat-rebake target version (r1-wsa-5 named 1.227.x;
    //   // the actual chosen exact version is whatever the rebake produces).
    //   // Reject `^`, `~`, or bare patterns that permit minor bumps:
    //   assert!(workspace_cargo.contains("wat = \"=") || workspace_cargo.contains("wat=\"="),
    //       "workspace `wat` dep MUST use `= ` exact-version pin per r4-r1-wsa-9 \
    //        (`^x.y.z`, `~x.y.z`, or bare `\"x.y.z\"` permit silent minor bumps that \
    //        may change emitted .wasm bytes + break cross-platform CID stability)");
    //
    //   // Forbid `wasm-tools` from sneaking in as a parallel dep (the
    //   // r4-r1-wsa-9 recalibration explicitly chose `wat` as the single
    //   // tool; presence of both would invite producer/consumer drift):
    //   assert!(!workspace_cargo.contains("\nwasm-tools = "),
    //       "workspace Cargo.toml MUST NOT declare a parallel `wasm-tools` dep \
    //        per r4-r1-wsa-9 single-tool commitment (would split build-time vs \
    //        test-time .wat-compilation paths)");
    //
    // OBSERVABLE consequence: a CI run on Linux x86_64 + macOS arm64
    // produces the same fixture CIDs (because `wat` version is exact-
    // pinned + .wasm bytes are committed + no parallel tool drift).
    // Defends r1-wsa-5 determinism contract + r4-r1-wsa-9 single-tool +
    // exact-version recalibration.
    //
    // Pairs with the AArch64 CI cell (also G17-B) which exercises
    // this assertion on the `macos-latest-arm64` runner.
    unimplemented!(
        "G17-B wires cross-platform CID-stability assertion + `wat` exact-version pin per r4-r1-wsa-9 (NOT `wasm-tools`; reject parallel-tool drift)"
    );
}
