//! SANDBOX escape-defense surface (Phase-3 G17-A1 wave-5b).
//!
//! Closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 MAJOR (ESC-16)
//! per phase-3-backlog §6.1 + D-E (R1 revision triage). The defenses
//! live alongside the SANDBOX subsystem because they need access to the
//! per-call wasmtime `Store` shape + the host-fn trampoline boundary;
//! routing through the typed [`crate::primitives::sandbox::SandboxError`]
//! variant keeps audit pipelines per-vector observable rather than
//! collapsing into a generic `ModuleInvalid` shape.
//!
//! ## Vectors
//!
//! - [`EscVector::Esc7FuelRefillViaReEntry`] — D-E + r1-wsa-1 BLOCKER:
//!   guest calls a host-fn whose dispatch path attempts to re-enter the
//!   guest `Store` and `add_fuel` mid-execution. Defense: a re-entry
//!   counter on [`SandboxStoreData`] (engine-side) trips the typed
//!   error before the fuel-refill takes effect.
//! - [`EscVector::Esc13StorePoison`] — D-E + r1-wsa-1 BLOCKER: a
//!   host-side fuel-meter callback panics or traps; the wasmtime
//!   `Store` enters a poisoned state. Defense: a panic-catcher around
//!   the fuel-meter callback maps callback-trap → typed
//!   `EscapeAttempt(Esc13StorePoison)` AND the next SANDBOX call
//!   constructs a fresh `Store` (poison-recovery — Phase-2b's
//!   per-call-`Store` lifecycle from D3-RESOLVED already gives the
//!   recovery path; ESC-13 just needs the typed-error attribution +
//!   the panic-catcher seam).
//! - [`EscVector::Esc16FingerprintCollapse`] — r1-wsa-4 MAJOR: guest
//!   reads a wallclock-correlated cell of guest-controlled linear
//!   memory (the fingerprint vector). Defense: the engine-side memory-
//!   read helper at [`crate::sandbox::fingerprint`] observes the read
//!   pattern; when the pattern matches the fingerprint-collapse
//!   signature, the next host-fn boundary fires the typed error
//!   BEFORE the side-channel becomes guest-observable.
//!
//! ## Why these are typed errors, not wasmtime traps
//!
//! Per sec-r1 D7, host-fn cap denials route through a typed-error
//! marker rather than a wasmtime trap so the engine accounting stays
//! clean (a wasmtime trap unwinds + may corrupt unrelated state). ESC
//! defenses follow the same discipline: detection happens at the
//! host-fn / fuel-meter boundary, NOT mid-instruction; the typed error
//! fires from the trampoline and the wasmtime `Store` is dropped
//! cleanly per the per-call lifecycle.
//!
//! ## Integration with the runtime arm
//!
//! Phase-2b SANDBOX shipped without any of these three defenses; the
//! corresponding ESC test bodies were deferred to phase-3-backlog
//! §6.1 + §6.4. G17-A1 ships:
//!
//! 1. The [`EscVector`] enum + [`SandboxError::EscapeAttempt`] wrapper
//!    (catalog code `E_SANDBOX_ESCAPE_ATTEMPT`).
//! 2. The defense [`run_esc7_check`] / [`run_esc13_check`] /
//!    [`run_esc16_check`] entry points (test-callable via
//!    [`crate::sandbox::testing_helpers`]; production-callable via the
//!    G17-A2 runtime arm wire-up + G20-A1 un-ignore of the §7.3.A.7
//!    test bodies that exercise the production fixture path).
//! 3. The trap-routing arm at
//!    [`crate::sandbox::trap_to_typed::map_call_error`] (the
//!    `EscapeAttempt` marker is unwrapped at the cause-chain walk).
//!
//! The phased approach matches §3.6b end-to-end pin discipline: G17-A1
//! ships the SURFACE (typed error + helper entry points + Vector enum +
//! cfg-gated test-helpers); G20-A1 wave-8a un-ignores the runtime test
//! bodies that drive committed `.wat` fixtures end-to-end.
//!
//! `#[cfg(not(target_arch = "wasm32"))]`-gated per sec-pre-r1-05; the
//! wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use crate::primitives::sandbox::SandboxError;
use serde::{Deserialize, Serialize};

/// Discriminating SANDBOX escape vector for
/// [`SandboxError::EscapeAttempt`] attribution. The ESC matrix in
/// `docs/SECURITY-POSTURE.md` enumerates 16 vectors; G17-A1 wave-5b
/// declares the three vectors we ship runtime defenses for in this
/// wave — the remaining 13 vectors (ESC-1..6, ESC-8, ESC-10..12,
/// ESC-14..15) either route through other typed errors (e.g. ESC-15
/// → `SandboxManifestUnknown`; ESC-1/3/5/11/12 → `SandboxModuleInvalid`
/// via wasmtime structural validation) or remain dormant pending
/// wave-8a G20-A1 un-ignore of the §7.3.A.1 test bodies.
///
/// **Naming canonical per r4-r1-wsa-5:** the variant names spell
/// `Esc<N><Description>` (e.g. `Esc7FuelRefillViaReEntry`,
/// `Esc13StorePoison`, `Esc16FingerprintCollapse`) — matching the
/// `tests/sandbox_esc_7.rs` + `tests/sandbox_esc_13.rs` + future
/// `tests/sandbox_esc_16.rs` body cite-targets. Renaming a variant
/// requires touching all body cite-targets in lock-step (the
/// `tests/esc_vector_variant_names_canonical.rs` architectural-shape
/// pin asserts this canonical naming).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EscVector {
    /// ESC-7: fuel-refill via host-fn re-entry. Guest calls a host-fn
    /// whose dispatch path attempts to re-enter the SANDBOX `Store`
    /// (or any path that calls `Store::add_fuel` during guest
    /// execution), effectively bypassing the fuel ceiling. Defense
    /// fires from the trampoline before the inner re-entry takes
    /// effect; `SandboxStoreData` carries a re-entry counter that
    /// must remain zero for the duration of guest execution.
    ///
    /// Closes r1-wsa-1 BLOCKER half-a per D-E (R1 revision triage).
    Esc7FuelRefillViaReEntry,
    /// ESC-13: trap during fuel-meter callback / Store-poison. A
    /// host-side fuel-meter callback (registered with wasmtime to
    /// interpose on fuel exhaustion) panics or traps; the wasmtime
    /// `Store` enters a poisoned state where subsequent execution
    /// produces undefined behavior (lingering fuel state, partial
    /// guest stack, leaked memory). Defense: a panic-catcher around
    /// the fuel-meter callback maps callback-trap → this typed error;
    /// per-call `Store` lifecycle (D3-RESOLVED) ensures the next
    /// SANDBOX call gets a fresh Store.
    ///
    /// Closes r1-wsa-1 BLOCKER half-b per D-E (R1 revision triage).
    Esc13StorePoison,
    /// ESC-16: fingerprint-collapse via wallclock-correlated state
    /// read. Guest reads a cell of linear memory the host wrote a
    /// wallclock-correlated value into (e.g. the diff between two
    /// `time` host-fn calls), then branches on the value to leak a
    /// side-channel. Defense: the engine-side memory-read helper at
    /// [`crate::sandbox::fingerprint::read_collapse_state`] observes
    /// the read pattern; the defense fires at the next host-fn
    /// boundary BEFORE the side-channel becomes guest-observable.
    ///
    /// Closes r1-wsa-4 MAJOR per phase-3-backlog §6.1.
    Esc16FingerprintCollapse,
}

impl EscVector {
    /// Stable string identifier for the vector — useful for log filters
    /// and audit-pipeline routing without committing to the
    /// `Debug` representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EscVector::Esc7FuelRefillViaReEntry => "ESC-7",
            EscVector::Esc13StorePoison => "ESC-13",
            EscVector::Esc16FingerprintCollapse => "ESC-16",
        }
    }
}

/// Per-call ESC-defense state attached to the SANDBOX `Store`.
/// Lives next to [`crate::primitives::sandbox::SandboxStoreData`] so
/// trampolines can mutate counters from `Caller<SandboxStoreData>`
/// without an extra layer of indirection. G17-A2 runtime-arm wave
/// threads this through the per-call store at construction time.
#[derive(Debug, Default)]
pub struct EscDefenseState {
    /// ESC-7: number of times the trampoline observed an attempt to
    /// re-enter the SANDBOX `Store` from within a host-fn dispatch
    /// path. Defense fires when this becomes >0 during guest
    /// execution (the only legitimate re-entry is the cleanup path
    /// after guest return, which sets `guest_active = false` first).
    pub re_entry_count: u32,
    /// ESC-7 + ESC-13 ancillary flag: `true` while the guest is
    /// executing inside `Instance::call`. The trampoline checks this
    /// before incrementing [`Self::re_entry_count`] (a re-entry
    /// during guest execution is the attack pattern; a re-entry
    /// after guest return is benign cleanup).
    pub guest_active: bool,
    /// ESC-13: the fuel-meter callback panicked or trapped during
    /// the most recent guest execution. Set by the panic-catcher in
    /// [`run_esc13_check`]; surfaced as
    /// [`SandboxError::EscapeAttempt(Esc13StorePoison)`].
    pub fuel_meter_callback_trapped: bool,
    /// ESC-16: number of times the guest read a wallclock-correlated
    /// memory cell during a single SANDBOX call. The fingerprint
    /// helper increments this; the defense fires when the pattern
    /// matches the fingerprint-collapse signature (see
    /// [`crate::sandbox::fingerprint`]).
    pub fingerprint_correlated_reads: u32,
}

impl EscDefenseState {
    /// Construct a fresh per-call state. `guest_active` is `false`
    /// until the trampoline transitions into `Instance::call`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark guest execution as active. Called by the SANDBOX executor
    /// immediately before `Instance::call`.
    pub fn enter_guest(&mut self) {
        self.guest_active = true;
    }

    /// Mark guest execution as ended. Called by the SANDBOX executor
    /// immediately after `Instance::call` returns (in either the Ok
    /// or trap path).
    pub fn exit_guest(&mut self) {
        self.guest_active = false;
    }
}

/// ESC-7 defense entry point — checks for fuel-refill via host-fn
/// re-entry on a per-call basis. Returns `Ok(())` if no attempt is
/// in flight; returns the typed [`SandboxError::EscapeAttempt`] when
/// the trampoline observed a guest-active re-entry attempt.
///
/// Called from the host-fn trampoline AFTER the trampoline detects a
/// re-entry attempt (specifically: the re-entry counter is bumped
/// from a host-fn dispatch path while `guest_active = true`). The
/// defense fires BEFORE any inner `add_fuel` call takes effect.
///
/// # Errors
/// Returns [`SandboxError::EscapeAttempt`] with
/// [`EscVector::Esc7FuelRefillViaReEntry`] when the state shows a
/// re-entry attempt during guest execution.
pub fn run_esc7_check(state: &EscDefenseState) -> Result<(), SandboxError> {
    if state.guest_active && state.re_entry_count > 0 {
        return Err(SandboxError::EscapeAttempt {
            vector: EscVector::Esc7FuelRefillViaReEntry,
            reason: format!(
                "host-fn dispatch attempted to re-enter the SANDBOX Store \
                 during guest execution ({} attempts observed); defense fires \
                 before fuel-refill takes effect per phase-3-backlog §6.1 + \
                 D-E + r1-wsa-1 BLOCKER closure",
                state.re_entry_count
            ),
        });
    }
    Ok(())
}

/// ESC-13 defense entry point — checks for fuel-meter callback trap /
/// Store-poison on a per-call basis. Returns `Ok(())` if the
/// fuel-meter callback executed cleanly; returns the typed
/// [`SandboxError::EscapeAttempt`] when the panic-catcher observed a
/// callback panic or trap.
///
/// The runtime arm pairs this defense with the per-call `Store`
/// lifecycle (D3-RESOLVED): a Store flagged poisoned is dropped after
/// the typed error fires, so the next SANDBOX call gets a fresh Store
/// (no cross-call poison leakage).
///
/// # Errors
/// Returns [`SandboxError::EscapeAttempt`] with
/// [`EscVector::Esc13StorePoison`] when the fuel-meter callback
/// trapped during the most recent guest execution.
pub fn run_esc13_check(state: &EscDefenseState) -> Result<(), SandboxError> {
    if state.fuel_meter_callback_trapped {
        return Err(SandboxError::EscapeAttempt {
            vector: EscVector::Esc13StorePoison,
            reason: "fuel-meter callback panicked or trapped during guest \
                     execution; the wasmtime Store is poisoned and being \
                     dropped per D3-RESOLVED per-call Store lifecycle. The \
                     next SANDBOX call gets a fresh Store; defense routes per \
                     phase-3-backlog §6.1 + D-E + r1-wsa-1 BLOCKER closure"
                .to_string(),
        });
    }
    Ok(())
}

/// ESC-16 defense entry point — checks for fingerprint-collapse via
/// wallclock-correlated state read. Returns `Ok(())` when the read
/// pattern is benign; returns the typed
/// [`SandboxError::EscapeAttempt`] when the engine-side memory-read
/// helper observed the fingerprint-collapse signature (a guest read
/// of memory the host wrote a wallclock-derived value into).
///
/// **The detection threshold is conservative** — a single correlated
/// read is suspicious but not yet conclusive (legitimate guests may
/// read host-written cells incidentally); the defense fires when the
/// counter exceeds the threshold defined in
/// [`crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD`].
/// The pairing fires at the NEXT host-fn boundary per
/// phase-3-backlog §6.1 (BEFORE the wallclock divergence becomes
/// guest-observable).
///
/// # Errors
/// Returns [`SandboxError::EscapeAttempt`] with
/// [`EscVector::Esc16FingerprintCollapse`] when the read counter
/// exceeds the threshold.
pub fn run_esc16_check(state: &EscDefenseState) -> Result<(), SandboxError> {
    use crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD;
    if state.fingerprint_correlated_reads >= FINGERPRINT_COLLAPSE_THRESHOLD {
        return Err(SandboxError::EscapeAttempt {
            vector: EscVector::Esc16FingerprintCollapse,
            reason: format!(
                "guest read a wallclock-correlated host-written memory cell \
                 {} times (threshold = {}); defense fires at next host-fn \
                 boundary per phase-3-backlog §6.1 + r1-wsa-4 closure, \
                 BEFORE the side-channel becomes guest-observable",
                state.fingerprint_correlated_reads, FINGERPRINT_COLLAPSE_THRESHOLD
            ),
        });
    }
    Ok(())
}

/// Aggregate ESC defense check fired at every host-fn boundary —
/// runs ESC-7, ESC-13, ESC-16 in priority order (highest-impact
/// first). Returns the first vector that fires, or `Ok(())` if no
/// vector tripped.
///
/// Priority rationale: ESC-13 (Store-poison) is the most catastrophic
/// (UB on subsequent calls); ESC-7 (fuel-refill) is next (resource
/// budget bypass); ESC-16 (fingerprint side-channel) is the lowest
/// priority because it leaks information rather than corrupting
/// state.
///
/// # Errors
/// Returns the first [`SandboxError::EscapeAttempt`] that fires from
/// the per-vector checks.
pub fn run_all_checks(state: &EscDefenseState) -> Result<(), SandboxError> {
    run_esc13_check(state)?;
    run_esc7_check(state)?;
    run_esc16_check(state)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_vector_as_str_canonical() {
        // r4-r1-wsa-5 architectural-shape pin: variant string
        // identifiers are stable + audit-pipeline routable.
        assert_eq!(EscVector::Esc7FuelRefillViaReEntry.as_str(), "ESC-7");
        assert_eq!(EscVector::Esc13StorePoison.as_str(), "ESC-13");
        assert_eq!(EscVector::Esc16FingerprintCollapse.as_str(), "ESC-16");
    }

    #[test]
    fn esc7_check_passes_when_no_re_entry() {
        let state = EscDefenseState::new();
        assert!(run_esc7_check(&state).is_ok());
    }

    #[test]
    fn esc7_check_fires_when_re_entry_during_guest_execution() {
        let mut state = EscDefenseState::new();
        state.enter_guest();
        state.re_entry_count = 1;
        let err = run_esc7_check(&state).unwrap_err();
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc7FuelRefillViaReEntry,
                ..
            }
        ));
    }

    #[test]
    fn esc7_check_silent_when_re_entry_after_guest_exit() {
        // Re-entry AFTER guest_active was cleared is the legitimate
        // cleanup path — no defense fires.
        let mut state = EscDefenseState::new();
        state.enter_guest();
        state.exit_guest();
        state.re_entry_count = 1;
        assert!(run_esc7_check(&state).is_ok());
    }

    #[test]
    fn esc13_check_fires_when_fuel_meter_callback_trapped() {
        let mut state = EscDefenseState::new();
        state.fuel_meter_callback_trapped = true;
        let err = run_esc13_check(&state).unwrap_err();
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc13StorePoison,
                ..
            }
        ));
    }

    #[test]
    fn esc13_check_passes_when_callback_clean() {
        let state = EscDefenseState::new();
        assert!(run_esc13_check(&state).is_ok());
    }

    #[test]
    fn esc16_check_fires_at_threshold() {
        use crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD;
        let mut state = EscDefenseState::new();
        state.fingerprint_correlated_reads = FINGERPRINT_COLLAPSE_THRESHOLD;
        let err = run_esc16_check(&state).unwrap_err();
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc16FingerprintCollapse,
                ..
            }
        ));
    }

    #[test]
    fn esc16_check_silent_below_threshold() {
        use crate::sandbox::fingerprint::FINGERPRINT_COLLAPSE_THRESHOLD;
        let mut state = EscDefenseState::new();
        state.fingerprint_correlated_reads = FINGERPRINT_COLLAPSE_THRESHOLD.saturating_sub(1);
        assert!(run_esc16_check(&state).is_ok());
    }

    #[test]
    fn run_all_checks_priority_order_esc13_first() {
        // ESC-13 is highest-priority: when both ESC-13 and ESC-7 are
        // tripped, ESC-13 fires first.
        let mut state = EscDefenseState::new();
        state.enter_guest();
        state.re_entry_count = 1;
        state.fuel_meter_callback_trapped = true;
        let err = run_all_checks(&state).unwrap_err();
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc13StorePoison,
                ..
            }
        ));
    }

    #[test]
    fn escape_attempt_routes_to_e_sandbox_escape_attempt() {
        let err = SandboxError::EscapeAttempt {
            vector: EscVector::Esc7FuelRefillViaReEntry,
            reason: "test".to_string(),
        };
        assert_eq!(err.code(), benten_errors::ErrorCode::SandboxEscapeAttempt);
    }
}
