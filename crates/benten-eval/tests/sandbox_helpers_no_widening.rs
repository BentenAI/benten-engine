//! G20-A1 §7.3.A.7 SANDBOX-escape testing helpers cfg-gating audit
//! (wave-8a; HIGH-risk security-shape per scope-real-03).
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

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn sandbox_escape_helpers_no_widening_of_production_attack_surface() {
    // §7.3.A.7 LOAD-BEARING pin per Phase-2a sec-r6r2-02 precedent.
    //
    // **G20-A1 wave-8a body** (Phase 3): scan the helper module
    // source + verify EVERY `pub` item in the file is gated behind
    // a cfg attribute that includes `feature = "test-helpers"` (or
    // `test` / `testing`).
    //
    // The audit is structural — the helper module itself carries a
    // FILE-LEVEL `#![cfg(any(test, feature = "test-helpers", feature = "testing"))]`
    // gate. With that gate at the top of the file, every `pub` item
    // inside the file is automatically cut from any build that does
    // NOT enable one of those legs.
    //
    // We assert:
    //   1. The helper file exists at the expected path.
    //   2. The file-level cfg gate is present + names the
    //      `test-helpers` feature.
    //   3. NO `pub` item appears in the file BEFORE the file-level
    //      cfg gate (a regression that moved the gate after a `pub`
    //      item would silently widen the surface).
    //   4. The `Cargo.toml` does NOT enable `test-helpers` in the
    //      `default` features — production builds do NOT pull the
    //      helpers in.
    let helpers_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("sandbox")
        .join("testing_helpers.rs");
    let src = std::fs::read_to_string(&helpers_path)
        .unwrap_or_else(|e| panic!("helper file MUST exist at {helpers_path:?}: {e}"));

    // Audit (1): file-level cfg gate present + names test-helpers.
    let gate_line = src
        .lines()
        .find(|l| l.starts_with("#![cfg(") && l.contains("test"));
    let gate = gate_line.unwrap_or_else(|| {
        panic!(
            "G20-A1 LOAD-BEARING: testing_helpers.rs MUST carry a \
             file-level `#![cfg(...)]` gate restricting it to test / \
             test-helpers builds (Phase-2a sec-r6r2-02 precedent)"
        )
    });
    assert!(
        gate.contains("feature = \"test-helpers\"") || gate.contains("feature = \"testing\""),
        "G20-A1 LOAD-BEARING: file-level cfg MUST name the \
         `test-helpers` (or `testing`) feature; got: {gate}"
    );
    assert!(
        gate.contains("test"),
        "G20-A1 LOAD-BEARING: file-level cfg MUST include the `test` \
         leg (so cargo-test in the same crate compiles cleanly); \
         got: {gate}"
    );

    // Audit (2): NO `pub` item in the file BEFORE the cfg gate. The
    // gate's index is the cutover point.
    let gate_pos = src.find(gate).expect("gate text was found by find() above");
    let preamble = &src[..gate_pos];
    let preamble_lines: Vec<&str> = preamble.lines().collect();
    for (lineno, line) in preamble_lines.iter().enumerate() {
        let trimmed = line.trim_start();
        assert!(
            !trimmed.starts_with("pub fn ")
                && !trimmed.starts_with("pub struct ")
                && !trimmed.starts_with("pub enum ")
                && !trimmed.starts_with("pub use ")
                && !trimmed.starts_with("pub const ")
                && !trimmed.starts_with("pub trait "),
            "G20-A1 LOAD-BEARING: NO `pub` item allowed BEFORE the \
             file-level cfg gate in testing_helpers.rs (a regression \
             would silently widen the production surface). Offending \
             line {}: {}",
            lineno + 1,
            line
        );
    }

    // Audit (3): Cargo.toml does NOT enable `test-helpers` in
    // default features.
    let cargo_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let cargo_src = std::fs::read_to_string(&cargo_path).expect("benten-eval Cargo.toml readable");

    // Walk the [features] section, find `default = [...]`, assert
    // it does NOT mention `test-helpers`.
    let mut in_features = false;
    for line in cargo_src.lines() {
        let trimmed = line.trim();
        if trimmed == "[features]" {
            in_features = true;
            continue;
        }
        if trimmed.starts_with('[') && in_features {
            in_features = false;
        }
        if in_features && trimmed.starts_with("default") {
            assert!(
                !trimmed.contains("test-helpers"),
                "G20-A1 LOAD-BEARING: benten-eval `default` features \
                 MUST NOT include `test-helpers` (production cdylib \
                 would compile in helpers); got: {trimmed}"
            );
        }
    }
}

#[test]
fn no_phase_3_destination_remaining_in_sandbox_or_attribution_test_ignores() {
    // **G20-A1 wave-8a body** (Phase 3): closure pin. After
    // un-ignoring the §7.3.A.1 + §7.3.A.7 cluster, NO `#[ignore]`
    // rationale in `tests/sandbox_*.rs` should still name "Phase 3"
    // as the destination.
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let entries = std::fs::read_dir(&test_dir).expect("test dir readable");
    let mut residuals: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        // Also include attribution + invariant_4_runtime + invariant_7_runtime
        // since they're part of §7.3.A.1.
        if !(name.starts_with("sandbox_")
            || name.starts_with("attribution_")
            || name.starts_with("invariant_4_runtime")
            || name.starts_with("invariant_7_runtime")
            || name.starts_with("proptest_sandbox_"))
        {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("test source readable");
        for (lineno, line) in src.lines().enumerate() {
            // Match `#[ignore` rationales naming §7.3.A.1 OR §7.3.A.7
            // — these are the G20-A1 wave-8a destinations + must be
            // empty post-close. §7.3.A.8 (Component-Model gated tests)
            // belongs to G20-A3 and is OUT OF G20-A1 scope; that
            // cluster carries `Phase 3+` rationales that this audit
            // intentionally does NOT pick up.
            let trimmed = line.trim_start();
            if trimmed.starts_with("#[ignore")
                && (line.contains("§7.3.A.1") || line.contains("§7.3.A.7"))
            {
                residuals.push(format!("{}:{}: {}", name, lineno + 1, line.trim()));
            }
        }
    }

    assert!(
        residuals.is_empty(),
        "G20-A1 incomplete: {} residual `#[ignore]` rationale(s) \
         naming \"Phase 3\" in sandbox_/attribution_/invariant_4_runtime \
         / invariant_7_runtime / proptest_sandbox_ test files:\n{}",
        residuals.len(),
        residuals.join("\n")
    );
}
