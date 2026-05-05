//! Phase-3 G14-pre-D + ds-11: `Hlc::update(remote)` MUST reject remote
//! stamps whose physical-clock component exceeds the configured skew
//! tolerance, surfacing the typed [`CoreError::HlcSkewExceeded`] which
//! maps to the [`ErrorCode::HlcSkewExceeded`] catalog code
//! `E_HLC_SKEW_EXCEEDED`.
//!
//! The local clock state must NOT be mutated when the typed error fires —
//! Phase-3 sync rejects the offending message and continues serving its
//! own `now()` calls.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::CoreError;
use benten_core::hlc::{BentenHlc, Hlc};
use benten_errors::ErrorCode;

static MOCK_MS: AtomicU64 = AtomicU64::new(0);
fn mock_clock() -> u64 {
    MOCK_MS.load(Ordering::SeqCst)
}

/// Remote 6 minutes in the future > 5-minute default → typed error fires.
#[test]
fn update_beyond_default_tolerance_fires_typed_error() {
    MOCK_MS.store(1_000_000, Ordering::SeqCst);
    let hlc = Hlc::new(1, mock_clock);
    let remote = BentenHlc::new(1_000_000 + 6 * 60 * 1000, 0, 2);
    let err = hlc.update(&remote).expect_err("must reject");
    match err {
        CoreError::HlcSkewExceeded {
            local_physical_ms,
            remote_physical_ms,
            tolerance_ms,
        } => {
            assert_eq!(local_physical_ms, 1_000_000);
            assert_eq!(remote_physical_ms, 1_000_000 + 6 * 60 * 1000);
            assert_eq!(tolerance_ms, Hlc::DEFAULT_SKEW_TOLERANCE_MS);
        }
        other => panic!("expected HlcSkewExceeded, got {other:?}"),
    }
}

/// Typed error maps to the stable catalog code `E_HLC_SKEW_EXCEEDED`.
/// Pins the catalog-string surface so a Phase-3 sync layer (or a future
/// drift-detector run) catches any silent rename.
#[test]
fn skew_error_maps_to_e_hlc_skew_exceeded_catalog_code() {
    MOCK_MS.store(0, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(1, mock_clock, 100);
    let remote = BentenHlc::new(50_000, 0, 2);
    let err = hlc.update(&remote).unwrap_err();
    let code = err.code();
    assert_eq!(code, ErrorCode::HlcSkewExceeded);
    assert_eq!(code.as_str(), "E_HLC_SKEW_EXCEEDED");
    // Round-trip via from_str so the parse-side mapping is also pinned.
    assert_eq!(
        ErrorCode::from_str("E_HLC_SKEW_EXCEEDED"),
        ErrorCode::HlcSkewExceeded
    );
}

/// Strict-inequality boundary: remote.physical_ms == local + tolerance + 1
/// is JUST OVER the boundary → typed error fires. Pins the boundary
/// direction (>, not >=) so a future refactor can't silently flip it.
#[test]
fn update_one_ms_past_boundary_fires_typed_error() {
    MOCK_MS.store(10_000, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(1, mock_clock, 1_000);
    // 10_000 + 1_000 + 1 = 11_001 — one ms past the inclusive boundary
    let remote = BentenHlc::new(11_001, 0, 2);
    let err = hlc.update(&remote).expect_err("just-past-boundary rejects");
    assert!(matches!(err, CoreError::HlcSkewExceeded { .. }));
}

/// Local state is NOT mutated when skew fires. The local HLC continues
/// serving its own `now()` stamps from the same physical_ms it had
/// before the rejected `update`.
#[test]
fn skew_rejection_does_not_mutate_local_state() {
    MOCK_MS.store(500_000, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(42, mock_clock, 1_000);
    let before = hlc.now();
    // before: (500_000, 0, 42)

    let far_future = BentenHlc::new(500_000 + 10_000, 99, 7);
    let _ = hlc.update(&far_future).expect_err("must reject");

    // Subsequent now() reflects local state UNAFFECTED by the rejected
    // remote — physical_ms stays at 500_000 (mock didn't move), logical
    // bumps from `before.logical` (0) by exactly 1.
    let after = hlc.now();
    assert_eq!(after.physical_ms(), 500_000);
    assert_eq!(after.logical(), 1, "logical bumped by +1, NOT +99");
    assert_eq!(after.node_id(), 42);
    assert!(after > before);
}
