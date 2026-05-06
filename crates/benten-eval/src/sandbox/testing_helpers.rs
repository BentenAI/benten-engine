//! §7.3.A.7 SANDBOX-escape testing helpers — HIGH-risk security
//! surface (Phase-3 G17-A1 wave-5b; phase-3-backlog §7.3.A.7).
//!
//! These helpers exist exclusively to drive the §7.3.A.1 + §7.3.A.7
//! ESC integration tests (G20-A1 wave-8a un-ignores them); they
//! manufacture the dispatch + state shapes that real attack vectors
//! produce so the engine-side defenses can be exercised end-to-end.
//!
//! ## CRITICAL — cfg-gating discipline (Phase-2a sec-r6r2-02 precedent)
//!
//! Every public item in this module MUST be gated behind
//! `cfg(any(test, feature = "test-helpers", feature = "testing"))`.
//! The production cdylib (default features, no `test-helpers` /
//! `testing` enabled) MUST NOT compile or export ANY symbol from this
//! file. The `tests/sandbox_helpers_no_widening.rs` load-bearing
//! security pin (G20-A1 wave-8a) audits this discipline on every CI
//! build. The `feature = "testing"` leg is included because the
//! existing `feature = "testing"` gates the broader `crate::testing`
//! helper surface — the §7.3.A.7 helpers travel together with that
//! family for downstream consumers (the integration-test binaries
//! that exercise both surfaces).
//!
//! Phase-2a `sec-r6r2-02` precedent: testing helpers that widen the
//! production attack surface are the most catastrophic ESC defense
//! bypass mode. The cfg-gating + audit pin pair was designed to make
//! this failure mode IMPOSSIBLE without surfacing as a hard CI failure.
//!
//! ## Helpers shipped at G17-A1
//!
//! 1. [`testing_revoke_cap_mid_call`] — revoke a cap inside a SANDBOX
//!    frame between two host-fn calls (drives ESC-9 closure /
//!    `sandbox_capability_check_per_call_after_revoke.rs`).
//! 2. [`testing_call_engine_dispatch`] — simulate a host-fn dispatching
//!    back into the engine (drives ESC-10 closure / nested-dispatch
//!    adversarial coverage). G17-A1 ships the SURFACE; G20-A1 wave-8a
//!    un-ignores the body.
//! 3. [`testing_inject_forged_cap_claim_section`] — inject a forged
//!    cap-claim WAT section into a fixture (drives ESC-14 closure).
//!    G17-A1 ships the SURFACE; G20-A1 wave-8a un-ignores the body.
//! 4. [`testing_register_uncounted_host_fn`] — register a host-fn that
//!    bypasses the `bypass_output_budget = false` discipline (drives
//!    ESC-7 fuel-refill via re-entry attack-pattern setup).
//!
//! ## Helper SURFACE narrative — r1-wsa-6 enumeration
//!
//! Per r1-wsa-6: the helpers are "spec the SURFACE in G17-A1 wave-5b;
//! un-ignore test bodies that consume the helpers in G20-A1 wave-8a."
//! G17-A1 ships:
//!
//! - The fn signatures + cfg-gating + return-type contracts.
//! - Stub bodies that surface a clear "not yet wired" signal (each
//!   helper either no-ops with a documented invariant or returns a
//!   typed error that the un-ignored test body can match against).
//! - Compile-time assertion via `tests/sandbox_helpers_no_widening.rs`
//!   (un-ignored at G20-A1) that EVERY `pub` item in this file is
//!   gated.
//!
//! G20-A1 wave-8a fills in the bodies that drive committed `.wat`
//! fixtures end-to-end (per phase-3-backlog §7.3.A.1 + §7.3.A.7).

#![cfg(any(test, feature = "test-helpers", feature = "testing"))]
#![cfg(not(target_arch = "wasm32"))]

use crate::primitives::sandbox::SandboxError;
use crate::sandbox::escape_defenses::{EscDefenseState, EscVector};

/// Marker error returned from helper bodies that have not yet been
/// wired (the SURFACE is shipped at G17-A1; the runtime arm wires +
/// fixture-driving comes in G20-A1 wave-8a).
///
/// Tests that consume the helper surface at G17-A1 should match
/// against this marker rather than asserting full fixture-driven
/// behavior.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("§7.3.A.7 helper surface not yet wired (un-ignored at G20-A1 wave-8a): {reason}")]
pub struct HelperSurfaceNotYetWired {
    /// Operator-actionable reason — names the specific G20-A1
    /// step that wires this helper.
    pub reason: String,
}

/// ESC-9 helper: revoke a cap inside a SANDBOX frame between two
/// host-fn calls. The G7-A surface declares the
/// [`crate::sandbox::HostFnContext::live_cap_check`] callback that
/// the trampoline consults BEFORE every host-fn invocation; this
/// helper provides a test-side handle that mutates a back-store the
/// callback reads.
///
/// **G17-A1 surface shape:** the helper takes a mutable `live_caps`
/// vec by reference + the cap-string to revoke; it removes the cap
/// from the vec in place. The integration test paired with this
/// helper at `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs`
/// (un-ignored at G17-A1 per phase-3-backlog §6.3) drives a SANDBOX
/// frame that calls `kv:read` twice with this helper firing between
/// the two calls — the second call observes the revocation.
///
/// **G20-A1 widening** (wave-8a): the helper signature stays
/// stable; the underlying live-policy hookup is replaced with a
/// real engine-backed callable that consults the engine's revoked-
/// actors set + future grant-store.
///
/// Returns `Ok(())` if the cap was present and removed; returns
/// the marker error if the cap was not present (helps tests detect
/// setup mistakes).
///
/// # Errors
/// Returns [`HelperSurfaceNotYetWired`] if the live-caps vec did
/// not contain the named cap (test-setup mistake).
pub fn testing_revoke_cap_mid_call(
    live_caps: &mut Vec<String>,
    cap_to_revoke: &str,
) -> Result<(), HelperSurfaceNotYetWired> {
    let prior_len = live_caps.len();
    live_caps.retain(|c| c != cap_to_revoke);
    if live_caps.len() == prior_len {
        return Err(HelperSurfaceNotYetWired {
            reason: format!(
                "testing_revoke_cap_mid_call: cap-string {:?} not present in live_caps; \
                 either the test set up the wrong cap or the SANDBOX frame already \
                 consumed the cap from the live set. live_caps={:?}",
                cap_to_revoke, live_caps,
            ),
        });
    }
    Ok(())
}

/// ESC-10 helper SURFACE: simulate a host-fn dispatching back into
/// the engine (the nested-dispatch attack pattern). G17-A1 ships
/// the SURFACE; G20-A1 wave-8a wires the body to drive the actual
/// nested-dispatch adversarial fixture.
///
/// **Phase-2b D19-RESOLVED:** nested dispatch is denied by typed
/// error (`SandboxError::NestedDispatchDenied`); ESC-10 closure
/// asserts the typed error fires for the adversarial case where a
/// host-fn callback attempts `Engine::call`. G17-A1 returns the
/// marker error so the un-ignored G20-A1 body has a stable hook.
///
/// # Errors
/// Returns [`HelperSurfaceNotYetWired`] — G20-A1 wave-8a fills in
/// the body to construct an adversarial host-fn dispatch + assert
/// `SandboxError::NestedDispatchDenied`.
pub fn testing_call_engine_dispatch() -> Result<(), HelperSurfaceNotYetWired> {
    Err(HelperSurfaceNotYetWired {
        reason: "testing_call_engine_dispatch: G20-A1 wave-8a wires the \
                 adversarial host-fn dispatch fixture per phase-3-backlog \
                 §7.3.A.7 ESC-10 closure. SURFACE shipped at G17-A1 wave-5b."
            .to_string(),
    })
}

/// ESC-14 helper SURFACE: inject a forged cap-claim WAT section
/// into a fixture so the integration test can assert the engine
/// rejects forged claims at module-validation time.
///
/// **G17-A1 surface shape:** takes the fixture bytes + the forged
/// claim text; returns the modified fixture bytes ready to feed
/// `Sandbox::execute`. G17-A1 returns the marker error (un-wired
/// SURFACE); G20-A1 wave-8a fills in the body.
///
/// # Errors
/// Returns [`HelperSurfaceNotYetWired`] — G20-A1 wave-8a wires
/// the body that mutates the fixture's custom-section table.
pub fn testing_inject_forged_cap_claim_section(
    _fixture_bytes: &[u8],
    _forged_claim: &str,
) -> Result<Vec<u8>, HelperSurfaceNotYetWired> {
    Err(HelperSurfaceNotYetWired {
        reason: "testing_inject_forged_cap_claim_section: G20-A1 wave-8a wires \
                 the WAT custom-section mutator + integrates with \
                 sandbox_esc14_forged_cap_claim_section.rs. SURFACE shipped at \
                 G17-A1 wave-5b."
            .to_string(),
    })
}

/// ESC-7 helper SURFACE: register a host-fn that simulates the
/// fuel-refill-via-re-entry attack pattern. The helper sets up an
/// [`EscDefenseState`] in the "guest active + re-entry observed"
/// shape so [`crate::sandbox::escape_defenses::run_esc7_check`]
/// fires the typed error.
///
/// **G17-A1 surface shape:** takes a mutable [`EscDefenseState`];
/// flips `guest_active = true` + bumps `re_entry_count`. Tests
/// then call `run_esc7_check(state)` and assert the typed
/// `SandboxError::EscapeAttempt(Esc7FuelRefillViaReEntry)` fires.
///
/// This is the simulation surface — the real attack pattern (a
/// guest module calling a host-fn that itself calls
/// `Store::add_fuel`) is exercised end-to-end by the G20-A1 wave-8a
/// fixture-driven test, but the simulation surface lets the
/// G17-A1 wave-5b unit test pin the typed-error fires WITHOUT
/// needing the .wat fixture.
pub fn testing_register_uncounted_host_fn(state: &mut EscDefenseState) {
    state.enter_guest();
    state.re_entry_count = state.re_entry_count.saturating_add(1);
}

/// ESC-13 helper SURFACE: simulate a fuel-meter callback trap.
/// Sets `fuel_meter_callback_trapped = true` on the state so
/// [`crate::sandbox::escape_defenses::run_esc13_check`] fires the
/// typed error.
///
/// G17-A1 wave-5b ships the simulation surface; G20-A1 wave-8a
/// drives the real attack pattern via committed
/// `esc_13_trap_during_fuel_meter_callback.wat` fixture.
pub fn testing_simulate_fuel_meter_callback_trap(state: &mut EscDefenseState) {
    state.fuel_meter_callback_trapped = true;
}

/// ESC-16 helper SURFACE: simulate the fingerprint-collapse pattern
/// (3+ reads of wallclock-correlated cells in one call) so
/// [`crate::sandbox::escape_defenses::run_esc16_check`] fires the
/// typed error.
pub fn testing_simulate_fingerprint_collapse_pattern(state: &mut EscDefenseState) {
    use crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD;
    state.fingerprint_correlated_reads = FINGERPRINT_COLLAPSE_THRESHOLD;
}

/// Helper to construct a typed `SandboxError::EscapeAttempt` for
/// test assertions without exposing the full
/// `crate::primitives::sandbox::SandboxError` surface to test code.
#[must_use]
pub fn make_escape_attempt_error(vector: EscVector, reason: &str) -> SandboxError {
    SandboxError::EscapeAttempt {
        vector,
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_revoke_cap_mid_call_removes_cap() {
        let mut live = vec![
            "host:compute:time".to_string(),
            "host:compute:kv:read".to_string(),
        ];
        testing_revoke_cap_mid_call(&mut live, "host:compute:kv:read").unwrap();
        assert_eq!(live, vec!["host:compute:time".to_string()]);
    }

    #[test]
    fn testing_revoke_cap_mid_call_errors_on_absent_cap() {
        let mut live = vec!["host:compute:time".to_string()];
        let err = testing_revoke_cap_mid_call(&mut live, "host:compute:kv:read").unwrap_err();
        assert!(err.reason.contains("not present"));
    }

    #[test]
    fn testing_register_uncounted_host_fn_sets_esc7_attack_state() {
        let mut state = EscDefenseState::new();
        testing_register_uncounted_host_fn(&mut state);
        assert!(state.guest_active);
        assert_eq!(state.re_entry_count, 1);
    }

    #[test]
    fn testing_simulate_fuel_meter_callback_trap_sets_esc13_state() {
        let mut state = EscDefenseState::new();
        testing_simulate_fuel_meter_callback_trap(&mut state);
        assert!(state.fuel_meter_callback_trapped);
    }

    #[test]
    fn testing_simulate_fingerprint_collapse_pattern_meets_threshold() {
        use crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD;
        let mut state = EscDefenseState::new();
        testing_simulate_fingerprint_collapse_pattern(&mut state);
        assert_eq!(
            state.fingerprint_correlated_reads,
            FINGERPRINT_COLLAPSE_THRESHOLD
        );
    }

    #[test]
    fn make_escape_attempt_error_produces_typed_variant() {
        let err = make_escape_attempt_error(EscVector::Esc7FuelRefillViaReEntry, "test");
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc7FuelRefillViaReEntry,
                ..
            }
        ));
    }
}
