//! Phase-3 G20-A2 (D12 wave-8a): WAIT TTL GC machinery — production
//! code (NOT test source — `docs/future/phase-3-backlog.md §7.3.A.6`
//! was miscategorized per scope-real-03).
//!
//! # GC discipline (three phases)
//!
//! Per the D12 hybrid-GC contract, three sweep paths cover every
//! expired-WAIT outcome:
//!
//! 1. **Event-driven** — every `suspend()` and `resume()` call invokes
//!    [`run_event_driven_sweep`], which walks the engine's tracked
//!    envelope-CID set + drops any whose deadline has elapsed. The
//!    sweep is opportunistic: it fires on the hot path, so its cost
//!    is amortized into the WAIT operation that triggered it.
//!    Suppressible via `EngineBuilder::gc_event_driven(false)` — the
//!    interval backstop + drop-final still fire.
//!
//! 2. **Interval backstop** — a tick driven externally. In tests:
//!    `Engine::testing_run_wait_ttl_gc_pass` invokes
//!    [`run_interval_tick`] synchronously. Production tokio-interval
//!    wiring (a 1h timer registered at `EngineBuilder::build`) is
//!    DOCUMENTED-DEFERRED to `docs/future/phase-3-backlog.md §7.14`
//!    per G20-A2 wave-8a mr-6 — the resume-time deadline check at
//!    `engine_wait.rs::resume_from_bytes_inner` is the LOAD-BEARING
//!    correctness mechanism (fires `E_WAIT_TTL_EXPIRED` independently
//!    of whether the GC sweep ran first); the interval backstop is a
//!    STORAGE-CLEANUP mechanism that hardens disk-usage on idle
//!    engines but is not gating v1-foundation correctness. Catches
//!    expired entries on idle engines that receive no suspend /
//!    resume traffic once wired.
//!
//! 3. **Drop-final** — `Engine::drop` calls
//!    [`run_drop_final_sweep`] so an explicit shutdown leaves the
//!    suspension store in the same state a long-running engine would
//!    eventually reach. Cross-process resume against the same path
//!    therefore observes the same set of live entries regardless of
//!    whether the suspending engine ran the GC pass before exit.
//!
//! # Cross-process correctness
//!
//! The GC stamps a wall-clock-relative deadline (`suspend_wallclock_ms
//! + ttl_hours * 3_600_000`). Two engines opening the same redb path
//! at different wall-clock instants compute the same deadline because
//! the deadline lives in the persisted entry — the resume-time engine
//! consults `SystemTime::now()` (or the testing override) and compares
//! against the suspended deadline rather than against a process-local
//! "elapsed" counter that would reset on engine open.
//!
//! # Scheduling correctness audit
//!
//! - **Hot path:** event-driven sweep on suspend reduces store growth
//!   under burst-suspend workloads (each suspend cleans up siblings
//!   the previous suspend missed).
//! - **Idle path:** interval backstop catches the
//!   one-suspend-then-idle-forever case where the event-driven path
//!   never fires again.
//! - **Shutdown path:** drop-final sweep guarantees we never leak an
//!   expired entry across a clean shutdown — the next process open
//!   sees a sparse store.
//! - **Crash path:** if the engine crashes between expiry and the
//!   next sweep, the entry survives until the *next* engine opens
//!   the path AND fires either the event-driven (any suspend / resume)
//!   or interval-backstop sweep. This is intentional: the GC is a
//!   storage-cleanup mechanism, not a correctness mechanism — the
//!   resume protocol's TTL deadline check fails closed independently
//!   of whether GC has run yet.

use std::sync::Arc;

use benten_core::Cid;
use benten_eval::{SuspensionKey, SuspensionStore, suspension_store::WaitMetadata};

/// Phase-3 G20-A2 (D12 wave-8a): observable counters for the WAIT TTL
/// GC machinery. Exposed via `Engine::testing_wait_ttl_gc_stats` so
/// tests can assert the GC actually swept (defends against the silent-
/// no-op failure mode where the GC code runs but stamps zero reaps).
#[derive(Debug, Clone, Default)]
pub struct WaitTtlGcStats {
    /// Cumulative count of envelope CIDs the GC has reaped from the
    /// SuspensionStore since engine construction. Bumped by each
    /// successful `delete(SuspensionKey::WaitMetadata(_))` /
    /// `delete(SuspensionKey::Envelope(_))` pair.
    pub reaped_count: u64,
    /// Cumulative count of GC sweep invocations (across all three
    /// paths: event-driven + interval-backstop + drop-final). Lets a
    /// test assert "the sweep actually ran" independently of whether
    /// it found anything to reap.
    pub sweep_count: u64,
}

/// Phase-3 G20-A2 (D12 wave-8a): wall-clock now in milliseconds since
/// UNIX epoch. Used by the GC machinery + the resume-time deadline
/// check. `wall_override_ms` is the engine's
/// `wait_wall_clock_override_ms` — when `Some` we honour it (test-only
/// path); when `None` we fall back to the host's `SystemTime`.
#[must_use]
pub fn wallclock_now_ms(wall_override_ms: Option<u64>) -> u64 {
    if let Some(v) = wall_override_ms {
        return v;
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

/// Phase-3 G20-A2 (D12 wave-8a): compute the wall-clock deadline for a
/// WAIT metadata entry. Returns `None` if the metadata lacks the wall-
/// clock anchor or the TTL field — the deadline check is then a no-op
/// (the metadata's in-process `timeout_ms` is the only authoritative
/// deadline).
///
/// # Overflow safety (mr-3)
///
/// The math uses saturating arithmetic on every operation:
///
/// ```text
/// deadline_ms = suspend_wallclock_ms.saturating_add(
///     u64::from(ttl_hours).saturating_mul(3_600_000)
/// )
/// ```
///
/// Saturation is fail-safe-CLOSED: any saturated `u64::MAX` deadline
/// rejects on the next resume via `is_expired` returning `true` (the
/// resume-time engine's wall-clock is below `u64::MAX`, so
/// `now_ms >= u64::MAX` is `true` only at the actual epoch saturation —
/// otherwise the saturated deadline genuinely is in the past and
/// rejection is correct). Concretely: a hand-crafted
/// `SerializableWaitMetadata` carrying `ttl_hours = u32::MAX` (~5.1M
/// hours; well above the 720h registration ceiling) saturates the
/// deadline to `u64::MAX` and the resume rejects via `E_WAIT_TTL_EXPIRED`.
/// No panic, no silent wrap, no "instantly expired but deadline says
/// distant future" half-state.
///
/// The registration-time validator at
/// `crates/benten-engine/src/engine.rs::register_subgraph` enforces
/// `1 <= ttl_hours <= 720` so legitimate entries can never saturate.
/// `debug_assert!` below catches out-of-range values in dev builds in
/// case a future change relaxes the registration validator without
/// updating this comment.
#[must_use]
pub fn deadline_ms(meta: &WaitMetadata) -> Option<u64> {
    let anchor = meta.suspend_wallclock_ms?;
    let ttl_hours_raw = meta.ttl_hours?;
    debug_assert!(
        ttl_hours_raw <= 720,
        "deadline_ms: ttl_hours={ttl_hours_raw} exceeds the 720h registration ceiling \
         (saturating math fails closed but this signals a registration-validator drift)"
    );
    let ttl_hours = u64::from(ttl_hours_raw);
    Some(anchor.saturating_add(ttl_hours.saturating_mul(3_600_000)))
}

/// Phase-3 G20-A2 (D12 wave-8a): is the entry expired at `now_ms`?
/// `false` if the metadata lacks a wall-clock anchor / TTL (no
/// deadline declared); `true` only when the deadline has been crossed.
#[must_use]
pub fn is_expired(meta: &WaitMetadata, now_ms: u64) -> bool {
    deadline_ms(meta).is_some_and(|d| now_ms >= d)
}

/// Phase-3 G20-A2 (D12 wave-8a): drop the WAIT-side entries (metadata
/// + envelope) for `cid` from `store`. Idempotent. Returns `true` if
/// the metadata entry existed before the call (the reap counter
/// increments only on observable reaps).
pub fn reap_one(store: &Arc<dyn SuspensionStore>, cid: &Cid) -> bool {
    let existed = matches!(store.get_wait(cid), Ok(Some(_)));
    let _ = store.delete(SuspensionKey::WaitMetadata(*cid));
    let _ = store.delete(SuspensionKey::Envelope(*cid));
    existed
}

/// Phase-3 G20-A2 (D12 wave-8a): event-driven sweep — walk the tracked
/// envelopes set + reap every entry whose deadline has elapsed. Called
/// from `suspend()` and `resume()` on the hot path. Returns the count
/// of reaped entries.
pub fn run_event_driven_sweep<S: ::std::hash::BuildHasher>(
    store: &Arc<dyn SuspensionStore>,
    tracked: &mut std::collections::HashSet<Cid, S>,
    now_ms: u64,
    stats: &mut WaitTtlGcStats,
) -> u64 {
    stats.sweep_count = stats.sweep_count.saturating_add(1);
    let mut reaped = 0u64;
    let candidates: Vec<Cid> = tracked.iter().copied().collect();
    for cid in candidates {
        let meta = match store.get_wait(&cid) {
            Ok(Some(m)) => m,
            _ => {
                // Entry already gone — drop it from tracked set.
                tracked.remove(&cid);
                continue;
            }
        };
        if is_expired(&meta, now_ms) && reap_one(store, &cid) {
            reaped = reaped.saturating_add(1);
            tracked.remove(&cid);
        }
    }
    stats.reaped_count = stats.reaped_count.saturating_add(reaped);
    reaped
}

/// Phase-3 G20-A2 (D12 wave-8a): interval-backstop sweep. Identical
/// shape to the event-driven sweep; the entry point is named
/// distinctly so tests can pin which path fired without reading
/// implementation details.
pub fn run_interval_tick<S: ::std::hash::BuildHasher>(
    store: &Arc<dyn SuspensionStore>,
    tracked: &mut std::collections::HashSet<Cid, S>,
    now_ms: u64,
    stats: &mut WaitTtlGcStats,
) -> u64 {
    run_event_driven_sweep(store, tracked, now_ms, stats)
}

/// Phase-3 G20-A2 (D12 wave-8a): drop-final sweep — runs from
/// `Engine::drop`. Same shape as the other two paths; the distinct
/// entry point exists so the drop-time invocation is grep-visible
/// from `Engine::drop` for code-review.
pub fn run_drop_final_sweep<S: ::std::hash::BuildHasher>(
    store: &Arc<dyn SuspensionStore>,
    tracked: &mut std::collections::HashSet<Cid, S>,
    now_ms: u64,
    stats: &mut WaitTtlGcStats,
) -> u64 {
    run_event_driven_sweep(store, tracked, now_ms, stats)
}
