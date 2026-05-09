//! Phase-3 G14-pre-D: `Hlc::update(remote)` MUST accept remote stamps whose
//! physical-clock component is within the configured skew tolerance.
//!
//! The reciprocal test (skew exceeded → typed error) lives in
//! `hlc_clock_skew_exceeded_fires_e_hlc_skew_exceeded.rs` next door; the
//! pair pins both sides of the boundary.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::hlc::{BentenHlc, Hlc};

// Per-test mock clocks: each test owns its own `static AtomicU64` + `fn`
// pointer, eliminating the cross-test race that the previous shared-
// `static MOCK_MS` shape carried (phase-3-backlog §7.18 + §7.16 same-
// class precedent). `Hlc::new` takes a `fn() -> u64` pointer (not an
// `impl Fn` closure), so each test needs its own free `fn` wrapping a
// distinct static.

static MOCK_DEFAULT: AtomicU64 = AtomicU64::new(0);
fn clock_default() -> u64 {
    MOCK_DEFAULT.load(Ordering::SeqCst)
}

static MOCK_CUSTOM: AtomicU64 = AtomicU64::new(0);
fn clock_custom() -> u64 {
    MOCK_CUSTOM.load(Ordering::SeqCst)
}

static MOCK_BOUNDARY: AtomicU64 = AtomicU64::new(0);
fn clock_boundary() -> u64 {
    MOCK_BOUNDARY.load(Ordering::SeqCst)
}

static MOCK_PAST: AtomicU64 = AtomicU64::new(0);
fn clock_past() -> u64 {
    MOCK_PAST.load(Ordering::SeqCst)
}

/// Remote stamp ahead of local by 1 second; default 5-minute tolerance
/// → accepted. The post-update local stamp must (a) be strictly greater
/// than the remote, (b) carry the local node-id (the stamp is the LOCAL
/// clock's view, advanced by the remote message), and (c) leave local
/// state mutated so subsequent `now()` calls also exceed the remote.
#[test]
fn update_within_default_tolerance_accepts() {
    MOCK_DEFAULT.store(1_000_000, Ordering::SeqCst);
    let hlc = Hlc::new(0xAAAA_AAAA_AAAA_AAAA, clock_default);
    let remote = BentenHlc::new(1_001_000, 3, 0xBBBB_BBBB_BBBB_BBBB);

    let after = hlc.update(&remote).expect("within tolerance must accept");
    assert!(after > remote, "post-update must dominate remote");
    assert_eq!(after.node_id(), 0xAAAA_AAAA_AAAA_AAAA);

    let next = hlc.now();
    assert!(next > after, "subsequent now() advances past update result");
}

/// Custom skew tolerance: a 1-second window. Remote within the window is
/// accepted; the post-update HLC inherits the remote's logical+1 when the
/// remote's physical clock dominates.
#[test]
fn update_within_custom_tolerance_accepts() {
    MOCK_CUSTOM.store(50_000, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(7, clock_custom, 1_000);
    let remote = BentenHlc::new(50_500, 11, 9);

    let after = hlc
        .update(&remote)
        .expect("within 1s tolerance must accept");
    assert_eq!(after.physical_ms(), 50_500);
    assert_eq!(after.logical(), 12, "remote.logical (11) + 1");
    assert_eq!(after.node_id(), 7);
}

/// Boundary case: remote.physical_ms == local + tolerance. The boundary
/// is INCLUSIVE — exactly-at-tolerance is accepted; only strictly-greater
/// fires the typed error. This pins the inequality direction so a future
/// refactor can't silently flip it.
#[test]
fn update_at_exact_tolerance_boundary_accepts() {
    MOCK_BOUNDARY.store(100_000, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(1, clock_boundary, 5_000);
    let remote = BentenHlc::new(105_000, 0, 2); // exactly local + 5_000

    let after = hlc
        .update(&remote)
        .expect("exact-tolerance boundary accepts");
    assert_eq!(after.physical_ms(), 105_000);
}

/// Remote in the past (older than local) is always accepted — the skew
/// check caps how far INTO THE FUTURE a remote may carry the HLC, not how
/// far in the past. A peer's queued message arriving late is the normal
/// case, not an adversarial one.
#[test]
fn update_with_past_remote_accepts_regardless_of_tolerance() {
    MOCK_PAST.store(1_000_000, Ordering::SeqCst);
    let hlc = Hlc::with_skew_tolerance(1, clock_past, 100); // very tight
    let ancient_remote = BentenHlc::new(0, 0, 99);
    let _ = hlc
        .update(&ancient_remote)
        .expect("past remote always accepts");
}
