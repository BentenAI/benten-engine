//! Phase-3 G21-T4 pin: drives `run_read_view_with_lint` against a fixture
//! root containing a `.rs` file with both canonical (`content_listing_<label>`
//! / `system:ivm:content_listing_<label>`) and non-canonical
//! `read_view_with("<view_id>", ...)` callsites. Compromise #11
//! (`docs/SECURITY-POSTURE.md`) closure relies on registry-driven label-hint
//! resolution at
//! `crates/benten-engine/src/engine_views.rs::resolve_read_view_label_hint`;
//! this lint is a regression-prevention sanity layer for new code that
//! drifts back to passing a non-canonical literal.
//!
//! pim-2 §3.6b end-to-end pin: drives the production entry point + asserts
//! observable behavioral consequence (specific drift detected at specific
//! callsite); a silent-no-op detector would fail this test.

use std::fs;

use cite_drift_detector::{FindingKind, run_read_view_with_lint};

#[test]
fn read_view_with_lint_finds_known_drift_fixture() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // Plant a `.rs` source under crates/ so the walker picks it up.
    let crates_dir = root.join("crates").join("test-fixture-crate").join("src");
    fs::create_dir_all(&crates_dir).unwrap();

    // Mix canonical + non-canonical callsites + a non-literal callsite
    // (the lint must skip the non-literal because static analysis cannot
    // resolve runtime expressions).
    let src = "\
//! Test fixture with intentional Compromise #11 regression patterns.

fn canonical_calls() {
    let _ = engine.read_view_with(\"content_listing_post\", strict);
    let _ = engine.read_view_with(\"system:ivm:content_listing_user\", strict);
}

fn non_canonical_calls() {
    let _ = engine.read_view_with(\"my_arbitrary_view\", strict);
    let _ = engine.read_view_with(\"random_id\", opts);
}

fn runtime_view_id_is_skipped() {
    let id = compute_view_id();
    let _ = engine.read_view_with(&id, opts);
}
";
    fs::write(crates_dir.join("fixture.rs"), src).unwrap();

    let findings = run_read_view_with_lint(root);

    // -- behavioral consequence assertions -------------------------------
    let non_canonical_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == FindingKind::NonCanonicalReadViewWithViewId)
        .collect();

    assert_eq!(
        non_canonical_findings.len(),
        2,
        "expected exactly 2 non-canonical findings (my_arbitrary_view + random_id); got {:?}",
        findings
    );

    let flagged_my_arbitrary = non_canonical_findings
        .iter()
        .any(|f| f.message.contains("my_arbitrary_view"));
    assert!(
        flagged_my_arbitrary,
        "expected `my_arbitrary_view` to be flagged; got {:?}",
        findings
    );

    let flagged_random_id = non_canonical_findings
        .iter()
        .any(|f| f.message.contains("random_id"));
    assert!(
        flagged_random_id,
        "expected `random_id` to be flagged; got {:?}",
        findings
    );

    // Canonical callsites NOT flagged.
    let canonical_falsely_flagged = non_canonical_findings.iter().any(|f| {
        f.message.contains("content_listing_post")
            || f.message.contains("system:ivm:content_listing_user")
    });
    assert!(
        !canonical_falsely_flagged,
        "canonical callsite was falsely flagged: {:?}",
        findings
    );

    // Runtime expression (`&id`) NOT flagged — static analysis cannot
    // resolve the value.
    let runtime_falsely_flagged = non_canonical_findings
        .iter()
        .any(|f| f.message.contains("compute_view_id") || f.message.contains("&id"));
    assert!(
        !runtime_falsely_flagged,
        "runtime view-id expression was falsely flagged: {:?}",
        findings
    );

    // Source-of-cite location is observable.
    for f in &non_canonical_findings {
        assert!(
            f.path.ends_with("fixture.rs"),
            "finding source-path is not the planted source: {:?}",
            f
        );
        assert!(
            f.line > 0,
            "finding line must be 1-indexed and non-zero: {:?}",
            f
        );
    }
}
