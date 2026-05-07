// Phase 3 G20-A3 — un-ignored: lifts D16 minimal-CID-pin to a
// green-phase end-to-end driver per dispatch-conventions §3.6b.
// Production code at
// `crates/benten-engine/src/engine_modules.rs::install_module`
// already returns `EngineError::ModuleManifestCidMismatch { expected,
// computed, summary }`; this test pins the rendered Display contains
// BOTH CIDs + a 1-line manifest summary so operators can diagnose
// from logs alone.
//
//! Phase 3 G20-A3 (Phase 2b R4-FP B-3 origin) — D16 minimal-CID-pin:
//! `Engine::install_module` mismatch error includes BOTH the expected
//! and computed CIDs plus a 1-line manifest summary.
//!
//! Pin sources:
//!   - `r1-security-auditor.json` D16 RESOLVED-FURTHER — the error
//!     MUST surface BOTH CIDs + a 1-line manifest summary
//!     (provides-subgraphs name + module count + caps count).
//!   - `r2-test-landscape.md` §1.8.
//!   - `docs/future/phase-3-backlog.md §7.3.A.4` (CLOSED at G20-A3).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::manifest_signing::ManifestVerifyArgs;
use benten_engine::testing::{
    testing_compute_manifest_cid, testing_make_distinct_dummy_cid, testing_make_minimal_manifest,
};

#[test]
fn install_module_error_body_carries_both_cids_and_summary_line() {
    // D16 RESOLVED-FURTHER — minimal pin: the rendered error string
    // (the form operators see in logs / stderr) MUST contain:
    //   (a) the EXPECTED CID (the one the caller passed),
    //   (b) the COMPUTED CID (the one the engine derived from the bytes),
    //   (c) a 1-line manifest summary anchored on the manifest name.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let m = testing_make_minimal_manifest("acme.posts");
    let computed = testing_compute_manifest_cid(&m);
    let expected_wrong = testing_make_distinct_dummy_cid();
    assert_ne!(
        computed, expected_wrong,
        "test fixture invariant: distinct dummy CID must differ from computed"
    );

    let err = engine
        .install_module(
            m,
            expected_wrong,
            ManifestVerifyArgs::unsigned_development(),
        )
        .expect_err("CID mismatch must error, not silently install");

    let rendered = err.to_string();
    assert!(
        rendered.contains(&computed.to_string()),
        "rendered error MUST include the COMPUTED CID; got: {rendered}"
    );
    assert!(
        rendered.contains(&expected_wrong.to_string()),
        "rendered error MUST include the EXPECTED CID (caller-supplied); got: {rendered}"
    );
    assert!(
        rendered.contains("acme.posts"),
        "rendered error MUST include the manifest summary anchor (name); got: {rendered}"
    );
}
