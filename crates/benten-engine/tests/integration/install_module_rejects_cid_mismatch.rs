//! Phase 2b R3 (R3-E) — G10-B `install_module` cross-crate integration.
//!
//! TDD red-phase. Pin source: plan §3 G10-B + D16-RESOLVED-FURTHER
//! (REQUIRED — not Optional — `expected_cid: Cid` arg on
//! `Engine::install_module(manifest, expected_cid)`; on mismatch the
//! error includes BOTH the expected and computed CIDs plus a 1-line
//! manifest summary for operator-actionable diff).
//!
//! Per §10 ownership disambiguation, R3-C owns the security-framing
//! Rust-side forge driver (`security/manifest_forge.rs`); R3-E owns the
//! cross-crate integration variant — the path that exercises the FULL
//! engine boundary (manifest → CID-check → typed-error surface →
//! operator-readable error body) so we know the wire-shape lands intact
//! end-to-end, not only at the unit-shape pin.
//!
//! Two assertions:
//!   1. `install_module` REJECTS a manifest whose computed CID differs
//!      from the supplied `expected_cid`.
//!   2. The error body carries BOTH CIDs and a manifest summary line.
//!
//! **Status:** RED-PHASE (Phase 2b G10-B pending). `Engine::install_module`
//! does not yet exist.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error` —
/// plan §3 G10-B must-pass + R2 §1.8.
#[test]
#[ignore = "Phase 2b G10-B pending — Engine::install_module + ModuleManifest unimplemented"]
fn install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error() {
    let (_dir, mut engine) = fresh_engine();

    // Build a manifest via the test helper (G10-B owns the constructor;
    // testing_compute_manifest_cid is a §9 backdoor for tests).
    let manifest = benten_engine::testing::testing_make_minimal_manifest("acme.posts");
    let true_cid = benten_engine::testing::testing_compute_manifest_cid(&manifest);

    // Construct an obviously-wrong CID — the error path must reject this.
    let wrong_cid = benten_engine::testing::testing_make_distinct_dummy_cid();
    assert_ne!(
        true_cid, wrong_cid,
        "test setup invariant — wrong_cid must differ from true_cid"
    );

    let err = engine
        .install_module(manifest, wrong_cid)
        .expect_err("CID mismatch must surface as a typed error, not silent install");

    // Operator-actionable diff requires BOTH CIDs visible in the error
    // body + a 1-line manifest summary. We assert against the rendered
    // error string (the wire-shape that operators see in logs).
    let rendered = err.to_string();
    assert!(
        rendered.contains(&true_cid.to_base32()),
        "error body must include the COMPUTED manifest CID for operator diff: {}",
        rendered
    );
    assert!(
        rendered.contains(&wrong_cid.to_base32()),
        "error body must include the EXPECTED manifest CID for operator diff: {}",
        rendered
    );
    assert!(
        rendered.contains("acme.posts"),
        "error body must include a manifest summary line (e.g. the \
         provides-subgraphs key 'acme.posts') so operators can identify \
         which manifest mis-installed: {}",
        rendered
    );
}
