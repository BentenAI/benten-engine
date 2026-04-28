//! Phase 2b G10-B — D16-RESOLVED-FURTHER minimal CID-pin enforcement on
//! `Engine::install_module`.
//!
//! Pin sources: D16-RESOLVED-FURTHER (Engine::install_module(manifest,
//! expected_cid: Cid) — REQUIRED CID arg; error includes BOTH expected +
//! computed CIDs + 1-line manifest summary for operator-actionable diff);
//! sec-pre-r1-01 (manifest forge / supply-chain attack class);
//! r1-security-auditor.json D16; plan §3 G10-B (EXCLUSIVE OWNER per
//! wsa-r1-5 plan-internal conflict resolution); r2-test-landscape.md §1.8
//! `install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::ErrorCode;
use benten_engine::testing::{
    testing_compute_manifest_cid, testing_make_distinct_dummy_cid, testing_make_minimal_manifest,
};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error() {
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.posts");
    let computed = testing_compute_manifest_cid(&m);
    let wrong = testing_make_distinct_dummy_cid();
    assert_ne!(computed, wrong, "test fixture invariant");

    let err = engine
        .install_module(m, wrong)
        .expect_err("CID mismatch must error, not silently install");
    assert_eq!(err.error_code(), ErrorCode::ModuleManifestCidMismatch);
    let rendered = err.to_string();
    assert!(
        rendered.contains(&computed.to_base32()),
        "rendered err must include the COMPUTED CID for operator diff; got: {rendered}"
    );
    assert!(
        rendered.contains(&wrong.to_base32()),
        "rendered err must include the EXPECTED CID for operator diff; got: {rendered}"
    );
    assert!(
        rendered.contains("acme.posts"),
        "rendered err must include the manifest summary anchor; got: {rendered}"
    );
}

#[test]
fn module_manifest_swap_after_review_rejected() {
    // sec-pre-r1-01 — manifest-swap-after-review attack class. Reviewer
    // pins the audited CID; an operator (intentional or not) calls
    // install_module with a DIFFERENT manifest plus the audited CID.
    // The CID-pin gate catches the swap.
    let (_dir, engine) = fresh_engine();
    let a = testing_make_minimal_manifest("acme.audited");
    let b = testing_make_minimal_manifest("acme.attacker.swap");
    let audited_cid = testing_compute_manifest_cid(&a);
    let cid_b = testing_compute_manifest_cid(&b);
    assert_ne!(audited_cid, cid_b);

    let err = engine
        .install_module(b, audited_cid)
        .expect_err("CID-pin gate must catch the swap");
    assert_eq!(err.error_code(), ErrorCode::ModuleManifestCidMismatch);
    // The audited_cid is the EXPECTED arg; cid_b is what the engine
    // computes. Both appear in the rendered err.
    let rendered = err.to_string();
    assert!(rendered.contains(&audited_cid.to_base32()));
    assert!(rendered.contains(&cid_b.to_base32()));
    assert!(!engine.is_module_installed(&cid_b));
    assert!(!engine.is_module_installed(&audited_cid));
}

#[test]
fn module_manifest_supply_chain_by_cid_confusion_rejected() {
    // sec-pre-r1-01 — by-CID-confusion attack class.
    // Operator copies cid_a from a trusted source but has FETCHED
    // attacker-substituted bytes B. The CID pin catches the mismatch.
    let (_dir, engine) = fresh_engine();
    let a = testing_make_minimal_manifest("acme.legit");
    // B has same name but elevated requires — privilege-escalation
    // shape attackers would use.
    let b = benten_engine::ModuleManifest {
        name: "acme.legit".into(),
        version: "0.0.1".into(),
        modules: vec![benten_engine::ModuleManifestEntry {
            name: "acme.legit.handler".into(),
            cid: "bafy_dummy_module_for_acme.legit".into(),
            requires: vec!["host:network:*".into()],
        }],
        migrations: vec![],
        signature: None,
    };
    let cid_a = testing_compute_manifest_cid(&a);
    let cid_b = testing_compute_manifest_cid(&b);
    assert_ne!(cid_a, cid_b);

    let err = engine
        .install_module(b, cid_a)
        .expect_err("by-CID-confusion attack must be denied by CID-pin gate");
    assert_eq!(err.error_code(), ErrorCode::ModuleManifestCidMismatch);
}
