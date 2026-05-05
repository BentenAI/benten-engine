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
