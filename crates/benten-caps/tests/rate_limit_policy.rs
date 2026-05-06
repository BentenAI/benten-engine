//! G14-B rate-limit policy plug integration tests (D-F +
//! D-PHASE-3-26 + D-PHASE-3-30).
//!
//! Source pins (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-B
//! + §7 rate-limit landscape):
//!
//! - `capability_policy_per_actor_write_rate_limit_enforced` — D-F + D-PHASE-3-26 (unit)
//! - `capability_policy_per_peer_bandwidth_budget_at_atrium_boundary` — D-F + D-PHASE-3-26 (unit)
//! - `cross_peer_back_pressure_via_rate_limit_policy` — D-F (unit)
//! - `rate_limit_policy_plug_per_actor_writes_sec_per_zone` — D-F + D-PHASE-3-26 (unit)
//! - `rate_limit_policy_plug_per_peer_bandwidth_budget_at_atrium_boundary` — D-F (unit)
//!
//! ## Architectural intent
//!
//! Per CLAUDE.md baked-in #7, capability system is a pluggable
//! policy with `CapabilityPolicy` pre-write hook. D-F (R1
//! capability-system) adds rate-limits as a Phase-3 general-purpose
//! plug — separate `RateLimitPolicy` trait composed alongside
//! `CapabilityPolicy`:
//!
//! - per-actor writes/sec/zone enforcement
//! - per-peer bandwidth budget at Atrium boundary
//! - cross-peer back-pressure surfacing
//!
//! G14-B lands the policy plug in `benten-caps`; G16-A consumes it
//! at the Atrium boundary; G16-B consumes for Loro merge throttling
//! (`crates/benten-sync/tests/rate_limit_consumption.rs` lands at
//! G16-A canary).
//!
//! ## Atrium-boundary integration coordination
//!
//! `capability_policy_per_peer_bandwidth_budget_at_atrium_boundary`
//! lands here as a UNIT test driving the
//! [`benten_caps::InMemoryRateLimitPolicy`] plug surface directly.
//! The cross-crate "consumed by Atrium" integration test lands at
//! G16-A wave-5b in
//! `crates/benten-sync/tests/rate_limit_consumption.rs` per the R3-B
//! coordination note.

#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use benten_caps::{CapError, InMemoryRateLimitPolicy, RateLimitPolicy};

/// Test clock helper — advances on demand so the 1s sliding-window
/// boundary is observable without sleeping.
#[derive(Clone)]
struct TestClock {
    inner: Arc<Mutex<Instant>>,
}

impl TestClock {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Instant::now())),
        }
    }

    fn advance(&self, by: Duration) {
        let mut g = self.inner.lock().unwrap();
        *g += by;
    }

    fn closure(&self) -> impl Fn() -> Instant + Send + Sync + 'static {
        let inner = self.inner.clone();
        move || *inner.lock().unwrap()
    }
}

#[test]
fn capability_policy_per_actor_write_rate_limit_enforced() {
    // D-F + D-PHASE-3-26 pin. The rate-limit policy plug enforces
    // per-actor (DID-keyed) writes/second/zone. Bursts above the
    // limit fire CapError::RateLimitExceeded.
    let actor = "did:test:actor-a";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_writes_per_second(actor, "/zone/posts", 10)
        .build();

    for i in 0..10 {
        policy
            .check_writes_per_sec(actor, "/zone/posts")
            .unwrap_or_else(|e| panic!("write {i} within budget unexpectedly rejected: {e:?}"));
    }
    let err = policy
        .check_writes_per_sec(actor, "/zone/posts")
        .unwrap_err();
    assert!(
        matches!(err, CapError::RateLimitExceeded { .. }),
        "11th write within 1-second window MUST reject; got {err:?}"
    );
}

#[test]
fn capability_policy_per_peer_bandwidth_budget_at_atrium_boundary() {
    // D-F + D-PHASE-3-26 cross-wave pin (UNIT shape — driving the
    // plug surface directly). The full Atrium-boundary integration
    // test lands at G16-A wave-5b in
    // `crates/benten-sync/tests/rate_limit_consumption.rs`.
    let peer = "did:test:peer-a";
    let policy = InMemoryRateLimitPolicy::builder()
        .peer_bandwidth_bytes_per_second(peer, 1_000_000)
        .build();

    policy.check_peer_bandwidth(peer, 500_000).unwrap();
    let err = policy.check_peer_bandwidth(peer, 800_000).unwrap_err();
    assert!(
        matches!(err, CapError::PeerBandwidthExceeded { .. }),
        "over-budget chunk MUST reject; got {err:?}"
    );
}

#[test]
fn cross_peer_back_pressure_via_rate_limit_policy() {
    // D-F pin. When peer A's bandwidth budget is exhausted, the
    // policy MUST surface back-pressure to the local sync engine so
    // it stops accepting inbound work from A while still serving
    // peers B + C.
    let peer_a = "did:test:peer-a";
    let peer_b = "did:test:peer-b";
    let policy = InMemoryRateLimitPolicy::builder()
        .peer_bandwidth_bytes_per_second(peer_a, 1_000_000)
        .peer_bandwidth_bytes_per_second(peer_b, 1_000_000)
        .build();

    // Saturate peer A — first 800k passes, second 800k tips over.
    policy.check_peer_bandwidth(peer_a, 800_000).unwrap();
    let _ = policy.check_peer_bandwidth(peer_a, 800_000);
    assert!(
        policy.is_peer_back_pressured(peer_a),
        "peer A MUST be back-pressured after saturating its budget"
    );
    assert!(
        !policy.is_peer_back_pressured(peer_b),
        "peer B is independent — MUST NOT be back-pressured"
    );
    // Peer B remains within budget.
    policy.check_peer_bandwidth(peer_b, 500_000).unwrap();
}

#[test]
fn rate_limit_policy_plug_per_actor_writes_sec_per_zone() {
    // D-F + D-PHASE-3-26 pin. Plug surface accepts (actor_did, zone,
    // writes_per_second) triples; per-(actor, zone) bucket isolation.
    let clock = TestClock::new();
    let actor_a = "did:test:actor-a";
    let actor_b = "did:test:actor-b";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_writes_per_second(actor_a, "/zone/posts", 10)
        .actor_writes_per_second(actor_a, "/zone/admin", 1)
        .actor_writes_per_second(actor_b, "/zone/posts", 100)
        .with_clock(clock.closure())
        .build();

    // Each (actor, zone) pair has its own bucket.
    for _ in 0..10 {
        policy.check_writes_per_sec(actor_a, "/zone/posts").unwrap();
    }
    policy.check_writes_per_sec(actor_a, "/zone/admin").unwrap();
    for _ in 0..50 {
        policy.check_writes_per_sec(actor_b, "/zone/posts").unwrap();
    }

    // Actor A's posts bucket exhausted; admin bucket exhausted at 1.
    assert!(policy.check_writes_per_sec(actor_a, "/zone/posts").is_err());
    assert!(
        policy.check_writes_per_sec(actor_a, "/zone/admin").is_err(),
        "actor A admin bucket (budget=1) MUST exhaust on second call"
    );

    // Actor B's posts bucket has 50 writes left; the next 50 all pass.
    for _ in 0..50 {
        policy.check_writes_per_sec(actor_b, "/zone/posts").unwrap();
    }
    // 101st rejects.
    assert!(policy.check_writes_per_sec(actor_b, "/zone/posts").is_err());

    // Window slide: advance the clock past 1s and the buckets reset.
    clock.advance(Duration::from_millis(1_100));
    policy.check_writes_per_sec(actor_a, "/zone/posts").unwrap();
}

#[test]
fn rate_limit_policy_plug_per_peer_bandwidth_budget_at_atrium_boundary() {
    // D-F pin. Plug surface for per-peer bandwidth budget; per-peer
    // heterogeneous budgets isolate at the policy layer.
    let peer_a = "did:test:peer-a";
    let peer_b = "did:test:peer-b";
    let policy = InMemoryRateLimitPolicy::builder()
        .peer_bandwidth_bytes_per_second(peer_a, 1_000_000)
        .peer_bandwidth_bytes_per_second(peer_b, 100_000)
        .build();

    policy.check_peer_bandwidth(peer_a, 800_000).unwrap();
    let err_b = policy.check_peer_bandwidth(peer_b, 200_000).unwrap_err();
    assert!(
        matches!(err_b, CapError::PeerBandwidthExceeded { .. }),
        "peer B over budget MUST reject"
    );
    assert!(!policy.is_peer_back_pressured(peer_a));
    assert!(policy.is_peer_back_pressured(peer_b));
}

// =====================================================================
// Bucket-state-isolation regression: a failing write under one actor
// MUST NOT change another actor's bucket. Defends against the
// "single global bucket" footgun that
// `feedback_3_plus_recurrence_deep_sweep` names as the kind of error
// the per-(actor, zone) bucket isolation pin guards. Independent test
// so a future refactor that conflates the buckets can't sneak past
// the basic exhaustion test above.
// =====================================================================

#[test]
fn rate_limit_per_actor_zone_buckets_isolated_under_failure() {
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_writes_per_second("did:test:loud-actor", "/zone/posts", 1)
        .actor_writes_per_second("did:test:quiet-actor", "/zone/posts", 1)
        .build();
    // Loud actor exhausts first.
    policy
        .check_writes_per_sec("did:test:loud-actor", "/zone/posts")
        .unwrap();
    let _ = policy.check_writes_per_sec("did:test:loud-actor", "/zone/posts");
    // Quiet actor's budget MUST remain untouched.
    policy
        .check_writes_per_sec("did:test:quiet-actor", "/zone/posts")
        .expect("quiet actor's bucket MUST be independent of loud actor's exhaustion");
}

// =====================================================================
// Read-budget axis — symmetric coverage to writes_per_sec (per mini-
// review g14b-mr-4). InMemoryRateLimitPolicy::check_read_budget is
// fully implemented; these tests pin the three exhaustion / slide /
// isolation invariants the writes_per_sec axis already pins. Concrete
// enforcement at G14-D / G16 read-path wiring; these tests pin the
// plug surface NOW so a future refactor of the bucket internals can't
// silently regress the read budget.
// =====================================================================

#[test]
fn read_budget_under_limit_passes() {
    let actor = "did:test:reader-a";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_reads_per_second(actor, 5)
        .build();
    for i in 0..5 {
        policy
            .check_read_budget(actor)
            .unwrap_or_else(|e| panic!("read {i} within budget unexpectedly rejected: {e:?}"));
    }
}

#[test]
fn read_budget_at_limit_passes_then_next_call_rejects() {
    let actor = "did:test:reader-a";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_reads_per_second(actor, 3)
        .build();
    for _ in 0..3 {
        policy.check_read_budget(actor).unwrap();
    }
    // 4th call MUST reject — bucket exhausted.
    let err = policy.check_read_budget(actor).unwrap_err();
    assert!(
        matches!(err, CapError::RateLimitExceeded { .. }),
        "4th read within 1-second window MUST reject; got {err:?}"
    );
}

#[test]
fn read_budget_resets_after_window_slide() {
    let clock = TestClock::new();
    let actor = "did:test:reader-a";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_reads_per_second(actor, 2)
        .with_clock(clock.closure())
        .build();
    // Exhaust the bucket.
    policy.check_read_budget(actor).unwrap();
    policy.check_read_budget(actor).unwrap();
    assert!(
        policy.check_read_budget(actor).is_err(),
        "post-budget call MUST reject"
    );
    // Slide past the 1-second window — bucket resets.
    clock.advance(Duration::from_millis(1_100));
    policy
        .check_read_budget(actor)
        .expect("post-window-slide call MUST pass — bucket resets on window boundary");
}

#[test]
fn read_budget_per_actor_buckets_isolated() {
    let actor_a = "did:test:reader-a";
    let actor_b = "did:test:reader-b";
    let policy = InMemoryRateLimitPolicy::builder()
        .actor_reads_per_second(actor_a, 1)
        .actor_reads_per_second(actor_b, 1)
        .build();
    // Actor A exhausts first.
    policy.check_read_budget(actor_a).unwrap();
    let _ = policy.check_read_budget(actor_a);
    // Actor B's budget MUST remain untouched (isolation across actors).
    policy
        .check_read_budget(actor_b)
        .expect("actor B's read bucket MUST be independent of actor A's exhaustion");
}
