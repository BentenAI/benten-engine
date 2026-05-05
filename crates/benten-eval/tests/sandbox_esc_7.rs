//! R3-D RED-PHASE pins for ESC-7 (Fuel-refill via host-fn re-entry)
//! (G17-A1 wave 5b; D-E + r1-wsa-1 BLOCKER).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 + §8 ESC-7+ESC-13 + r1-revision-triage D-E):
//!
//! - `tests/esc_7_fuel_refill_via_host_fn_re_entry_blocked` — D-E + r1-wsa-1 BLOCKER
//! - `tests/esc_07_fuel_refill_via_host_fn_re_entry_traps_typed_error` — D-E
//!
//! ## ESC-7 closure shape (D-E added at R1 revision)
//!
//! ESC-7 is the "fuel-refill via host-fn re-entry" escape vector: a
//! guest that calls a host-fn which, in its dispatch path, re-enters
//! the same SANDBOX `Store` (or any path that calls `Store::add_fuel`
//! during guest execution) effectively bypasses the fuel ceiling.
//!
//! Phase-2b SANDBOX shipped without the ESC-7 defense; r1-wasmtime-sandbox
//! flagged this as r1-wsa-1 BLOCKER. D-E (R1-revision triage) pulls
//! ESC-7 + ESC-13 into G17-A1 wave-5b for honest 16/16 ESC coverage.
//!
//! Defense lives at `crates/benten-eval/src/sandbox/escape_defenses.rs`:
//! a helper + fixture pair that exercises a guest re-entering through
//! a host function and asserts a typed trap fires before fuel refill
//! takes effect.
//!
//! ## Why two distinct pin functions
//!
//! Per pim-2 §3.6b end-to-end test pin requirement:
//!
//! - `..._blocked` asserts the engine-side defense actually runs the
//!   guest fixture and observes an attack denial. (The engine-side
//!   integration: the ESC-7 callback fires DURING re-entry, not after.)
//! - `..._traps_typed_error` asserts the typed-error variant routing
//!   surfaces the attack to the caller (rather than panic / silent
//!   fuel-refill / generic trap). Defends pim-2 — the runtime arm is
//!   wired AND the typed error reaches outer dispatch.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b authors ESC-7 fuel-refill-via-re-entry defense per D-E + r1-wsa-1 BLOCKER"]
fn esc_7_fuel_refill_via_host_fn_re_entry_blocked() {
    // r1-wsa-1 BLOCKER pin. G17-A1 implementer wires this:
    //
    // PRECONDITION — fixture committed:
    //   crates/benten-eval/tests/fixtures/sandbox/esc_07_fuel_refill_via_host_fn_re_entry.wat
    //   (paired .wasm via G17-B build.rs)
    //
    // SHAPE:
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //
    //   // Build a host-fn that, in its dispatch arm, attempts to
    //   // re-enter the guest Store (the attack vector). The §7.3.A.7
    //   // helper SURFACE provides `testing_register_uncounted_host_fn`
    //   // that demonstrates this attack.
    //   let module = load_fixture_wat_or_wasm("esc_07_fuel_refill_via_host_fn_re_entry");
    //   let sandbox = Sandbox::new(/* config with low fuel ceiling */);
    //   let result = sandbox.execute(module);
    //
    //   // Defense observed the re-entry attempt + denied:
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::EscapeAttempt {
    //           vector: benten_eval::EscVector::Esc7FuelRefillViaReEntry,
    //           ..
    //       }
    //   ));
    //
    // OBSERVABLE consequence: a guest that performs the ESC-7 attack
    // pattern (re-entering the Store through a host-fn during fuel
    // exhaustion) is DENIED before the fuel-refill takes effect. The
    // exit-criterion-7 narrative honestly says 16/16 ESC vectors.
    //
    // Defends r1-wsa-1 BLOCKER directly.
    unimplemented!("G17-A1 wires ESC-7 escape_defenses helper + fixture + integration test");
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b routes ESC-7 trap to a typed error (D-E)"]
fn esc_07_fuel_refill_via_host_fn_re_entry_traps_typed_error() {
    // D-E pim-2 LOAD-BEARING pin. G17-A1 implementer wires this:
    //
    //   // Same fixture as above, but assert the OUTER dispatch path
    //   // (caller of `Sandbox::execute`) receives a typed error, not
    //   // a generic Trap or a panic.
    //   let module = load_fixture_wat_or_wasm("esc_07_fuel_refill_via_host_fn_re_entry");
    //   let result = sandbox.execute(module);
    //   let err = result.expect_err("ESC-7 attempt must surface an error");
    //
    //   // Engine-side outer dispatch sees a typed variant that
    //   // distinguishes ESC-7 from a generic fuel-exhaustion trap:
    //   assert!(matches!(
    //       err,
    //       benten_eval::SandboxError::EscapeAttempt { vector, .. }
    //         if vector == benten_eval::EscVector::Esc7FuelRefillViaReEntry
    //   ));
    //   // (Distinguished from the plain `SandboxError::FuelExhausted`
    //   //  variant a benign fuel-overrun would surface.)
    //
    // OBSERVABLE consequence: a regression that re-routes ESC-7 to the
    // generic fuel-exhausted variant (e.g. via a defense that traps
    // but routes through the wrong arm) silently loses ESC vector
    // attribution. This pin fails. Defends pim-2 — typed routing
    // surfaces to caller.
    unimplemented!("G17-A1 wires ESC-7 typed-error routing + fixture-driving assertion");
}
