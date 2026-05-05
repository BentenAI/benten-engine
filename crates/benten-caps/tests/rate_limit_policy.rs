//! R3-B RED-PHASE pins for `benten-caps` rate-limit CapabilityPolicy
//! plug (G14-B wave-4b; D-F + D-PHASE-3-26).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-B
//! + §7 rate-limit landscape):
//!
//! - `tests/capability_policy_per_actor_write_rate_limit_enforced` — D-F + D-PHASE-3-26 (unit)
//! - `tests/capability_policy_per_peer_bandwidth_budget_at_atrium_boundary` — D-F + D-PHASE-3-26 (G14-B + G16-A integration)
//! - `tests/cross_peer_back_pressure_via_rate_limit_policy` — D-F (G14-B + G16-A integration)
//! - `tests/rate_limit_policy_plug_per_actor_writes_sec_per_zone` — D-F + D-PHASE-3-26 (unit)
//! - `tests/rate_limit_policy_plug_per_peer_bandwidth_budget_at_atrium_boundary` — D-F (unit)
//!
//! ## Architectural intent
//!
//! Per CLAUDE.md baked-in #7, capability system is a pluggable policy
//! with `CapabilityPolicy` pre-write hook. D-F (R1 capability-system)
//! adds rate-limits as a Phase-3 general-purpose plug:
//! - per-actor writes/sec/zone enforcement
//! - per-peer bandwidth budget at Atrium boundary
//! - cross-peer back-pressure surfacing
//!
//! G14-B lands the policy plug in `benten-caps`; G16-A consumes it at
//! the Atrium boundary; G16-B consumes for Loro merge throttling.
//!
//! ## Coordination note
//!
//! `rate_limit_policy_consumed_by_g16_b_loro_merge_throttle` lives at
//! `crates/benten-sync/tests/rate_limit_consumption.rs` (R3-C territory
//! since the `benten-sync` crate doesn't exist at R3-B landing time).
//! R3-C creates that file when the G16-A canary lands the crate.
//!
//! ## RED-PHASE discipline
//!
//! Stays `#[ignore]`'d until G14-B implementer un-ignores. Per
//! §3.6b pim-2, integration tests must drive the production
//! Atrium-boundary entry point — not a stub `policy.check()` call.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-B — D-F + D-PHASE-3-26 — per-actor write rate-limit enforced"]
fn capability_policy_per_actor_write_rate_limit_enforced() {
    // D-F + D-PHASE-3-26 pin. The rate-limit policy plug enforces
    // per-actor (DID-keyed) writes/second/zone. Bursts above the limit
    // fire a typed error.
    //
    // G14-B implementer wires:
    //
    //   let actor_did = ...;
    //   let policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .actor_writes_per_second(actor_did.clone(), "/zone/posts", 10)
    //       .build();
    //
    //   // 10 writes within 1s succeed:
    //   for _ in 0..10 {
    //       policy.pre_write(&actor_did, "/zone/posts", &node).unwrap();
    //   }
    //   // 11th write within same window: typed error.
    //   let err = policy.pre_write(&actor_did, "/zone/posts", &node).unwrap_err();
    //   assert!(matches!(err, benten_caps::CapError::RateLimitExceeded { .. }));
    //
    // OBSERVABLE consequence: the 11th write within a 1-second window
    // observably rejects with a typed RateLimitExceeded variant
    // carrying the actor + zone for diagnostics.
    unimplemented!(
        "G14-B wires per-actor write-rate-limit enforcement at CapabilityPolicy::pre_write"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B + G16-A — D-F + D-PHASE-3-26 — per-peer bandwidth budget at Atrium boundary"]
fn capability_policy_per_peer_bandwidth_budget_at_atrium_boundary() {
    // D-F + D-PHASE-3-26 cross-wave pin. The Atrium boundary consults
    // the policy plug to enforce per-peer (DID-keyed) bandwidth
    // budgets. Defense against a malicious or buggy peer flooding the
    // sync channel.
    //
    // Implementer wires:
    //
    //   let peer_did = ...;
    //   let policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .peer_bandwidth_bytes_per_second(peer_did.clone(), 1_000_000) // 1 MB/s
    //       .build();
    //
    //   // Atrium boundary delivers an inbound sync chunk; policy
    //   // accounts the bytes. Within budget: passes.
    //   policy.account_peer_inbound(&peer_did, 500_000).unwrap();
    //   // Subsequent chunk pushes over budget: typed error.
    //   let err = policy.account_peer_inbound(&peer_did, 800_000).unwrap_err();
    //   assert!(matches!(err, benten_caps::CapError::PeerBandwidthExceeded { .. }));
    //
    // OBSERVABLE consequence: per-peer bandwidth accounting observably
    // rejects over-budget chunks at the policy plug.
    unimplemented!("G14-B + G16-A wires per-peer bandwidth budget enforcement at Atrium boundary");
}

#[test]
#[ignore = "RED-PHASE: G14-B + G16-A — D-F — cross-peer back-pressure surfaces via policy"]
fn cross_peer_back_pressure_via_rate_limit_policy() {
    // D-F pin. When peer A's bandwidth budget is exhausted, the
    // policy MUST surface back-pressure to the local sync engine so
    // it stops accepting inbound work from A while still serving
    // peers B + C.
    //
    // Implementer wires:
    //
    //   let peer_a = ...;
    //   let peer_b = ...;
    //   let policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .peer_bandwidth_bytes_per_second(peer_a.clone(), 1_000_000)
    //       .peer_bandwidth_bytes_per_second(peer_b.clone(), 1_000_000)
    //       .build();
    //
    //   // Saturate peer A:
    //   for _ in 0..2 {
    //       let _ = policy.account_peer_inbound(&peer_a, 800_000);
    //   }
    //   assert!(policy.is_peer_back_pressured(&peer_a));
    //   // Peer B is unaffected:
    //   assert!(!policy.is_peer_back_pressured(&peer_b));
    //   policy.account_peer_inbound(&peer_b, 500_000).unwrap();
    //
    // OBSERVABLE consequence: the policy surfaces per-peer back-
    // pressure state; the sync engine can drain peer B while peer A
    // is throttled. Cross-peer isolation is the load-bearing property.
    unimplemented!(
        "G14-B + G16-A wires per-peer back-pressure surfacing via is_peer_back_pressured()"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — D-F + D-PHASE-3-26 — plug per-actor writes/sec/zone"]
fn rate_limit_policy_plug_per_actor_writes_sec_per_zone() {
    // D-F + D-PHASE-3-26 pin. The plug surface (the API for adding a
    // rate-limit at construction time) accepts `(actor_did, zone,
    // writes_per_second)` triples. This test pins the plug API shape.
    //
    // Implementer wires:
    //
    //   let policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .actor_writes_per_second(actor_a.clone(), "/zone/posts", 10)
    //       .actor_writes_per_second(actor_a.clone(), "/zone/admin", 1)
    //       .actor_writes_per_second(actor_b.clone(), "/zone/posts", 100)
    //       .build();
    //
    //   // Each (actor, zone) pair has its own bucket:
    //   for _ in 0..10 { policy.pre_write(&actor_a, "/zone/posts", &node).unwrap(); }
    //   policy.pre_write(&actor_a, "/zone/admin", &node).unwrap();
    //   for _ in 0..50 { policy.pre_write(&actor_b, "/zone/posts", &node).unwrap(); }
    //
    //   // Actor A's posts bucket is exhausted; admin bucket isn't:
    //   assert!(policy.pre_write(&actor_a, "/zone/posts", &node).is_err());
    //   policy.pre_write(&actor_a, "/zone/admin", &node).unwrap_err(); // 1/sec exhausted
    //
    // OBSERVABLE consequence: per-(actor, zone) bucket isolation;
    // policy accepts the plug surface for fine-grained limits.
    unimplemented!(
        "G14-B wires per-(actor, zone) bucket isolation in RateLimitPolicy plug surface"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — D-F — plug per-peer bandwidth budget at Atrium boundary"]
fn rate_limit_policy_plug_per_peer_bandwidth_budget_at_atrium_boundary() {
    // D-F pin. Plug surface for per-peer bandwidth budget. Pins the
    // API shape `(peer_did, bytes_per_second)`.
    //
    // Implementer wires:
    //
    //   let policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .peer_bandwidth_bytes_per_second(peer_a.clone(), 1_000_000)
    //       .peer_bandwidth_bytes_per_second(peer_b.clone(), 100_000) // smaller
    //       .build();
    //
    //   // Different peers have different budgets:
    //   policy.account_peer_inbound(&peer_a, 800_000).unwrap();
    //   assert!(policy.account_peer_inbound(&peer_b, 200_000).is_err()); // > 100k
    //
    //   // Bandwidth budgets are PER-PEER:
    //   assert!(!policy.is_peer_back_pressured(&peer_a));
    //   assert!(policy.is_peer_back_pressured(&peer_b));
    //
    // OBSERVABLE consequence: plug API accepts heterogeneous per-peer
    // budgets; budgets isolate at the policy layer.
    unimplemented!(
        "G14-B wires per-peer heterogeneous bandwidth budget plug surface in RateLimitPolicy"
    );
}
