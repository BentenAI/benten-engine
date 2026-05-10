//! Phase-3 G14-pre-D: 10 000-case proptest verifying that
//! [`benten_core::hlc::Hlc::now`] returns strictly-monotonic stamps under
//! tight-loop sampling, regardless of how the underlying physical clock
//! behaves (advances, stalls, rewinds).
//!
//! Replaces the historical `prop_hlc_monotonic_placeholder.rs` which was
//! an `#[ignore]`'d empty test pinning the M16 deferral.
//!
//! Strategy: drive the HLC with a deterministic `fn() -> u64` mock clock
//! that follows a property-generated sequence of physical-clock readings
//! (monotonic, stalled, or rewound at each step). For every step, sample
//! `Hlc::now()` and assert each new stamp is strictly greater than the
//! previous, even under the adversarial schedules where the wallclock
//! does not cooperate.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::hlc::{BentenHlc, Hlc};
use proptest::prelude::*;

// The `Hlc::new` API takes a `fn() -> u64` (not a closure or trait object),
// so each mock-clock state lives in its own module-level static. Per-test
// statics (NOT a single shared `MOCK_PHYSICAL_MS` across proptests)
// preclude cross-test interference under any future schedule that might
// run the proptest binary's tests in parallel — defends structurally
// rather than via comment-only constraint. Sibling decomposition applied
// to the inline `mod tests` at `crates/benten-core/src/hlc.rs:380+` per
// phase-3-backlog §7.18 closure (R6 R2 hlc-r6-r2-2 sibling-site closure).

// ---- prop_hlc_now_is_strictly_monotonic ----
static MOCK_PROP_NOW_MS: AtomicU64 = AtomicU64::new(0);
fn prop_now_clock() -> u64 {
    MOCK_PROP_NOW_MS.load(Ordering::SeqCst)
}

// ---- prop_hlc_tight_loop_is_strictly_monotonic ----
static MOCK_PROP_TIGHT_LOOP_MS: AtomicU64 = AtomicU64::new(0);
fn prop_tight_loop_clock() -> u64 {
    MOCK_PROP_TIGHT_LOOP_MS.load(Ordering::SeqCst)
}

/// Strategy: a sequence of physical-clock deltas. Each delta is signed in
/// effect (`i64`) but represented as `(advance: bool, magnitude: u32)` to
/// keep the strategy small + cheap. The proptest body then either advances
/// the wallclock by `magnitude` ms (if `advance == true`) or rewinds by
/// `magnitude` ms (if `advance == false`, saturating at zero so the mock
/// never underflows).
fn arb_clock_delta() -> impl Strategy<Value = (bool, u32)> {
    (any::<bool>(), 0u32..1_000)
}

proptest! {
    // 10 000 cases per the G14-pre-D brief. Each case drives ~16 sampled
    // stamps through the HLC under a property-generated clock schedule,
    // so the test exercises ~160 000 `now()` invocations end-to-end.
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        .. ProptestConfig::default()
    })]

    /// Strict monotonicity: every `Hlc::now()` call returns a stamp
    /// strictly greater than the previous one, regardless of how the
    /// physical clock behaves (advance / stall / rewind).
    #[test]
    fn prop_hlc_now_is_strictly_monotonic(
        deltas in prop::collection::vec(arb_clock_delta(), 1..16),
    ) {
        // Reset the mock to a known starting point — different cases must
        // not share `last_emitted` state.
        MOCK_PROP_NOW_MS.store(1_000_000, Ordering::SeqCst);
        let hlc = Hlc::new(0xABCD_EF01_2345_6789, prop_now_clock);

        // Always sample once before any deltas — pins the initial physical_ms.
        let first = hlc.now();
        let mut prev: BentenHlc = first;

        for (advance, magnitude) in deltas {
            // Apply the property-generated delta to the mock clock.
            if advance {
                MOCK_PROP_NOW_MS.fetch_add(u64::from(magnitude), Ordering::SeqCst);
            } else {
                let cur = MOCK_PROP_NOW_MS.load(Ordering::SeqCst);
                let new_val = cur.saturating_sub(u64::from(magnitude));
                MOCK_PROP_NOW_MS.store(new_val, Ordering::SeqCst);
            }
            let next = hlc.now();
            // Strict monotonicity invariant — load-bearing for Loro
            // per-property LWW + Inv-14 device-grain attribution + every
            // future Phase-3 consumer of HLC.
            prop_assert!(
                next > prev,
                "HLC::now must be strictly monotonic; prev={:?}, next={:?}",
                prev,
                next
            );
            prev = next;
        }
    }

    /// Tight-loop sampling: `now()` called repeatedly without any external
    /// clock changes still returns strictly-monotonic stamps. Stresses the
    /// logical-counter bump path.
    #[test]
    fn prop_hlc_tight_loop_is_strictly_monotonic(n in 2usize..256) {
        MOCK_PROP_TIGHT_LOOP_MS.store(2_000_000, Ordering::SeqCst);
        let hlc = Hlc::new(1, prop_tight_loop_clock);
        let stamps: Vec<BentenHlc> = (0..n).map(|_| hlc.now()).collect();
        for w in stamps.windows(2) {
            prop_assert!(
                w[1] > w[0],
                "tight-loop stamps must be monotonic; w[0]={:?}, w[1]={:?}",
                w[0],
                w[1]
            );
        }
    }
}
