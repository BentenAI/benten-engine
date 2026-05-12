//! Phase 4-Foundation R3 (Family A — G22-A §13.6 fix-shape (d)). RED-PHASE
//! grep-assert: at R5 G22-A merge time, `docs/history/PHASE-2a.md` MUST
//! be retensed with the actual exit-criterion verification dates for
//! exit-criteria 1 + 2 (WAIT-resume determinism + four new invariants
//! firing). Today those criteria are doc-claimed but never test-verified
//! post-`phase-2a-close` because the `phase_2a_pending_apis` feature
//! gate left their canary tests off the default-feature run.
//!
//! # Charter
//!
//! Per `docs/future/phase-3-backlog.md` §13.6 fix-shape (d) +
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.1 G22-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 wave-1 G22-A.
//! When G22-A un-gates the 8 `phase_2a_pending_apis`-gated tests
//! (`inv_8_11_13_14_firing` + `wait_resume_determinism` chief among
//! them — these are headline canaries for exit-criteria 1 + 2), the
//! retrospective MUST be updated with the date on which the criteria
//! were ACTUALLY verified by a passing test under default features,
//! not just doc-claimed.
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! `docs/history/PHASE-2a.md` MUST contain a verification-date marker
//! that names the actual date on which exit-criteria 1 + 2 were
//! verified by passing canary tests. The marker shape is a substring
//! search for `exit-criterion 1 verified` AND `exit-criterion 2
//! verified` — the actual line wording is up to the retrospective
//! author at R5 close-out but the substrings are the load-bearing
//! grep anchor.
//!
//! Removing the marker (or never adding it) leaves the doc claim
//! standing without a paired test-evidence date, which is the failure
//! mode §13.6 fix-shape (d) names — this test catches it.
//!
//! # RED-PHASE
//!
//! At write-time (R3 Family A; base SHA `f3930e1`) the doc does NOT
//! yet contain either marker. Therefore this test's assertion fails
//! against current HEAD — it is intentionally `#[ignore]`-marked with
//! a RED-PHASE tag so CI stays green; R5 G22-A un-ignores when the
//! retense lands.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer. Closes at R5 G22-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

/// Workspace root resolved from `CARGO_MANIFEST_DIR` of the
/// `benten-engine` crate (`crates/benten-engine`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-A (phase_2a_pending_apis lapsing). §13.6 fix-shape (d): retense PHASE-2a.md with actual exit-criterion 1 + 2 verification dates."]
fn phase_2a_retro_carries_exit_criterion_1_verification_marker() {
    let doc = workspace_root().join("docs/history/PHASE-2a.md");
    let body = std::fs::read_to_string(&doc)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", doc.display()));
    let needle = "exit-criterion 1 verified";
    assert!(
        body.contains(needle),
        "expected `{needle}` marker in {} (G22-A §13.6 fix-shape (d) \
         retense not yet landed — exit-criterion 1 'WAIT-resume \
         determinism' is doc-claimed but never test-verified \
         post-phase-2a-close; R5 G22-A adds the verification-date \
         marker once the un-gated canary `wait_resume_determinism` test \
         passes under default features). Un-ignore + un-block this test \
         at R5 G22-A close.",
        doc.display(),
    );
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-A (phase_2a_pending_apis lapsing). §13.6 fix-shape (d): retense PHASE-2a.md with actual exit-criterion 1 + 2 verification dates."]
fn phase_2a_retro_carries_exit_criterion_2_verification_marker() {
    let doc = workspace_root().join("docs/history/PHASE-2a.md");
    let body = std::fs::read_to_string(&doc)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", doc.display()));
    let needle = "exit-criterion 2 verified";
    assert!(
        body.contains(needle),
        "expected `{needle}` marker in {} (G22-A §13.6 fix-shape (d) \
         retense not yet landed — exit-criterion 2 'four new invariants \
         firing' is doc-claimed but never test-verified \
         post-phase-2a-close; R5 G22-A adds the verification-date \
         marker once the un-gated canary `inv_8_11_13_14_firing` test \
         passes under default features). Un-ignore + un-block this test \
         at R5 G22-A close.",
        doc.display(),
    );
}
