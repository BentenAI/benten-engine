//! Phase 2b R4-FP B-4 — `docs/ARCHITECTURE.md` 8-crate count drift detector.
//!
//! TDD red-phase. Pin source: architect-r1 carry item (the
//! `benten-dsl-compiler` crate lands as part of plan §3.2 G12-B;
//! ARCHITECTURE.md currently states "Seven crates" / "## Seven crates"
//! and must be updated to "Eight crates" once G12-B lands).
//!
//! Drift discipline: doc-as-source-of-truth on the workspace shape
//! must agree with the actual `crates/` directory layout. Without
//! this test, ARCHITECTURE.md would silently drift from the workspace
//! manifest the way Phase-2a R7 audits caught aspirational-prose-but-
//! dead-code regressions repeatedly (CLAUDE.md: "Verify, don't trust
//! docs").
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `architecture_md_says_eight_crates_after_g12b` — architect-r1 carry.
///
/// Asserts ARCHITECTURE.md says "Eight crates" (or "8 crates") in the
/// section header AND in the prose body, after G12-B lands the
/// `benten-dsl-compiler` crate. The Phase-1/2a phrasing was "Seven
/// crates"; if the doc still says "Seven" after G12-B, operators
/// reading the doc will miss the dsl-compiler boundary.
#[test]
#[ignore = "Phase 2b G12-B pending — benten-dsl-compiler crate + ARCHITECTURE.md update unimplemented"]
fn architecture_md_says_eight_crates_after_g12b() {
    let root = workspace_root();
    let doc_path = root.join("docs/ARCHITECTURE.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/ARCHITECTURE.md not found at {} ({}); this is a \
             load-bearing doc — pre-existing in Phase 1.",
            doc_path.display(),
            e
        );
    });

    let lower = doc_src.to_ascii_lowercase();

    // After G12-B, the "Seven crates" phrasing MUST be gone.
    assert!(
        !lower.contains("seven crates")
            && !lower.contains("## 7 crates")
            && !lower.contains("# seven"),
        "docs/ARCHITECTURE.md still contains pre-G12-B phrasing 'Seven \
         crates' / '## 7 crates' / '# Seven'. After benten-dsl-compiler \
         lands per architect-r1 carry, the doc MUST update to 'Eight \
         crates' (G11-2b-A doc sweep — paired with G12-B landing)."
    );

    // Should explicitly assert the new count.
    let says_eight = lower.contains("eight crates")
        || lower.contains("## 8 crates")
        || lower.contains("# eight");
    assert!(
        says_eight,
        "docs/ARCHITECTURE.md MUST explicitly state 'Eight crates' / \
         '## 8 crates' / similar after G12-B lands the benten-dsl-compiler \
         crate (architect-r1 carry; G11-2b-A doc sweep)."
    );

    // The dsl-compiler crate itself must be listed.
    assert!(
        lower.contains("benten-dsl-compiler"),
        "docs/ARCHITECTURE.md MUST mention `benten-dsl-compiler` by \
         name after G12-B lands the crate."
    );
}

/// Workspace-shape sanity check — verifies the actual `crates/` layout
/// matches the doc. R5 G12-B landing will make this pass; until then
/// the dsl-compiler dir does not exist and this fails to enforce the
/// dependency.
#[test]
#[ignore = "Phase 2b G12-B pending — benten-dsl-compiler crate not yet created"]
fn workspace_has_benten_dsl_compiler_crate_dir() {
    let root = workspace_root();
    let crate_dir = root.join("crates/benten-dsl-compiler");

    assert!(
        crate_dir.is_dir(),
        "crates/benten-dsl-compiler/ MUST exist after G12-B lands \
         (architect-r1 carry; plan §3.2 G12-B). Without the directory, \
         the 8-crates phrasing in ARCHITECTURE.md would be aspirational \
         (the regression Phase-1 R7 audit caught repeatedly)."
    );
}
