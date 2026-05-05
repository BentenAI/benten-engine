//! Phase-3 R4-FP orchestrator-direct (renamed 2026-05-05) —
//! `docs/ARCHITECTURE.md` 10-crate count drift detector.
//!
//! TDD red-phase. Pin source: architect-r1 carry item (the
//! `benten-dsl-compiler` crate landed at Phase-2b G12-B taking the
//! workspace from 7 → 8 crates; Phase-3 R3-A + R3-C added
//! `benten-id` + `benten-sync` as canary stubs taking it 8 → 10).
//! ARCHITECTURE.md must enumerate all 10 crates by name with
//! `benten-sync` flagged native-only per CLAUDE.md baked-in #17.
//!
//! **Renamed from `architecture_md_8_crate_count_after_dsl_compiler.rs`**
//! at the cite-drift detector source-of-truth bump (8 → 10) per
//! `tools/cite-drift-detector/src/lib.rs::numeric_claims_source_of_truth`.
//! Test still `#[ignore]`'d per `docs/future/phase-3-backlog.md §7.3.A.5`;
//! body lift to executable is a Phase-3 deliverable.
//!
//! Drift discipline: doc-as-source-of-truth on the workspace shape
//! must agree with the actual `crates/` directory layout. Without
//! this test, ARCHITECTURE.md would silently drift from the workspace
//! manifest the way Phase-2a R7 audits caught aspirational-prose-but-
//! dead-code regressions repeatedly (CLAUDE.md: "Verify, don't trust
//! docs").
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4
//! (8-crate form); renamed + retensed to 10-crate at orchestrator-direct
//! cross-cutting cleanup 2026-05-05.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `architecture_md_says_ten_crates_after_phase_3_canaries` —
/// architect-r1 carry, retensed for the Phase-3 R3 canary stubs.
///
/// Asserts ARCHITECTURE.md says "Ten crates" (or "10 crates") in the
/// section header AND in the prose body, after R3-A + R3-C land
/// `benten-id` + `benten-sync` as workspace members. The Phase-2b
/// phrasing was "Eight crates"; if the doc still says "Eight" after the
/// canary stubs land, operators reading the doc will miss the
/// `benten-id` + `benten-sync` boundaries.
#[test]
#[ignore = "Phase 3 — ARCHITECTURE.md 10-crate doc-drift body deferred per docs/future/phase-3-backlog.md §7.3.A.5 (G12-B + G11-2b-A both landed; R3-A + R3-C added 9th/10th canary crates; doc-drift detector body lands Phase 3 alongside G20-B FINAL transition pin)"]
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

    // After R3-A + R3-C land, the "Eight crates" / "## 8 crates"
    // phrasing MUST be gone.
    assert!(
        !lower.contains("eight crates")
            && !lower.contains("## 8 crates")
            && !lower.contains("# eight"),
        "docs/ARCHITECTURE.md still contains pre-Phase-3 phrasing 'Eight \
         crates' / '## 8 crates' / '# Eight'. After benten-id + benten-sync \
         land per architect-r1 carry + arch-r1-3 BLOCKER ladder, the doc \
         MUST update to 'Ten crates' (G20-B FINAL doc sweep — paired with \
         the orchestrator-direct cite-drift detector source-of-truth bump)."
    );

    // Should explicitly assert the new count.
    let says_ten =
        lower.contains("ten crates") || lower.contains("## 10 crates") || lower.contains("# ten");
    assert!(
        says_ten,
        "docs/ARCHITECTURE.md MUST explicitly state 'Ten crates' / \
         '## 10 crates' / similar after R3-A + R3-C land the benten-id + \
         benten-sync canary crates (architect-r1 carry; G20-B doc sweep)."
    );

    // The two new crates must be listed by name.
    assert!(
        lower.contains("benten-id"),
        "docs/ARCHITECTURE.md MUST mention `benten-id` by name after \
         R3-A lands the crate (Phase-3 G14-A1 canary)."
    );
    assert!(
        lower.contains("benten-sync"),
        "docs/ARCHITECTURE.md MUST mention `benten-sync` by name after \
         R3-C lands the crate (Phase-3 G16-A canary)."
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
        "docs/ARCHITECTURE.md MUST mention `benten-dsl-compiler` by \
         name after G12-B landed the crate (Phase-2b)."
    );
}

/// Workspace-shape sanity check — verifies the actual `crates/` layout
/// matches the doc. R5 G14-A1 + G16-A landings made the canary dirs
/// exist; this test asserts both directories are present so the
/// 10-crate doc claim is not aspirational.
#[test]
#[ignore = "Phase 3 — workspace-has-canary-crate-dirs body deferred per docs/future/phase-3-backlog.md §7.3.A.5 (G14-A1 + G16-A canary stub crates landed at R3-A + R3-C respectively; doc-drift detector body lands Phase 3)"]
fn workspace_has_phase_3_canary_crate_dirs() {
    let root = workspace_root();
    let dsl_compiler_dir = root.join("crates/benten-dsl-compiler");
    let id_dir = root.join("crates/benten-id");
    let sync_dir = root.join("crates/benten-sync");

    assert!(
        dsl_compiler_dir.is_dir(),
        "crates/benten-dsl-compiler/ MUST exist after G12-B lands \
         (architect-r1 carry; plan §3.2 G12-B). Without the directory, \
         the 10-crates phrasing in ARCHITECTURE.md would be aspirational \
         (the regression Phase-1 R7 audit caught repeatedly)."
    );
    assert!(
        id_dir.is_dir(),
        "crates/benten-id/ MUST exist after R3-A lands the Phase-3 \
         G14-A1 canary stub. Without the directory, the 10-crates \
         phrasing in ARCHITECTURE.md would be aspirational."
    );
    assert!(
        sync_dir.is_dir(),
        "crates/benten-sync/ MUST exist after R3-C lands the Phase-3 \
         G16-A canary stub. Without the directory, the 10-crates \
         phrasing in ARCHITECTURE.md would be aspirational."
    );
}
