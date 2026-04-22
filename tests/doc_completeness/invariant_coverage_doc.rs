//! Phase 2a R4 qa-r4-7 / cov-3 + phil-r1-4: `docs/INVARIANT-COVERAGE.md`
//! exists with a row per active invariant.
//!
//! The doc is a G11-A deliverable. TDD red-phase: this test fails until
//! the doc lands.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .and_then(|p| p.parent()) // repo root
        .expect("repo root")
        .join("docs/INVARIANT-COVERAGE.md")
}

#[test]
fn invariant_coverage_doc_exists_with_every_active_invariant() {
    let contents = fs::read_to_string(doc_path()).expect(
        "docs/INVARIANT-COVERAGE.md must exist (G11-A deliverable; see plan §3 G11-A + phil-r1-4)",
    );

    // The 14 structural invariants (INV-1..14) must each appear as a row
    // identifier. The format is "| Invariant | ..." table header + one row
    // per invariant whose row-label contains `Inv-N` or `INV-N`.
    for n in 1..=14_u8 {
        let long = format!("Inv-{n}");
        let upper = format!("INV-{n}");
        assert!(
            contents.contains(&long) || contents.contains(&upper),
            "docs/INVARIANT-COVERAGE.md is missing a row for invariant {n} \
             (expected `Inv-{n}` or `INV-{n}` somewhere in the file)"
        );
    }

    // A header row anchors the table.
    assert!(
        contents.contains("| Invariant |") || contents.contains("|Invariant|"),
        "docs/INVARIANT-COVERAGE.md must carry a markdown table with a \
         `Invariant` header column"
    );
}
