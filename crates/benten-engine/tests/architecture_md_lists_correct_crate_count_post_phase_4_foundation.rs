//! `docs/ARCHITECTURE.md` crate-count drift detector — retensed across
//! the phase progression to the current fifteen-crate workspace shape.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 1
//!   (G26-A docs retense + tag-prep).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter
//!   (G26-A docs-shape set).
//! - exit-criterion 10 (docs retense complete at phase-close).
//!
//! ## What this pin asserts
//!
//! Phase 4-Foundation added `benten-platform-foundation` (11th) +
//! `benten-renderer-tauri` (12th); Phase 4-Meta-Core added
//! `benten-crypto-suite` (13th), `benten-drop` (14th), and
//! `benten-membership-set` (15th, F-full). `docs/ARCHITECTURE.md` must
//! reflect the current fifteen-crate count in the section header AND
//! list the new crates by name.
//!
//! State at HEAD: ARCHITECTURE.md is retensed to "Fifteen crates
//! (post-Phase-4-Meta-Core F-full)" with all new crates named. This pin
//! is a permanent regression-guard against future drift (a retense edit
//! could accidentally revert the count). The §3.6b would-FAIL-if-no-op'd
//! arm: the pin FAILs if a retense edit dropped the "Fifteen crates"
//! phrasing OR removed a new-crate name.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn architecture_md_lists_correct_crate_count_post_phase_4_foundation() {
    let root = workspace_root();
    let doc_path = root.join("docs/ARCHITECTURE.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/ARCHITECTURE.md not found at {} ({}); this is a load-bearing doc.",
            doc_path.display(),
            e
        );
    });
    let lower = doc_src.to_ascii_lowercase();

    // Older count phrasings ("Ten crates" / "## 10 crates" / "Twelve
    // crates" / etc.) MUST be retensed away from the canonical heading
    // (historical narrative describing past states is allowed). The
    // current canonical phrasing is "Fifteen crates".
    let says_fifteen = lower.contains("fifteen crates")
        || lower.contains("## 15 crates")
        || lower.contains("# fifteen")
        || lower.contains("fifteen rust crates");
    assert!(
        says_fifteen,
        "docs/ARCHITECTURE.md MUST state 'Fifteen crates' / '## 15 crates' post-Phase-4-Meta-Core \
         F-full retense. After benten-membership-set joins the workspace as the 15th crate, \
         the section header MUST reflect the fifteen-crate shape (paired with cite-drift-detector \
         source-of-truth)."
    );

    // The new crates must be listed by name.
    for (name, note) in [
        ("benten-platform-foundation", "11th crate per D-4F-2 ratification"),
        ("benten-renderer-tauri", "12th crate per CLAUDE.md #19 engine-extension"),
        ("benten-crypto-suite", "13th crate, Phase 4-Meta-Core G-CORE-2"),
        ("benten-drop", "14th crate, Phase 4-Meta-Core G-CORE-3f"),
        ("benten-membership-set", "15th crate, Phase 4-Meta-Core F-full"),
    ] {
        assert!(
            lower.contains(name),
            "docs/ARCHITECTURE.md MUST mention `{name}` by name ({note})."
        );
    }

    // The pre-existing Phase-3 crates must remain.
    assert!(
        lower.contains("benten-id"),
        "docs/ARCHITECTURE.md MUST continue to mention `benten-id` (Phase-3 9th crate)."
    );
    assert!(
        lower.contains("benten-sync"),
        "docs/ARCHITECTURE.md MUST continue to mention `benten-sync` (Phase-3 10th crate)."
    );

    // SHAPE+SUBSTANCE pair (pim-18 §3.6f): the crates/ directories must
    // actually exist (not aspirational prose).
    for name in [
        "benten-platform-foundation",
        "benten-renderer-tauri",
        "benten-crypto-suite",
        "benten-drop",
        "benten-membership-set",
    ] {
        let dir = root.join("crates").join(name);
        assert!(
            dir.is_dir(),
            "crates/{name}/ MUST exist on disk — without it the fifteen-crate \
             phrasing in ARCHITECTURE.md is aspirational (the regression \
             Phase-1 R7 audit caught repeatedly)."
        );
    }
}
