//! G12-C sec-pre-r1-13 carry: assert the Phase-2a `sec-r6r2-02` test-helpers
//! cfg-gating (`#[cfg(any(test, feature = "test-helpers"))]` etc.) is NOT
//! silently dropped during the `Subgraph` type relocation from `benten-eval`
//! to `benten-core`.
//!
//! Per `r1-security-auditor.json` sec-pre-r1-13: Phase-2a security closures
//! "are MUST-NOT-REOPEN in Phase 2b. Specifically: ... G12-C migration MUST
//! preserve the Phase-2a `#[cfg(any(test, feature = "test-helpers"))]` gates
//! on `testing_*` surfaces (no surface should silently drop a gate during
//! the Subgraph type relocation)."
//!
//! Per `00-implementation-plan.md` line 568 sec-pre-r1-13 paragraph reinforces:
//! "G12-C migration MUST preserve the Phase-2a `#[cfg(any(test, feature =
//! "test-helpers"))]` gates on `testing_*` surfaces."
//!
//! Test approach: scan source trees for every `pub fn testing_*` /
//! `pub method testing_*` and assert each is preceded (within N lines) by
//! a recognised cfg-gate attribute. Catches gate-drop regressions during
//! the type relocation.
//!
//! TDD red-phase. Owner: R5 G12-C (qa-r4-02 sec-pre-r1-13 carry; R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-C red-phase: cfg-gate sweep across testing_* surfaces not yet wired"]
fn every_pub_testing_helper_in_benten_engine_carries_cfg_test_or_test_helpers_gate() {
    // Walk `crates/benten-engine/src/**/*.rs`; for each `pub fn testing_*`
    // or `pub fn ...testing_...` (methods), assert a cfg attribute on a nearby
    // line matches one of:
    //   - #[cfg(any(test, feature = "test-helpers"))]
    //   - #[cfg(any(test, feature = "envelope-cache-test-grade"))]
    //   - #[cfg(test)]
    //   - module-level cfg gating the surrounding mod
    todo!(
        "R5 G12-C: implement source-tree scanner; assert all testing_* surfaces \
         in benten-engine carry a recognised cfg-gate post-migration"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: cfg-gate sweep across benten-eval not yet wired"]
fn every_pub_testing_helper_in_benten_eval_carries_cfg_test_or_testing_feature_gate() {
    // benten-eval uses `feature = "testing"` (not `test-helpers`); see crate
    // Cargo.toml. Helpers must be gated by:
    //   - #[cfg(any(test, feature = "testing"))]
    //   - #[cfg(any(test, debug_assertions, feature = "testing"))]
    //   - #[cfg(test)]
    todo!(
        "R5 G12-C: implement source-tree scanner against benten-eval; \
         assert testing_* surfaces stay gated under the testing feature"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: post-migration testing_* surface count parity check not yet wired"]
fn count_of_testing_helpers_post_migration_matches_pre_migration_inventory() {
    // sec-pre-r1-13 reinforcement: the count of `testing_*` items in
    // benten-eval + benten-engine post-migration matches the pre-migration
    // count (modulo intentional renames per `r3-testing-helpers.md`). Catches
    // accidental drops during the relocation.
    let _expected_testing_helper_count_pre_migration: usize = 85; // per r3-testing-helpers.md
    todo!(
        "R5 G12-C: count `testing_*` items across benten-eval + benten-engine; \
         assert |delta| <= 2 (allowing for intentional renames documented \
         in r3-testing-helpers.md R4-FP-A audit)"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: parse_counter cfg-gate preserved test not yet wired"]
fn g12c_parse_counter_cfg_gate_preserved_post_subgraph_migration() {
    // sec-pre-r1-13 explicit named carry from R2 §1.9:
    // `g12c_parse_counter_cfg_gate_preserved_post_subgraph_migration`. Pins
    // that the Phase-2a sec-r6r3-02 parse-counter cfg-gate
    // (`testing_parse_counter` / `testing_reset_parse_counter`) survives the
    // Subgraph relocation.
    todo!(
        "R5 G12-C: locate testing_parse_counter + testing_reset_parse_counter; \
         assert each is preceded by a recognised cfg(any(test, feature = ...)) gate"
    )
}
