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
