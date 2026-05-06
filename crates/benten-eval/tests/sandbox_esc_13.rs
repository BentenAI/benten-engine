//! ESC-13 (Trap during fuel-meter callback / Store-poison) closure pins
//! (G17-A1 wave-5b; D-E + r1-wsa-1 BLOCKER).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 + §8 ESC-7+ESC-13 +
//! r1-revision-triage D-E):
//!
//! - `tests/esc_13_trap_during_fuel_meter_callback_store_poison_observable` — D-E + r1-wsa-1 BLOCKER
//! - `tests/esc_13_trap_during_fuel_meter_callback_does_not_poison_store` — D-E
//!
//! ## ESC-13 closure shape
//!
//! ESC-13 is the "trap during fuel-meter callback" escape vector: if a
//! host-side fuel-meter callback (registered with wasmtime to interpose
//! on fuel exhaustion) panics or traps, the wasmtime `Store` enters
//! a poisoned state.
//!
//! G17-A1 wave-5b ships:
//!
//! 1. The [`benten_eval::sandbox::EscDefenseState::fuel_meter_callback_trapped`]
//!    flag.
//! 2. The [`benten_eval::sandbox::run_esc13_check`] defense entry
//!    point that fires the typed `SandboxError::EscapeAttempt` with
//!    [`benten_eval::sandbox::EscVector::Esc13StorePoison`].
//! 3. The
//!    [`benten_eval::testing::testing_simulate_fuel_meter_callback_trap`]
//!    simulation helper that drives the trapped-callback state.
//!
//! ## Recovery-path pin
//!
//! The runtime arm pairs ESC-13 detection with the per-call `Store`
//! lifecycle (D3-RESOLVED): a Store flagged poisoned is dropped after
//! the typed error fires, so the next SANDBOX call gets a fresh Store.
//! G17-A1 wave-5b asserts the recovery-path SHAPE at the state level:
//! a fresh `EscDefenseState::new()` does NOT carry the poisoned flag,
//! so subsequent simulated calls do not falsely fire ESC-13.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::SandboxError;
use benten_eval::sandbox::{EscDefenseState, EscVector, run_esc13_check};
use benten_eval::testing::testing_simulate_fuel_meter_callback_trap;

#[test]
fn esc_13_trap_during_fuel_meter_callback_store_poison_observable() {
    // r1-wsa-1 BLOCKER pin. The simulation helper sets
    // `fuel_meter_callback_trapped = true`; the defense fires the
    // typed ESC-13 attribution.
    let mut state = EscDefenseState::new();
    testing_simulate_fuel_meter_callback_trap(&mut state);

    let err = run_esc13_check(&state).expect_err("ESC-13 attack must surface as Err");

    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc13StorePoison,
                ..
            }
        ),
        "ESC-13 attack MUST surface as EscapeAttempt(Esc13StorePoison); got: {err:?}"
    );

    // ESC-13 attribution surfaces as `E_SANDBOX_ESCAPE_ATTEMPT`, not
    // a generic Internal / panic.
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::SandboxEscapeAttempt,
        "ESC-13 typed error routes to E_SANDBOX_ESCAPE_ATTEMPT, not a generic variant"
    );
}

#[test]
fn esc_13_trap_during_fuel_meter_callback_does_not_poison_store() {
    // D-E pim-2 LOAD-BEARING recovery-path pin.
    //
    // Step 1 — trigger ESC-13 (as in the previous pin):
    let mut state_attack = EscDefenseState::new();
    testing_simulate_fuel_meter_callback_trap(&mut state_attack);
    let _ = run_esc13_check(&state_attack); // expected ESC-13 error

    // Step 2 — a FRESH state (analogue of "next SANDBOX call gets a
    // fresh Store" per D3-RESOLVED per-call lifecycle) does NOT carry
    // the poisoned flag forward.
    let state_benign = EscDefenseState::new();
    assert!(
        run_esc13_check(&state_benign).is_ok(),
        "ESC-13 recovery: a fresh state (analogous to the next SANDBOX \
         call's fresh Store per D3-RESOLVED) MUST NOT carry the poisoned \
         flag forward; a regression that persists fuel_meter_callback_trapped \
         across calls would silently fire ESC-13 on benign subsequent calls"
    );
}

#[test]
fn esc_13_state_pin_canonical_field_name() {
    // Architectural-shape pin per r4-r1-wsa-5: the field name
    // `fuel_meter_callback_trapped` is canonical (matches the
    // narrative in `crates/benten-eval/src/sandbox/escape_defenses.rs`).
    // A rename-without-narrative-update would fail this pin.
    let mut state = EscDefenseState::new();
    state.fuel_meter_callback_trapped = true;
    assert!(state.fuel_meter_callback_trapped);
}
