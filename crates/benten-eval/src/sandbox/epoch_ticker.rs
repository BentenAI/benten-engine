//! D24 wallclock-axis epoch ticker (Phase 2b Wave-8b).
//!
//! wasmtime's epoch-interruption mechanism uses a monotonically-increasing
//! u64 epoch counter on the [`wasmtime::Engine`]. Per-call we set a
//! deadline via `Store::set_epoch_deadline(N)` where `N` is the epoch
//! count after which the runtime traps. Some external thread must
//! periodically advance the engine's epoch counter — that's what this
//! module owns.
//!
//! Cadence: D24 default 1ms (1000 ticks per second). Per-call wallclock
//! deadlines are expressed in milliseconds and converted to ticks via
//! [`epoch_ticks_for_ms`].
//!
//! Spawn discipline: process-wide singleton. The first call to
//! [`spawn_epoch_ticker`] starts a daemon thread that runs forever,
//! ticking [`shared_engine`](super::instance::shared_engine) every
//! [`EPOCH_TICK_INTERVAL`]. Subsequent calls are no-ops thanks to
//! [`OnceLock`].
//!
//! The ticker thread is a daemon — there's no shutdown channel because
//! the process lifetime is the only meaningful lifetime here. wasmtime's
//! Engine::increment_epoch is lock-free + cheap.
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// D24 epoch tick cadence — 10ms means a 30s wallclock budget needs
/// `3_000` epoch ticks. The default 30s wallclock budget cares about
/// ms-level precision only weakly; 10ms is plenty of resolution while
/// dramatically reducing the daemon ticker thread's idle-CPU footprint.
///
/// **wsa-w8b-3 fix-pass:** widened from 1ms (the original cadence) to
/// 10ms after a parallel-test-run flake on
/// `wait_signal_arrives_after_timeout_fires_e_wait_timeout`. Once any
/// test in the process triggers SANDBOX, the ticker thread runs forever
/// (process-wide singleton via [`OnceLock`]); a 1ms cadence burned
/// continuous CPU and competed with wall-clock-sensitive WAIT-timing
/// assertions. 10ms preserves correctness on every existing budget
/// fixture while eliminating the timing collision.
pub const EPOCH_TICK_INTERVAL: Duration = Duration::from_millis(10);

/// Convert a millisecond budget to epoch tick count using
/// [`EPOCH_TICK_INTERVAL`].
///
/// Saturating-arithmetic: a budget so large it would overflow u64
/// (effectively never trip) saturates at `u64::MAX`; a 0ms budget
/// returns 0 (which would trap immediately on the first tick — the
/// SANDBOX executor rejects 0 at config-construction time per
/// [`super::primitives_sandbox::SandboxConfig::with_wallclock_ms`]).
#[must_use]
pub fn epoch_ticks_for_ms(ms: u64) -> u64 {
    let interval_ms = u64::try_from(EPOCH_TICK_INTERVAL.as_millis()).unwrap_or(1);
    if interval_ms == 0 {
        return ms;
    }
    ms.saturating_mul(1).saturating_div(interval_ms.max(1))
        + u64::from(!ms.is_multiple_of(interval_ms))
}

static TICKER_HANDLE: OnceLock<JoinHandle<()>> = OnceLock::new();

/// Spawn the process-wide epoch ticker thread. Idempotent: subsequent
/// calls are no-ops (the [`OnceLock`] guards single-spawn).
///
/// The thread loops `sleep(EPOCH_TICK_INTERVAL); engine.increment_epoch()`
/// forever. Returns immediately on first call (after spawning); on
/// subsequent calls also returns immediately (the lock is already
/// initialized).
pub fn spawn_epoch_ticker() {
    let _ = TICKER_HANDLE.get_or_init(|| {
        thread::Builder::new()
            .name("benten-sandbox-epoch-ticker".to_string())
            .spawn(|| {
                loop {
                    thread::sleep(EPOCH_TICK_INTERVAL);
                    let engine = super::instance::shared_engine();
                    engine.increment_epoch();
                }
            })
            .expect("epoch ticker thread spawn failed")
    });
}

/// `true` if the ticker thread has been spawned (diagnostic).
#[must_use]
pub fn ticker_running() -> bool {
    TICKER_HANDLE.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_ticks_for_ms_basic_conversion() {
        // 10ms cadence (wsa-w8b-3) — 30s budget = 3_000 ticks.
        assert_eq!(epoch_ticks_for_ms(30_000), 3_000);
        // 1ms budget rounds up to 1 tick (partial-tick ceiling).
        assert_eq!(epoch_ticks_for_ms(1), 1);
        // 0ms budget = 0 ticks.
        assert_eq!(epoch_ticks_for_ms(0), 0);
    }

    #[test]
    fn epoch_ticks_for_ms_saturating_at_u64_max() {
        // u64::MAX should saturate, not overflow.
        let big = epoch_ticks_for_ms(u64::MAX);
        assert!(big > 0);
    }

    #[test]
    fn epoch_ticks_for_ms_rounds_up_for_partial_ticks() {
        // With a 10ms interval (wsa-w8b-3), a 1ms budget is a partial
        // tick that rounds up to 1; a 5ms budget likewise rounds up to 1.
        assert_eq!(epoch_ticks_for_ms(1), 1);
        assert_eq!(epoch_ticks_for_ms(5), 1);
        // A 10ms budget is a clean 1 tick.
        assert_eq!(epoch_ticks_for_ms(10), 1);
        // 11ms straddles boundaries: 1 full tick + 1 partial = 2.
        assert_eq!(epoch_ticks_for_ms(11), 2);
    }

    #[test]
    fn spawn_is_idempotent() {
        spawn_epoch_ticker();
        spawn_epoch_ticker();
        spawn_epoch_ticker();
        assert!(ticker_running());
    }
}
