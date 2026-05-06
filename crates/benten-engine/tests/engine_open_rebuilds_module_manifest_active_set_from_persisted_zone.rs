//! R6FP-Group-1 (r6-arch-1) regression pin —
//! `Engine::open` rebuilds the in-memory `installed_modules` active set
//! from the durable `system:ModuleManifest` zone.
//!
//! Pre-R6FP-G1, `install_module` persisted manifests to the
//! `system:ModuleManifest` zone (durable across engine restart) but
//! the in-memory `installed_modules` BTreeMap was NOT rebuilt at
//! engine open — so a freshly-restarted Engine returned `false` from
//! `is_module_installed` for previously-installed CIDs. The
//! install_module docstring claimed "the manifest survives engine
//! restart and is sync-eligible for Phase-3 federation"; the code
//! only honoured the "survives engine restart" half on disk, not in
//! the in-memory indexes the dispatcher consults at SANDBOX entry.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::manifest_signing::ManifestVerifyArgs;
use benten_engine::{Engine, ModuleManifest, ModuleManifestEntry};

fn make_manifest(name: &str) -> ModuleManifest {
    let module_cid =
        benten_core::Cid::from_blake3_digest(*blake3::hash(name.as_bytes()).as_bytes());
    let entry = ModuleManifestEntry {
        name: format!("{name}:mod0"),
        cid: module_cid.to_base32(),
        requires: vec!["host:compute:time".to_string()],
    };
    ModuleManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        modules: vec![entry],
        signature: None,
        migrations: Vec::new(),
    }
}

/// R6FP-G1 regression: `install_module` then `drop Engine` then
/// `Engine::open(same_path)` then `is_module_installed(installed_cid)` ⇒ true.
#[test]
fn engine_open_rebuilds_module_manifest_active_set_from_persisted_zone() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");

    let manifest = make_manifest("test:rehydration");
    let computed_cid;

    // Phase 1: open engine, install manifest, drop engine.
    {
        let engine = Engine::open(&path).expect("open engine");
        computed_cid = engine
            .compute_manifest_cid(&manifest)
            .expect("compute manifest cid");
        engine
            .install_module(
                manifest.clone(),
                computed_cid,
                ManifestVerifyArgs::unsigned_development(),
            )
            .expect("install manifest");
        assert!(
            engine.is_module_installed(&computed_cid),
            "post-install, the manifest must appear in the in-memory active set"
        );
        // Engine drops at end of scope — closing the redb backend.
    }

    // Phase 2: re-open engine against the same path. Pre-R6FP-G1,
    // is_module_installed returned false because the in-memory active
    // set was NOT rebuilt from the durable system:ModuleManifest zone.
    {
        let engine = Engine::open(&path).expect("re-open engine");
        assert!(
            engine.is_module_installed(&computed_cid),
            "post-restart, Engine::open MUST rebuild the in-memory \
             installed_modules active set from the persisted \
             system:ModuleManifest zone (R6FP-G1 r6-arch-1: pre-fix \
             the docstring claim 'manifest survives engine restart' \
             was honoured on disk only — the dispatcher's in-memory \
             active set was empty)"
        );
    }
}
