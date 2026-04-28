//! Phase 2b G11-2b — `docs/INVARIANT-COVERAGE.md` MUST mark Inv-4 +
//! Inv-7 as ACTIVE at Phase 2b close (their Phase-1 stub status is
//! removed in this wave).
//!
//! Inv-4 = SANDBOX nest-depth ceiling.
//! Inv-7 = SANDBOX `output_max_bytes` range.
//!
//! Both went live in G7-B alongside the SANDBOX runtime.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn invariant_coverage_doc_lists_inv_4_and_inv_7_active() {
    let root = workspace_root();
    let doc_path = root.join("docs/INVARIANT-COVERAGE.md");
    let body = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/INVARIANT-COVERAGE.md MUST exist at Phase-2b close ({}); \
             error: {}. G11-2b-A owns this file per plan §3.",
            doc_path.display(),
            e
        );
    });

    // Inv-4 row MUST mention the active-state marker.
    assert!(
        body.contains("**SANDBOX nest-depth ceiling — ACTIVE")
            || body.contains("Inv-4 — ACTIVE")
            || (body.contains("SANDBOX nest-depth") && body.contains("ACTIVE")),
        "INVARIANT-COVERAGE.md MUST mark Inv-4 (SANDBOX nest-depth \
         ceiling) as ACTIVE at Phase 2b close — the Phase-1 stub \
         status is removed by G11-2b. Doc body did NOT contain the \
         active marker."
    );

    // Inv-7 row MUST mention the active-state marker.
    assert!(
        body.contains("**SANDBOX `output_max_bytes` range — ACTIVE")
            || body.contains("Inv-7 — ACTIVE")
            || (body.contains("output_max_bytes") && body.contains("ACTIVE")),
        "INVARIANT-COVERAGE.md MUST mark Inv-7 (SANDBOX \
         `output_max_bytes` range) as ACTIVE at Phase 2b close — \
         the Phase-1 stub status is removed by G11-2b."
    );

    // The doc MUST NOT carry residual "Phase 2b" stub markers in
    // either row — those are explicitly scheduled to be removed by
    // G11-2b per the brief.
    let phase_2b_stub_markers = [
        "Inv-4 stub (Phase 2b)",
        "Inv-7 stub (Phase 2b)",
        "Phase 2b — stub",
        "stub (Phase 2b)",
    ];
    for marker in phase_2b_stub_markers {
        assert!(
            !body.contains(marker),
            "INVARIANT-COVERAGE.md MUST NOT carry Phase-2b stub \
             marker {marker:?} after G11-2b — Inv-4 + Inv-7 are now \
             active."
        );
    }
}
