//! Self-test for the R6-R2-FP-C glob-cite phantom detection pass.
//!
//! Per `feedback_pim_n_sweep_completeness_self_verify` + §3.6j: a scanner
//! that claims to detect drift MUST include a known-phantom fixture that
//! the scanner detects. This test plants a known phantom + asserts the
//! scanner catches it.
//!
//! Companion to:
//!   - `cite_drift_detector_finds_known_drift_fixture.rs` (cite-drift)
//!   - `numeric_claim_drift_lint_finds_known_drift_fixture.rs` (numeric)
//!   - `read_view_with_lint_finds_known_drift_fixture.rs` (read-view)
//!
//! Memory: `feedback_pim_n_cite_grep_verify_at_author_time.md`.

use std::path::PathBuf;

use cite_drift_detector::{FindingKind, run_glob_cite_check};

/// Plant a synthetic workspace with one valid glob (matches files) +
/// one phantom glob (matches zero files) + one bare-basename glob
/// (should be skipped). Assert the scanner returns exactly one finding
/// for the phantom + skips the others.
#[test]
fn glob_cite_scanner_detects_phantom_glob_in_synthetic_tree() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // Plant a `crates/foo-crate/tests/real_pin_a.rs` so the
    // `crates/foo-crate/tests/real_pin_*.rs` glob is non-phantom.
    let tests_dir = root.join("crates").join("foo-crate").join("tests");
    std::fs::create_dir_all(&tests_dir).unwrap();
    std::fs::write(tests_dir.join("real_pin_a.rs"), "//! real pin\n").unwrap();
    std::fs::write(tests_dir.join("real_pin_b.rs"), "//! real pin\n").unwrap();

    // Plant a `docs/` doc that cites:
    //   (a) `crates/foo-crate/tests/real_pin_*.rs` — real, should not flag
    //   (b) `crates/foo-crate/tests/phantom_*.rs` — phantom, should flag
    //   (c) `phantom_basename_*.rs` — bare basename, should skip silently
    //   (d) `crates/foo-crate/src/**/*.rs` — recursive `**`, should skip
    //   (e) `crates/foo-crate/tests/exempt_*.rs` with `<!-- cite-drift-exempt -->` — should skip
    let docs_dir = root.join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::write(
        docs_dir.join("FIXTURE.md"),
        "\
# Fixture\n\
\n\
See `crates/foo-crate/tests/real_pin_*.rs` for the real pins.\n\
See `crates/foo-crate/tests/phantom_*.rs` for the (PHANTOM) pins.\n\
See `phantom_basename_*.rs` for bare-basename (should-skip) glob.\n\
See `crates/foo-crate/src/**/*.rs` for recursive (should-skip) glob.\n\
See `crates/foo-crate/tests/exempt_*.rs` <!-- cite-drift-exempt --> for exempt.\n\
",
    )
    .unwrap();

    let findings = run_glob_cite_check(root);
    // Expect exactly ONE finding — the phantom glob at line 4 of FIXTURE.md.
    assert_eq!(
        findings.len(),
        1,
        "expected 1 glob phantom finding; got {} :: {:#?}",
        findings.len(),
        findings
    );
    let f = &findings[0];
    assert_eq!(f.kind, FindingKind::LineCiteGlobNoMatch);
    assert!(
        f.message.contains("phantom_*.rs"),
        "finding message should mention the phantom glob; got: {}",
        f.message
    );
    let path_str = f.path.to_string_lossy();
    assert!(
        path_str.ends_with("FIXTURE.md"),
        "finding should cite the source doc; got: {path_str}"
    );
    assert_eq!(
        f.line, 4,
        "phantom is on line 4 of fixture; got line {}",
        f.line
    );
}

/// Validate the basename-glob matcher handles single-`*`, leading-`*`,
/// trailing-`*`, and mid-`*` patterns correctly. (Covers the parser
/// directly via `glob_has_match` which is the only public seam to
/// validate matcher semantics from a test crate.)
#[test]
fn glob_basename_matcher_handles_common_patterns() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let dir = root.join("crates").join("x").join("tests");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("prefix_foo_suffix.rs"), "//\n").unwrap();
    std::fs::write(dir.join("prefix_bar_suffix.rs"), "//\n").unwrap();
    std::fs::write(dir.join("totally_different.rs"), "//\n").unwrap();

    // We test indirectly by running the full scanner against a doc
    // containing each glob shape; one glob phantom = one finding.
    let docs_dir = root.join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();

    // Each line is its own glob cite; we verify match-vs-no-match per glob.
    std::fs::write(
        docs_dir.join("MATCHER.md"),
        "\
# Matcher tests\n\
\n\
Line A: `crates/x/tests/prefix_*.rs` matches prefix_foo + prefix_bar.\n\
Line B: `crates/x/tests/*_suffix.rs` matches both _suffix files.\n\
Line C: `crates/x/tests/prefix_*_suffix.rs` matches both.\n\
Line D: `crates/x/tests/nonexistent_*.rs` matches NOTHING (PHANTOM).\n\
Line E: `crates/x/tests/*.rs` matches all 3 files.\n\
",
    )
    .unwrap();

    let findings = run_glob_cite_check(root);
    // Only Line D should be phantom.
    assert_eq!(
        findings.len(),
        1,
        "expected 1 phantom from Line D; got {} :: {:#?}",
        findings.len(),
        findings
    );
    assert_eq!(findings[0].line, 6); // Line D in the doc
    assert!(findings[0].message.contains("nonexistent_*.rs"));
}

/// The scanner walker excludes the detector's own subtree (the
/// `cite-drift-detector` skip in `walk_ext_recursive`); without this,
/// the scanner's own test fixtures would self-flag. Validate by
/// running the scanner against the LIVE workspace root and confirming
/// that the scanner's own doc-comments-with-glob-shaped-strings (if
/// any) don't trip a finding.
#[test]
fn scanner_self_does_not_phantom_against_its_own_source_globs() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur = manifest_dir.as_path();
    let workspace_root = loop {
        if cur.join("crates").is_dir() && cur.join("packages").is_dir() {
            break cur.to_path_buf();
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => panic!(
                "could not locate workspace root from CARGO_MANIFEST_DIR={}",
                manifest_dir.display()
            ),
        }
    };
    let findings = run_glob_cite_check(&workspace_root);
    // Strictly assert workspace cleanliness post-R6-R2-FP-C. If a future
    // change introduces a new phantom glob, this test fails with the
    // exact location (the pin enforces the §3.6j cite-grep-verify
    // discipline at CI time).
    assert!(
        findings.is_empty(),
        "expected zero glob-phantom findings on live workspace tree post-R6-R2-FP-C; \
         got {} finding(s):\n{:#?}",
        findings.len(),
        findings
    );
}
