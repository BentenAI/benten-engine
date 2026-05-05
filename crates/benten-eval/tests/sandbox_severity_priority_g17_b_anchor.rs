//! R3-D RED-PHASE pin — G17-B AArch64 CI cell anchors the existing
//! `sandbox_severity_priority` test (r1-wsa-9 MINOR; r2-test-landscape §2.5 G17-B).
//!
//! ## Why this file exists
//!
//! r2-test-landscape §2.5 G17-B lists `tests/sandbox_severity_priority`
//! as a verification anchor for the AArch64 CI cell:
//!
//! ```text
//! cargo nextest run -p benten-eval --target aarch64-apple-darwin \
//!   --test sandbox_basic --test sandbox_escape_attempts_denied \
//!   --test sandbox_severity_priority
//! ```
//!
//! r1-wsa-9 verified that `sandbox_severity_priority` (the existing
//! Phase-2b R3-B test file at
//! `crates/benten-eval/tests/sandbox_severity_priority.rs`) ALREADY
//! contains live test bodies — this is NOT a new file to author at
//! G17-B; the AArch64 cell merely RUNS what's already there.
//!
//! This anchor file pins the existence of the AArch64 cell narrative
//! itself (i.e. that the workflow YAML names the three test files
//! correctly + the file references in this comment match reality).
//!
//! ## Why a Rust source-cite pin (not just YAML)
//!
//! Per pim-3 §3.9 (R2 lens-menu correctness coverage) + pim-1 §3.5b
//! HARDENED (post-fix doc-coupling pre-flight): if the AArch64 YAML
//! gets renamed or relocated, the Rust-side narrative pinning the
//! cell + naming the three test files goes stale. This Rust-side
//! source-cite pin grep-asserts the YAML still names the three test
//! files. Defends against silent CI drift.
//!
//! ## R4-FP recalibration per r4-r1-wsa-7 (MINOR — shape-hardening)
//!
//! r4-r1-wsa-7 flagged that substring-presence of the test file names
//! is insufficient: a workflow refactor that switches to
//! `cargo test --workspace` (which would compile + run ALL tests,
//! dramatically expanding cell time and breaking the targeted-3-tests
//! intent) wouldn't fail the pin — the YAML still names the 3 test
//! files (perhaps in a comment) and the cell still exists. Per pim-2
//! §3.6b the test pins the INVOCATION SHAPE not just the file names.
//! The recalibrated pin asserts:
//!   1. `cargo nextest run` is the invocation tool (NOT `cargo test`).
//!   2. `--target aarch64-apple-darwin` flag is present.
//!   3. Each `--test <name>` flag appears with its expected name (regex-
//!      anchored on `--test\s+sandbox_basic`, NOT bare substring
//!      `sandbox_basic`).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-B wave 5b extends multi-arch-cargo-check.yml with macos-latest-arm64 cell per phase-3-backlog §6.7 + r4-r1-wsa-7 shape-hardening (cargo nextest run + --target + flag-position --test invocation)"]
fn aarch64_sandbox_runtime_ci_cell_green() {
    // r2-test-landscape §2.5 G17-B AArch64 cell pin. G17-B implementer wires:
    //
    //   let workflow = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join(".github").join("workflows")
    //           .join("multi-arch-cargo-check.yml")
    //   ).unwrap();
    //
    //   // The AArch64 cell exists:
    //   assert!(workflow.contains("macos-latest-arm64") || workflow.contains("macos-arm64"),
    //       "multi-arch-cargo-check.yml must declare macos-latest-arm64 cell per phase-3-backlog §6.7");
    //
    //   // SHAPE assertion 1: `cargo nextest run` is the invocation tool
    //   // (per pim-2 §3.6b — NOT `cargo test --workspace`). Per
    //   // r4-r1-wsa-7 a workflow refactor to `cargo test --workspace`
    //   // would still substring-match the file names but would expand
    //   // cell time + break the targeted-3-tests intent.
    //   assert!(workflow.contains("cargo nextest run") || workflow.contains("cargo-nextest"),
    //       "AArch64 cell MUST invoke `cargo nextest run` per r4-r1-wsa-7 shape-pin (not `cargo test`)");
    //   assert!(!workflow.contains("cargo test --workspace"),
    //       "AArch64 cell MUST NOT use `cargo test --workspace` (broadens scope; defeats targeted-3-tests intent per r4-r1-wsa-7)");
    //
    //   // SHAPE assertion 2: `--target aarch64-apple-darwin` flag-position
    //   // present (not just substring of the triple).
    //   //
    //   // Use a regex-anchored match (would need `regex` dev-dep at G17-B time;
    //   // for now the implementer pins the cleaner shape):
    //   //
    //   //   let target_re = regex::Regex::new(r"--target\s+aarch64-apple-darwin").unwrap();
    //   //   assert!(target_re.is_match(&workflow),
    //   //       "AArch64 cell MUST pass `--target aarch64-apple-darwin` flag per r4-r1-wsa-7");
    //   //
    //   // Or via lightweight char-window scan if dev-dep absent: find
    //   // `aarch64-apple-darwin` substring; assert preceding ~16 chars
    //   // contain `--target`.
    //
    //   // SHAPE assertion 3: each `--test <name>` flag appears in
    //   // flag-position form (NOT bare substring). Regex-anchored on
    //   // `--test\s+<name>`:
    //   for test_name in &[
    //       "sandbox_basic",
    //       "sandbox_escape_attempts_denied",
    //       "sandbox_severity_priority",
    //   ] {
    //       // let test_re = regex::Regex::new(&format!(r"--test\s+{}", test_name)).unwrap();
    //       // assert!(test_re.is_match(&workflow),
    //       //     "AArch64 cell MUST pass `--test {}` flag per r4-r1-wsa-7 \
    //       //      (bare substring presence is insufficient — flag-position required)",
    //       //     test_name);
    //       //
    //       // Lightweight alternative: locate `<name>` substring; assert preceding
    //       // ~10 chars contain `--test`:
    //       let mut found_in_flag_position = false;
    //       for (idx, _) in workflow.match_indices(test_name) {
    //           let preceding = &workflow[idx.saturating_sub(10)..idx];
    //           if preceding.contains("--test") {
    //               found_in_flag_position = true;
    //               break;
    //           }
    //       }
    //       assert!(found_in_flag_position,
    //           "AArch64 cell MUST pass `--test {}` flag per r4-r1-wsa-7 \
    //            (flag-position required; bare substring presence in YAML \
    //            comments is INSUFFICIENT to defend the targeted-3-tests intent)",
    //           test_name);
    //   }
    //
    //   // SHAPE assertion 4: the named test files exist on disk:
    //   for test_file in &[
    //       "sandbox_basic.rs",
    //       "sandbox_escape_attempts_denied.rs",
    //       "sandbox_severity_priority.rs",
    //   ] {
    //       let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("tests").join(test_file);
    //       assert!(path.exists(),
    //           "AArch64 cell references missing test file {}", test_file);
    //   }
    //
    // OBSERVABLE consequence: a workflow rename/relocation OR a refactor
    // to `cargo test --workspace` that breaks the three-file targeted
    // invocation contract fails this pin. Defends pim-1 §3.5b
    // doc-coupling + pim-3 §3.9 R2 lens-menu correctness + r4-r1-wsa-7
    // shape-hardening (flag-position required, not substring presence).
    unimplemented!(
        "G17-B wires multi-arch-cargo-check.yml AArch64 cell + Rust-side YAML invocation-shape assertion (cargo nextest run + --target aarch64-apple-darwin + per-test --test flag-position per r4-r1-wsa-7)"
    );
}
