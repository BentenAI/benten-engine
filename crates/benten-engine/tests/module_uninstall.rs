//! Phase 2b R4-FP B-3 — G10-B `Engine::uninstall_module(cid)` happy-path
//! + idempotence tests.
//!
//! TDD red-phase. Pin sources:
//!   - `r2-test-landscape.md` §1.8 —
//!     `module_uninstall_respects_capability_retraction`,
//!     `module_install_with_migrations_rejects_on_wasm32`.
//!   - `.addl/phase-2b/00-implementation-plan.md` §3.2 G10-B —
//!     uninstall releases caps; subscriptions / IVM views referencing
//!     modules from this manifest get cleaned up.
//!   - `r1-wasm-target.json` — G10-B is in-memory-only on wasm32 in
//!     Phase 2b; persistence (IndexedDB) defers to Phase 3.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::Engine::install_module
//   benten_engine::Engine::uninstall_module(cid: Cid) -> Result<(), EngineError>
//   benten_engine::Engine::is_module_installed(cid: &Cid) -> bool
//   benten_engine::testing::testing_compute_manifest_cid
//   benten_engine::testing::testing_make_minimal_manifest

#[test]
#[ignore = "Phase 2b G10-B pending — uninstall happy path"]
fn uninstall_module_happy_path_after_install() {
    // R5 G10-B wires:
    //   1. Install manifest M with cid_m.
    //   2. ASSERT engine.is_module_installed(&cid_m) == true.
    //   3. engine.uninstall_module(cid_m).expect("uninstall must succeed");
    //   4. ASSERT engine.is_module_installed(&cid_m) == false.
    //   5. ASSERT a subsequent register_subgraph referencing the manifest
    //      by name FAILS with E_SANDBOX_MANIFEST_UNKNOWN (the manifest is
    //      no longer resolvable).
    todo!("R5 G10-B — assert uninstall_module removes the manifest from the active set");
}

#[test]
#[ignore = "Phase 2b G10-B pending — uninstall idempotence"]
fn uninstall_module_is_idempotent() {
    // R2 §1.8 — uninstalling an already-uninstalled module MUST be a
    // no-op (return Ok(())). Without idempotence, tear-down loops in
    // operator scripts would have to track install/uninstall state out-
    // of-band, which is error-prone.
    //
    // R5 G10-B wires:
    //   1. Install M; uninstall M (first call -> Ok).
    //   2. uninstall_module(cid_m) AGAIN -> MUST return Ok(())
    //      (NOT Err with "not installed").
    //   3. uninstall_module on a CID that was NEVER installed -> ALSO
    //      Ok(()) (the idempotence boundary is the CID, not the install
    //      history).
    todo!("R5 G10-B — assert uninstall_module is idempotent across repeat + never-installed");
}

#[test]
#[ignore = "Phase 2b G10-B pending — uninstall releases capability declarations"]
fn module_uninstall_respects_capability_retraction() {
    // R2 §1.8 — when a manifest's `requires` block declared caps that
    // are scoped to the manifest's lifetime, uninstall MUST release
    // those declarations from the active capability set. Otherwise,
    // ghost-cap drift accumulates across install/uninstall cycles.
    //
    // R5 G10-B wires:
    //   1. Install manifest M that requires `host:compute:time`.
    //   2. ASSERT the engine's active-cap set lists
    //      `host:compute:time` as a manifest-scoped declaration.
    //   3. Uninstall M.
    //   4. ASSERT the engine's active-cap set NO LONGER lists
    //      `host:compute:time` from the manifest-scoped declaration.
    //   5. (Edge-case) IF another manifest N still requires
    //      `host:compute:time`, the cap MUST remain in the active set
    //      after uninstalling M -- only the M-scoped declaration is
    //      retracted, not the cap entirely.
    todo!("R5 G10-B — assert uninstall releases manifest-scoped cap declarations");
}

#[test]
#[ignore = "Phase 2b G10-B pending — install with migrations rejects on wasm32"]
fn module_install_with_migrations_rejects_on_wasm32() {
    // R2 §1.8 + r1-wasm-target.json — Phase-2b in-memory-only constraint.
    // Manifests that declare migration steps (which would persist across
    // sessions) MUST be rejected on wasm32-unknown-unknown because there
    // is no persistent backing store. The IndexedDB persistence layer
    // defers to Phase 3.
    //
    // R5 G10-B wires:
    //   1. Build manifest M with at least one `migrations: [...]` entry.
    //   2. On wasm32-unknown-unknown ONLY:
    //        engine.install_module(M, cid_of_M)
    //        -> MUST return Err with E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE
    //           (or whichever typed code G10-B coins).
    //   3. The same install on native (redb-backed) MUST SUCCEED.
    //
    // Test gating: this case may need #[cfg(target_arch = "wasm32")] or
    // a runtime feature check; R5 G10-B picks the gating shape.
    todo!("R5 G10-B — reject migrations-bearing install on wasm32-unknown-unknown");
}
