//! R3-D RED-PHASE pins for ESC-13 (Trap during fuel-meter callback /
//! Store-poison) (G17-A1 wave 5b; D-E + r1-wsa-1 BLOCKER).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 + §8 ESC-7+ESC-13 + r1-revision-triage D-E):
//!
//! - `tests/esc_13_trap_during_fuel_meter_callback_store_poison_observable` — D-E + r1-wsa-1 BLOCKER
//! - `tests/esc_13_trap_during_fuel_meter_callback_does_not_poison_store` — D-E
//!
//! ## ESC-13 closure shape (D-E added at R1 revision)
//!
//! ESC-13 is the "trap during fuel-meter callback" escape vector: if a
//! host-side fuel-meter callback (registered with wasmtime to interpose
//! on fuel exhaustion) panics or traps, the wasmtime `Store` enters
//! a poisoned state. A subsequent SANDBOX execution against a poisoned
//! Store produces undefined behavior (in practice: lingering fuel
//! state, partial guest stack, leaked memory).
//!
//! Phase-2b SANDBOX shipped without the ESC-13 defense; r1-wasmtime-sandbox
//! flagged this as r1-wsa-1 BLOCKER. D-E (R1-revision triage) pulls
//! ESC-13 into G17-A1 wave-5b alongside ESC-7 for honest 16/16 ESC
//! coverage.
//!
//! Defense lives at `crates/benten-eval/src/sandbox/escape_defenses.rs`:
//! a panic-catcher around the fuel-meter callback that converts a
//! callback trap into a typed `Esc13StorePoison` error AND ensures the
//! Store is rebuilt fresh for the next execution (so poison does not
//! leak across SANDBOX invocations).
//!
//! ## Why two distinct pin functions
//!
//! - `..._store_poison_observable` exercises the ATTACK PATH: a
//!   fixture that arranges for the fuel-meter callback to trap; the
//!   ESC-13 attribution is observable in the typed error.
//! - `..._does_not_poison_store` exercises the RECOVERY PATH: after
//!   ESC-13 fires, a *subsequent* SANDBOX execution succeeds (Store
//!   is rebuilt). Defends against "defense fires once but Store stays
//!   poisoned" failure shape.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b authors ESC-13 fuel-meter-callback Store-poison defense per D-E + r1-wsa-1 BLOCKER"]
fn esc_13_trap_during_fuel_meter_callback_store_poison_observable() {
    // r1-wsa-1 BLOCKER pin. G17-A1 implementer wires this:
    //
    // PRECONDITION — fixture committed:
    //   crates/benten-eval/tests/fixtures/sandbox/esc_13_trap_during_fuel_meter_callback.wat
    //   (paired .wasm via G17-B build.rs)
    //
    // SHAPE:
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //
    //   // Build a SANDBOX whose host-side fuel-meter callback is
    //   // arranged (via §7.3.A.7 helper SURFACE) to trap when invoked.
    //   let module = load_fixture_wat_or_wasm("esc_13_trap_during_fuel_meter_callback");
    //   let sandbox = Sandbox::new_with_traping_fuel_meter(/* config */);
    //   let result = sandbox.execute(module);
    //
    //   // ESC-13 fires + is attributed:
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::EscapeAttempt {
    //           vector: benten_eval::EscVector::Esc13StorePoison,
    //           ..
    //       }
    //   ));
    //
    // OBSERVABLE consequence: a fuel-meter callback trap surfaces as a
    // typed ESC-13 attribution, not a panic and not a generic
    // `SandboxError::Internal`. Defends r1-wsa-1 BLOCKER directly.
    unimplemented!(
        "G17-A1 wires ESC-13 panic-catcher around fuel-meter callback + integration test"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b ESC-13 recovery path — Store NOT poisoned cross-invocation (D-E)"]
fn esc_13_trap_during_fuel_meter_callback_does_not_poison_store() {
    // D-E pim-2 LOAD-BEARING recovery-path pin. G17-A1 implementer:
    //
    //   // Step 1 — trigger ESC-13 (as in the previous pin):
    //   let module_attack = load_fixture_wat_or_wasm("esc_13_trap_during_fuel_meter_callback");
    //   let _ = sandbox.execute(module_attack); // expected ESC-13 error
    //
    //   // Step 2 — execute a benign module on the SAME engine:
    //   let module_benign = load_fixture_wat_or_wasm("benign_kv_read");
    //   let result = sandbox.execute(module_benign);
    //
    //   // The benign module succeeds — Store was rebuilt clean:
    //   assert!(result.is_ok(),
    //       "ESC-13 recovery path must rebuild Store cleanly per D-E + pim-2; \
    //        a poisoned Store leaking into the next execution would silently \
    //        produce undefined behavior + fail this assertion");
    //
    // OBSERVABLE consequence: ESC-13 attribution + Store recovery are
    // BOTH observable. A regression that fires the typed error but
    // forgets to rebuild the Store passes the previous pin and fails
    // this one. Distinct end-to-end observable consequence per pim-2.
    unimplemented!(
        "G17-A1 wires ESC-13 Store-rebuild recovery path + sequential-execution assertion"
    );
}
