//! `docs/ARCHITECTURE.md` 10-crate count drift detector.
//!
//! The workspace landed at 10 crates after `benten-dsl-compiler`
//! (Phase 2b) and the `benten-id` + `benten-sync` additions (Phase 3).
//! ARCHITECTURE.md enumerates all 10 crates by name with `benten-sync`
//! flagged native-only per CLAUDE.md baked-in #17. The cite-drift
//! detector source-of-truth is bumped to 10 per
//! `tools/cite-drift-detector/src/lib.rs::numeric_claims_source_of_truth`.
//!
//! Drift discipline: doc-as-source-of-truth on the workspace shape
//! must agree with the actual `crates/` directory layout. Without
//! this test, ARCHITECTURE.md would silently drift from the workspace
//! manifest the way Phase-1 R7 audits caught aspirational-prose-but-
//! dead-code regressions repeatedly (CLAUDE.md: "Verify, don't trust
//! docs").

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `architecture_md_says_ten_crates_after_phase_3_canaries`.
///
/// Asserts ARCHITECTURE.md says "Ten crates" (or "10 crates") in the
/// section header AND in the prose body, with `benten-id` +
/// `benten-sync` listed as workspace members. The pre-Phase-3 phrasing
/// was "Eight crates"; if the doc still says "Eight", operators
/// reading the doc miss the `benten-id` + `benten-sync` boundaries.
#[test]
fn architecture_md_says_ten_crates_after_phase_3_canaries() {
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

    // The "Eight crates" / "## 8 crates" phrasing MUST be gone.
    assert!(
        !lower.contains("eight crates")
            && !lower.contains("## 8 crates")
            && !lower.contains("# eight"),
        "docs/ARCHITECTURE.md still contains pre-Phase-3 phrasing 'Eight \
         crates' / '## 8 crates' / '# Eight'. After benten-id + benten-sync \
         join the workspace, the doc MUST say 'Ten crates' (paired with \
         the cite-drift detector source-of-truth)."
    );

    // Should explicitly assert the new count.
    let says_ten =
        lower.contains("ten crates") || lower.contains("## 10 crates") || lower.contains("# ten");
    assert!(
        says_ten,
        "docs/ARCHITECTURE.md MUST explicitly state 'Ten crates' / \
         '## 10 crates' / similar with benten-id + benten-sync as \
         workspace members."
    );

    // The two new crates must be listed by name.
    assert!(
        lower.contains("benten-id"),
        "docs/ARCHITECTURE.md MUST mention `benten-id` by name."
    );
    assert!(
        lower.contains("benten-sync"),
        "docs/ARCHITECTURE.md MUST mention `benten-sync` by name."
    );

    // benten-sync must be marked native-only per CLAUDE.md baked-in #17.
    assert!(
        lower.contains("native-only") || lower.contains("native only"),
        "docs/ARCHITECTURE.md MUST declare benten-sync as native-only \
         per CLAUDE.md baked-in #17 (excluded from wasm32 targets)."
    );

    // The pre-existing dsl-compiler row must remain.
    assert!(
        lower.contains("benten-dsl-compiler"),
        "docs/ARCHITECTURE.md MUST mention `benten-dsl-compiler` by name."
    );
}

/// Workspace-shape sanity check — verifies the actual `crates/` layout
/// matches the doc. Asserts the three Phase-2b+Phase-3 crate
/// directories are present so the 10-crate doc claim is not
/// aspirational.
#[test]
fn workspace_has_phase_3_canary_crate_dirs() {
    let root = workspace_root();
    let dsl_compiler_dir = root.join("crates/benten-dsl-compiler");
    let id_dir = root.join("crates/benten-id");
    let sync_dir = root.join("crates/benten-sync");

    assert!(
        dsl_compiler_dir.is_dir(),
        "crates/benten-dsl-compiler/ MUST exist. Without the directory, \
         the 10-crates phrasing in ARCHITECTURE.md would be aspirational \
         (the regression Phase-1 R7 audit caught repeatedly)."
    );
    assert!(
        id_dir.is_dir(),
        "crates/benten-id/ MUST exist. Without the directory, the \
         10-crates phrasing in ARCHITECTURE.md would be aspirational."
    );
    assert!(
        sync_dir.is_dir(),
        "crates/benten-sync/ MUST exist. Without the directory, the \
         10-crates phrasing in ARCHITECTURE.md would be aspirational."
    );
}
