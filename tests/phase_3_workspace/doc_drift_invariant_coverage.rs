//! G15-B (wave-5a) — `INVARIANT-COVERAGE.md` retense post-G15-A
//! generalization.
//!
//! Pre-G15-A the Algorithm B production-registration note named a
//! `ContentListingView` fallback for non-canonical user-defined view IDs.
//! Post-G15-A's generalized kernel (Strategy::B keyed on arbitrary
//! `(label_pattern, projection)` triples), that note is stale: user-defined
//! views run under Strategy::B with their actual label patterns.
//!
//! G15-B retenses the doc + this test pins the post-G15-A surface so a
//! future drift that re-introduces the stale phrase fails CI rather than
//! audit (pim-1 §3.5b post-fix doc-coupling).

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is `<repo>/tests/phase_3_workspace`; the
    // workspace root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
fn invariant_coverage_md_retensed_to_g15_a_close() {
    let path = workspace_root().join("docs/INVARIANT-COVERAGE.md");
    let body = std::fs::read_to_string(&path).expect("read INVARIANT-COVERAGE.md");

    // The stale phrase MUST be GONE.
    assert!(
        !body.contains("ContentListingView fallback"),
        "INVARIANT-COVERAGE.md still names the canonical-only \
         `ContentListingView fallback`; G15-B must retense to the \
         post-G15-A generalized-kernel narrative"
    );

    // The post-G15-A retense MUST be present (one of two acceptable
    // spellings — keeps the test forgiving of editorial choice while
    // pinning the architectural claim).
    let retensed = body.contains("generalized at Phase 3 G15-A")
        || body.contains("Strategy::B with their actual label patterns");
    assert!(
        retensed,
        "INVARIANT-COVERAGE.md must carry the post-G15-A retense — \
         either the phrase 'generalized at Phase 3 G15-A' or \
         'Strategy::B with their actual label patterns'"
    );
}
