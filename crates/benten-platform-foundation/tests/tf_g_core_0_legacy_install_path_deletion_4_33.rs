//! ADDL Phase-4-Meta-Core R4.1-FP-1 (Class B coverage) — §4.33 legacy
//! `module_ecosystem::install_plugin*` path DELETION + 4-test-file
//! migration to `plugin_lifecycle::install_plugin` — G-CORE-0 verify-
//! pass factual state-check pin (§1.A.FROZEN item 7 deliverable).
//!
//! ## RED-PHASE — un-ignore at G-CORE-0
//!
//! This file is the R4.1-FP-1 closure of L1 MAJOR finding
//! `coverage-completeness-r4.1-2` (§4.33 legacy install-path DELETION
//! had ZERO test-pin coverage at HEAD `ed03729a`). It models on the
//! `tf10_c6_pii_verify_pass_factual_state_check.rs` template
//! (factual-state-check idiom): a `git grep` over the live source tree
//! asserts the FACTUAL POST-STATE that the G-CORE-0 wave is supposed
//! to have produced. It performs NO migration and NO deletion (those
//! are the wave's deliverables — this pin is purely the closing-wave
//! verifier).
//!
//! ============================================================================
//! ⚠️ THIS IS A VERIFY-PASS FACTUAL ASSERTION — **NOT** A RE-RUN SWEEP.
//! ============================================================================
//!
//! Per L1 r4.1-2 fix_now_action + plan §1.A.FROZEN item 7 + plan §3
//! G-CORE-0 group def + `docs/future/phase-4-backlog.md §4.33`:
//!
//!   - §1.A.FROZEN item 7: "§4.33 legacy `module_ecosystem::install_plugin*`
//!     path DELETED — hard deadline = Core opening wave (the v1
//!     platform-shippable assessment cannot tolerate two install paths
//!     with different security envelopes; this is a deletion, not a
//!     deprecation, per HARD RULE 12 clause-(a))."
//!   - Plan §3 G-CORE-0: "§4.33 legacy `module_ecosystem::install_plugin*`
//!     DELETION + 4-test-file migration to
//!     `plugin_lifecycle::install_plugin` (~+100 LOC net). HARD-RULE-12
//!     clause-(a) deletion."
//!   - phase-4-backlog §4.33 names the migration target as
//!     `crates/benten-platform-foundation/src/plugin_lifecycle.rs::
//!     install_plugin` and the 4 test files that consume the legacy path:
//!     (a) `plugin_content_cid_mismatch_rejected_on_receive.rs`;
//!     (b) `plugin_manifest_substitution_at_install_rejected.rs`;
//!     (c) `plugin_heterogeneity_incompatible.rs`;
//!     (d) `g24d_substantive_pipeline.rs`.
//!     All currently carry `#![allow(deprecated)]` headers + import
//!     `module_ecosystem::{InstallerShape, install_plugin}` /
//!     `install_plugin_persisting_did` (the legacy symbols slated for
//!     DELETION).
//!
//! ## SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 §3.6f)
//!
//! Each arm below is a FACTUAL STATE CHECK over the live source tree
//! via `std::fs::read_to_string` walks (file-system substrate — no
//! external `git` dependency, mirrors the C6 template's idiom while
//! avoiding `Command::new("git")` for sandboxed-runner portability).
//! Real assertion + observable would-FAIL consequence (post-G-CORE-0
//! the legacy `pub fn install_plugin` declarations MUST be absent
//! from `module_ecosystem.rs`; the file may even be deleted entirely).
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): exercises the SPECIFIC
//!     deletion-deliverable arm (factual post-state check; would-FAIL
//!     pre-deletion when the symbol is still present).
//!   - pim-12 (§3.6e): RED-PHASE staged-pin; reviewer verifies LANDING
//!     status (not just spec-pin presence) at G-CORE-0 closing-wave
//!     sweep.
//!   - pim-18 (§3.6f): substantive arm — concrete factual-state grep
//!     with observable would-FAIL signal, NOT a sentinel.
//!   - HARD-RULE-12 clause-(a): the destination is G-CORE-0 (the
//!     opening-wave deletion deliverable named in plan §3 + §1.A.FROZEN
//!     item 7); zero phantom destinations.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!   - pattern-induction r4.1 candidate-2 (deletion-as-deliverable):
//!     this is the additive factual-state-check pin establishing the
//!     idiom for deletion deliverables — the C6 template was already a
//!     factual-state check for an ALREADY-DELETED rename target; this
//!     file extends the idiom to a NOT-YET-DELETED target.
//!
//! Pins: G-CORE-0 · §1.A.FROZEN item 7 · phase-4-backlog §4.33.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // crates/benten-platform-foundation/ → repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_or_absent(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

// ---------------------------------------------------------------------------
// §4.33 FACTUAL ARM 1 — `pub fn install_plugin` MUST be ABSENT from
// `crates/benten-platform-foundation/src/module_ecosystem.rs`.
// This is the structural post-deletion factual state-check.
// Post-G-CORE-0: the file may be deleted entirely (read_to_string =>
// None → arm passes); OR the file may persist with the legacy
// install_plugin* surface deleted (read returns Some(...) but does NOT
// contain `pub fn install_plugin`).
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-0 (§4.33 legacy module_ecosystem::install_plugin* DELETION factual state-check, NOT a re-run)"]
fn g_core_0_verify_legacy_install_plugin_symbol_deleted() {
    let module_ecosystem_path = repo_root()
        .join("crates")
        .join("benten-platform-foundation")
        .join("src")
        .join("module_ecosystem.rs");

    let contents = read_or_absent(&module_ecosystem_path);
    match contents {
        None => {
            // FACTUAL POST-STATE: the file was deleted entirely. This
            // is the cleanest post-G-CORE-0 outcome and trivially
            // satisfies the deletion deliverable.
        }
        Some(body) => {
            // FACTUAL POST-STATE: the file persists but the legacy
            // `pub fn install_plugin` and `pub fn install_plugin_persisting_did`
            // declarations MUST be absent.
            assert!(
                !body.contains("pub fn install_plugin"),
                "§4.33 verify-pass: `pub fn install_plugin` MUST be \
                 ABSENT from `crates/benten-platform-foundation/src/\
                 module_ecosystem.rs` post-G-CORE-0 (HARD-RULE-12 \
                 clause-(a) DELETION). The legacy symbol bypasses \
                 Layer-2 consent + Layer-1 cap cascade per CLAUDE.md \
                 #18; two install paths with different security \
                 envelopes cannot coexist into the v1 freeze \
                 (§1.A.FROZEN item 7). NOTE: do NOT 'fix' this by \
                 re-adding the symbol — investigate the regression \
                 source instead; the migration target is \
                 `plugin_lifecycle::install_plugin` per phase-4-backlog \
                 §4.33."
            );
            // The persisting-did variant is the second deletion target.
            assert!(
                !body.contains("pub fn install_plugin_persisting_did"),
                "§4.33 verify-pass: `pub fn install_plugin_persisting_did` \
                 MUST be ABSENT post-G-CORE-0 (sibling deletion target \
                 per phase-4-backlog §4.33)."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// §4.33 FACTUAL ARM 2 — the migration target `pub fn install_plugin`
// MUST be PRESENT in `crates/benten-platform-foundation/src/plugin_lifecycle.rs`
// (the canonical post-G-CORE-0 install path with full Layer-2 consent +
// Layer-1 cap cascade + caller-mint-first contract).
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-0 (§4.33 migration target present factual state-check)"]
fn g_core_0_verify_plugin_lifecycle_install_plugin_is_canonical() {
    let plugin_lifecycle_path = repo_root()
        .join("crates")
        .join("benten-platform-foundation")
        .join("src")
        .join("plugin_lifecycle.rs");

    let body = read_or_absent(&plugin_lifecycle_path).expect(
        "§4.33 verify-pass: `crates/benten-platform-foundation/src/\
         plugin_lifecycle.rs` MUST exist — it is the canonical \
         post-G-CORE-0 install path (HARD-RULE-12 clause-(a) target). \
         Absence indicates the migration premise is stale; surface to \
         the orchestrator.",
    );
    assert!(
        body.contains("pub fn install_plugin"),
        "§4.33 verify-pass: `pub fn install_plugin` MUST be PRESENT on \
         the `benten-platform-foundation::plugin_lifecycle` canonical \
         surface (the 11-step install pipeline with full Layer-2 \
         consent + Layer-1 cap cascade per CLAUDE.md #18). Absence \
         means the verify-pass premise is stale — surface to the \
         orchestrator, do NOT re-run the migration sweep."
    );
}

// ---------------------------------------------------------------------------
// §4.33 FACTUAL ARM 3 — the 4 test files named in phase-4-backlog §4.33
// MUST no longer import the legacy `module_ecosystem::install_plugin` /
// `install_plugin_persisting_did` symbols (the 4-test-file migration
// deliverable). Each file MAY have been deleted entirely OR have
// migrated its imports to `plugin_lifecycle::install_plugin`; either is
// acceptable post-G-CORE-0.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-0 (§4.33 4-test-file migration factual state-check)"]
fn g_core_0_verify_4_test_files_migrated_off_legacy_install_path() {
    const LEGACY_TEST_FILES: &[&str] = &[
        "crates/benten-platform-foundation/tests/plugin_content_cid_mismatch_rejected_on_receive.rs",
        "crates/benten-platform-foundation/tests/plugin_manifest_substitution_at_install_rejected.rs",
        "crates/benten-platform-foundation/tests/plugin_heterogeneity_incompatible.rs",
        "crates/benten-platform-foundation/tests/g24d_substantive_pipeline.rs",
    ];

    let root = repo_root();
    for rel in LEGACY_TEST_FILES {
        let path = root.join(rel);
        let Some(body) = read_or_absent(&path) else {
            // POST-STATE: file deleted entirely (one acceptable
            // migration outcome).
            continue;
        };
        // POST-STATE: file persists but no longer imports the legacy
        // `module_ecosystem` install path. The simplest stable signal
        // is the `use ...module_ecosystem...install_plugin` import
        // line — its absence is the migration's structural marker.
        let legacy_import_a = "module_ecosystem::install_plugin";
        let legacy_import_b = "module_ecosystem::{InstallerShape, install_plugin";
        let legacy_import_c = "install_plugin_persisting_did";
        assert!(
            !body.contains(legacy_import_a)
                && !body.contains(legacy_import_b)
                && !body.contains(legacy_import_c),
            "§4.33 verify-pass: test file `{rel}` MUST no longer import \
             from the legacy `module_ecosystem::install_plugin*` path \
             post-G-CORE-0. Migrate to `plugin_lifecycle::install_plugin` \
             per phase-4-backlog §4.33. NOTE: do NOT 'fix' this by \
             re-adding the legacy import — the legacy path is a \
             HARD-RULE-12 clause-(a) DELETION target; the v1 \
             platform-shippable assessment cannot tolerate two install \
             paths with different security envelopes."
        );
    }
}

// ---------------------------------------------------------------------------
// §4.33 SCOPE-FENCE NOTE (assertion-as-documentation): this file is the
// G-CORE-0 verify-pass factual-state pin per §1.A.FROZEN item 7. The
// DELETION itself + the 4-test-file MIGRATION are G-CORE-0 deliverables,
// NOT this R3 pin's deliverables. This trivially-true assertion
// documents the fence so a future reader does not mistake the
// factual-state-check pin for ownership of the deletion work.
// ---------------------------------------------------------------------------
#[test]
fn g_core_0_scope_fence_4_33_deletion_is_g_core_0_not_r3() {
    let r3_owns_4_33 = false;
    assert!(
        !r3_owns_4_33,
        "scope-fence: §4.33 legacy install-path DELETION + 4-test-file \
         migration is G-CORE-0's deliverable (plan §3 G-CORE-0 + \
         §1.A.FROZEN item 7 + phase-4-backlog §4.33); this R3 pin is \
         the closing-wave verify-pass only."
    );
}
