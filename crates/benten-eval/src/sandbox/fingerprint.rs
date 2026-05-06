//! ESC-16 fingerprint-collapse engine-side memory-read helper
//! (Phase-3 G17-A1 wave-5b; r1-wsa-4 MAJOR + phase-3-backlog §6.1).
//!
//! ESC-16 is the "fingerprint-collapse via wallclock-correlated state
//! read" escape vector: the guest reads a cell of guest-controlled
//! linear memory that the host wrote a wallclock-correlated value
//! into (e.g. the diff between two `time` host-fn calls), then
//! branches on the value to leak a side-channel.
//!
//! The defense lives in two halves:
//!
//! - **engine-side memory-read helper** (this module): observes the
//!   guest's reads of host-written cells AND increments the
//!   per-call read counter on
//!   [`crate::sandbox::escape_defenses::EscDefenseState`].
//! - **boundary-firing defense** (in
//!   [`crate::sandbox::escape_defenses::run_esc16_check`]): fires the
//!   typed [`crate::primitives::sandbox::SandboxError::EscapeAttempt`]
//!   at the next host-fn boundary when the read counter exceeds
//!   [`FINGERPRINT_COLLAPSE_THRESHOLD`], BEFORE the wallclock
//!   divergence becomes guest-observable.
//!
//! ## Why the helper lives at engine-side, not in the trampoline
//!
//! Per r1-wsa-4 architecture pin: the read pattern detection cannot
//! happen inside the wasm guest body (the guest is by-design
//! isolated). The host trampolines that WRITE wallclock-correlated
//! values (currently: `time` host-fn) tag the destination cells in a
//! side-table; the engine-side memory-read helper consults the
//! side-table when the next host-fn observation occurs (e.g. when
//! the guest passes a memory pointer to a host-fn the host can
//! inspect). The check is thus boundary-coupled rather than
//! mid-instruction.
//!
//! ## Threshold rationale
//!
//! A single read of a wallclock-correlated cell is suspicious but
//! not conclusive — legitimate guests may read host-written cells
//! incidentally. The threshold ([`FINGERPRINT_COLLAPSE_THRESHOLD`])
//! is set conservatively at 3 reads-within-one-call: an observation
//! pattern compatible with a guest sampling the wallclock-derived
//! cell to amplify the side-channel.
//!
//! G20-A1 wave-8a un-ignores the committed `.wat` fixture
//! (`crates/benten-eval/tests/fixtures/sandbox/esc_16_fingerprint_collapse.wat`)
//! that drives a 3-read pattern + asserts the typed error fires.
//!
//! `#[cfg(not(target_arch = "wasm32"))]`-gated per sec-pre-r1-05.

#![cfg(not(target_arch = "wasm32"))]

use crate::sandbox::escape_defenses::EscDefenseState;

/// Threshold for ESC-16 fingerprint-collapse defense — number of
/// reads of host-written wallclock-correlated cells observed within
/// a single SANDBOX call before the typed error fires.
///
/// Set at 3 (conservative): legitimate guests may incidentally read
/// host-written cells; 3+ reads-within-one-call is a pattern
/// compatible with sampling the wallclock-derived cell to amplify
/// the side-channel.
pub const FINGERPRINT_COLLAPSE_THRESHOLD: u32 = 3;

/// Side-table marker — the host writes a wallclock-correlated value
/// into a memory address; the engine-side helper records the
/// address so subsequent guest reads of the same address can be
/// detected.
///
/// In the G17-A1 wave-5b SURFACE shape: the side-table is a
/// `BTreeSet<u32>` of memory addresses living on the per-call
/// `crate::primitives::sandbox::SandboxStoreData` (private; see crate root). G17-A2's
/// runtime-arm wave widens the trampoline to populate the
/// side-table from inside the `time` host-fn (which writes the
/// wallclock-derived value into guest memory at a guest-pointed
/// destination).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WallclockTaintedAddress(pub u32);

/// Observe a guest read of a memory address. Increments the read
/// counter on [`EscDefenseState`] when the address matches a
/// host-written wallclock-correlated cell; the boundary-firing
/// defense at [`crate::sandbox::escape_defenses::run_esc16_check`]
/// consults the counter at the next host-fn boundary.
///
/// # Why this is read_collapse_state (not read_address)
///
/// Per r1-wsa-4 the helper is named after the OBSERVABLE EFFECT
/// (collapsing the wallclock fingerprint into a side-channel) rather
/// than the mechanical action (memory read). The naming makes
/// audit-pipeline traces self-describing: a log line containing
/// `read_collapse_state` immediately signals "this is the ESC-16
/// detection path" rather than "this is one of N read paths".
pub fn read_collapse_state(
    state: &mut EscDefenseState,
    addr: WallclockTaintedAddress,
    tainted_addresses: &[WallclockTaintedAddress],
) {
    if tainted_addresses.contains(&addr) {
        state.fingerprint_correlated_reads = state.fingerprint_correlated_reads.saturating_add(1);
    }
}

/// Record a host write of a wallclock-correlated value into a guest
/// memory address. Called from the `time` host-fn trampoline (and
/// any future host-fn that writes wallclock-derived values into
/// guest memory) to seed the tainted-address side-table.
///
/// Returns the address as a [`WallclockTaintedAddress`] so callers
/// can store it on the per-call side-table without re-wrapping.
#[must_use]
pub fn record_wallclock_write(addr: u32) -> WallclockTaintedAddress {
    WallclockTaintedAddress(addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_collapse_state_increments_on_tainted_read() {
        let mut state = EscDefenseState::new();
        let tainted = vec![WallclockTaintedAddress(0x1000)];
        read_collapse_state(&mut state, WallclockTaintedAddress(0x1000), &tainted);
        assert_eq!(state.fingerprint_correlated_reads, 1);
    }

    #[test]
    fn read_collapse_state_silent_on_untainted_read() {
        let mut state = EscDefenseState::new();
        let tainted = vec![WallclockTaintedAddress(0x1000)];
        read_collapse_state(&mut state, WallclockTaintedAddress(0x2000), &tainted);
        assert_eq!(state.fingerprint_correlated_reads, 0);
    }

    #[test]
    fn read_collapse_state_threshold_pin() {
        // Sanity-check the threshold is non-trivial (not 0 / not 1
        // — guards against a regression that would make EVERY read
        // fire the defense, including legitimate incidental reads).
        const _: () = assert!(
            FINGERPRINT_COLLAPSE_THRESHOLD >= 2,
            "FINGERPRINT_COLLAPSE_THRESHOLD must be >=2 to avoid firing on a single legitimate read"
        );
    }

    #[test]
    fn record_wallclock_write_round_trips_address() {
        let tainted = record_wallclock_write(0xDEAD_BEEF);
        assert_eq!(tainted, WallclockTaintedAddress(0xDEAD_BEEF));
    }
}
