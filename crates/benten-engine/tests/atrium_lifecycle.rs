//! R3-C RED-PHASE pins for Atrium open/close lifecycle + two-peer
//! bidirectional sync (G16-B wave-6b; per r2-test-landscape §2.4
//! G16-B + plan §3 G16-B row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_open_close_lifecycle` +
//!   `atrium_sync_subgraph_two_peer_bidirectional`.
//! - plan §3 G16-B row line "`Engine::open_atrium` +
//!   `Atrium::sync_subgraph` + `Atrium::merge_remote_change`
//!   surface".
//! - exit-criterion 1 LOAD-BEARING (atrium two-peer bidirectional
//!   sync end-to-end).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b lands atrium lifecycle + sync"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — plan §3 G16-B — atrium open/close lifecycle"]
fn atrium_open_close_lifecycle() {
    // plan §3 G16-B pin. G16-B implementer wires this against the
    // public Engine surface:
    //
    //   let mut engine = test_engine();
    //   let atrium = engine.open_atrium(AtriumConfig {
    //       atrium_id: "test-atrium",
    //       peer_did: my_peer_did(),
    //       device_did: my_device_did(),
    //   }).unwrap();
    //
    //   // Atrium handle exposes status + peer list:
    //   let status = atrium.status();
    //   assert!(status.is_active());
    //
    //   // Close atrium:
    //   atrium.close().unwrap();
    //   // Subsequent operations on the closed handle return typed errors:
    //   match atrium.sync_subgraph("zone:posts") {
    //       Err(e) if e.code() == ErrorCode::E_ATRIUM_CLOSED => {}
    //       other => panic!("expected E_ATRIUM_CLOSED, got {other:?}"),
    //   }
    //
    // OBSERVABLE consequence: the Atrium handle exposes a clean
    // open/close lifecycle; closed handles return typed errors,
    // not panics or stale state.
    unimplemented!("G16-B wires Engine::open_atrium + Atrium::close lifecycle");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — exit-criterion 1 LOAD-BEARING — two-peer bidirectional sync"]
fn atrium_sync_subgraph_two_peer_bidirectional() {
    // exit-criterion 1 LOAD-BEARING pin (Phase 3 atrium two-peer
    // bidirectional sync). G16-B implementer wires this against
    // two engines + a shared atrium handle:
    //
    //   let mut engine_a = test_engine_with_peer_did(peer_a_did);
    //   let mut engine_b = test_engine_with_peer_did(peer_b_did);
    //   let atrium_a = engine_a.open_atrium(shared_config()).unwrap();
    //   let atrium_b = engine_b.open_atrium(shared_config()).unwrap();
    //
    //   // peer_a writes to a zone; peer_b syncs:
    //   engine_a.write_node_in_zone("zone:posts", make_post("p1")).unwrap();
    //   atrium_a.sync_subgraph("zone:posts").unwrap();
    //   atrium_b.sync_subgraph("zone:posts").unwrap();
    //
    //   // peer_b sees peer_a's write:
    //   let p1_on_b = engine_b.read_node_by_label_in_zone("zone:posts", "post:p1").unwrap();
    //   assert!(p1_on_b.is_some());
    //
    //   // Bidirectional: peer_b writes; peer_a syncs:
    //   engine_b.write_node_in_zone("zone:posts", make_post("p2")).unwrap();
    //   atrium_b.sync_subgraph("zone:posts").unwrap();
    //   atrium_a.sync_subgraph("zone:posts").unwrap();
    //
    //   let p2_on_a = engine_a.read_node_by_label_in_zone("zone:posts", "post:p2").unwrap();
    //   assert!(p2_on_a.is_some());
    //
    // OBSERVABLE consequence: two peers in an atrium round-trip
    // writes through `sync_subgraph` end-to-end. This is the
    // load-bearing exit-criterion-1 pin for Phase 3.
    unimplemented!(
        "G16-B wires two-peer bidirectional sync end-to-end (exit-criterion 1 LOAD-BEARING)"
    );
}
