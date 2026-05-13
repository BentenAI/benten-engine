//! R4-FP-3 RED-PHASE pin: `docs/future/phase-4-backlog.md` skeleton
//! created at G26-A wave-10.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 5.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - meth-r1-15: phase-4-backlog.md is the destination doc for HARD
//!   RULE rule-12 clause-(b) BELONGS-NAMED-NOW deferrals targeting
//!   Phase-4-Meta + the v1-assessment-window. Without the file, every
//!   Phase-4-Foundation deferral to "Phase-4-Meta" is a phantom
//!   destination.
//!
//! ## What this pin asserts
//!
//! `docs/future/phase-4-backlog.md` MUST exist as a skeleton with:
//!
//! - A top-level heading.
//! - A §1 (or §0) "Phase-4-Meta-targeted carries" section ready to
//!   receive named-carry entries.
//! - A §N "v1-assessment-window items" section.
//! - At least the inherited entries from Phase-3 (Phase-3-deferred
//!   items inherited per CLAUDE.md status table — wasmtime Component-
//!   Model re-evaluation; engine impl-block generic-cascade lift;
//!   light-client mode-(b) range-query proof; light-client mode-(c)
//!   signed checkpoint).
//!
//! Without this file at HEAD, every Phase-4-Foundation finding that
//! reads "carry to Phase-4-Meta" is a HARD RULE rule-12 violation
//! (phantom destination). The file MUST exist + the entries MUST land
//! NOW per BELONGS-NAMED-NOW clause-(b).

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-A wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.12 row 5 + meth-r1-15. phase-4-backlog.md skeleton \
    is the named-destination for Phase-4-Meta carries; without it Phase-4-Foundation findings \
    citing 'Phase-4-Meta' are phantom destinations (HARD RULE rule-12 violation)."]
fn docs_future_phase_4_backlog_md_skeleton_created() {
    let path = workspace_root().join("docs/future/phase-4-backlog.md");

    assert!(
        path.is_file(),
        "docs/future/phase-4-backlog.md MUST exist at {} after G26-A wave-10 — \
         meth-r1-15 named-destination for Phase-4-Meta carries. Without it every \
         'Phase-4-Meta' deferral is a phantom destination (HARD RULE rule-12 \
         clause-(b) violation).",
        path.display()
    );

    let body = fs::read_to_string(&path).unwrap();

    // SHAPE: must be non-empty (smoke).
    assert!(
        body.len() > 200,
        "phase-4-backlog.md MUST be a substantive skeleton (not empty placeholder); \
         got {} bytes",
        body.len()
    );

    // SUBSTANCE: must contain headings/labels for the canonical
    // destination sections (Phase-4-Meta + v1-assessment-window).
    let mentions_phase_4_meta = body.contains("Phase-4-Meta") || body.contains("Phase 4-Meta");
    assert!(
        mentions_phase_4_meta,
        "phase-4-backlog.md MUST mention 'Phase-4-Meta' as the carry destination per \
         CLAUDE.md status table"
    );

    let mentions_v1_window =
        body.contains("v1-assessment-window") || body.contains("v1 assessment window");
    assert!(
        mentions_v1_window,
        "phase-4-backlog.md MUST mention 'v1-assessment-window' as the carry destination \
         for identity-recovery + missing_docs sweep + small architectural cleanups per \
         CLAUDE.md baked-in #15 v1-milestone-gate framing"
    );

    // SUBSTANCE — inherited Phase-3-deferred items must each be listed.
    // Per CLAUDE.md status table: wasmtime Component-Model re-eval +
    // engine impl-block generic-cascade lift + light-client mode-(b)
    // range-query + light-client mode-(c) signed checkpoint.
    let inherited_markers: &[&str] = &[
        "wasmtime",
        "Component-Model",
        "impl-block",
        "generic-cascade",
        "light-client",
    ];
    let missing_inherited: Vec<&&str> = inherited_markers
        .iter()
        .filter(|m| !body.contains(**m))
        .collect();
    assert!(
        missing_inherited.is_empty(),
        "phase-4-backlog.md MUST inherit Phase-3-deferred items per CLAUDE.md status table; \
         missing markers: {:?}",
        missing_inherited,
    );
}
