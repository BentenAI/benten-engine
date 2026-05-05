//! R3-B RED-PHASE pins: durable module-bytes registry CID validation
//! (G14-C wave-4b; D-PHASE-3-12 + plan §3 G14-C).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-C):
//!
//! - `tests/register_module_bytes_validates_cid_per_d_phase_3_12_resolution` — D-PHASE-3-12 (unit)
//! - `tests/module_bytes_durable_across_engine_restart` — plan §3 G14-C (integration)
//!
//! ## Architectural intent
//!
//! Compromise #17 (durable module-bytes registry) closes at G14-C.
//! Per D-PHASE-3-12 the implementation MUST:
//! 1. Validate the supplied CID against the actual content hash on
//!    `register_module_bytes`; mismatch → typed error.
//! 2. Persist module-bytes durably so they survive engine restart.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-C
//! implementer un-ignores. Per §3.6b pim-2 these tests must drive the
//! production `Engine::register_module_bytes` entry point and assert
//! observable consequences: CID-mismatch rejects; restart preserves.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-C — D-PHASE-3-12 — register_module_bytes CID validation"]
fn register_module_bytes_validates_cid_per_d_phase_3_12_resolution() {
    // D-PHASE-3-12 pin. G14-C implementer wires this:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let bytes = include_bytes!("../fixtures/module.wasm");
    //   let actual_cid = benten_core::cid::compute(bytes);
    //
    //   // Correct CID + bytes: succeeds.
    //   engine.register_module_bytes(&actual_cid, bytes).unwrap();
    //
    //   // Wrong CID for these bytes: typed error.
    //   let wrong_cid = benten_core::cid::compute(b"different bytes");
    //   let err = engine.register_module_bytes(&wrong_cid, bytes).unwrap_err();
    //   assert!(matches!(err, benten_engine::EngineError::ModuleBytesCidMismatch { .. }));
    //
    // OBSERVABLE consequence: a caller passing mismatched (CID,
    // bytes) gets a typed `ModuleBytesCidMismatch` error. Defends
    // against the silent-corruption attack class where a peer writes
    // module-bytes under a CID different from their hash.
    unimplemented!("G14-C wires register_module_bytes CID/content validation per D-PHASE-3-12");
}

#[test]
#[ignore = "RED-PHASE: G14-C — plan §3 G14-C — module-bytes durable across restart"]
fn module_bytes_durable_across_engine_restart() {
    // plan §3 G14-C pin. Compromise #17 closure: registered module-
    // bytes survive engine restart so handlers continue to resolve
    // their `requires` SANDBOX manifests on the second run.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let bytes = include_bytes!("../fixtures/module.wasm");
    //   let cid = benten_core::cid::compute(bytes);
    //
    //   {
    //       let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //       engine.register_module_bytes(&cid, bytes).unwrap();
    //       assert!(engine.fetch_module_bytes(&cid).is_some());
    //       // engine drops; durable-store flush
    //   }
    //
    //   // Re-open at same path; the bytes MUST persist:
    //   {
    //       let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //       let fetched = engine.fetch_module_bytes(&cid).unwrap();
    //       assert_eq!(fetched.as_slice(), bytes);
    //   }
    //
    // OBSERVABLE consequence: re-opening the engine at the same store
    // path resurrects the registered module bytes. Compromise #17
    // closes when this test goes green.
    unimplemented!(
        "G14-C wires module-bytes durability across Engine::open() restart per Compromise #17"
    );
}
