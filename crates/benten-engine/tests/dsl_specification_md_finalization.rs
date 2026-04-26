//! Phase 2b R4-FP B-4 — `docs/DSL-SPECIFICATION.md` finalization gate.
//!
//! TDD red-phase. Pin source: plan §3.2 G12-B (DSL compiler + devserver
//! routing) — once G12-B lands the dsl-compiler crate, the DSL spec
//! transitions from "draft" / "in progress" to "FINAL" / "stable".
//!
//! Drift detector: prevents the spec from shipping at Phase-2b close
//! still marked as a working document. Operators reading the spec
//! after Phase-2b close need the load-bearing surface frozen — Phase-3
//! sync depends on the DSL being stable.
//!
//! Format pin: the doc must NOT contain `Status: DRAFT` /
//! `Status: WIP` / `Status: in progress` / `(draft)` / `[draft]` in
//! the top-of-doc front matter; SHOULD contain `Status: FINAL` (or
//! `Status: stable`) after G12-B + G11-2b-A lands.
//!
//! Owned by R3-E (CI workflow tests); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `dsl_specification_md_marked_final_post_g12b` — plan §3.2 G12-B +
/// G11-2b-A doc sweep.
#[test]
#[ignore = "Phase 2b G12-B + G11-2b-A pending — DSL-SPECIFICATION.md finalization unimplemented"]
fn dsl_specification_md_marked_final_post_g12b() {
    let root = workspace_root();
    let doc_path = root.join("docs/DSL-SPECIFICATION.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/DSL-SPECIFICATION.md not found at {} ({}); this is a \
             load-bearing Phase-1 doc per CLAUDE.md key-reading list.",
            doc_path.display(),
            e
        );
    });

    // Inspect only the first 50 lines (front matter + intro).
    let head: String = doc_src.lines().take(50).collect::<Vec<_>>().join("\n");
    let lower = head.to_ascii_lowercase();

    let draft_markers = [
        "status: draft",
        "status: wip",
        "status: in progress",
        "status: in-progress",
        "(draft)",
        "[draft]",
        "**draft**",
    ];
    let found_draft: Vec<_> = draft_markers
        .iter()
        .filter(|m| lower.contains(*m))
        .collect();
    assert!(
        found_draft.is_empty(),
        "docs/DSL-SPECIFICATION.md still contains draft markers in the \
         first 50 lines: {:?}. After G12-B lands the dsl-compiler crate \
         + G11-2b-A doc sweep, the spec MUST transition to FINAL/stable. \
         Phase-3 sync depends on the DSL being frozen.",
        found_draft
    );

    let final_markers = [
        "status: final",
        "status: stable",
        "status: phase 2b final",
        "status: phase-2b final",
    ];
    let has_final = final_markers.iter().any(|m| lower.contains(*m));
    assert!(
        has_final,
        "docs/DSL-SPECIFICATION.md MUST explicitly mark itself FINAL/stable \
         in the front matter (one of: {:?}) after G12-B + G11-2b-A. \
         Front-matter pin prevents future editors silently regressing the \
         status.",
        final_markers
    );
}
