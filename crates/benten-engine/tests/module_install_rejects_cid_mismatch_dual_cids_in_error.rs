//! Phase 2b R4-FP B-3 — D16 minimal-CID-pin assertion:
//! `Engine::install_module` mismatch error includes BOTH the expected
//! and computed CIDs plus a 1-line manifest summary.
//!
//! TDD red-phase. Pin sources:
//!   - `r1-security-auditor.json` D16 RESOLVED-FURTHER — the error MUST
//!     surface BOTH CIDs + a 1-line manifest summary (provides-subgraphs
//!     name + module count + caps count) so an operator can diagnose
//!     the mismatch from logs alone (no source-code dive).
//!   - `r2-test-landscape.md` §1.8.
//!
//! NOTE: this file is the FOCUSED minimal-CID-pin assertion called out
//! by the R4-FP B-3 brief (qa-r4-05 fix). The broader security-class
//! mismatch suite (manifest-swap-after-review attack, by-CID confusion
//! attack) lives in `install_module_rejects_cid_mismatch.rs` (R3-C
//! ownership). The cross-crate integration variant lives in
//! `tests/integration/install_module_rejects_cid_mismatch.rs` (R3-E).
//! All three exist intentionally per `r3-consolidation.md` §2 #8 dual-
//! ownership precedent.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::Engine::install_module
//   benten_engine::testing::testing_compute_manifest_cid
//   benten_engine::testing::testing_make_minimal_manifest
//   benten_engine::testing::testing_make_distinct_dummy_cid
//   benten_errors::ErrorCode::ModuleManifestCidMismatch (E_MODULE_MANIFEST_CID_MISMATCH)

#[test]
#[ignore = "Phase 2b G10-B pending — D16 dual-CID + summary in mismatch error body"]
fn install_module_error_body_carries_both_cids_and_summary_line() {
    // D16 RESOLVED-FURTHER — minimal pin: the rendered error string
    // (the form operators see in logs / stderr) MUST contain:
    //   (a) the EXPECTED CID (the one the caller passed),
    //   (b) the COMPUTED CID (the one the engine derived from the bytes),
    //   (c) a 1-line manifest summary (provides-subgraphs name +
    //       module count + caps count).
    //
    // This is the OPERATOR-ACTIONABLE pin: without (a)+(b), the operator
    // cannot tell which side is wrong; without (c), the operator cannot
    // tell WHICH manifest was being installed.
    //
    // R5 G10-B wires:
    //   1. let m = testing_make_minimal_manifest("acme.posts");
    //   2. let computed = testing_compute_manifest_cid(&m);
    //   3. let expected_wrong = testing_make_distinct_dummy_cid();
    //      assert_ne!(computed, expected_wrong, "test fixture invariant");
    //   4. let err = engine.install_module(m, expected_wrong)
    //          .expect_err("CID mismatch must error, not silently install");
    //   5. let rendered = err.to_string();
    //   6. ASSERT rendered.contains(&computed.to_string());
    //   7. ASSERT rendered.contains(&expected_wrong.to_string());
    //   8. ASSERT rendered.contains("acme.posts"); // summary anchor
    //
    // The exact summary format is documented in
    // `docs/MODULE-MANIFEST.md` (G10-B doc-drift item); the test only
    // pins the load-bearing components (manifest name appears).
    todo!(
        "R5 G10-B — assert CID-mismatch error string contains BOTH CIDs + \
         manifest summary"
    );
}
