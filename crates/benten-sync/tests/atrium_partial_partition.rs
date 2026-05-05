//! R3-C RED-PHASE pin for partial-partition asymmetric reachability
//! (G16-B + G16-C wave-6b; per r2-test-landscape §2.4 G16-B + plan
//! §3 G16-B row + net-major-3).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `atrium_partial_partition_asymmetric_reachability_observable_state_explicit`.
//! - `net-major-3` (asymmetric reachability — peer A can reach peer
//!   B but B can't reach A — must surface as an observable explicit
//!   state via engine atrium-status).
//!
//! ## What this pins
//!
//! Partial network partitions where peers have asymmetric
//! reachability (peer-A can reach peer-B; peer-B cannot reach
//! peer-A) MUST surface as an observable explicit state via the
//! engine's atrium-status surface — NOT silently as "all peers
//! healthy" which would give operators a false picture.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B + G16-C wave-6b expose partial-partition state"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B + G16-C wave-6b — net-major-3 — partial-partition asymmetric reachability observable"]
fn atrium_partial_partition_asymmetric_reachability_observable_state_explicit() {
    // net-major-3 pin. G16-B implementer wires this against a fixture
    // that injects asymmetric reachability:
    //
    //   let mut peer_a = test_peer(peer_a_did);
    //   let mut peer_b = test_peer(peer_b_did);
    //   peer_a.atrium_join(shared_atrium()).await.unwrap();
    //   peer_b.atrium_join(shared_atrium()).await.unwrap();
    //
    //   // Inject asymmetric reachability:
    //   inject_partition_asymmetric(/* a → b OK; b → a BLOCKED */).await;
    //
    //   // peer_a's atrium-status surfaces partial-partition state:
    //   let status_a = peer_a.atrium_status();
    //   assert!(matches!(
    //       status_a.peer_health(peer_b_did),
    //       PeerHealth::AsymmetricallyReachable { incoming: false, outgoing: true } | PeerHealth::PartiallyPartitioned
    //   ));
    //
    //   // peer_b's atrium-status surfaces the inverse asymmetry:
    //   let status_b = peer_b.atrium_status();
    //   assert!(matches!(
    //       status_b.peer_health(peer_a_did),
    //       PeerHealth::AsymmetricallyReachable { incoming: true, outgoing: false } | PeerHealth::PartiallyPartitioned
    //   ));
    //
    // OBSERVABLE consequence: partial-partition is explicitly visible
    // at both peers; defends against the failure shape where the
    // atrium-status surface reports "healthy" while writes are
    // silently failing in one direction.
    unimplemented!("G16-B wires partial-partition asymmetric-reachability state observability");
}
