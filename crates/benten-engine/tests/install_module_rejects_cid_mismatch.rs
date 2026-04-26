//! Phase 2b R3-C — D16-RESOLVED-FURTHER minimal CID-pin enforcement on
//! `Engine::install_module` (G10-B / G7-C).
//!
//! Pin sources: D16-RESOLVED-FURTHER (Engine::install_module(manifest,
//! expected_cid: Cid) — REQUIRED CID arg; error includes BOTH expected +
//! computed CIDs + 1-line manifest summary for operator-actionable diff);
//! sec-pre-r1-01 (manifest forge / supply-chain attack class);
//! r1-security-auditor.json D16; plan §3 G10-B (EXCLUSIVE OWNER per
//! wsa-r1-5 plan-internal conflict resolution); r2-test-landscape.md §1.8
//! `install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error`.
//!
//! Cross-territory: per R2 §10 + R3-C brief, R3-C owns the Rust-side
//! forge driver; TS-side install round-trip lives in `manifest.test.ts`
//! (R3-E).
//!
//! NOTE: file kept under `install_module_rejects_cid_mismatch.rs` per
//! the R3-C brief; the broader G10-B suite (matching-CID accepts,
//! manifest-summary error, helper round-trip) lives in
//! `tests/module_install.rs` (R3-B / R3-E ownership). This file is the
//! security-class slice ONLY.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::Engine::install_module
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::testing::testing_compute_manifest_cid (R2 §9 helper)
//   benten_core::Cid
//   benten_engine::error::EngineError or benten_errors::ErrorCode::ModuleManifestCidMismatch
//   (catalog code: E_MODULE_MANIFEST_CID_MISMATCH)

#[test]
#[ignore = "Phase 2b G10-B pending — D16 REQUIRED CID arg + dual-CID diff"]
fn install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error() {
    // D16-RESOLVED-FURTHER — `install_module` REQUIRES the expected_cid
    // arg (NOT Optional). On mismatch, the error message MUST include
    // BOTH the expected and computed CIDs + a 1-line manifest summary,
    // making the failure operator-actionable WITHOUT a source-code dive.
    //
    // R5 wires:
    //   1. Build a valid ModuleManifest m; let computed_cid =
    //      testing_compute_manifest_cid(&m).
    //   2. Compute a wrong_cid (e.g. testing_compute_manifest_cid(&other)
    //      for some other manifest).
    //   3. Call engine.install_module(m, wrong_cid).
    //   4. ASSERT: returns Err.
    //   5. ASSERT: the err carries (or maps to) the variant
    //      ModuleManifestCidMismatch (catalog code
    //      E_MODULE_MANIFEST_CID_MISMATCH).
    //   6. ASSERT: the err's Display impl includes BOTH cid strings:
    //         err_msg.contains(&computed_cid.to_string()) AND
    //         err_msg.contains(&wrong_cid.to_string())
    //   7. ASSERT: the err's Display includes a 1-line manifest summary
    //      (e.g. the manifest name + module count + provides-subgraphs
    //      count). The summary format is documented in
    //      `docs/MODULE-MANIFEST.md` per G10-B doc-drift.
    //
    // CRITICAL property: D16 chose REQUIRED-arg over Optional precisely
    // BECAUSE Optional becomes the lazy-developer footgun
    // (`install_module(m, None)` ships in production); REQUIRED forces
    // the operator to think about the pinned CID at every install site.
    //
    // sec-pre-r1-01 closure: the forged-manifest attack class —
    // attacker swaps an audited manifest for an attacker-controlled one
    // between review and install — is closed by this CID pin.
    todo!(
        "R5 G10-B — assert ModuleManifestCidMismatch err includes BOTH CIDs + \
         manifest summary"
    );
}

#[test]
#[ignore = "Phase 2b G10-B pending — D16 manifest-swap-after-review attack closure"]
fn module_manifest_swap_after_review_rejected() {
    // sec-pre-r1-01 — manifest-swap-after-review attack class.
    //
    // R5 wires:
    //   1. Reviewer audits ModuleManifest A; pins audited_cid =
    //      testing_compute_manifest_cid(&A).
    //   2. Operator (intentionally or not) calls
    //      engine.install_module(B, audited_cid) where B != A.
    //   3. ASSERT: ModuleManifestCidMismatch fires; B is NOT installed.
    //   4. ASSERT: the audited_cid in the error is presented FIRST
    //      (the "expected" position) and computed_cid == cid(B) is the
    //      "actual" — so the operator sees "we tried to install
    //      something OTHER than what we reviewed."
    //
    // Pin: the CID-pin gate is integrity-protective even when the
    // operator is not adversarial — it catches accidental swaps too.
    todo!("R5 G10-B — assert install of B with audited_cid(A) fires CidMismatch");
}

#[test]
#[ignore = "Phase 2b G10-B pending — D16 by-CID confusion attack closure"]
fn module_manifest_supply_chain_by_cid_confusion_rejected() {
    // sec-pre-r1-01 — by-CID-confusion attack class.
    //
    // Threat model: an attacker publishes a registry entry that maps a
    // legitimate CID to a malicious manifest. The operator copies the
    // CID from a trusted source (without recomputing) and passes it to
    // install_module. The CID pin is supposed to catch this — the
    // computed CID of the bytes the operator FETCHED won't match the
    // CID they PASSED.
    //
    // R5 wires:
    //   1. Construct legitimate ModuleManifest A; cid_a =
    //      testing_compute_manifest_cid(&A).
    //   2. Construct attacker-substituted ModuleManifest B (e.g. same
    //      provides-subgraphs but different requires-caps that include
    //      host:network:* — privilege escalation).
    //   3. Operator passes cid_a as expected_cid + B as the manifest
    //      bytes (the "fetched" payload).
    //   4. ASSERT: install_module(B, cid_a) returns CidMismatch.
    //
    // Pin: the gate works AS THE PRIMARY DEFENSE against by-CID
    // confusion until Phase 3's full Ed25519 surface lands.
    todo!("R5 G10-B — assert by-CID-confusion attack denied by CID-pin gate");
}
