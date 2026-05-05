//! R3-C RED-PHASE pin for INVARIANT-COVERAGE.md retense (G15-B
//! wave-5a; per r2-test-landscape §2.3 + plan §3 G15-B row).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.3 G15-B row
//!   `invariant_coverage_md_retensed_to_g15_a_close`.
//! - plan §3 G15-B row line "drop Algorithm B canonical-only
//!   compromise note; retense to 'generalized at Phase 3 G15-A'".
//!
//! ## What this pins
//!
//! `docs/INVARIANT-COVERAGE.md` carries a Phase-2b note that
//! Algorithm B's canonical-only fallback covers user-defined views
//! by coercing them to ContentListingView semantics. Post-G15-A
//! generalization, that note is stale: user-defined views run
//! under Strategy::B with their actual label patterns. G15-B
//! retenses the doc.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G15-B wave-5a retenses INVARIANT-COVERAGE.md"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-B wave-5a — retense INVARIANT-COVERAGE.md per plan §3 G15-B"]
fn invariant_coverage_md_retensed_to_g15_a_close() {
    // plan §3 G15-B pin. G15-B implementer:
    //
    //   1. Edits docs/INVARIANT-COVERAGE.md.
    //   2. Drops the "user-defined view IDs hit a `ContentListingView`
    //      fallback" note.
    //   3. Adds a "Algorithm B generalized at Phase 3 G15-A; user-defined
    //      view IDs run under Strategy::B with their actual label patterns"
    //      replacement.
    //
    // This test asserts the post-G15-B doc state:
    //
    //   let inv = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("docs").join("INVARIANT-COVERAGE.md")
    //   ).unwrap();
    //   // The stale phrase must be GONE:
    //   assert!(
    //       !inv.contains("ContentListingView fallback"),
    //       "INVARIANT-COVERAGE.md still names the canonical-only fallback; G15-B must retense"
    //   );
    //   // The new phrase must be PRESENT:
    //   assert!(
    //       inv.contains("generalized at Phase 3 G15-A") || inv.contains("Strategy::B with their actual label patterns"),
    //       "INVARIANT-COVERAGE.md must carry the post-G15-A retense"
    //   );
    //
    // OBSERVABLE consequence: a future refactor that re-introduces
    // the stale note fails this test. Defends against pim-1
    // (post-fix doc-coupling) drift on the IVM correctness narrative.
    unimplemented!("G15-B wires INVARIANT-COVERAGE.md retense assertion");
}
