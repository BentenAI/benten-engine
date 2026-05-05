//! R3-E RED-PHASE pins for G19-C1 testing_advance_wait_clock napi binding
//! (wave-7 parallel; §7.1.4 + r6-napi-2 closure).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C1 must-pass column):
//!
//! - `tests/testing_advance_wait_clock_napi_binding_present` — §7.1.4
//!
//! ## What G19-C1 establishes (§7.1.4)
//!
//! The Phase-2b state per r6-napi-2: engine has
//! `testing_set_iteration_budget` pattern but no wallclock-advance hook on
//! the TS surface. G19-C1 adds `bindings/napi/src/wait.rs::testingAdvanceWaitClock`
//! as a test-only napi method, gated behind the `test-helpers` Cargo
//! feature so the production cdylib does NOT compile it (the cfg-gating
//! audit precedent from Phase-2a sec-r6r2-02).
//!
//! ## RED-PHASE discipline
//!
//! The napi method does not yet exist. R5 implementer wires it +
//! drops `#[ignore]`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C1 wave-7 wires testing_advance_wait_clock napi method (test-helpers gated)"]
fn testing_advance_wait_clock_napi_binding_present() {
    // §7.1.4 pin per r6-napi-2 closure. G19-C1 implementer wires this:
    //
    //   // Verify the napi binding compiles + is reachable through the
    //   // testing helper surface (cfg-gated to test-helpers feature).
    //   #[cfg(any(test, feature = "test-helpers"))]
    //   {
    //       use benten_napi::testing::testing_advance_wait_clock;
    //       // Call shape: takes a delta_ms u64; returns Result<()>.
    //       let result = testing_advance_wait_clock(0);
    //       assert!(result.is_ok(),
    //           "testing_advance_wait_clock with zero-delta must succeed");
    //   }
    //
    // Production cdylib does NOT carry this method — defends against
    // the sec-r6r2-02 cfg-gating audit failure shape (test helpers
    // widening production attack surface).
    unimplemented!(
        "G19-C1 wires testing_advance_wait_clock napi binding (test-helpers feature gate)"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-C1 wave-7 — testing_advance_wait_clock advances WAIT TTL clock for resume tests"]
fn testing_advance_wait_clock_advances_engine_wait_clock_for_ttl_expiry_path() {
    // §3.6b end-to-end pin: drives the production-grade entry point +
    // asserts an observable consequence. G19-C1 implementer wires this:
    //
    //   let engine = benten_napi::testing::open_in_memory_engine().unwrap();
    //   // Build a TTL-bearing handler:
    //   let sg = engine.register_subgraph_with_ttl_wait("ttl-test", 60_000).unwrap();
    //   let suspended = engine.call_with_suspension(sg, "main", json!({})).unwrap();
    //
    //   // Advance past TTL:
    //   benten_napi::testing::testing_advance_wait_clock(70_000).unwrap();
    //
    //   // Resume now triggers TTL-expired branch:
    //   let result = engine.resume_with_meta(suspended.envelope, "never-arrives");
    //   assert!(result.is_err());
    //   let err_payload: serde_json::Value =
    //       serde_json::from_str(&result.err().unwrap().message()).unwrap();
    //   assert_eq!(err_payload["code"], "E_WAIT_TIMEOUT");
    //
    // OBSERVABLE consequence: clock-advance triggers the TTL-expiry
    // branch in the production runtime. Sentinel-presence (the function
    // exists) does NOT suffice per §3.6b pim-2.
    unimplemented!("G19-C1 wires testing_advance_wait_clock end-to-end TTL-expiry pin");
}
