//! Phase 2b G10-B — `Engine::uninstall_module(cid)` happy-path +
//! idempotence + capability-retraction tests (R2 §1.8).
//!
//! Pin sources:
//!   - `r2-test-landscape.md` §1.8.
//!   - `.addl/phase-2b/00-implementation-plan.md` §3.2 G10-B —
//!     uninstall releases caps; subscriptions / IVM views referencing
//!     modules from this manifest get cleaned up.
//!   - `r1-wasm-target.json` — G10-B in-memory-only on wasm32 in
//!     Phase 2b; persistence (IndexedDB) defers to Phase 3.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::testing::{
    testing_compute_manifest_cid, testing_make_manifest_with_caps, testing_make_minimal_manifest,
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
fn uninstall_module_happy_path_after_install() {
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.posts");
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    assert!(engine.is_module_installed(&cid));
    engine
        .uninstall_module(cid)
        .expect("uninstall must succeed");
    assert!(
        !engine.is_module_installed(&cid),
        "post-uninstall, is_module_installed must report false"
    );
}

#[test]
fn uninstall_module_is_idempotent_repeat() {
    // R2 §1.8 — uninstalling an already-uninstalled module returns Ok(()).
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.repeat");
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    engine.uninstall_module(cid).unwrap();
    // Second uninstall on the same CID — must NOT error.
    engine.uninstall_module(cid).expect("idempotent on repeat");
}

#[test]
fn uninstall_module_is_idempotent_never_installed() {
    // The idempotence boundary is the CID, not the install history —
    // uninstalling a CID that was NEVER installed is also Ok(()).
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.never");
    let cid = testing_compute_manifest_cid(&m);
    engine
        .uninstall_module(cid)
        .expect("never-installed CID must be Ok");
    assert!(!engine.is_module_installed(&cid));
}

#[test]
fn module_uninstall_respects_capability_retraction() {
    // R2 §1.8 — when a manifest's `requires` block declares caps, the
    // engine's manifest-scoped active-cap set carries them while
    // installed and retracts them on uninstall.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_manifest_with_caps("acme.caps", &["host:compute:time"]);
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    let active = engine.active_module_capabilities();
    assert!(
        active.contains("host:compute:time"),
        "post-install, active manifest-scoped caps must contain host:compute:time; got {:?}",
        active
    );
    engine.uninstall_module(cid).unwrap();
    let active_after = engine.active_module_capabilities();
    assert!(
        !active_after.contains("host:compute:time"),
        "post-uninstall, active manifest-scoped caps must NOT contain host:compute:time; got {:?}",
        active_after
    );
}

#[test]
fn module_uninstall_preserves_caps_required_by_sibling_manifest() {
    // The cap-overlap edge case — uninstalling M MUST NOT retract a
    // cap that a sibling manifest N still requires.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_manifest_with_caps("acme.m", &["host:compute:time"]);
    let n = testing_make_manifest_with_caps("acme.n", &["host:compute:time"]);
    let cid_m = testing_compute_manifest_cid(&m);
    let cid_n = testing_compute_manifest_cid(&n);
    engine.install_module(m, cid_m).unwrap();
    engine.install_module(n, cid_n).unwrap();
    engine.uninstall_module(cid_m).unwrap();
    let active = engine.active_module_capabilities();
    assert!(
        active.contains("host:compute:time"),
        "sibling N still requires host:compute:time; cap must survive M's uninstall; got {:?}",
        active
    );
    // Now uninstall N — cap should retract.
    engine.uninstall_module(cid_n).unwrap();
    let active_after = engine.active_module_capabilities();
    assert!(
        !active_after.contains("host:compute:time"),
        "after both uninstalled, cap must be retracted; got {:?}",
        active_after
    );
}
