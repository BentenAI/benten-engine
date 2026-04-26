//! Phase 2b R4-FP B-3 — G10-B `uninstall_module` cap-retraction
//! integration test.
//!
//! TDD red-phase. Pin sources:
//!   - `r2-test-landscape.md` §1.8
//!     `module_uninstall_respects_capability_retraction`.
//!   - Plan §3.2 G10-B — uninstall releases caps.
//!   - `r1-security-auditor.json` D9 — manifest `requires` block is
//!     authoritative for cap declarations; ghost-cap drift across
//!     install/uninstall cycles would defeat the policy hook.
//!
//! This test is the cross-crate INTEGRATION variant of the
//! `module_uninstall_respects_capability_retraction` unit pin in
//! `tests/module_uninstall.rs`. The unit test asserts the engine's
//! introspection accessor; this integration test asserts the actual
//! capability-policy behavior (a subsequent operation that requires the
//! retracted cap is DENIED).
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

use benten_engine::Engine;

// R5 surfaces consumed:
//   benten_engine::Engine::install_module
//   benten_engine::Engine::uninstall_module
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::testing::testing_make_minimal_manifest
//   benten_engine::testing::testing_compute_manifest_cid
//   (capability-policy boundary: a subsequent action requiring the
//    retracted cap MUST be denied; exact denial path lives in the
//    G10-B impl + capability-policy hook).

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
#[ignore = "Phase 2b G10-B pending — uninstall retracts manifest-scoped caps end-to-end"]
fn module_uninstall_releases_capabilities_end_to_end() {
    // R5 G10-B + capability-policy wires:
    //   1. Install manifest M that requires `host:compute:time`.
    //   2. Register a handler whose body calls a host fn that requires
    //      `host:compute:time`. The handler should run successfully
    //      while M is installed.
    //   3. Uninstall M.
    //   4. ASSERT a subsequent invocation of the same handler is
    //      DENIED -- the policy hook MUST see the cap as no longer
    //      declared (assuming no other manifest declares it; if another
    //      manifest does, the cap survives -- see the multi-manifest
    //      cap-overlap row in module_install_uninstall_round_trip.rs
    //      row 5).
    //
    // The denial code is the standard capability-policy denial
    // (E_CAP_DENIED or whichever code G10-B + the policy hook agree on).
    let (_dir, mut engine) = fresh_engine();
    let _ = &mut engine;
    todo!(
        "R5 G10-B — assert post-uninstall handler invocation denied because \
         manifest-scoped cap was retracted from active set"
    );
}

#[test]
#[ignore = "Phase 2b G10-B pending — sibling manifest preserves overlapping cap"]
fn module_uninstall_does_not_retract_cap_required_by_sibling_manifest() {
    // The cap-overlap edge case: when two manifests M and N both
    // require `host:compute:time`, uninstalling M MUST NOT retract
    // the cap declaration that N still requires.
    //
    // R5 G10-B wires:
    //   1. Install M (requires `host:compute:time`).
    //   2. Install N (also requires `host:compute:time`).
    //   3. Uninstall M.
    //   4. ASSERT a handler that depends on `host:compute:time` STILL
    //      runs successfully (because N still declares the cap).
    //   5. Uninstall N.
    //   6. NOW the handler invocation MUST be denied.
    let (_dir, mut engine) = fresh_engine();
    let _ = &mut engine;
    todo!(
        "R5 G10-B — assert cap-overlap rule: sibling manifest preserves \
         shared cap through partial uninstall"
    );
}
