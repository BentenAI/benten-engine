//! Phase-3 G13-pre-A pin: drives `run_numeric_claim_check_with_truth` against
//! a fixture root containing a doc that disagrees with the source-of-truth
//! numeric claims. Closes `docs/future/phase-2-backlog.md` §8.2 (cross-doc
//! numeric-claim drift lint) by reusing this tool's parser/validator.
//!
//! pim-2 §3.6b end-to-end pin: drives the production entry point + asserts
//! observable behavioral consequence (specific drift detected at specific
//! locations); a silent-no-op detector would fail this test.

use std::fs;

use cite_drift_detector::{FindingKind, NumericClaim, run_numeric_claim_check_with_truth};

#[test]
fn numeric_claim_drift_lint_finds_known_drift_fixture() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    fs::create_dir_all(root.join("docs")).unwrap();
    let doc = "\
# Numeric drift fixture

This doc claims 13 operation primitives — wrong; truth is 12.

CLAUDE.md commits to 17 invariants — wrong; truth is 14.

The workspace has 8 crates — correct.

Running 12 operation primitives somewhere correct on this line.
";
    fs::write(root.join("docs/NUMERIC-DRIFT-FIXTURE.md"), doc).unwrap();

    // Use the production source-of-truth shape, but explicitly named here
    // so the test does NOT silently track edits to the production map
    // (the test pin is for the lint mechanism, not the truth values).
    let truth: Vec<NumericClaim> = vec![
        NumericClaim {
            label: "primitives",
            value: 12,
            phrasings: &["{N} operation primitives", "{N} primitives"],
        },
        NumericClaim {
            label: "invariants",
            value: 14,
            phrasings: &["{N} invariants"],
        },
        NumericClaim {
            label: "crates",
            value: 8,
            phrasings: &["{N} crates"],
        },
    ];

    let findings = run_numeric_claim_check_with_truth(root, &truth);

    // -- behavioral consequence assertions -------------------------------
    // We expect at least one drift each for `primitives` (13 ≠ 12) and
    // `invariants` (17 ≠ 14), and ZERO drift for `crates` (8 == 8).
    let primitive_drift_found = findings.iter().any(|f| {
        f.kind == FindingKind::NumericClaimDrift
            && f.message.contains("primitives")
            && f.message.contains("13")
            && f.message.contains("12")
    });
    assert!(
        primitive_drift_found,
        "expected primitives-drift finding (13 vs 12); got {:?}",
        findings
    );

    let invariant_drift_found = findings.iter().any(|f| {
        f.kind == FindingKind::NumericClaimDrift
            && f.message.contains("invariants")
            && f.message.contains("17")
            && f.message.contains("14")
    });
    assert!(
        invariant_drift_found,
        "expected invariants-drift finding (17 vs 14); got {:?}",
        findings
    );

    let crate_drift_falsely_found = findings
        .iter()
        .any(|f| f.kind == FindingKind::NumericClaimDrift && f.message.contains("crates"));
    assert!(
        !crate_drift_falsely_found,
        "did not expect crates-drift finding (8 == 8); got {:?}",
        findings
    );

    // Source-of-cite location is observable.
    for f in &findings {
        assert!(
            f.path.ends_with("docs/NUMERIC-DRIFT-FIXTURE.md"),
            "finding source-path is not the planted doc: {:?}",
            f
        );
        assert!(
            f.line > 0,
            "finding line must be 1-indexed and non-zero: {:?}",
            f
        );
    }

    // The clean-line phrasing "Running 12 operation primitives" must NOT
    // be flagged (12 == 12).
    let clean_line_falsely_flagged = findings.iter().any(|f| {
        f.message.contains("12")
            && f.message.contains("source-of-truth")
            && !f.message.contains("13")
            && !f.message.contains("17")
    });
    assert!(
        !clean_line_falsely_flagged,
        "clean numeric phrasing (12 primitives) was falsely flagged: {:?}",
        findings
    );
}
