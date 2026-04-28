//! Phase 2b G10-B — `Engine::install_module` happy-path tests
//! (D16-RESOLVED-FURTHER + R2 §1.8).
//!
//! Pin sources:
//!   - `r1-security-auditor.json` D16 RESOLVED-FURTHER —
//!     `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES
//!     the expected CID arg at compile time (NOT Optional). The
//!     compile-time requirement closes the lazy-developer footgun
//!     (`install_module(m, None)` shipping in production).
//!   - `r2-test-landscape.md` §1.8 — install-module unit pins.
//!   - `.addl/phase-2b/00-implementation-plan.md` §3.2 G10-B + §1
//!     exit criterion #4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
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
fn install_module_accepts_matching_cid() {
    // D16 RESOLVED-FURTHER — when expected_cid matches the canonical
    // CID of the manifest, install_module SUCCEEDS and returns the
    // installed CID.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.posts");
    let cid = testing_compute_manifest_cid(&m);
    let installed = engine
        .install_module(m, cid)
        .expect("happy path: matching CID must succeed");
    assert_eq!(installed, cid);
    assert!(
        engine.is_module_installed(&cid),
        "post-install, is_module_installed must report true"
    );
}

#[test]
fn install_module_persists_in_system_zone() {
    // The install lifecycle writes a `system:ModuleManifest` Node via
    // the privileged path. We verify the install round-trips through
    // the in-memory active set (the system-zone backend write is
    // covered indirectly — failure to write would surface as Err here).
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.persist");
    let cid = testing_compute_manifest_cid(&m);
    let installed = engine.install_module(m, cid).unwrap();
    assert_eq!(installed, cid);
    assert!(engine.is_module_installed(&cid));
}

#[test]
fn install_module_requires_cid_arg_at_compile_time() {
    // D16 RESOLVED-FURTHER — the `expected_cid` parameter MUST be a
    // REQUIRED positional arg, not `Option<Cid>`. The non-existence of
    // a one-arg overload is enforced by the fact this file's body
    // calls `install_module(m, cid)` — a single-arg overload would not
    // change this body, BUT a future contributor adding an
    // `install_module(m)` overload would still have to do so in code
    // review (this comment + the test name are the documentation
    // anchors).
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.shape");
    let cid = testing_compute_manifest_cid(&m);
    // The body simply calls install_module with both args; this asserts
    // the 2-arg signature compiles.
    let _ = engine.install_module(m, cid).unwrap();
}

#[test]
fn install_module_error_includes_manifest_summary() {
    // D16 RESOLVED-FURTHER — on CID mismatch the error MUST include a
    // 1-line manifest summary so the operator can identify *which*
    // manifest mis-installed.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.posts");
    let wrong = testing_make_distinct_dummy_cid();
    let err = engine.install_module(m, wrong).unwrap_err();
    let rendered = err.to_string();
    assert!(
        rendered.contains("acme.posts"),
        "err must contain manifest name (summary anchor); got: {rendered}"
    );
    assert!(
        rendered.contains("modules=1"),
        "err must contain module count from summary; got: {rendered}"
    );
    assert!(
        rendered.contains("caps="),
        "err must contain caps count from summary; got: {rendered}"
    );
}

#[test]
fn install_module_compute_cid_helper_round_trips() {
    // R2 §1.8 — the testing helper MUST compute the SAME CID that
    // install_module computes internally (and the same that
    // engine.compute_manifest_cid produces).
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.helper");
    let cid_via_helper = testing_compute_manifest_cid(&m);
    let cid_via_engine = engine
        .compute_manifest_cid(&m)
        .expect("compute_manifest_cid must agree with the helper");
    assert_eq!(cid_via_helper, cid_via_engine);
    let installed = engine
        .install_module(m, cid_via_helper)
        .expect("helper-CID must match engine-internal CID");
    assert_eq!(installed, cid_via_helper);
}

#[test]
fn install_module_idempotent_on_repeat() {
    // Re-installing the same CID is Ok — no double-write, no churn.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.idempotent");
    let cid = testing_compute_manifest_cid(&m);
    let first = engine.install_module(m.clone(), cid).unwrap();
    let second = engine.install_module(m, cid).unwrap();
    assert_eq!(first, second);
    assert!(engine.is_module_installed(&cid));
}
