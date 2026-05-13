//! R4-FP-3 RED-PHASE pin: `docs/SECURITY-POSTURE.md` Compromise #11
//! reaffirmation against the new materializer pipeline.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 4.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - sec-3.5-r1-13: Compromise #11 (IVM views coarse-grained read-gate;
//!   CLOSED at Phase-3 G15-A) requires reaffirmation against the
//!   Phase-4-Foundation materializer pipeline (G23-B). The materializer
//!   surfaces materialized views to client UIs; the row-level gate
//!   shipped at G15-A composes with the materializer's per-row
//!   capability check (mat-r1-3 + mat-r1-7).
//!
//! ## What this pin asserts
//!
//! Compromise #11 reaffirmation MUST appear in SECURITY-POSTURE.md
//! Phase-4-Foundation section, explicitly stating that the row-level
//! read-gate continues to compose with the new materializer surface
//! (per-row gate independent of delivery, dual-gate composition). This
//! defends against the failure shape where a future materializer
//! refactor implicitly weakens the gate without surfacing the trade-off
//! in the posture doc.

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
    Pin source: r2-test-landscape.md §2.12 row 4 + sec-3.5-r1-13. Compromise #11 reaffirmation \
    against new materializer pipeline (G23-B) landed at Phase-4-Foundation close."]
fn security_posture_compromise_11_reaffirmation_landed() {
    let posture = workspace_root().join("docs/SECURITY-POSTURE.md");
    let body = fs::read_to_string(&posture).expect("read SECURITY-POSTURE.md");

    // Compromise #11 must continue to be referenced (it was CLOSED at
    // Phase-3 G15-A; that closure remains in the doc).
    assert!(
        body.contains("Compromise #11"),
        "SECURITY-POSTURE.md MUST continue to mention Compromise #11 (IVM views \
         coarse-grained read-gate) — closure narrative preserved across phases"
    );

    // The reaffirmation narrative MUST name the materializer or G23-B.
    let mentions_materializer =
        body.contains("materializer") || body.contains("Materializer") || body.contains("G23-B");
    assert!(
        mentions_materializer,
        "SECURITY-POSTURE.md MUST reaffirm Compromise #11 against the new materializer \
         pipeline (G23-B) at Phase-4-Foundation close per sec-3.5-r1-13. The reaffirmation \
         narrative must mention 'materializer' / 'G23-B' so readers can grep for the \
         composition narrative."
    );

    // Compose-with-row-level-gate narrative.
    let mentions_row_gate = body.contains("row-level") || body.contains("per-row");
    assert!(
        mentions_row_gate,
        "SECURITY-POSTURE.md MUST mention 'row-level' / 'per-row' gate semantics in the \
         Compromise #11 reaffirmation context — the materializer surface composes with the \
         per-row capability gate shipped at G15-A"
    );

    // Reaffirmation marker — "reaffirmed" or "composes with" language
    // that connects the historical closure to the new surface.
    let mentions_reaffirmation = body.contains("reaffirm")
        || body.contains("composes with")
        || body.contains("composition")
        || body.contains("re-affirm");
    assert!(
        mentions_reaffirmation,
        "SECURITY-POSTURE.md MUST explicitly reaffirm or describe composition of Compromise #11's \
         G15-A row-level gate with the new G23-B materializer surface — the connection narrative \
         is the load-bearing artifact, not the historical compromise text alone"
    );
}
