//! ESC-7 (Fuel-refill via host-fn re-entry) SHAPE pins
//! (G17-A1 wave-5b — SCAFFOLDING ONLY; r1-wsa-1 BLOCKER closure
//! RECALLED to wave-5c per mini-review at
//! `.addl/phase-3/r5-w5b-g17-a1-mini-review.json`).
//!
//! These tests audit the [`benten_eval::sandbox::run_esc7_check`]
//! helper logic against synthetic [`benten_eval::sandbox::EscDefenseState`]
//! values — they are NOT end-to-end load-bearing closure for r1-wsa-1
//! BLOCKER. The end-to-end pin (driving a real SANDBOX dispatch +
//! observing the runtime arm fire from a production trampoline) lands
//! at wave-5c ESC runtime-arm wiring per phase-3-backlog §6.1-followup.
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 + §8 ESC-7+ESC-13 +
//! r1-revision-triage D-E):
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
//! G17-A1 wave-5b ships:
//!
//! 1. The [`benten_eval::sandbox::EscDefenseState`] state shape that
//!    carries the re-entry counter + guest_active flag.
//! 2. The [`benten_eval::sandbox::run_esc7_check`] defense entry point
//!    that fires the typed `SandboxError::EscapeAttempt` variant with
//!    [`benten_eval::sandbox::EscVector::Esc7FuelRefillViaReEntry`]
//!    discriminator.
//! 3. The [`benten_eval::testing::testing_register_uncounted_host_fn`]
//!    simulation helper that drives the attack-pattern state shape.
//!
//! The full fixture-driven runtime test (driving an `.wat` fixture
//! end-to-end through `Sandbox::execute`) un-ignores at G20-A1 wave-8a
//! per phase-3-backlog §7.3.A.7. G17-A1 wave-5b ships the SURFACE +
//! the simulation-level closure pin.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::SandboxError;
use benten_eval::sandbox::{EscDefenseState, EscVector, run_esc7_check};
use benten_eval::testing::testing_register_uncounted_host_fn;

#[test]
fn esc_7_fuel_refill_via_host_fn_re_entry_blocked() {
    // r1-wsa-1 BLOCKER pin — the defense fires when the simulated
    // attack-pattern state is set up by
    // `testing_register_uncounted_host_fn` (which sets
    // `guest_active = true` + bumps `re_entry_count`).
    let mut state = EscDefenseState::new();
    testing_register_uncounted_host_fn(&mut state);

    let result = run_esc7_check(&state);

    let err = result.expect_err("ESC-7 attack pattern MUST trip the defense");
    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc7FuelRefillViaReEntry,
                ..
            }
        ),
        "ESC-7 attack MUST surface as EscapeAttempt(Esc7FuelRefillViaReEntry); got: {err:?}"
    );

    // Defends r1-wsa-1 BLOCKER directly: the typed error preserves
    // ESC-vector attribution rather than collapsing into a generic
    // fuel-exhaustion variant.
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::SandboxEscapeAttempt,
        "ESC-7 typed error routes to E_SANDBOX_ESCAPE_ATTEMPT, not E_SANDBOX_FUEL_EXHAUSTED"
    );
}

#[test]
fn esc_07_fuel_refill_via_host_fn_re_entry_traps_typed_error() {
    // D-E pim-2 LOAD-BEARING typed-routing pin. The OUTER dispatch
    // path observes a typed error (not a panic / generic trap /
    // collapsed FuelExhausted).
    let mut state = EscDefenseState::new();
    testing_register_uncounted_host_fn(&mut state);

    let err = run_esc7_check(&state).expect_err("ESC-7 attack must surface as Err");

    // The discriminating EscVector survives end-to-end. A regression
    // that re-routes ESC-7 to FuelExhausted (e.g. via a defense that
    // fires but maps through the wrong arm) silently loses ESC vector
    // attribution; this pin fails such a regression.
    let vector = match &err {
        SandboxError::EscapeAttempt { vector, .. } => *vector,
        other => panic!("expected EscapeAttempt, got: {other:?}"),
    };
    assert_eq!(
        vector,
        EscVector::Esc7FuelRefillViaReEntry,
        "ESC-7 vector attribution survives the typed-error routing per pim-2 §3.6b"
    );
    assert_eq!(vector.as_str(), "ESC-7");
}

#[test]
fn esc_07_post_guest_exit_re_entry_is_silent() {
    // Recovery-path pin: re-entry AFTER `guest_active = false`
    // (legitimate cleanup) does NOT trip the defense. A regression
    // that fires on EVERY non-zero re_entry_count would falsely
    // accuse legitimate cleanup paths of an attack pattern.
    let mut state = EscDefenseState::new();
    state.enter_guest();
    state.exit_guest();
    state.re_entry_count = 1;

    assert!(
        run_esc7_check(&state).is_ok(),
        "ESC-7 defense must be silent on re-entry after guest_active = false \
         (legitimate cleanup path); a regression that fires on this state \
         would block legitimate cleanup paths"
    );
}
