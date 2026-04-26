//! Phase 2b R4-FP B-3 — G10-B install/uninstall round-trip 5-row fixture
//! matrix. Closes plan §1 exit criterion #4.
//!
//! TDD red-phase. Pin sources:
//!   - `.addl/phase-2b/00-implementation-plan.md` §1 exit criterion #4
//!     (install + uninstall round-trip with `requires`-cap propagation).
//!   - `r2-test-landscape.md` §2.4
//!     `module_install_round_trip_5_row_fixture_matrix`.
//!
//! The 5-row matrix exercises the {install, uninstall} cross-product
//! against a representative spread of manifest shapes (no caps, single
//! cap, multi-cap, multi-module, cap-overlap with sibling manifest)
//! so that no single shape monopolizes the pin coverage.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

use benten_engine::Engine;

// R5 surfaces consumed (same as module_install.rs unit tests):
//   benten_engine::Engine::install_module
//   benten_engine::Engine::uninstall_module
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::testing::testing_compute_manifest_cid

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
#[ignore = "Phase 2b G10-B pending — exit criterion #4 5-row install/uninstall matrix"]
fn module_install_uninstall_round_trip_5_row_fixture_matrix() {
    // Plan §1 exit criterion #4 — full install/uninstall round-trip
    // across 5 representative manifest shapes. EACH row MUST:
    //   (a) install successfully with the matching CID,
    //   (b) report `is_module_installed(cid) == true` post-install,
    //   (c) propagate `requires`-cap declarations into the engine's
    //       active capability set,
    //   (d) uninstall successfully,
    //   (e) report `is_module_installed(cid) == false` post-uninstall,
    //   (f) retract the `requires`-cap declarations from the active set
    //       (subject to the multi-manifest cap-overlap rule -- see row 5).
    //
    // The 5 rows:
    //   Row 1: empty-caps, single-module
    //   Row 2: single-cap, single-module ("host:compute:time")
    //   Row 3: multi-cap, single-module ("host:compute:time" + "host:fs:read")
    //   Row 4: empty-caps, multi-module (3 modules, no caps)
    //   Row 5: multi-cap + cap-OVERLAP with a sibling manifest already
    //          installed -- uninstalling row 5 must NOT retract caps that
    //          a sibling manifest still requires.
    //
    // R5 G10-B wires the row table + driver loop. Suggested table:
    //   let rows: [(&str, Vec<&str>, usize); 5] = [
    //     ("empty.caps.single", vec![], 1),
    //     ("single.cap.single", vec!["host:compute:time"], 1),
    //     ("multi.cap.single", vec!["host:compute:time", "host:fs:read"], 1),
    //     ("empty.caps.multi", vec![], 3),
    //     ("multi.cap.overlap.sibling", vec!["host:compute:time"], 1),
    //   ];
    //
    // For row 5, install a separately-named sibling manifest that ALSO
    // requires "host:compute:time" BEFORE the row-5 install/uninstall;
    // assert the sibling's cap declaration survives the row-5 uninstall.
    let (_dir, mut engine) = fresh_engine();
    let _ = &mut engine; // suppress unused warning until R5 wires the driver
    todo!(
        "R5 G10-B — drive 5-row install/uninstall fixture matrix per exit \
         criterion #4 (install + cap-propagation + uninstall + cap-retraction)"
    );
}
