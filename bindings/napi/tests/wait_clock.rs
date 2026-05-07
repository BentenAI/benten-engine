//! GREEN-PHASE pin for G19-C1 testing_advance_wait_clock napi binding
//! (wave-7 parallel; §7.1.4 + r6-napi-2 closure).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C1 must-pass column):
//!
//! - `tests/testing_advance_wait_clock_napi_binding_present` — §7.1.4
//!
//! ## What G19-C1 establishes (§7.1.4)
//!
//! Per r6-napi-2: the engine carries `testing_set_iteration_budget` but
//! had no wallclock-advance hook on the TS surface. G19-C1 adds
//! `bindings/napi/src/lib.rs::testing.testing_advance_wait_clock` (free
//! function in the rlib-only `testing` module) + a sibling
//! `Engine::testingAdvanceWaitClock` `#[napi]` method gated behind the
//! `test-helpers` Cargo feature so the production cdylib does NOT
//! compile the latter (cfg-gating audit precedent from Phase-2a
//! sec-r6r2-02).
//!
//! Body remains a forward-compatible deterministic no-op until the
//! D12 `MockMonotonicSource` injection plumbing lands; the binding
//! shape is the load-bearing pin (r6-napi-2 named the GAP, not the
//! deterministic-vs-real-clock semantics).

#![allow(clippy::unwrap_used)]

#[cfg(feature = "in-process-test")]
#[test]
fn testing_advance_wait_clock_napi_binding_present() {
    // §7.1.4 pin per r6-napi-2 closure. Free function in the
    // rlib-only `testing` module — the cdylib does NOT carry this
    // binding (sec-r6r2-02 cfg-gating defense-in-depth).
    use benten_napi::testing::testing_advance_wait_clock;
    let result = testing_advance_wait_clock(0);
    assert!(
        result.is_ok(),
        "testing_advance_wait_clock with zero delta MUST succeed; \
         got {result:?}"
    );

    // Non-zero delta also succeeds (forward-compatible no-op until
    // D12 wires the real MockMonotonicSource advance).
    let nonzero = testing_advance_wait_clock(70_000);
    assert!(
        nonzero.is_ok(),
        "testing_advance_wait_clock with nonzero delta MUST succeed \
         under the deterministic no-op shape; got {nonzero:?}"
    );
}
