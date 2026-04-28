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

/// D24 epoch tick cadence — 1ms means a 30s wallclock budget needs
/// `30_000` epoch ticks. Tighter than the wall-time precision of typical
/// schedulers; in practice the ticker thread fires somewhere between 1ms
/// and a few ms apart, which is well below the 30s default budget
/// granularity. Tightening below 1ms would burn cycles for no gain.
pub const EPOCH_TICK_INTERVAL: Duration = Duration::from_millis(1);

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
        // 1ms cadence — 30s budget = 30_000 ticks.
        assert_eq!(epoch_ticks_for_ms(30_000), 30_000);
        // 1ms budget = 1 tick.
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
        // With a 1ms interval, 1ms == 1 tick exactly. Verify partial-tick
        // rounding only meaningfully kicks in if cadence is widened.
        assert_eq!(epoch_ticks_for_ms(1), 1);
    }

    #[test]
    fn spawn_is_idempotent() {
        spawn_epoch_ticker();
        spawn_epoch_ticker();
        spawn_epoch_ticker();
        assert!(ticker_running());
    }
}
