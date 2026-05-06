//! Phase-3 G13-pre-A pin: drives a fixture root containing planted line-cite
//! drift + planted symbol-cite drift + a planted high-churn surface bare
//! line cite. Asserts `run_cite_drift_check` emits the EXPECTED kinds with
//! the EXPECTED source-of-cite locations.
//!
//! pim-2 §3.6b end-to-end pin: drives the production entry point
//! (`run_cite_drift_check`) + asserts observable behavioral consequence
//! (specific finding kinds at specific locations); a silent-no-op detector
//! would fail this test.

use std::fs;

use cite_drift_detector::{FindingKind, run_cite_drift_check};

#[test]
fn cite_drift_detector_finds_known_drift_fixture() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // -- planted target file (exists; has 5 lines) -----------------------
    // `lib.rs` is on the high-churn list (it's a load-bearing convention
    // surface), so we use a non-high-churn basename here for the
    // line-range + symbol drift assertions; the high-churn check lives
    // on a separate fixture file (`packages/engine/src/dsl.ts` below).
    fs::create_dir_all(root.join("crates/foo/src")).unwrap();
    fs::write(
        root.join("crates/foo/src/utility.rs"),
        "// line 1\n\
         // line 2\n\
         pub fn real_symbol() {}\n\
         // line 4\n\
         // line 5\n",
    )
    .unwrap();

    // -- planted high-churn target (basename = `dsl.ts`) -----------------
    fs::create_dir_all(root.join("packages/engine/src")).unwrap();
    fs::write(
        root.join("packages/engine/src/dsl.ts"),
        "// dsl.ts content\nexport function realFn() {}\n",
    )
    .unwrap();

    // -- doc planting drift: 4 distinct findings -------------------------
    fs::create_dir_all(root.join("docs")).unwrap();
    let doc = "\
# Drift fixture

Line cite that points at a missing file:
crates/foo/src/missing.rs:7

Line cite where the file exists but the line is out of range:
crates/foo/src/utility.rs:9999

Line cite against a HIGH-CHURN surface (must use symbol form):
packages/engine/src/dsl.ts:34

Symbol cite where the file is missing:
crates/bar/src/lib.rs::ghost_symbol

Symbol cite where the file exists but the symbol does not:
crates/foo/src/utility.rs::nonexistent_symbol

Clean line cite (file exists, line in range, NOT high-churn):
crates/foo/src/utility.rs:3

Clean symbol cite (file exists, symbol defined):
crates/foo/src/utility.rs::real_symbol
";
    fs::write(root.join("docs/DRIFT-FIXTURE.md"), doc).unwrap();

    // -- run the detector ------------------------------------------------
    let findings = run_cite_drift_check(root);

    // -- behavioral consequence assertions -------------------------------
    // (pim-2 §3.6b: each assertion would fail if the detector silently
    // no-op'd, NOT just sentinel-presence.)

    // Drift kinds we MUST see:
    let kinds: Vec<FindingKind> = findings.iter().map(|f| f.kind).collect();
    assert!(
        kinds.contains(&FindingKind::LineCiteFileMissing),
        "expected LineCiteFileMissing finding; got {:?}",
        kinds
    );
    assert!(
        kinds.contains(&FindingKind::LineCiteLineOutOfRange),
        "expected LineCiteLineOutOfRange finding; got {:?}",
        kinds
    );
    assert!(
        kinds.contains(&FindingKind::LineCiteOnHighChurnSurface),
        "expected LineCiteOnHighChurnSurface finding (dsl.ts:34); got {:?}",
        kinds
    );
    assert!(
        kinds.contains(&FindingKind::SymbolCiteFileMissing),
        "expected SymbolCiteFileMissing finding; got {:?}",
        kinds
    );
    assert!(
        kinds.contains(&FindingKind::SymbolCiteSymbolMissing),
        "expected SymbolCiteSymbolMissing finding; got {:?}",
        kinds
    );

    // Clean cites must NOT generate findings — confirms the detector
    // distinguishes drift from valid cites (a silent always-pass detector
    // would not fail; a silent always-fail detector WOULD fail this).
    let clean_line_cite_falsely_flagged = findings.iter().any(|f| {
        f.message.contains("crates/foo/src/utility.rs:3")
            && (f.kind == FindingKind::LineCiteFileMissing
                || f.kind == FindingKind::LineCiteLineOutOfRange)
    });
    assert!(
        !clean_line_cite_falsely_flagged,
        "clean line cite (lib.rs:3) was falsely flagged: {:?}",
        findings
    );

    let clean_symbol_cite_falsely_flagged = findings
        .iter()
        .any(|f| f.message.contains("crates/foo/src/utility.rs::real_symbol"));
    assert!(
        !clean_symbol_cite_falsely_flagged,
        "clean symbol cite (lib.rs::real_symbol) was falsely flagged: {:?}",
        findings
    );

    // Source-of-cite locations are observable: every finding must point
    // back to the doc file we planted.
    for f in &findings {
        assert!(
            f.path.ends_with("docs/DRIFT-FIXTURE.md"),
            "finding source-path is not the planted doc: {:?}",
            f
        );
        assert!(
            f.line > 0,
            "finding line must be 1-indexed and non-zero: {:?}",
            f
        );
    }
}

// r4-r2-ivm-5 fixture-extension RED-PHASE pin (added 2026-05-05).
//
// At G15-A landing, this fixture is extended with a SECURITY-POSTURE
// Compromise #11 mini-fixture that exercises the cite-drift detector
// against the proptest-symbol-of-record + materialization-gate
// symbol-of-record. The narrative covers TWO symbol-form cites that
// MUST be findable post-G15-A:
//
//   crates/benten-ivm/tests/algorithm_b_drift_detector.rs::prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern
//   crates/benten-engine/tests/<TBD>::ivm_view_per_row_read_gate_against_actor_cap_set
//
// Per ivm-major-2 narrative + §3.5b HARDENED point-1: when SECURITY-
// POSTURE.md Compromise #11 closure narrative cites these symbols, the
// cite-drift detector verifies the symbols still resolve at HEAD. A
// regression that renames or deletes either symbol triggers
// SymbolCiteSymbolMissing finding through this fixture.
//
// G15-A wave-5a implementer wires this:
//
//   #[test]
//   fn cite_drift_detector_finds_security_posture_compromise_11_proptest_drift() {
//       let tmp = tempfile::tempdir().expect("tempdir");
//       let root = tmp.path();
//
//       // Plant the proptest target file with the actual G15-B symbol:
//       fs::create_dir_all(root.join("crates/benten-ivm/tests")).unwrap();
//       fs::write(
//           root.join("crates/benten-ivm/tests/algorithm_b_drift_detector.rs"),
//           // ... real file contents with the proptest symbol present ...
//       ).unwrap();
//
//       // Plant a SECURITY-POSTURE.md fragment that cites the symbol AND
//       // a stale (renamed/missing) symbol — assert the detector flags
//       // the stale one but NOT the live one:
//       fs::create_dir_all(root.join("docs")).unwrap();
//       let posture = "\
// # SECURITY-POSTURE.md (fixture)
//
// ## Compromise #11 — IVM views coarse-grained read-gate
//
// CLOSED at G15-A via materialization-time gate composing with G14-D
// delivery-time gate. Closure narrative cites:
//   - crates/benten-ivm/tests/algorithm_b_drift_detector.rs::prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern (LIVE)
//   - crates/benten-ivm/tests/algorithm_b_drift_detector.rs::prop_old_renamed_symbol (DRIFTED — should flag)
// ";
//       fs::write(root.join("docs/SECURITY-POSTURE.md"), posture).unwrap();
//
//       let findings = run_cite_drift_check(root);
//
//       // The live symbol cite must NOT flag:
//       assert!(!findings.iter().any(|f|
//           f.message.contains("prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern")
//           && f.kind == FindingKind::SymbolCiteSymbolMissing
//       ));
//       // The drifted symbol cite MUST flag:
//       assert!(findings.iter().any(|f|
//           f.message.contains("prop_old_renamed_symbol")
//           && f.kind == FindingKind::SymbolCiteSymbolMissing
//       ));
//   }
//
// OBSERVABLE consequence: a refactor that renames the proptest-symbol-
// of-record post-G15-A (and forgets to retense SECURITY-POSTURE.md
// Compromise #11 closure narrative) is caught by this fixture pin. Per
// pim-1 §3.5b HARDENED post-fix doc-coupling. r4-r2-ivm-5 fixture-
// extension closure.
