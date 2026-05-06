//! Rate-limit policy plug (G14-B wave-4b; D-F + D-PHASE-3-26 +
//! D-PHASE-3-30).
//!
//! ## Architectural intent
//!
//! Per CLAUDE.md baked-in #7, the capability system is a pluggable
//! policy with a `CapabilityPolicy` pre-write hook. D-F (R1
//! capability-system reviewer) adds rate-limits as a Phase-3 general-
//! purpose plug via a separate trait — [`RateLimitPolicy`] — that the
//! durable UCAN backend calls into pre-write + the Atrium boundary
//! consults at peer-inbound chunk receipt.
//!
//! Three dimensions land at the trait surface (D-F + D-PHASE-3-26):
//!
//! - **Per-actor writes/sec/zone** — the `(actor_did, zone)` bucket
//!   the durable UCAN backend (or any consumer) checks pre-write to
//!   bound user-issued WRITE traffic.
//! - **Per-actor read-budget** — the typed-shape API for token-bucket
//!   read budgets; concrete enforcement lives at G14-D / G16 read-path
//!   wiring, but the plug surface lands now per D-PHASE-3-26.
//! - **Per-peer bandwidth bytes/sec** — Atrium-boundary defense
//!   against a malicious / buggy peer flooding the sync channel
//!   (per D-PHASE-3-30 thin-client edge consumption).
//!
//! ## G14-B scope
//!
//! G14-B lands the trait surface + a [`NullRateLimitPolicy`] that
//! always permits (the no-op default for backends that don't want
//! rate-limiting wired). A simple [`InMemoryRateLimitPolicy`] is
//! provided alongside for tests + the in-process default
//! configuration. Concrete production implementations (sliding-window
//! counters backed by durable state, distributed token-bucket across
//! Atrium peers) land at G14-D / G16 per the plan §3 row.
//!
//! ## Composition with [`crate::backends::ucan::UCANBackend`]
//!
//! `UCANBackend` carries an `Arc<dyn RateLimitPolicy>` (boxed because
//! the trait is `dyn`-safe; concrete instances of the in-memory impl
//! erase to the same Arc shape). `UCANBackend::pre_write_with_actor`
//! consults the policy before recording a grant; `record_grant`
//! itself stays untouched so engine-privileged paths skip the
//! rate-limit check (consistent with the [`crate::policy::WriteAuthority::EnginePrivileged`]
//! Inv-13 dispatch precedent).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::CapError;

/// Pluggable rate-limit policy trait. See module docs for the
/// architectural intent + composition with
/// [`crate::backends::ucan::UCANBackend`].
///
/// `dyn`-safe so the durable UCAN backend can carry an
/// `Arc<dyn RateLimitPolicy>` without forcing the generic-cascade
/// chain to grow another type parameter (per D-F D-PHASE-3-26 trait-
/// surface shape).
pub trait RateLimitPolicy: Send + Sync {
    /// Permit or deny a per-actor write at `zone`. Concrete impls
    /// account the call against a token-bucket / sliding-window /
    /// counter and either return `Ok(())` (within budget) or
    /// [`CapError::RateLimitExceeded`].
    ///
    /// # Errors
    ///
    /// Returns [`CapError::RateLimitExceeded`] when the actor's
    /// writes/sec/zone budget is exhausted.
    fn check_writes_per_sec(&self, actor: &str, zone: &str) -> Result<(), CapError>;

    /// Per-actor read-budget check. Plug surface only at G14-B —
    /// concrete enforcement lands at G14-D / G16 read-path wiring per
    /// D-PHASE-3-26 sub-track scoping.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::RateLimitExceeded`] when the actor's
    /// read budget is exhausted.
    fn check_read_budget(&self, actor: &str) -> Result<(), CapError>;

    /// Account `bytes` against `peer`'s inbound bandwidth budget.
    /// Atrium-boundary defense (D-F + D-PHASE-3-30 thin-client edge
    /// consumption).
    ///
    /// # Errors
    ///
    /// Returns [`CapError::PeerBandwidthExceeded`] when the per-peer
    /// budget is exhausted.
    fn check_peer_bandwidth(&self, peer: &str, bytes: usize) -> Result<(), CapError>;

    /// Probe the back-pressure flag for `peer`. Used by the sync
    /// engine to drain other peers while throttled peers wait for
    /// their bucket to refill.
    fn is_peer_back_pressured(&self, peer: &str) -> bool;
}

/// No-op rate-limit policy — every check passes. Default plug for
/// backends that don't wire rate-limiting; preserves the
/// `Option`-free shape on `UCANBackend` so the plug is always
/// callable without a `match` ladder.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullRateLimitPolicy;

impl NullRateLimitPolicy {
    /// Construct a fresh no-op policy.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl RateLimitPolicy for NullRateLimitPolicy {
    fn check_writes_per_sec(&self, _actor: &str, _zone: &str) -> Result<(), CapError> {
        Ok(())
    }

    fn check_read_budget(&self, _actor: &str) -> Result<(), CapError> {
        Ok(())
    }

    fn check_peer_bandwidth(&self, _peer: &str, _bytes: usize) -> Result<(), CapError> {
        Ok(())
    }

    fn is_peer_back_pressured(&self, _peer: &str) -> bool {
        false
    }
}

/// Sliding-window in-memory rate-limit policy with per-bucket budgets.
///
/// Implements 1-second sliding windows for both per-`(actor, zone)`
/// write counters and per-peer byte counters. Adequate for tests +
/// in-process default configuration; production deployments wanting
/// distributed enforcement land their own [`RateLimitPolicy`] impl
/// at G14-D / G16.
///
/// Use [`InMemoryRateLimitPolicy::builder`] to configure per-bucket
/// budgets. Actors / peers without a configured budget pass through
/// (`Ok(())`).
pub struct InMemoryRateLimitPolicy {
    actor_writes: HashMap<(String, String), u32>,
    actor_reads: HashMap<String, u32>,
    peer_bandwidth: HashMap<String, usize>,
    state: Mutex<RateLimitState>,
    /// Test-only clock override; in production the real
    /// `Instant::now()` drives the sliding window.
    clock: Box<dyn Fn() -> Instant + Send + Sync>,
}

impl std::fmt::Debug for InMemoryRateLimitPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryRateLimitPolicy")
            .field("actor_writes", &self.actor_writes)
            .field("actor_reads", &self.actor_reads)
            .field("peer_bandwidth", &self.peer_bandwidth)
            .field("state", &"<mutex>")
            .field("clock", &"<closure>")
            .finish()
    }
}

#[derive(Debug, Default)]
#[allow(
    clippy::struct_field_names,
    reason = "the `_buckets` postfix is load-bearing: each field stores a HashMap-of-Bucket per dimension, and the postfix disambiguates the per-dimension bucket map from the trait-shape (writes/reads/bandwidth). Per-dimension symmetry — `actor_write_buckets` / `actor_read_buckets` / `peer_bw_buckets` — reads naturally at every call site."
)]
struct RateLimitState {
    actor_write_buckets: HashMap<(String, String), Bucket<u32>>,
    actor_read_buckets: HashMap<String, Bucket<u32>>,
    peer_bw_buckets: HashMap<String, Bucket<usize>>,
}

#[derive(Debug)]
struct Bucket<T> {
    window_start: Instant,
    count: T,
    /// Sticky-within-window flag set when a check rejected against
    /// this bucket. Used by [`InMemoryRateLimitPolicy::is_peer_back_pressured`]
    /// so a saturated peer remains back-pressured for the rest of
    /// the window even after the rejected chunk leaves the bucket
    /// count untouched. Resets on window slide.
    saturated_in_window: bool,
}

impl InMemoryRateLimitPolicy {
    /// Construct a policy builder.
    #[must_use]
    pub fn builder() -> InMemoryRateLimitPolicyBuilder {
        InMemoryRateLimitPolicyBuilder::default()
    }

    /// Construct an empty policy (every check passes — equivalent to
    /// [`NullRateLimitPolicy`] but with the runtime mutex shape so
    /// callers wanting to swap in budgets later do not have to
    /// re-hand the policy across the engine).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            actor_writes: HashMap::new(),
            actor_reads: HashMap::new(),
            peer_bandwidth: HashMap::new(),
            state: Mutex::new(RateLimitState::default()),
            clock: Box::new(Instant::now),
        }
    }
}

/// Builder for [`InMemoryRateLimitPolicy`].
#[derive(Default)]
pub struct InMemoryRateLimitPolicyBuilder {
    actor_writes: HashMap<(String, String), u32>,
    actor_reads: HashMap<String, u32>,
    peer_bandwidth: HashMap<String, usize>,
    clock: Option<Box<dyn Fn() -> Instant + Send + Sync>>,
}

impl InMemoryRateLimitPolicyBuilder {
    /// Set the per-`(actor, zone)` writes/sec budget.
    #[must_use]
    pub fn actor_writes_per_second(
        mut self,
        actor: impl Into<String>,
        zone: impl Into<String>,
        budget: u32,
    ) -> Self {
        self.actor_writes
            .insert((actor.into(), zone.into()), budget);
        self
    }

    /// Set the per-actor read budget per second.
    #[must_use]
    pub fn actor_reads_per_second(mut self, actor: impl Into<String>, budget: u32) -> Self {
        self.actor_reads.insert(actor.into(), budget);
        self
    }

    /// Set the per-peer bandwidth budget in bytes/sec.
    #[must_use]
    pub fn peer_bandwidth_bytes_per_second(
        mut self,
        peer: impl Into<String>,
        bytes_per_sec: usize,
    ) -> Self {
        self.peer_bandwidth.insert(peer.into(), bytes_per_sec);
        self
    }

    /// Override the clock for tests so sliding-window boundaries can
    /// be deterministically advanced. Production paths leave this
    /// unset and the policy uses the real wallclock.
    #[must_use]
    pub fn with_clock(mut self, clock: impl Fn() -> Instant + Send + Sync + 'static) -> Self {
        self.clock = Some(Box::new(clock));
        self
    }

    /// Finalize the builder.
    #[must_use]
    pub fn build(self) -> InMemoryRateLimitPolicy {
        InMemoryRateLimitPolicy {
            actor_writes: self.actor_writes,
            actor_reads: self.actor_reads,
            peer_bandwidth: self.peer_bandwidth,
            state: Mutex::new(RateLimitState::default()),
            clock: self.clock.unwrap_or_else(|| Box::new(Instant::now)),
        }
    }
}

const WINDOW: Duration = Duration::from_secs(1);

impl RateLimitPolicy for InMemoryRateLimitPolicy {
    fn check_writes_per_sec(&self, actor: &str, zone: &str) -> Result<(), CapError> {
        let key = (actor.to_string(), zone.to_string());
        let Some(budget) = self.actor_writes.get(&key).copied() else {
            return Ok(());
        };
        let now = (self.clock)();
        let mut state = self.state.lock().expect("rate-limit state mutex poisoned");
        let bucket = state
            .actor_write_buckets
            .entry(key.clone())
            .or_insert_with(|| Bucket {
                window_start: now,
                count: 0,
                saturated_in_window: false,
            });
        if now.duration_since(bucket.window_start) >= WINDOW {
            bucket.window_start = now;
            bucket.count = 0;
            bucket.saturated_in_window = false;
        }
        if bucket.count >= budget {
            return Err(CapError::RateLimitExceeded {
                actor: actor.to_string(),
                zone: zone.to_string(),
            });
        }
        bucket.count = bucket.count.saturating_add(1);
        Ok(())
    }

    fn check_read_budget(&self, actor: &str) -> Result<(), CapError> {
        let Some(budget) = self.actor_reads.get(actor).copied() else {
            return Ok(());
        };
        let now = (self.clock)();
        let mut state = self.state.lock().expect("rate-limit state mutex poisoned");
        let bucket = state
            .actor_read_buckets
            .entry(actor.to_string())
            .or_insert_with(|| Bucket {
                window_start: now,
                count: 0,
                saturated_in_window: false,
            });
        if now.duration_since(bucket.window_start) >= WINDOW {
            bucket.window_start = now;
            bucket.count = 0;
            bucket.saturated_in_window = false;
        }
        if bucket.count >= budget {
            return Err(CapError::RateLimitExceeded {
                actor: actor.to_string(),
                zone: String::new(),
            });
        }
        bucket.count = bucket.count.saturating_add(1);
        Ok(())
    }

    fn check_peer_bandwidth(&self, peer: &str, bytes: usize) -> Result<(), CapError> {
        let Some(budget) = self.peer_bandwidth.get(peer).copied() else {
            return Ok(());
        };
        let now = (self.clock)();
        let mut state = self.state.lock().expect("rate-limit state mutex poisoned");
        let bucket = state
            .peer_bw_buckets
            .entry(peer.to_string())
            .or_insert_with(|| Bucket {
                window_start: now,
                count: 0_usize,
                saturated_in_window: false,
            });
        if now.duration_since(bucket.window_start) >= WINDOW {
            bucket.window_start = now;
            bucket.count = 0;
            bucket.saturated_in_window = false;
        }
        let new_total = bucket.count.saturating_add(bytes);
        if new_total > budget {
            bucket.saturated_in_window = true;
            return Err(CapError::PeerBandwidthExceeded {
                peer: peer.to_string(),
                bytes,
            });
        }
        bucket.count = new_total;
        Ok(())
    }

    fn is_peer_back_pressured(&self, peer: &str) -> bool {
        let Some(budget) = self.peer_bandwidth.get(peer).copied() else {
            return false;
        };
        let now = (self.clock)();
        let state = self.state.lock().expect("rate-limit state mutex poisoned");
        let Some(bucket) = state.peer_bw_buckets.get(peer) else {
            return false;
        };
        // If the window is fresh (no traffic this second), not
        // back-pressured.
        if now.duration_since(bucket.window_start) >= WINDOW {
            return false;
        }
        // Sticky in-window saturation flag covers the case where an
        // over-budget chunk rejected (leaving the bucket count
        // unchanged but the peer should still be back-pressured for
        // the remainder of the window). Pure count-based check
        // alone misses this — see `cross_peer_back_pressure_via_rate_limit_policy`.
        bucket.saturated_in_window || bucket.count >= budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_policy_allows_everything() {
        let p = NullRateLimitPolicy::new();
        p.check_writes_per_sec("a", "/zone/posts").unwrap();
        p.check_read_budget("a").unwrap();
        p.check_peer_bandwidth("peer-a", 1_000_000).unwrap();
        assert!(!p.is_peer_back_pressured("peer-a"));
    }

    #[test]
    fn empty_policy_passes_unconfigured_actors() {
        let p = InMemoryRateLimitPolicy::empty();
        // No buckets configured — passes:
        p.check_writes_per_sec("a", "/zone/posts").unwrap();
        p.check_peer_bandwidth("peer-x", 100).unwrap();
    }

    #[test]
    fn writes_per_sec_budget_exhausts() {
        let p = InMemoryRateLimitPolicy::builder()
            .actor_writes_per_second("did:test:a", "/zone/posts", 3)
            .build();
        for _ in 0..3 {
            p.check_writes_per_sec("did:test:a", "/zone/posts").unwrap();
        }
        let err = p
            .check_writes_per_sec("did:test:a", "/zone/posts")
            .unwrap_err();
        assert!(matches!(err, CapError::RateLimitExceeded { .. }));
    }
}
