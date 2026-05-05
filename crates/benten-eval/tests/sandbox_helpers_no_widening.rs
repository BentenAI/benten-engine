//! R3-E RED-PHASE pins for G20-A1 §7.3.A.7 SANDBOX-escape testing helpers
//! cfg-gating audit (wave-8a; HIGH-risk security-shape per scope-real-03).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-A1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G20-A1 must-pass column):
//!
//! - `tests/sandbox_escape_helpers_no_widening_of_production_attack_surface` —
//!   §7.3.A.7 LOAD-BEARING security pin per Phase-2a sec-r6r2-02 precedent
//!
//! ## What G20-A1 establishes (§7.3.A.7)
//!
//! G17-A1 wave-5b shipped the helper SURFACE (per seq-minor-2). G20-A1
//! wave-8a un-ignores the test bodies AND verifies that the helper
//! cfg-gating discipline holds: testing helpers are visible ONLY in
//! test / `feature = "test-helpers"` builds, NEVER in the production
//! cdylib.
//!
//! Per Phase-2a `sec-r6r2-02` precedent + memory `feedback_understand_lint_root_cause`:
//! cfg-gating audit MUST be a load-bearing pin (not a sentinel-presence
//! check), because testing-helper widening into production is the most
//! catastrophic ESC defense bypass mode.
//!
//! ## RED-PHASE discipline
//!
//! Production-side audit doesn't exist yet as a load-bearing pin. R5
//! implementer drops `#[ignore]` after wiring G20-A1.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-A1 wave-8a wires LOAD-BEARING cfg-gating audit pin (§7.3.A.7)"]
fn sandbox_escape_helpers_no_widening_of_production_attack_surface() {
    // §7.3.A.7 LOAD-BEARING pin per Phase-2a sec-r6r2-02 precedent.
    // G20-A1 implementer wires this:
    //
    //   // Audit 1: scan crates/benten-eval/src/sandbox/testing_helpers.rs
    //   // (or the helper module) and verify EVERY `pub` item is gated
    //   // behind `#[cfg(any(test, feature = "test-helpers"))]`:
    //   let helpers_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("src").join("sandbox").join("testing_helpers.rs");
    //   let src = std::fs::read_to_string(&helpers_path).unwrap();
    //
    //   // Every `pub fn`, `pub struct`, `pub enum`, `pub use` in this
    //   // file must be preceded by a cfg-gate matching the audit:
    //   for item in extract_pub_items(&src) {
    //       assert!(item.has_cfg_gate(),
    //           "testing helper {} is `pub` without cfg(any(test, feature = \
    //            \"test-helpers\"))) gate — Phase-2a sec-r6r2-02 violation",
    //           item.name);
    //   }
    //
    //   // Audit 2: production cdylib (default features) does NOT compile
    //   // any testing helper. This is verified by a build-time check —
    //   // the cdylib build with `--no-default-features` (or default
    //   // features without test-helpers) MUST NOT export any helper symbol.
    //
    //   // Audit 3: the integration tests at tests/sandbox_*.rs which
    //   // CONSUME the helpers MUST be gated themselves (tests are
    //   // already in the test-only namespace, but the helper imports
    //   // from feature = "test-helpers" must be cfg-gated to avoid
    //   // bringing the helper into a production rlib).
    //
    // OBSERVABLE consequence: production attack surface stays LOCKED
    // — testing helpers cannot widen it. Defends against the
    // catastrophic "test helper accidentally exported" failure mode
    // that Phase-2a sec-r6r2-02 named.
    unimplemented!("G20-A1 wires LOAD-BEARING cfg-gating audit (Phase-2a sec-r6r2-02 precedent)");
}

#[test]
#[ignore = "RED-PHASE: G20-A1 wave-8a — §7.3.A.7 + §7.3.A.1 test bodies un-ignored regression check"]
fn no_phase_3_destination_remaining_in_sandbox_or_attribution_test_ignores() {
    // G20-A1 closure pin. After wave-8a un-ignores all §7.3.A.1
    // (runtime SANDBOX invariant + attribution-frame) + §7.3.A.7
    // (testing helpers integration) test bodies, NO `#[ignore]`
    // rationale should still name "Phase 3" as the destination.
    //
    //   let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("tests");
    //   let mut residuals = Vec::new();
    //
    //   for entry in std::fs::read_dir(&test_dir).unwrap() {
    //       let path = entry.unwrap().path();
    //       if !path.extension().map_or(false, |e| e == "rs") { continue; }
    //       let name = path.file_name().unwrap().to_string_lossy().to_string();
    //       if !name.starts_with("sandbox_") { continue; }
    //       let src = std::fs::read_to_string(&path).unwrap();
    //       for line in src.lines() {
    //           if line.contains("#[ignore") && line.contains("Phase 3") {
    //               residuals.push(format!("{}: {}", name, line.trim()));
    //           }
    //       }
    //   }
    //
    //   assert!(residuals.is_empty(),
    //       "G20-A1 incomplete: residual Phase-3-destination ignores in \
    //        sandbox_*.rs tests:\n{}", residuals.join("\n"));
    //
    // OBSERVABLE consequence: the wave-8j R6 residuals fully clear at
    // G20-A1 close. End-to-end pin per §3.6b — would FAIL if the
    // un-ignore sweep missed any sandbox-test file.
    unimplemented!("G20-A1 wires no-Phase-3-residual-ignore audit for sandbox test files");
}
