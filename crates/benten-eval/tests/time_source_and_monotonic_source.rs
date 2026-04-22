//! R3 unit tests for G3-B / T1 / sec-r1-2: `TimeSource` + `MonotonicSource`
//! traits — FROZEN interfaces.
//!
//! `TimeSource` (default: `uhlc::HLC` wrapper) supplies HLC stamps.
//! `MonotonicSource` (default: `std::time::Instant` wrapper) drives TOCTOU
//! cadence. Mock impls must compile for test injection.
//!
//! TDD red-phase: neither trait exists yet in `benten_eval`. Tests fail to
//! compile until G3-B lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.3 T1, sec-r1-2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{MonotonicSource, TimeSource};
use std::time::Duration;

struct MockTime {
    value: u64,
}
impl TimeSource for MockTime {
    fn hlc_stamp(&self) -> u64 {
        self.value
    }
}

struct MockMono {
    start: u64,
}
impl MonotonicSource for MockMono {
    fn elapsed_since_start(&self) -> Duration {
        Duration::from_secs(self.start)
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn time_source_mock_usable() {
    // Test-side impl compiles + is injectable behind a trait object.
    let m: Box<dyn TimeSource> = Box::new(MockTime { value: 42 });
    assert_eq!(m.hlc_stamp(), 42);
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn time_source_default_is_hlc_wrapper() {
    // The default production TimeSource wraps `uhlc::HLC`. The wrapper is
    // exposed via `benten_eval::default_time_source()`.
    let t = benten_eval::default_time_source();
    // Two consecutive stamps must be monotonic-non-decreasing (HLC property).
    let first = t.hlc_stamp();
    let second = t.hlc_stamp();
    assert!(
        second >= first,
        "HLC stamps must be monotonically non-decreasing"
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn monotonic_source_trait_present() {
    let m: Box<dyn MonotonicSource> = Box::new(MockMono { start: 10 });
    assert_eq!(m.elapsed_since_start(), Duration::from_secs(10));
}

#[test]
fn monotonic_source_default_is_instant_wrapper() {
    // `benten_eval::default_monotonic_source()` returns the `Instant`-backed
    // wrapper. Two reads must be non-decreasing on monotonic time.
    let m = benten_eval::default_monotonic_source();
    let a = m.elapsed_since_start();
    let b = m.elapsed_since_start();
    assert!(
        b >= a,
        "std::time::Instant wrapper must be monotonic non-decreasing"
    );
}
