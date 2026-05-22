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

// ===========================================================================
// R3-W5 EXTENSION (Phase-4-Meta-Core R3 — cross-wave-integration partition):
// the #506 builder verify-pass CLOSURE arm.
//
// R2-test-landscape §7 W5 row instructs: "extend
// `crates/benten-core/tests/tf10_c6_pii_verify_pass_factual_state_check.rs`
// (untracked) for the #506 builder closure".
//
// Reading: the existing file (above) covers the FACTUAL post-state of
// PR #1295's superset (no `legacy_minimal`; `to_canonical_bytes`
// present) and explicitly fences #506 OUT of R3 ownership. The W5
// extension adds the COMPLEMENTARY arm: a RED-PHASE staged-pin that
// FAILS until G-CORE-6 (verify-pass) actually lands the #506 builder
// `.build()` single-fallible-point. The fence remains: #506 is
// G-CORE-6's deliverable; THIS pin is a closure-observable on the
// G-CORE-6 wave producing the deliverable, not an R3 attempt to do
// the work.
//
// §3.6e pim-12 RED-PHASE: un-ignore at G-CORE-6 verify-pass wave
// (the §3 G-CORE-6 group def: "SOLE residual P-II mechanical item =
// #506 builder `.build()` single-fallible-point — that is the only
// sweep work in this group"). The closure observable is: a
// `.build()` method exists on the canonical builder + every
// fallibility surface is collapsed to that ONE call (not spread
// across N intermediate setters). §3.6b sub-rule-4: the SPECIFIC
// arm is the single-fallible-point property; the OBSERVABLE is the
// shape of the builder's public API; WOULD-FAIL pre-G-CORE-6 (the
// builder still has multi-point fallibility per #506).
//
// §3.5n pass: R3-W5 verified that as of HEAD `c9c11c56`, #506 has
// NOT yet shipped (PR #1295 landed the to_canonical_bytes/#990/#733
// superset; #506 is the residual P-II mechanical work remaining,
// per plan §3 G-CORE-6). The pin is genuinely RED at HEAD.
// ===========================================================================

/// R3-W5 EXTENSION — #506 builder single-fallible-point closure
/// arm. RED-PHASE staged-pin. Un-ignore at G-CORE-6 verify-pass
/// wave producing the .build() single-fallible-point deliverable.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-6 verify-pass wave (the #506 builder `.build()` single-fallible-point landing — the SOLE residual P-II mechanical item per plan §3 G-CORE-6). R3-W5 cross-wave-integration partition."]
fn c6_pii_verify_506_builder_build_is_single_fallible_point() {
    // Body intent at un-ignore (G-CORE-6 verify-pass produces the
    // surface, then this body grep-asserts the shape):
    //
    //   let root = repo_root();
    //   // The canonical builder source (the G-CORE-6 implementer
    //   // names the canonical home; likely benten-core).
    //   let builder_src = std::fs::read_to_string(
    //       root.join("crates/benten-core/src/builder.rs")
    //   ).expect("post-G-CORE-6 the canonical builder ships");
    //
    //   // The single-fallible-point property:
    //   //   (a) `pub fn build(self) -> Result<X, BuilderError>` exists.
    //   //   (b) NO other `pub fn` on the builder returns Result.
    //   //   (c) Intermediate setters return `Self` (chainable, infallible).
    //
    //   assert!(builder_src.contains("pub fn build(self)") &&
    //           builder_src.contains("-> Result<"),
    //           "#506: .build() MUST be the fallible point");
    //
    //   // Count non-build pub fn returning Result: must be 0.
    //   let non_build_results = builder_src.matches("pub fn ")
    //       .filter(|_| /* ... */)
    //       .count();
    //   // (Detailed scan logic the implementer wires; pattern is the
    //   //  same as `tf12_benten_engine_freeze_conformance_caps_no_regression_469.rs`
    //   //  source-scan style.)
    //
    // RED at HEAD: #506 builder closure not yet shipped per plan
    // §3 G-CORE-6 (it is the SOLE residual P-II mechanical item).
    panic!(
        "RED-PHASE: #506 builder `.build()` single-fallible-point \
         closure observable. Un-ignore at G-CORE-6 verify-pass wave \
         (the SOLE residual P-II mechanical item per plan §3 \
         G-CORE-6). Closure shape: (a) `pub fn build(self) -> \
         Result<_>` exists, (b) NO other `pub fn` on the builder \
         returns Result, (c) intermediate setters return `Self` \
         (infallible/chainable). Would-FAIL post-G-CORE-6 if the \
         builder ships with multi-point fallibility (the wrong shape \
         #506 explicitly closes). Scope-fence: this is a CLOSURE \
         OBSERVABLE on G-CORE-6's deliverable, NOT an R3 attempt to \
         do the work (the original fence_506_is_g_core_6_not_r3 \
         above remains)."
    );
}

/// R3-W5 EXTENSION — #506 closure regression-guard: post-verify-pass,
/// any re-introduction of fallibility on an intermediate setter (a
/// post-freeze regression that would re-fragment the
/// single-fallible-point property) MUST fail. This pin couples with
/// the cross-language ErrorCode rule (§3.5g): a forgotten regen on
/// the TS side after a Rust-side builder shape change would also
/// fail this assertion's `.d.ts` mirror sub-check.
///
/// §3.6f waiver: structural-source-scan pin (same waiver class as
/// tf12_benten_engine_freeze_conformance_caps_no_regression_469.rs).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-6 verify-pass wave; this pin then runs as a GREEN regression-guard."]
fn c6_pii_verify_506_no_regression_intermediate_setter_fallibility_post_close() {
    panic!(
        "RED-PHASE: #506 single-fallible-point regression-guard. \
         Un-ignore at G-CORE-6 verify-pass wave (then becomes a \
         GREEN guard against future regressions). Body at un-ignore: \
         scan the canonical builder source; assert no `pub fn` other \
         than `build()` returns Result. Mirrors the §4.69 \
         no-regression freeze-invariant pattern from \
         tf12_benten_engine_freeze_conformance_caps_no_regression_469.rs \
         at the builder surface."
    );
}
