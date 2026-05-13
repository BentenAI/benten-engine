//! R4-FP-3 RED-PHASE pin: Phase 4-Foundation docs retense narrative
//! complete at G26-A wave-10.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 2.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - exit-criterion 10 (docs retense complete).
//!
//! ## What this pin asserts
//!
//! At Phase-4-Foundation close, the load-bearing top-of-funnel docs
//! (PRIMER.md, VISION.md, FULL-ROADMAP.md, ARCHITECTURE.md, README.md)
//! all carry the Phase-4-Foundation retense narrative — they mention
//! the phase as SHIPPED (or in-flight per current state) and the new
//! v1-platform surface (admin UI v0, plugin manifest, materializer,
//! schema-driven rendering, IVM generalization).
//!
//! Grep-shape regression-guard: walks the canonical docs + asserts the
//! retense-narrative markers are present in each. Defends against the
//! failure shape where ONE doc gets retensed but siblings drift.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-A wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.12 row 2 + exit-criterion 10. Phase-4-Foundation \
    retense narrative present across 5 load-bearing top-of-funnel docs."]
fn phase_4_foundation_docs_retense_complete() {
    let root = workspace_root();

    // The 5 load-bearing top-of-funnel docs. Each MUST mention
    // Phase-4-Foundation as a labelled phase + at least one of the
    // headline surfaces (admin UI / plugin manifest / materializer /
    // schema rendering) so the retense is substantive not name-only.
    let docs: &[(&str, &[&str])] = &[
        (
            "docs/PRIMER.md",
            &["Phase 4-Foundation", "phase-4-foundation"],
        ),
        (
            "docs/VISION.md",
            &["Phase 4-Foundation", "phase-4-foundation"],
        ),
        (
            "docs/FULL-ROADMAP.md",
            &["Phase 4-Foundation", "phase-4-foundation"],
        ),
        (
            "docs/ARCHITECTURE.md",
            &["Phase 4-Foundation", "phase-4-foundation"],
        ),
        ("README.md", &["Phase 4-Foundation", "phase-4-foundation"]),
    ];

    // Headline surfaces — at least ONE must appear in EACH retensed doc.
    let headline_markers: &[&str] = &[
        "admin UI v0",
        "admin-ui-v0",
        "plugin manifest",
        "plugin-manifest",
        "materializer",
        "schema-driven render",
        "Renderer trait",
    ];

    let mut missing_phase_mention: Vec<&str> = Vec::new();
    let mut missing_headline: Vec<&str> = Vec::new();

    for (doc, phase_aliases) in docs {
        let path = root.join(doc);
        let body = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "{} MUST exist as load-bearing top-of-funnel doc ({})",
                path.display(),
                e
            );
        });

        let mentions_phase = phase_aliases.iter().any(|alias| body.contains(alias));
        if !mentions_phase {
            missing_phase_mention.push(doc);
        }

        let mentions_headline = headline_markers.iter().any(|m| body.contains(m));
        if !mentions_headline {
            missing_headline.push(doc);
        }
    }

    assert!(
        missing_phase_mention.is_empty(),
        "Phase-4-Foundation retense MUST mention the phase by name in each \
         load-bearing top-of-funnel doc; missing in: {:?}",
        missing_phase_mention,
    );

    assert!(
        missing_headline.is_empty(),
        "Phase-4-Foundation retense MUST mention at least one headline surface \
         (admin UI v0 / plugin manifest / materializer / schema rendering / Renderer trait) \
         in each retensed doc so the retense is substantive — missing headline in: {:?}",
        missing_headline,
    );
}
