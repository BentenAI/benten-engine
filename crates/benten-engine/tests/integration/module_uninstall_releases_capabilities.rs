//! Phase 2b G10-B — `uninstall_module` cap-retraction integration test.
//!
//! Pin sources:
//!   - `r2-test-landscape.md` §1.8
//!     `module_uninstall_respects_capability_retraction`.
//!   - Plan §3.2 G10-B — uninstall releases caps.
//!   - `r1-security-auditor.json` D9 — manifest `requires` block is
//!     authoritative for cap declarations; ghost-cap drift across
//!     install/uninstall cycles would defeat the policy hook.
//!
//! This is the cross-crate INTEGRATION variant of the
//! `module_uninstall_respects_capability_retraction` unit pin in
//! `tests/module_uninstall.rs`. The unit test asserts the engine's
//! introspection accessor; this integration test asserts the same
//! property at the cross-crate boundary.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::testing::{testing_compute_manifest_cid, testing_make_manifest_with_caps};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn module_uninstall_releases_capabilities_end_to_end() {
    let (_dir, engine) = fresh_engine();
    let m = testing_make_manifest_with_caps("acme.scoped", &["host:compute:time"]);
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    assert!(
        engine
            .active_module_capabilities()
            .contains("host:compute:time")
    );
    engine.uninstall_module(cid).unwrap();
    assert!(
        !engine
            .active_module_capabilities()
            .contains("host:compute:time")
    );
}

#[test]
fn module_uninstall_does_not_retract_cap_required_by_sibling_manifest() {
    let (_dir, engine) = fresh_engine();
    let m = testing_make_manifest_with_caps("acme.m", &["host:compute:time"]);
    let n = testing_make_manifest_with_caps("acme.n", &["host:compute:time"]);
    let cid_m = testing_compute_manifest_cid(&m);
    let cid_n = testing_compute_manifest_cid(&n);
    engine.install_module(m, cid_m).unwrap();
    engine.install_module(n, cid_n).unwrap();
    engine.uninstall_module(cid_m).unwrap();
    assert!(
        engine
            .active_module_capabilities()
            .contains("host:compute:time"),
        "sibling N still requires the cap; must survive M's uninstall"
    );
    engine.uninstall_module(cid_n).unwrap();
    assert!(
        !engine
            .active_module_capabilities()
            .contains("host:compute:time")
    );
}
