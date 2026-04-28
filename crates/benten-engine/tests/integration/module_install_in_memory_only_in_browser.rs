//! Phase 2b G10-B — browser-target in-memory-only constraint
//! (Compromise #N+8).
//!
//! Pin sources:
//!   - `r1-wasm-target.json` — G10-B in-memory-only on
//!     wasm32-unknown-unknown in Phase 2b; persistence (IndexedDB,
//!     OPFS, etc.) defers to Phase 3.
//!   - `r2-test-landscape.md` §2.4
//!     `module_install_in_memory_only_in_browser_session`.
//!
//! On native (non-wasm32) targets: this file's tests assert the
//! same-session install + uninstall lifecycle works end-to-end. The
//! page-reload behaviour (post-Phase-3 IndexedDB) lives in the
//! browser-target G10-A worktree.
//!
//! On wasm32-unknown-unknown: a manifest declaring migrations MUST
//! reject with E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE because there
//! is no persistent backing store in Phase 2b.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::testing::{testing_compute_manifest_cid, testing_make_minimal_manifest};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn module_install_lifecycle_within_a_single_session() {
    // The native-target version of the in-session install+uninstall
    // pin. The wasm32-unknown-unknown variant lives in G10-A-browser.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.posts");
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    assert!(engine.is_module_installed(&cid));
    engine.uninstall_module(cid).unwrap();
    assert!(!engine.is_module_installed(&cid));
}

#[test]
fn module_uninstall_during_session_takes_effect_immediately() {
    // R2 §2.4 — uninstall takes effect within the same session
    // (no engine restart required) so operators can hot-swap modules
    // during devserver iteration.
    let (_dir, engine) = fresh_engine();
    let m = testing_make_minimal_manifest("acme.hotswap");
    let cid = testing_compute_manifest_cid(&m);
    engine.install_module(m, cid).unwrap();
    assert!(engine.is_module_installed(&cid));
    engine.uninstall_module(cid).unwrap();
    // Same engine handle — no restart.
    assert!(!engine.is_module_installed(&cid));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn module_install_with_migrations_rejects_on_wasm32() {
    use benten_engine::{MigrationStep, ModuleManifest};
    let (_dir, engine) = fresh_engine();
    let m = ModuleManifest {
        name: "acme.migrate".into(),
        version: "0.0.1".into(),
        modules: vec![],
        migrations: vec![MigrationStep {
            id: "add-author-index-2026-04".into(),
            description: None,
        }],
        signature: None,
    };
    let cid = testing_compute_manifest_cid(&m);
    let err = engine
        .install_module(m, cid)
        .expect_err("migrations on wasm32 must reject");
    assert_eq!(
        err.error_code(),
        benten_engine::ErrorCode::ModuleMigrationsRequirePersistence
    );
}
