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

/// ESC-10 helper: simulate a host-fn dispatching back into the
/// engine (the nested-dispatch attack pattern). The simulation
/// shape stamps the [`EscDefenseState`] with the "guest active +
/// re-entry observed" markers a real attack would produce, so a
/// caller can then invoke
/// [`crate::sandbox::escape_defenses::run_esc7_check`] (the
/// fuel-refill-via-re-entry vector ESC-10 maps to in the
/// EscDefenseState matrix per audit-5 finding #2 + the inline
/// narrative at `sandbox_escape_attempts_denied.rs::
/// sandbox_escape_reentrancy_via_host_fn_denied`) and assert the
/// typed `SandboxError::EscapeAttempt(Esc7FuelRefillViaReEntry)`
/// fires.
///
/// **Phase-3 G21-T3 fill (audit-5 ESC-10 carry):** prior to G21-T3
/// the helper returned [`HelperSurfaceNotYetWired`] as a structural-
/// hook stub; this fill drives the EscDefenseState transition that
/// matches what a real nested-dispatch attempt would observe. The
/// production defense path (D19-RESOLVED — no host-fn callback
/// re-enters `Engine::call`) is asserted at the engine layer by
/// `crates/benten-engine/tests/integration/esc_subscribe_integration.rs::
/// helper_smoke_call_engine_dispatch_routes_through_production_call`
/// + by the absence of a re-entry path in `Engine::call` itself.
/// This eval-side helper exists so the eval-only adversarial
/// integration tests can drive the EscDefenseState transition
/// without taking an engine dependency.
pub fn testing_call_engine_dispatch(state: &mut EscDefenseState) {
    // Mirror the EscDefenseState transitions the host-fn
    // trampoline would observe if a guest module invoked a
    // host-fn that itself attempted nested dispatch:
    //   1. Guest is currently executing (enter_guest).
    //   2. The host-fn dispatched back, observed as a re-entry.
    state.enter_guest();
    state.re_entry_count = state.re_entry_count.saturating_add(1);
}

/// ESC-14 helper: inject a forged cap-claim custom-section into a
/// wasm module's bytes so the integration test can assert the
/// engine SILENTLY IGNORES the forged section (cap derivation is
/// EXCLUSIVELY from the call-time manifest per ESC-14 closure).
///
/// Takes the well-formed wasm module bytes + the forged claim text
/// (e.g. `"requires:host:*:*"`); returns the bytes with a
/// custom-section appended. The wasm binary format permits
/// arbitrary `id=0` "custom sections" trailing the well-formed
/// module body; they are spec-mandated to be IGNORED by execution
/// AND by cap derivation. This helper's appended section is exactly
/// what a malicious module-author would attempt — engineering risk
/// being a future wasmtime feature OR a codegen mistake reads the
/// section + treats it as authoritative.
///
/// **Phase-3 G21-T3 fill (audit-5 ESC-14 carry):** prior to G21-T3
/// the helper returned [`HelperSurfaceNotYetWired`]; this fill
/// mutates the bytes per the WASM custom-section grammar
/// (`SECTION_ID_CUSTOM=0` + LEB128 size + LEB128 name-len +
/// name-bytes + payload). Mirrors the inline `append_forged_custom_section`
/// in `crates/benten-eval/tests/sandbox_esc14_forged_cap_claim_section.rs`.
///
/// **Caveat:** the input bytes MUST be a well-formed wasm module
/// (header + valid section sequence). If the input is malformed,
/// the appended section is appended verbatim — wasmtime will
/// reject the resulting module at parse time, which is also a
/// valid outcome (the forged cap-claim is not consulted because
/// the module was rejected before any cap derivation).
#[must_use]
pub fn testing_inject_forged_cap_claim_section(
    fixture_bytes: &[u8],
    forged_claim: &str,
) -> Vec<u8> {
    // Custom-section format: id=0 (1 byte) + section size (LEB128 u32) +
    // name length (LEB128 u32) + name (UTF-8) + payload bytes.
    fn leb128_u32(mut x: u32, out: &mut Vec<u8>) {
        loop {
            let mut byte = (x & 0x7f) as u8;
            x >>= 7;
            if x != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if x == 0 {
                break;
            }
        }
    }

    let name = b"benten:forged_caps";
    let payload = forged_claim.as_bytes();

    let mut name_len_buf: Vec<u8> = Vec::new();
    leb128_u32(u32::try_from(name.len()).unwrap_or(u32::MAX), &mut name_len_buf);

    // Section content = [name_len_LEB128 | name_bytes | payload_bytes].
    let mut content: Vec<u8> = Vec::with_capacity(name_len_buf.len() + name.len() + payload.len());
    content.extend_from_slice(&name_len_buf);
    content.extend_from_slice(name);
    content.extend_from_slice(payload);

    let mut size_buf: Vec<u8> = Vec::new();
    leb128_u32(
        u32::try_from(content.len()).unwrap_or(u32::MAX),
        &mut size_buf,
    );

    let mut out: Vec<u8> = Vec::with_capacity(fixture_bytes.len() + 1 + size_buf.len() + content.len());
    out.extend_from_slice(fixture_bytes);
    out.push(0u8); // SECTION_ID_CUSTOM
    out.extend_from_slice(&size_buf);
    out.extend_from_slice(&content);
    out
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

    /// G21-T3 ESC-10 helper fill: `testing_call_engine_dispatch`
    /// drives the EscDefenseState through the nested-dispatch
    /// attack-pattern transition (enter_guest + re_entry_count bump).
    #[test]
    fn testing_call_engine_dispatch_simulates_nested_dispatch_attack_state() {
        let mut state = EscDefenseState::new();
        assert!(!state.guest_active, "fresh state must have guest_active=false");
        assert_eq!(state.re_entry_count, 0);

        testing_call_engine_dispatch(&mut state);

        assert!(
            state.guest_active,
            "ESC-10 helper MUST flip guest_active=true"
        );
        assert_eq!(
            state.re_entry_count, 1,
            "ESC-10 helper MUST bump re_entry_count to 1"
        );
    }

    /// G21-T3 ESC-14 helper fill:
    /// `testing_inject_forged_cap_claim_section` appends a custom
    /// section to the wasm bytes per the WASM custom-section grammar.
    #[test]
    fn testing_inject_forged_cap_claim_section_appends_custom_section() {
        // Minimal well-formed wasm header (magic + version).
        let bytes: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let forged = testing_inject_forged_cap_claim_section(&bytes, "requires:host:*:*");

        // Original bytes preserved verbatim at the head.
        assert!(
            forged.starts_with(&bytes),
            "forged bytes MUST preserve the original module bytes verbatim"
        );
        // Trailing section starts with id=0 (custom-section).
        assert_eq!(
            forged[bytes.len()],
            0u8,
            "appended section MUST be id=0 (custom-section)"
        );
        // The forged claim payload is present in the trailing bytes.
        let tail = &forged[bytes.len()..];
        assert!(
            tail.windows(b"requires:host:*:*".len())
                .any(|w| w == b"requires:host:*:*"),
            "forged-claim payload MUST be embedded in the appended section"
        );
        // The section's name marker is present.
        assert!(
            tail.windows(b"benten:forged_caps".len())
                .any(|w| w == b"benten:forged_caps"),
            "forged section MUST carry the `benten:forged_caps` name marker"
        );
    }
}
