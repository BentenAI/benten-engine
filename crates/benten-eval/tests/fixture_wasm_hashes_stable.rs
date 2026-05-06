//! Phase-2b G7-B / D26-RESOLVED — committed `.wasm` fixture drift detector.
//!
//! D26 ships pre-built `.wasm` bytes in the repo (NOT a CI build step) to
//! side-step the wsa-12 cross-platform shell-portability issues with
//! invoking `wat2wasm` from CI runners. The contract is:
//!
//!   - The `.wat` source is the canonical input (human-editable).
//!   - The `.wasm` binary is the cached output (machine-generated).
//!   - Both are committed to git side-by-side under
//!     `crates/benten-eval/tests/fixtures/sandbox/`.
//!   - This test pins the BLAKE3 hash of every committed `.wasm` so that
//!     drift between the `.wat` source and the cached bytes is caught at
//!     test time.
//!
//! ## Phase-3 G17-B canonicalisation (phase-3-backlog §6.2 + r4-r1-wsa-9)
//!
//! The canonical regenerator is now `cargo bench-wat-rebake` (alias →
//! `tools/bench-wat-rebake/`), which uses the workspace-locked exact-
//! version `wat` crate (`=1.248.0` per `[workspace.dependencies] wat`). The legacy
//! `scripts/build_wasm.sh` invoked the host `wabt` binary (`wat2wasm`)
//! whose output bytes can differ from the `wat` crate's even on
//! semantically-equivalent modules — this is the producer/consumer drift
//! r4-r1-wsa-9 named (the recalibration to a single tool closes it).
//! Phase-3 G17-B regenerated `depth_nest_2`, `depth_nest_3_negative`,
//! and `output_overflow_2048` via the new pipeline; the BLAKE3 hashes
//! below moved to match.
//!
//! Updating a fixture: edit the `.wat`, run `cargo bench-wat-rebake`,
//! copy the new BLAKE3 hash from the test failure into the constants
//! below in the SAME commit. The `.wasm` and the pin in this file MUST
//! land together.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "dev-only --ignored helper prints fixture hashes for first-run \
              fill-in of the PINNED_FIXTURES table"
)]

use std::path::PathBuf;

/// Map of (relative-path, expected-blake3-hex) for every G7-B-owned
/// `.wasm` fixture. Sibling subdirectories under
/// `crates/benten-eval/tests/fixtures/sandbox/` (e.g. `escape/`) are
/// owned by other G7-* briefs and pinned in their own drift-detector
/// tests.
const PINNED_FIXTURES: &[(&str, &str)] = &[
    (
        // Phase-3 G17-B: regenerate via `cargo bench-wat-rebake` and
        // update on intentional `.wat` source change. Hash matches the
        // `wat` crate's emitted bytes at workspace pin `=1.248.0`.
        "depth_nest_1.wasm",
        "1ca2cf216d0dc5dd6b1bfc4e49748fee72d4b508ce5087d6e04d009f8e17c77b",
    ),
    (
        "depth_nest_2.wasm",
        "f6ff215bcdb20c787e04148c6c053d9a1617fb46e01e2e11fb6ef616178626fc",
    ),
    (
        "depth_nest_3_negative.wasm",
        "7d91e61b96ad6d49e9514ce272d4236c28f1f6b0b2a497d9545640e839fa79f7",
    ),
    (
        "output_overflow_2048.wasm",
        "2ca7a13910ace849a86b6c0b8715298a977d6fef7f4fea477d4c52fdd9a8adb8",
    ),
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sandbox")
}

#[test]
fn fixture_wasm_hashes_stable() {
    let root = fixture_root();
    let mut drifted: Vec<String> = Vec::new();
    for (relative, expected_hex) in PINNED_FIXTURES {
        let path = root.join(relative);
        assert!(
            path.exists(),
            "missing committed .wasm at {} — run `cargo bench-wat-rebake`",
            path.display()
        );
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("read {}: {e}", path.display());
        });
        let hash = blake3::hash(&bytes);
        let actual_hex = hash.to_hex().to_string();
        if actual_hex != *expected_hex {
            drifted.push(format!(
                "  {relative}\n    expected: {expected_hex}\n    actual:   {actual_hex}"
            ));
        }
    }
    if !drifted.is_empty() {
        let body = drifted.join("\n");
        panic!(
            "G7-B .wasm fixture drift detected — \n{body}\n\n\
             If the .wat source changed intentionally: copy the actual \
             hashes above into PINNED_FIXTURES in this file in the \
             SAME commit as the .wasm change. If you didn't change a \
             .wat, the committed .wasm has been corrupted — restore from \
             a clean checkout.",
        );
    }
}

/// First-run helper — print every committed fixture's BLAKE3 hash so the
/// PINNED_FIXTURES constants above can be filled in mechanically. Run
/// with `cargo test -p benten-eval --test fixture_wasm_hashes_stable -- \
/// --ignored print_committed_fixture_hashes --nocapture`.
#[test]
#[ignore = "dev-only — print hashes for PINNED_FIXTURES initial fill-in"]
fn print_committed_fixture_hashes() {
    let root = fixture_root();
    for (relative, _) in PINNED_FIXTURES {
        let path = root.join(relative);
        let bytes = std::fs::read(&path).unwrap();
        let hash = blake3::hash(&bytes);
        println!("{relative} -> {}", hash.to_hex());
    }
}
