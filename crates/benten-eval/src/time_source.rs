//! Phase 2a G3-B / T1 / sec-r1-2: `TimeSource` + `MonotonicSource` traits.
//!
//! Dual-source per §9.13 resolution: `MonotonicSource` drives TOCTOU
//! refresh cadence (drift-exploit-hard); `TimeSource` is consulted
//! alongside for HLC federation-correlation context.
//!
//! TODO(phase-2a-G3-B): back the defaults with `uhlc::HLC` (TimeSource)
//! and `std::time::Instant` (MonotonicSource).

use core::time::Duration;

/// HLC stamp source. Default impl wraps `uhlc::HLC`.
pub trait TimeSource: Send + Sync {
    /// Return the current HLC stamp.
    fn hlc_stamp(&self) -> u64;
}

/// Monotonic clock source. Default impl wraps `std::time::Instant`.
pub trait MonotonicSource: Send + Sync {
    /// Elapsed since the source was constructed.
    fn elapsed_since_start(&self) -> Duration;
}

/// `uhlc::HLC`-backed default `TimeSource`.
pub struct HlcTimeSource {
    // Phase-1 placeholder: use a monotonic counter so two calls differ.
    counter: std::sync::atomic::AtomicU64,
}

impl HlcTimeSource {
    /// Construct a new default source.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl Default for HlcTimeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSource for HlcTimeSource {
    fn hlc_stamp(&self) -> u64 {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

/// `std::time::Instant`-backed default `MonotonicSource`.
pub struct InstantMonotonicSource {
    start: std::time::Instant,
}

impl InstantMonotonicSource {
    /// Construct a source that measures elapsed time from this instant.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Default for InstantMonotonicSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicSource for InstantMonotonicSource {
    fn elapsed_since_start(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Default production `TimeSource`.
#[must_use]
pub fn default_time_source() -> Box<dyn TimeSource> {
    Box::new(HlcTimeSource::new())
}

/// Default production `MonotonicSource`.
#[must_use]
pub fn default_monotonic_source() -> Box<dyn MonotonicSource> {
    Box::new(InstantMonotonicSource::new())
}

/// Test-only mock clock. Shared `Arc<Mutex<...>>` state so multiple clones
/// observe `advance` changes.
#[derive(Clone)]
pub struct MockTimeSource {
    inner: std::sync::Arc<std::sync::Mutex<Duration>>,
}

impl MockTimeSource {
    /// Construct a mock clock pinned at the given duration.
    #[must_use]
    pub fn at(initial: Duration) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(initial)),
        }
    }

    /// Advance the clock by the given duration.
    pub fn advance(&self, by: Duration) {
        if let Ok(mut g) = self.inner.lock() {
            *g += by;
        }
    }

    /// Read the current duration.
    pub fn elapsed(&self) -> Duration {
        self.inner.lock().map_or(Duration::ZERO, |g| *g)
    }
}

impl TimeSource for MockTimeSource {
    fn hlc_stamp(&self) -> u64 {
        // Represent the elapsed duration in microseconds as the HLC stamp
        // stand-in. Purely for tests; the real HLC is injected separately.
        u64::try_from(self.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}

// ---------------------------------------------------------------------------
// G9-A-cont test helpers (Phase 2a G11-A Wave 1)
//
// The evaluator wallclock-refresh TOCTOU tests (wallclock_refresh_uses_
// monotonic_only / wallclock_refresh_ntp_slew_doesnt_skip) require a
// mockable `MonotonicSource` whose elapsed returns are caller-driven, plus
// a `MockTimeSource` constructor whose name ("frozen at epoch") reads at
// the call site as "the wall-clock never advances". Kept under the
// `testing` feature so release builds don't see the mock surface.
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "testing"))]
impl MockTimeSource {
    /// Frozen wall-clock mock used by `wallclock_refresh_uses_monotonic_only`:
    /// every `hlc_stamp()` call returns the same value because the
    /// underlying duration never changes. Semantically equivalent to
    /// `MockTimeSource::at(Duration::ZERO)` but reads at the call-site as
    /// "frozen at epoch", which is the language the sec-r1-2 test spec
    /// uses.
    #[must_use]
    pub fn frozen_at_epoch() -> Self {
        Self::at(Duration::ZERO)
    }

    /// Mock wall-clock pinned at an explicit HLC stamp. Accepts the raw
    /// HLC counter value (microseconds); lets the ntp-slew test rewind
    /// the clock to a specific point before advancing it forward again
    /// via `advance`.
    #[must_use]
    pub fn at_epoch(hlc_stamp: u64) -> Self {
        Self::at(Duration::from_micros(hlc_stamp))
    }

    /// Rewind the mock wall-clock backward by the given duration. Used by
    /// the NTP-slew test to simulate an adversarial backward jump. Saturates
    /// at `Duration::ZERO` so the mock never underflows.
    pub fn rewind_by(&self, by: Duration) {
        if let Ok(mut g) = self.inner.lock() {
            *g = g.saturating_sub(by);
        }
    }
}

/// Test-only mock `MonotonicSource` whose `elapsed_since_start` is
/// caller-driven. Cloneable + shared-state so the test harness and the
/// Engine can both observe advances.
///
/// Lives in `benten-eval` alongside `MockTimeSource` so the G9-A-cont
/// wallclock-refresh tests have a single import point; gated behind the
/// `testing` feature so release builds don't see it.
#[cfg(any(test, feature = "testing"))]
#[derive(Clone)]
pub struct MockMonotonicSource {
    inner: std::sync::Arc<std::sync::Mutex<Duration>>,
}

#[cfg(any(test, feature = "testing"))]
impl MockMonotonicSource {
    /// Construct a mock monotonic source pinned at zero elapsed.
    #[must_use]
    pub fn at_zero() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(Duration::ZERO)),
        }
    }

    /// Advance the mock's elapsed by `by`. Callers use this to jump the
    /// clock forward in one step.
    pub fn advance(&self, by: Duration) {
        if let Ok(mut g) = self.inner.lock() {
            *g += by;
        }
    }

    /// Read the current elapsed.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.inner.lock().map_or(Duration::ZERO, |g| *g)
    }
}

#[cfg(any(test, feature = "testing"))]
impl MonotonicSource for MockMonotonicSource {
    fn elapsed_since_start(&self) -> Duration {
        self.inner.lock().map_or(Duration::ZERO, |g| *g)
    }
}
