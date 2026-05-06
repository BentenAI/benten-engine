//! G13-E pin (LIVE since Phase-3 R5 wave-3): `docs/SECURITY-POSTURE.md`
//! Compromise #12 marked CLOSED at G13-E.
//!
//! Pin source: plan §3 G13-E row + r2-test-landscape §2.1 G13-E row
//! `security_posture_compromise_12_marked_closed`; closes
//! `docs/future/phase-2-backlog.md` §9.1.
//!
//! ## What this asserts
//!
//! Compromise #12 (`DurabilityMode::Group` gate-5 deferred) was opened
//! during Phase-2a arch-r1-1 triage; G13-E flips
//! `DurabilityMode::default()` to `Group` (closing the engine-surface
//! posture gap) and promotes benchmark CI to required (closing the
//! observability gap). Compromise #12's prose in
//! `docs/SECURITY-POSTURE.md` is updated to record the closure; this
//! test pins that the prose actually carries the CLOSED marker so a
//! future regression that re-opens Compromise #12 surfaces as a
//! test failure rather than a silent doc-vs-code drift.
//!
//! ## OBSERVABLE consequence (per pim-2 §3.6b)
//!
//! The test reads `docs/SECURITY-POSTURE.md` from disk and asserts the
//! Compromise #12 section contains the CLOSED-AT-G13-E phrase. If a
//! future change reverts the default-flip, the maintainer is expected
//! to also re-open the Compromise; if they only revert the code (or
//! only the doc) this test fires.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

/// Resolve `docs/SECURITY-POSTURE.md` relative to the workspace root.
/// `CARGO_MANIFEST_DIR` for this test crate is `crates/benten-graph`.
fn security_posture_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("docs")
        .join("SECURITY-POSTURE.md")
}

#[test]
fn security_posture_compromise_12_marked_closed() {
    let path = security_posture_path();
    let body = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    // Compromise #12's section header MUST exist (lineage marker).
    assert!(
        body.contains("Compromise #12"),
        "expected `Compromise #12` section header in {}",
        path.display()
    );

    // Find the Compromise #12 section + the next `### Compromise #` header
    // (or end of file) to scope the assertion to this section only.
    let start = body
        .find("### Compromise #12")
        .expect("Compromise #12 section header present");
    let after_start = &body[start + "### Compromise #12".len()..];
    let end_offset = after_start
        .find("### Compromise #")
        .map_or(body.len(), |o| start + "### Compromise #12".len() + o);
    let section = &body[start..end_offset];

    // OBSERVABLE: the section must declare the closure at G13-E. Tolerate
    // either `CLOSED-AT-G13-E` or `CLOSED at G13-E` phrasing — the
    // load-bearing claim is that the section communicates the closure.
    let has_closed_marker = section.contains("CLOSED-AT-G13-E")
        || section.contains("CLOSED at G13-E")
        || section.contains("CLOSED IN PHASE-3 G13-E")
        || section.contains("CLOSED-IN-PHASE-3-G13-E");
    assert!(
        has_closed_marker,
        "Compromise #12 section must record closure at G13-E (look for \
         `CLOSED-AT-G13-E` / `CLOSED at G13-E`); section currently:\n{section}"
    );
}
