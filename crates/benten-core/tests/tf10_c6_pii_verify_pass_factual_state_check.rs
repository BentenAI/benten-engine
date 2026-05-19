//! Phase-4-Meta-Core — R3-B6 BRIEF-ADDENDUM — C6 P-II verify-pass
//! FACTUAL state-check pin (r2 §4-C C6 CALLOUT).
//!
//! ============================================================================
//! ⚠️ THIS IS A VERIFY-PASS FACTUAL ASSERTION — **NOT** A RE-RUN SWEEP.
//! ============================================================================
//!
//! r2 §4-C (C6 row) — CALLOUT (LOAD-BEARING):
//!
//!   "C6 is a *verify-pass* (PR #1295 already landed the superset;
//!   re-running = P-III hazard). The 'test' is a **factual verify
//!   assertion** (grep `legacy_minimal`=EMPTY on the wave + zero
//!   canonical-byte/CID delta), NOT a re-run sweep. R3 must NOT write
//!   a test that re-executes the rename. Owned by G-CORE-6 verify-pass
//!   + G-CORE-0 verify-pass — assigned as a verify-assertion in
//!   R3-B6's brief addendum (not a new test family — a factual
//!   state-check pin)."
//!
//! Accordingly this file asserts ONLY the FACTUAL POST-STATE that PR
//! #1295 (`a15d6af4`, an ancestor of baseline `ed03729a`) is supposed
//! to have produced. It performs NO rename, NO mechanical pass over
//! the canonical-encoding surface, and NO #506 builder work (the SOLE
//! residual P-II mechanical item #506 is owned by **G-CORE-6
//! verify-pass**, NOT R3 — see plan §3 G-CORE-6). A second mechanical
//! pass over the canonical-encoding surface is a **P-III hazard**
//! (§3.5m: wire/CID-on-disk changes are Ben-scheduled, never an
//! orchestrator/agent side-effect) — this pin exists precisely to
//! make the verify-pass mechanical without anyone re-running it.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD ed03729a — orchestrator §3.5n re-confirmed
//! 2026-05-19; re-verified by the R3 author):
//! ============================================================================
//!
//!   `git grep legacy_minimal origin/main`  ⇒  EMPTY (the
//!     #990 `legacy_minimal → minimal` rename in the P-II superset
//!     landed; no occurrence survives).
//!   `to_canonical_bytes` is the standardized P-II name
//!     (RATIFIED-decisions-2026-05-17:24 "Standardize on
//!     to_canonical_bytes" — the settled which-name fork; no agent
//!     re-opens it).
//!
//! ============================================================================
//! WHY A TEST AND NOT JUST A GREP: this is the §3.6b sub-rule-4 +
//! §3.6e shape — the factual post-state is pinned as an executable
//! assertion so the G-CORE-6 / G-CORE-0 verify-pass is a green-test
//! check rather than a manual grep an agent might skip, AND so a
//! regression (a `legacy_minimal` reintroduction) is caught by CI.
//! It is RED-PHASE-ignored until the verify-pass wave so it rides the
//! same un-ignore discipline (§3.6e: reviewer verifies landing-status).
//! It does NOT — and must not — re-perform the rename.
//! ============================================================================

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    // crates/benten-core/ -> repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

// ---------------------------------------------------------------------------
// C6 FACTUAL ARM 1 — `legacy_minimal` is EMPTY across tracked sources.
// This is the literal r2 §4-C verify assertion ("grep
// `legacy_minimal`=EMPTY on the wave"). It is a STATE CHECK over the
// already-landed tree — NOT a re-run of the #990 rename.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-6/G-CORE-0 verify-pass (factual state-check, NOT a re-run)"]
fn c6_pii_verify_legacy_minimal_is_empty_tracked() {
    let root = repo_root();
    // `git grep` over tracked files only — excludes addl/docs narrative
    // (which legitimately *mentions* the historical name). The
    // factual post-state PR #1295 produced: zero `legacy_minimal`
    // identifier occurrences in tracked SOURCE.
    let out = Command::new("git")
        .arg("grep")
        .arg("-l")
        .arg("legacy_minimal")
        .arg("--")
        .arg("crates/")
        .arg("bindings/")
        .arg("packages/")
        .current_dir(&root)
        .output()
        .expect("run git grep");

    let hits = String::from_utf8_lossy(&out.stdout);
    let hits = hits.trim();
    assert!(
        hits.is_empty(),
        "C6 verify-pass: `legacy_minimal` MUST be EMPTY in tracked \
         source at HEAD (PR #1295 landed the #990 rename superset; a \
         non-empty result means a regression reintroduced it OR the \
         verify-pass premise is stale). Offending files:\n{hits}\n\
         NOTE: do NOT 'fix' this by re-running the rename sweep — that \
         is the P-III hazard r2 §4-C warns against; investigate the \
         regression source instead."
    );
    // `git grep` exits 1 (no matches) on the EMPTY (expected) case —
    // that is success here, so we assert on stdout content, not the
    // exit code.
}

// ---------------------------------------------------------------------------
// C6 FACTUAL ARM 2 — `to_canonical_bytes` is the live standardized
// name (the settled P-II which-name fork). A factual presence check
// on the already-landed superset — NOT a rename.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-6/G-CORE-0 verify-pass (factual state-check, NOT a re-run)"]
fn c6_pii_verify_to_canonical_bytes_is_the_standardized_name() {
    let root = repo_root();
    let out = Command::new("git")
        .arg("grep")
        .arg("-l")
        .arg("fn to_canonical_bytes")
        .arg("--")
        .arg("crates/benten-core/src/")
        .current_dir(&root)
        .output()
        .expect("run git grep");
    let hits = String::from_utf8_lossy(&out.stdout);
    assert!(
        hits.contains("subgraph.rs"),
        "C6 verify-pass: `fn to_canonical_bytes` MUST be present on \
         the benten-core canonical surface (RATIFIED-decisions-\
         2026-05-17:24 standardized name; PR #1295 superset). \
         Absence means the verify-pass premise is stale — surface to \
         the orchestrator, do NOT re-run the standardization sweep \
         (P-III hazard). Found in:\n{hits}"
    );
}

// ---------------------------------------------------------------------------
// C6 SCOPE-FENCE NOTE (assertion-as-documentation): the SOLE residual
// P-II mechanical item is #506 (builder `.build()`
// single-fallible-point) and it is owned by **G-CORE-6 verify-pass**,
// NOT this R3 addendum and NOT any R3 test. This trivially-true
// assertion documents the fence so a future reader does not mistake
// the C6 verify-pass addendum for ownership of the #506 work.
// ---------------------------------------------------------------------------
#[test]
fn c6_scope_fence_506_is_g_core_6_not_r3() {
    // Intentionally always-true: the load-bearing content is the
    // module + this comment recording that R3-B6's C6 addendum is a
    // FACTUAL verify-pin only; #506 builder single-fallible-point is
    // G-CORE-6's deliverable (plan §3 G-CORE-6: "SOLE residual P-II
    // mechanical item = #506 builder .build() single-fallible-point —
    // that is the only sweep work in this group").
    let r3_owns_506 = false;
    assert!(
        !r3_owns_506,
        "scope-fence: #506 builder single-fallible-point is G-CORE-6, \
         not an R3 test family"
    );
}
