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

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-B wave 5b extends multi-arch-cargo-check.yml with macos-latest-arm64 cell per phase-3-backlog §6.7"]
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
    //   // The cell runs the three named test files (per r1-wsa-9 verification):
    //   assert!(workflow.contains("sandbox_basic"),
    //       "AArch64 cell must run sandbox_basic per r2-test-landscape §2.5 G17-B");
    //   assert!(workflow.contains("sandbox_escape_attempts_denied"),
    //       "AArch64 cell must run sandbox_escape_attempts_denied per r2-test-landscape §2.5 G17-B");
    //   assert!(workflow.contains("sandbox_severity_priority"),
    //       "AArch64 cell must run sandbox_severity_priority per r1-wsa-9 verified test name");
    //
    //   // The named test files exist on disk:
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
    // OBSERVABLE consequence: a workflow rename/relocation that breaks
    // the three-file invocation contract fails this pin. Defends
    // pim-1 §3.5b doc-coupling + pim-3 §3.9 R2 lens-menu correctness
    // for the AArch64 cell.
    unimplemented!(
        "G17-B wires multi-arch-cargo-check.yml AArch64 cell + Rust-side YAML cite assertion"
    );
}
