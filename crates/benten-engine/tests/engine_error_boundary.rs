//! R3-A RED-PHASE pins for engine error-boundary erasure
//! (G13-B wave 2; D-PHASE-3-1a / D-B / arch-r1-1 BLOCKER).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column + plan §6 D-B):
//!
//! - `tests/engine_error_boundary_erases_backend_specific_error_to_dyn_std_error` — D-PHASE-3-1a / arch-r1-1 BLOCKER
//! - `tests/engine_error_carries_typed_backend_error_per_d_phase_3_1a` — D-B
//!
//! ## What G13-B / D-PHASE-3-1a establishes
//!
//! Per arch-r1-1 BLOCKER + D-B resolution: the engine PUBLIC boundary
//! erases backend-specific errors to `Box<dyn std::error::Error + Send +
//! Sync>` (preserves API stability + napi error-mapping). INSIDE the
//! `EngineGeneric<B>` impl, errors stay typed via `B::Error`.
//!
//! The two test pins assert BOTH directions of that contract:
//!
//! 1. **Public boundary erases:** the public `EngineError` enum carries
//!    a `Box<dyn std::error::Error + Send + Sync>` variant that wraps
//!    `B::Error` losslessly via `Error::source()` chain.
//!
//! 2. **Source chain preserves typed:** downcasting via
//!    `err.source().and_then(|e| e.downcast_ref::<RedbBackendError>())`
//!    recovers the typed error for diagnostics; the typed error is NOT
//!    irrevocably lost at the boundary.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B + D-PHASE-3-1a BLOCKER — public boundary erasure"]
fn engine_error_boundary_erases_backend_specific_error_to_dyn_std_error() {
    // D-PHASE-3-1a / arch-r1-1 BLOCKER pin. G13-B implementer wires this:
    //
    //   // Trigger a backend error (e.g. open a non-existent path or
    //   // induce a redb commit failure):
    //   let result = benten_engine::Engine::open("/dev/null/no-such-dir");
    //   let err: benten_engine::EngineError = result.unwrap_err();
    //
    //   // The public type's variant carrying the backend error MUST
    //   // expose it as Box<dyn std::error::Error + Send + Sync>:
    //   match &err {
    //       benten_engine::EngineError::Backend(boxed) => {
    //           // Compile-time pin: the Box is the dyn-erased shape.
    //           let _: &(dyn std::error::Error + Send + Sync) = &**boxed;
    //       }
    //       other => panic!("expected EngineError::Backend, got {:?}", other),
    //   }
    //
    // OBSERVABLE consequence: callers (napi binding; in-process Rust
    // consumers) receive a stable type even when B varies. Defends
    // against the API-instability failure mode where switching from
    // RedbBackend to BrowserBackend would break napi error-mapping
    // because the typed error variant changed shape.
    unimplemented!("G13-B wires EngineError::Backend dyn-erasure pin");
}

#[test]
#[ignore = "RED-PHASE: G13-B + D-B — typed backend error preserved in source chain"]
fn engine_error_carries_typed_backend_error_per_d_phase_3_1a() {
    // D-B resolution pin. The dyn-erased boundary MUST NOT lose the
    // typed error: `Error::source()` chain recovers the original
    // backend error for diagnostics + structured logging.
    //
    // G13-B implementer wires this:
    //   let result = benten_engine::Engine::open("/dev/null/no-such-dir");
    //   let err = result.unwrap_err();
    //   match &err {
    //       benten_engine::EngineError::Backend(boxed) => {
    //           let source: &dyn std::error::Error = &**boxed;
    //           // Walk the source chain to find the typed backend error:
    //           let typed = source.downcast_ref::<benten_graph::RedbBackendError>();
    //           // OR if the typed error is wrapped one level deeper:
    //           let chained = source.source().and_then(|s|
    //               s.downcast_ref::<benten_graph::RedbBackendError>());
    //           assert!(typed.is_some() || chained.is_some(),
    //               "typed RedbBackendError must be recoverable from the dyn-erased \
    //                EngineError::Backend variant via source-chain downcast");
    //       }
    //       _ => unreachable!(),
    //   }
    //
    // OBSERVABLE consequence: structured logging that wants to emit
    // backend-specific telemetry (e.g. "redb file-too-large vs redb
    // path-not-found vs redb corruption") can downcast at the napi
    // boundary or in test assertions. The dyn-erasure is for API
    // stability, not for information loss.
    unimplemented!("G13-B wires source-chain downcast assertion for typed backend error recovery");
}
