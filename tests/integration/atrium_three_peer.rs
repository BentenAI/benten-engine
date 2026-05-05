//! R3-C RED-PHASE end-to-end pin: three-peer atrium Loro convergence
//! under concurrent writes (G16-B wave-6b; per r2-test-landscape §2.4
//! G16-B + plan §3 G16-B row + C-10 + exit-criterion 15).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `integration/atrium_three_peer_loro_convergence_under_concurrent_writes`.
//! - plan §3 G16-B row line "per C-10 — multi-peer 3+-peer
//!   convergence pin".
//! - `C-10` (3+-peer Loro convergence under concurrent writes).
//! - exit-criterion 15 (atrium as sociotechnical unit;
//!   3+-peer membership + propagation per FULL-ROADMAP.md exit
//!   criterion sentence 3).
//! - `ds-r4-1` (R4 large-council Round 1 distributed-systems lens —
//!   Byzantine-class 3+-peer concurrent-writes-AND-revoke proptest
//!   sibling pin landed here at R4-FP/R3-C).
//!
//! ## Relocated R4-FP
//!
//! Originally placed in `tests/phase_3_workspace/`; relocated to
//! `tests/integration/` at R4-FP/R3-C per R3-CPC-1 + R2 §2.4 G16-B row.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b — C-10 + exit-criterion 15 LOAD-BEARING"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — C-10 + exit-criterion 15 — three-peer Loro convergence under concurrent writes"]
fn atrium_three_peer_loro_convergence_under_concurrent_writes() {
    // C-10 + exit-criterion 15 pin. G16-B implementer wires this:
    //
    //   1. Spin up THREE engines under three peer-DIDs.
    //   2. All three join the same Atrium via shared invite.
    //   3. All three concurrently write to the SAME node + property
    //      (e.g. /zone/posts/p1 title field) at staggered HLC times.
    //   4. Bidirectional sync triggers Loro merges.
    //   5. After convergence wave, all three peers observe the SAME
    //      canonical-bytes for the merged Version Node.
    //   6. The merged Version's AttributionFrame contains all three
    //      contributing peer-DIDs.
    //
    //   let mut peers: Vec<_> = (0..3).map(|i| test_peer_with_did(test_did(i))).collect();
    //   for peer in &mut peers {
    //       peer.atrium_join(shared_atrium()).await.unwrap();
    //   }
    //   // Concurrent writes:
    //   for (i, peer) in peers.iter_mut().enumerate() {
    //       peer.write_node_in_zone("/zone/posts",
    //           make_post_with_title("p1", &format!("title-from-peer-{i}"))).await.unwrap();
    //   }
    //   // Sync to convergence:
    //   for _ in 0..3 {
    //       for peer in &mut peers {
    //           peer.sync_subgraph("/zone/posts").await.unwrap();
    //       }
    //   }
    //   // All three converge:
    //   let bytes_set: BTreeSet<_> = peers.iter()
    //       .map(|p| p.read_current_bytes_for_anchor_in_zone("/zone/posts", "p1").unwrap())
    //       .collect();
    //   assert_eq!(bytes_set.len(), 1, "all three peers must converge on identical canonical bytes");
    //
    //   // The merged Version's AttributionFrame contains all 3 peer-DIDs:
    //   let p1 = peers[0].read_current_for_anchor_in_zone("/zone/posts", "p1").unwrap();
    //   let frame = p1.attribution_frame();
    //   for i in 0..3 {
    //       assert!(frame.contains_peer_did(&test_did(i)),
    //           "AttributionFrame must include peer-DID {i} per D-C");
    //   }
    //
    // OBSERVABLE consequence: a 3-peer Atrium under concurrent
    // writes converges on the same merged Version with all 3 peers'
    // attribution captured. This is the load-bearing
    // exit-criterion-15 pin.
    unimplemented!("G16-B wires three-peer concurrent-write Loro convergence end-to-end");
}

#[test]
#[ignore = "RED-PHASE: G16-B + G14-D wave-6b — ds-r4-1 — Byzantine 3+-peer concurrent-writes-AND-partial-revoke-AND-offline-reconnect proptest"]
fn atrium_three_peer_concurrent_writes_under_partial_revoke_with_offline_reconnect_converges() {
    // ds-r4-1 (R4 large-council Round 1 distributed-systems lens) pin.
    // Byzantine-class extension of the happy-path 3-peer convergence
    // pin above. R1 ds-5 was aliased into 'HLC orphan covered by D-A'
    // during R1 triage, but ds-5's substantive content is the realistic
    // operational shape: 3+ peers under concurrent writes WHILE one
    // peer's grant is partially revoked AND another peer is offline +
    // reconnecting. Without this pin, partial-revoke-mid-merge edge
    // cases ship unverified.
    //
    // Proptest shape (G16-B + G14-D joint owners; 10k iterations):
    //
    //   use proptest::prelude::*;
    //   proptest! {
    //       #![proptest_config(ProptestConfig {
    //           cases: 10_000, // Per-iteration calibration if budget tight
    //           ..ProptestConfig::default()
    //       })]
    //       #[test]
    //       fn prop_three_peer_under_partial_revoke_offline_reconnect_converges(
    //           // Sample: 0..3 peer-write counts, 0..16 writes per peer,
    //           // 0..2 revoke events, 0..1 offline-reconnect window per peer.
    //           writes_a in 0usize..16,
    //           writes_b in 0usize..16,
    //           writes_c in 0usize..16,
    //           num_revokes in 0usize..3,
    //           offline_peer_idx in 0usize..3,
    //           offline_window_start in 0usize..32,
    //           offline_window_len in 1usize..16,
    //       ) {
    //           let mut peers = make_three_peers();
    //           let revoke_schedule = sample_revoke_schedule(num_revokes, &peers);
    //           interleave_writes_and_revokes_with_offline_window(
    //               &mut peers,
    //               &[writes_a, writes_b, writes_c],
    //               &revoke_schedule,
    //               offline_peer_idx,
    //               offline_window_start,
    //               offline_window_len,
    //           );
    //           sync_to_convergence(&mut peers);
    //           // ALL converge on identical canonical-bytes for surviving zones:
    //           assert_three_peers_converge(&peers);
    //           // Revoked-peer's effective cap-set excludes revoked paths:
    //           for revoke_event in &revoke_schedule {
    //               for peer in &peers {
    //                   if peer.peer_did() == revoke_event.target_peer_did {
    //                       assert!(!peer.effective_cap_set()
    //                           .includes_path(&revoke_event.path));
    //                   }
    //               }
    //           }
    //           // No peer observes data under a stale grant during
    //           // offline-reconnect drain (per ds-r4-1 + net-blocker-3
    //           // companion):
    //           let drain_log = peers[offline_peer_idx].drain_log();
    //           let revoked_data_seen = drain_log.iter().any(|e| {
    //               revoke_schedule.iter().any(|r| {
    //                   matches!(e.kind(), MessageKind::Data)
    //                       && e.before_event_for(r)
    //                       && e.under_revoked_grant_for(r)
    //               })
    //           });
    //           assert!(!revoked_data_seen,
    //               "no peer must observe data under a revoked grant per ds-r4-1");
    //       }
    //   }
    //
    // OBSERVABLE consequence: under any interleaving of 3-peer
    // concurrent writes + partial revokes + offline-reconnect windows,
    // (a) all peers converge on identical canonical bytes for non-
    // revoked zones, (b) no peer observes data under a revoked grant.
    // Composes G16-B 3+-peer convergence + G14-D partial-revoke + G16-C
    // MST-diff-on-reconnect drain ordering. Defends against the
    // partial-revoke-mid-merge attack class that R1 ds-5 named.
    unimplemented!(
        "G16-B + G14-D wire ds-r4-1 Byzantine 3-peer concurrent-write + partial-revoke + offline-reconnect proptest (10k iterations)"
    );
}
