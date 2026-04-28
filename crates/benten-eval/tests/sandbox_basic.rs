//! Phase 2b R3-B — SANDBOX core unit tests (G7-A).
//!
//! **cr-g7a-mr-1 fix-pass:** 2 of 4 tests FLIPPED from `#[ignore]`
//! `todo!()` to live assertions against the G7-A-landed surface.
//! The remaining 2 (`sandbox_end_to_end`, `sandbox_no_state_persists_across_calls`)
//! need G7-C engine integration to fire — markers re-pointed to PR #33.
//!
//! Pin sources: plan §3 G7-A, wsa-15 (rename), wsa-20 (Engine singleton +
//! Module cache), D3-RESOLVED (per-call instance lifecycle), D22-precondition.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::instance::{module_cache_size, module_for_bytes, shared_engine};
use std::sync::Arc;

#[test]
#[ignore = "Phase 2b G7-C pending — full engine.sandbox_call surface lands at G7-C engine integration (PR #33). G7-A ships the per-call wasmtime lifecycle building blocks but not the public Engine method."]
fn sandbox_end_to_end() {
    // Plan §3 G7-A must-pass test #1 — minimal echo module, default caps,
    // returns the input verbatim through the SANDBOX primitive executor.
    todo!("G7-C PR #33 — wire echo fixture + sandbox_call public surface");
}

#[test]
#[ignore = "Phase 2b G7-C pending — wasm fixture exercising module-global memory mutation requires the Store + Instance per-call lifecycle that G7-C wires (G7-A scaffold's execute() returns Ok with empty output without invoking the module). Tracked at G7-C PR #33."]
fn sandbox_no_state_persists_across_calls() {
    // wsa-15 — pin the PROPERTY (no cross-call retention) not the
    // implementation. Test fixture writes to module-global memory in
    // call 1; call 2 of the same module observes the global at its
    // initial value. Needs G7-C Store+Instance dispatch.
    todo!("G7-C PR #33 — assert module global memory is reset between calls");
}

#[test]
fn sandbox_engine_singleton_lifetime() {
    // wsa-20 + D3-RESOLVED — `wasmtime::Engine` constructed ONCE per
    // benten Engine open (not per primitive call). White-box test:
    // `benten_eval::sandbox::instance::shared_engine()` returns the
    // same `&'static Engine` reference on every call within a benten
    // Engine's lifetime.
    let a = shared_engine();
    let b = shared_engine();
    assert!(
        std::ptr::eq(a, b),
        "wsa-20 — shared_engine MUST return the same singleton reference"
    );
}

#[test]
fn sandbox_module_cache_avoids_recompilation_on_repeated_call() {
    // wsa-20 — `wasmtime::Module` is content-CID-cached. The cold-start
    // budget (D22 ≤2ms p95 Linux x86_64) is unmeetable if Module
    // recompiles per call.
    let bytes = wat::parse_str("(module)").unwrap();
    let initial_size = module_cache_size();
    let m1 = module_for_bytes(&bytes).unwrap();
    let m2 = module_for_bytes(&bytes).unwrap();
    // Arc pointer equality — second call returns the cached entry.
    assert!(
        Arc::ptr_eq(&m1, &m2),
        "wsa-20 — Module cache MUST reuse the compiled artifact"
    );
    // Cache must have grown by at most 1 entry (the new fixture if
    // not previously cached).
    let after_size = module_cache_size();
    assert!(
        after_size <= initial_size + 1,
        "module cache must contain at most one new entry per CID"
    );
}
