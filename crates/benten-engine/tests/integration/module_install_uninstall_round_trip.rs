//! Phase 2b G10-B — install/uninstall round-trip 5-row fixture matrix.
//! Closes plan §1 exit criterion #4.
//!
//! Pin sources:
//!   - `.addl/phase-2b/00-implementation-plan.md` §1 exit criterion #4
//!     (install + uninstall round-trip with `requires`-cap propagation).
//!   - `r2-test-landscape.md` §2.4 `module_install_round_trip_5_row_fixture_matrix`.
//!
//! The 5-row matrix exercises the {install, uninstall} cross-product
//! against a representative spread of manifest shapes (no caps, single
//! cap, multi-cap, multi-module, cap-overlap with sibling manifest).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::{
    testing_compute_manifest_cid, testing_make_manifest_with_caps, testing_make_minimal_manifest,
};
use benten_engine::{Engine, ModuleManifest, ModuleManifestEntry};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

fn manifest_multi_module(name: &str, count: usize) -> ModuleManifest {
    ModuleManifest {
        name: name.to_string(),
        version: "0.0.1".into(),
        modules: (0..count)
            .map(|i| ModuleManifestEntry {
                name: format!("{name}.handler{i}"),
                cid: format!("bafy_dummy_module_for_{name}_{i}"),
                requires: vec![],
            })
            .collect(),
        migrations: vec![],
        signature: None,
    }
}

#[test]
fn module_install_uninstall_round_trip_5_row_fixture_matrix() {
    // Plan §1 exit criterion #4 — full install/uninstall round-trip
    // across 5 representative manifest shapes.
    let (_dir, engine) = fresh_engine();

    // Pre-install a sibling manifest used by Row 5's cap-overlap case.
    // The sibling requires "host:compute:time" — when Row 5 also
    // requires it, uninstalling Row 5 MUST NOT retract the cap (the
    // sibling still declares it).
    let sibling = testing_make_manifest_with_caps("acme.sibling", &["host:compute:time"]);
    let sibling_cid = testing_compute_manifest_cid(&sibling);
    engine.install_module(sibling.clone(), sibling_cid).unwrap();

    // Each row: (label, manifest, expected-cap-set-on-the-manifest).
    let rows: Vec<(&'static str, ModuleManifest, Vec<&'static str>)> = vec![
        // Row 1: empty-caps, single-module
        (
            "empty.caps.single",
            testing_make_minimal_manifest("row1.empty"),
            vec![],
        ),
        // Row 2: single-cap, single-module
        (
            "single.cap.single",
            testing_make_manifest_with_caps("row2.single", &["host:compute:time"]),
            vec!["host:compute:time"],
        ),
        // Row 3: multi-cap, single-module
        (
            "multi.cap.single",
            testing_make_manifest_with_caps("row3.multi", &["host:compute:time", "host:fs:read"]),
            vec!["host:compute:time", "host:fs:read"],
        ),
        // Row 4: empty-caps, multi-module (3 modules, no caps)
        (
            "empty.caps.multi",
            manifest_multi_module("row4.multimod", 3),
            vec![],
        ),
        // Row 5: multi-cap + cap-OVERLAP with the sibling already installed
        (
            "multi.cap.overlap.sibling",
            testing_make_manifest_with_caps("row5.overlap", &["host:compute:time"]),
            vec!["host:compute:time"],
        ),
    ];

    for (label, manifest, expected_caps) in rows {
        let cid = testing_compute_manifest_cid(&manifest);

        // (a) install with matching CID
        let installed = engine
            .install_module(manifest.clone(), cid)
            .unwrap_or_else(|e| panic!("[{label}] install must succeed: {e}"));
        assert_eq!(installed, cid, "[{label}] returned CID matches");

        // (b) is_module_installed reports true
        assert!(
            engine.is_module_installed(&cid),
            "[{label}] post-install, is_module_installed must report true"
        );

        // (c) caps propagated into the active set
        let active = engine.active_module_capabilities();
        for cap in &expected_caps {
            assert!(
                active.contains(*cap),
                "[{label}] post-install, active caps must contain {cap}; got {active:?}"
            );
        }

        // (d) uninstall succeeds
        engine
            .uninstall_module(cid)
            .unwrap_or_else(|e| panic!("[{label}] uninstall must succeed: {e}"));

        // (e) is_module_installed reports false
        assert!(
            !engine.is_module_installed(&cid),
            "[{label}] post-uninstall, is_module_installed must report false"
        );

        // (f) caps retracted UNLESS the sibling manifest still requires them.
        let active_after = engine.active_module_capabilities();
        for cap in &expected_caps {
            // Row 5 overlaps with sibling on host:compute:time — cap survives.
            let sibling_still_requires = sibling
                .modules
                .iter()
                .any(|m| m.requires.iter().any(|r| r == cap));
            if sibling_still_requires {
                assert!(
                    active_after.contains(*cap),
                    "[{label}] cap-overlap rule: sibling still requires {cap}; \
                     it MUST survive uninstall; got {active_after:?}"
                );
            } else {
                assert!(
                    !active_after.contains(*cap),
                    "[{label}] post-uninstall, cap {cap} must be retracted; got {active_after:?}"
                );
            }
        }
    }

    // Sanity: sibling manifest is still installed and its cap is still active.
    assert!(engine.is_module_installed(&sibling_cid));
    assert!(
        engine
            .active_module_capabilities()
            .contains("host:compute:time")
    );

    // Cleanup: uninstall sibling, cap retracts entirely.
    engine.uninstall_module(sibling_cid).unwrap();
    assert!(!engine.is_module_installed(&sibling_cid));
    assert!(
        !engine
            .active_module_capabilities()
            .contains("host:compute:time")
    );
}
