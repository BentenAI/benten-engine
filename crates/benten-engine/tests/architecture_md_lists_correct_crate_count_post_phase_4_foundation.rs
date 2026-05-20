//! R4-FP-3 RED-PHASE pin: `docs/ARCHITECTURE.md` crate-count drift
//! detector post-Phase-4-Foundation (13 crates).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 1
//!   (G26-A docs retense + tag-prep).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter
//!   (G26-A 5-pin docs-shape set; previously orphaned by R3 family
//!   charter — wave-10 docs not assigned to any R3 family).
//! - exit-criterion 10 (docs retense complete at phase-close).
//!
//! ## What this pin asserts
//!
//! Per `r1-triage.md` ratification #1: Phase 4-Foundation adds two
//! crates (`benten-platform-foundation` 11th + `benten-renderer-tauri`
//! 12th). `docs/ARCHITECTURE.md` must reflect the 13-crate count in
//! the section header AND list the new crates by name.
//!
//! State at HEAD: ARCHITECTURE.md already retensed to "Twelve crates
//! (post-Phase-4-Foundation)" with both new crates named — verified
//! 2026-05-11 LATE EVENING. This pin is a permanent regression-guard
//! against future drift (a Phase 4-Foundation retense edit could
//! accidentally revert the count). The §3.6b would-FAIL-if-no-op'd
//! arm: the pin would FAIL if a retense edit dropped the "Twelve
//! crates" phrasing OR removed either new-crate name.
//!
//! Mirrors `architecture_md_10_crate_count_post_phase_3_canaries.rs`
//! shape (the Phase-3-era ten-crates equivalent) — companion regression-
//! guard pattern.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-A wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.12 row 1 + exit-criterion 10. ARCHITECTURE.md \
    13-crate retense regression-guard. Companion to phase-3-era ten-crates pin per \
    `architecture_md_10_crate_count_post_phase_3_canaries.rs`."]
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

    // The "Ten crates" / "## 10 crates" phrasing MUST be retensed away
    // OR remain only in historical context. After G26-A retense the
    // canonical phrasing is "Twelve crates".
    let says_twelve = lower.contains("twelve crates")
        || lower.contains("## 13 crates")
        || lower.contains("# twelve");
    assert!(
        says_twelve,
        "docs/ARCHITECTURE.md MUST state 'Twelve crates' / '## 13 crates' post-Phase-4-Foundation \
         retense. After benten-platform-foundation + benten-renderer-tauri join the workspace, \
         the section header MUST reflect the 13-crate shape (paired with cite-drift-detector \
         source-of-truth)."
    );

    // The two new crates must be listed by name.
    assert!(
        lower.contains("benten-platform-foundation"),
        "docs/ARCHITECTURE.md MUST mention `benten-platform-foundation` by name \
         (11th crate per D-4F-2 ratification)."
    );
    assert!(
        lower.contains("benten-renderer-tauri"),
        "docs/ARCHITECTURE.md MUST mention `benten-renderer-tauri` by name \
         (12th crate per CLAUDE.md #19 engine-extension)."
    );

    // The pre-existing Phase-3 crates must remain.
    assert!(
        lower.contains("benten-id"),
        "docs/ARCHITECTURE.md MUST continue to mention `benten-id` (Phase-3 9th crate)."
    );
    assert!(
        lower.contains("benten-sync"),
        "docs/ARCHITECTURE.md MUST continue to mention `benten-sync` (Phase-3 10th crate)."
    );

    // SHAPE+SUBSTANCE pair (pim-18 §3.6f): the crates/ directory must
    // actually have these directories (not aspirational prose).
    let pf_dir = root.join("crates/benten-platform-foundation");
    let rt_dir = root.join("crates/benten-renderer-tauri");
    assert!(
        pf_dir.is_dir(),
        "crates/benten-platform-foundation/ MUST exist on disk — without it the \
         13-crate phrasing in ARCHITECTURE.md is aspirational (the regression \
         Phase-1 R7 audit caught repeatedly)."
    );
    assert!(
        rt_dir.is_dir(),
        "crates/benten-renderer-tauri/ MUST exist on disk — without it the \
         13-crate phrasing is aspirational."
    );
}
