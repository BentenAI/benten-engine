//! R3-C RED-PHASE pin: `benten-sync` persistent state is graph-encoded
//! (G16-A wave-6 canary; cag-2 architectural pin).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `benten_sync_persistent_state_graph_encoded`.
//! - `cag-2` (Code-as-Graph: persistent state is encoded as graph
//!   Nodes/Edges, not as opaque blobs alongside the graph).
//!
//! ## What this pins
//!
//! `benten-sync`'s persistent state (peer rosters, atrium membership,
//! pending sync state, MST diff cursors) is encoded as graph
//! Nodes/Edges in `benten-graph`'s storage layer — NOT as opaque
//! redb keys with no graph-side handle. This preserves Code-as-Graph
//! symmetry: every Atrium-side persistent fact can be queried, IVM
//! views can subscribe to it, and content-addressing applies.
//!
//! ## Per-shape granularity (cag-r4-3 MAJOR)
//!
//! The composite `benten_sync_persistent_state_graph_encoded` pin
//! enumerates 3 R1-named shapes (atrium membership / sync-cursor
//! HLC checkpoints / grant-cross-reference) but only walks 1 of 3.
//! Per cag-r4-3 (R4 large-council Round 1 + Round 2 carry), the
//! sync-cursor and grant-cross-reference MUST have INDIVIDUAL
//! per-shape pins — without them, a future implementer can store
//! cursors as opaque KVBackend.put() entries with composite keys,
//! defeating graph-traversal queries + IVM-view subscription.
//!
//! Per-shape sibling pins (cag-r4-3 closure):
//!
//! - `tests/atrium_sync_cursor_persisted_as_graph_node_keyed_by_peer_did_zone` — cag-r4-3
//! - `tests/atrium_grant_cross_reference_via_graph_edge_not_side_table` — cag-r4-3
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 wires graph-encoded persistent state; cag-2 audit at landing"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — cag-2 — persistent state graph-encoded"]
fn benten_sync_persistent_state_graph_encoded() {
    // cag-2 pin. G16-A implementer wires this against the
    // post-implementation atrium state + benten-graph backend:
    //
    //   use benten_graph::Backend;
    //   use benten_sync::transport::Endpoint;
    //   let backend = test_backend();
    //   let endpoint = Endpoint::with_backend(&backend).build().await.unwrap();
    //   endpoint.join_atrium(atrium_handle).await.unwrap();
    //
    //   // After joining an Atrium, the membership fact should be
    //   // queryable as a graph Node:
    //   let nodes = backend.query_nodes_by_label("atrium:membership").unwrap();
    //   assert!(!nodes.is_empty(), "atrium membership must be encoded as a graph Node");
    //   let membership = &nodes[0];
    //   assert_eq!(membership.property("atrium_id").unwrap(), atrium_handle.to_value());
    //   assert_eq!(membership.property("peer_did").unwrap(), endpoint.peer_did().to_value());
    //
    //   // IVM views can subscribe to it (Code-as-Graph end-to-end):
    //   let view = engine.register_user_view(
    //       "atrium_memberships",
    //       LabelPattern::exact("atrium:membership"),
    //       Projection::default(),
    //   ).unwrap().materialize();
    //   assert!(view.rows().iter().any(|n| n.property("peer_did").unwrap() == endpoint.peer_did().to_value()));
    //
    // OBSERVABLE consequence: the post-G16-A persistent state is
    // queryable through the same graph + IVM surfaces as user data;
    // Code-as-Graph symmetry preserved.
    unimplemented!("G16-A wires graph-encoded persistent atrium state + IVM view subscription");
}

#[test]
#[ignore = "RED-PHASE: G16-A / G16-B wave-6 — cag-r4-3 MAJOR — sync cursor as graph Node keyed by (peer_did, zone)"]
fn atrium_sync_cursor_persisted_as_graph_node_keyed_by_peer_did_zone() {
    // cag-r4-3 MAJOR pin (Charter 4 per-shape granularity for
    // sync-cursor). HLC checkpoints per-peer per-zone are the
    // load-bearing state for Loro CRDT replay (D-PHASE-3-22). The
    // cursor MUST be a graph Node keyed by (peer_did, zone) — NOT
    // a KVBackend.put() with a composite key. Without graph
    // encoding, IVM views cannot subscribe to cursor advances and
    // operator dashboards lose visibility.
    //
    // G16-A / G16-B implementer wires this:
    //
    //   use benten_graph::Backend;
    //   use benten_sync::transport::Endpoint;
    //   let backend = test_backend();
    //   let endpoint = Endpoint::with_backend(&backend).build().await.unwrap();
    //   endpoint.join_atrium(atrium_handle).await.unwrap();
    //
    //   let peer_did = endpoint.peer_did();
    //   let zone = "/zone/posts";
    //
    //   // Receive some Loro deltas; cursor advances:
    //   endpoint.process_remote_change(remote_loro_change).await.unwrap();
    //
    //   // The sync-cursor reads back as a Node with label
    //   // `sync:cursor` and structured properties:
    //   let cursors = backend.query_nodes_by_label("sync:cursor").unwrap();
    //   let cursor = cursors.iter()
    //       .find(|n| n.property("peer_did").map(|v| v == peer_did.to_value()).unwrap_or(false)
    //              && n.property("zone").map(|v| v == zone.into()).unwrap_or(false))
    //       .expect("sync cursor MUST be queryable as a graph Node keyed by (peer_did, zone) per cag-r4-3");
    //
    //   for required in &["peer_did", "zone", "hlc_checkpoint"] {
    //       assert!(cursor.properties().keys().any(|k| k == required),
    //           "sync:cursor Node MUST carry structured property `{}` per cag-r4-3", required);
    //   }
    //
    //   // IVM-view subscription: a subscriber can register on
    //   // `sync:cursor` and observe advances:
    //   let view = engine.register_user_view(
    //       "sync_cursors",
    //       LabelPattern::exact("sync:cursor"),
    //       Projection::default(),
    //   ).unwrap().materialize();
    //   assert!(view.rows().iter().any(|n|
    //       n.property("peer_did").unwrap() == peer_did.to_value()),
    //       "sync:cursor MUST be IVM-view-subscribable per cag-r4-3");
    //
    // OBSERVABLE consequence: cursor advances are observable through
    // the standard graph + IVM surfaces; operator dashboards can
    // visualize per-peer-per-zone HLC progress without bespoke
    // KVBackend probing. Defends against the architectural drift
    // toward "side-table state alongside the graph" per Charter 4.
    unimplemented!(
        "G16-A / G16-B wires sync-cursor graph-encoding pin: Node label `sync:cursor` + \
         structured properties (peer_did/zone/hlc_checkpoint) + IVM-view-subscribable per cag-r4-3"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-A / G16-B wave-6 — cag-r4-3 MAJOR — grant-cross-reference via graph Edge, not side table"]
fn atrium_grant_cross_reference_via_graph_edge_not_side_table() {
    // cag-r4-3 MAJOR pin (Charter 4 per-shape granularity for
    // grant-cross-reference). UCAN grants scoped to which Atriums
    // MUST be encoded as an Edge from the grant Node to the
    // Atrium Node — NOT as a KVBackend.put() with a composite key
    // like `(grant_cid, atrium_id) → ()`. Without graph encoding,
    // queries like "list all grants scoped to Atrium X" require
    // bespoke KV iteration; with graph encoding, they're standard
    // outgoing-edge traversals.
    //
    // G16-A / G16-B implementer wires this:
    //
    //   use benten_graph::Backend;
    //   use benten_id::ucan;
    //   let backend = test_backend();
    //
    //   let issuer_kp = benten_id::keypair::Keypair::generate();
    //   let audience_kp = benten_id::keypair::Keypair::generate();
    //   let atrium_id = engine.create_atrium("alice-atrium").unwrap();
    //
    //   // Issue a UCAN grant scoped to a specific Atrium:
    //   let ucan = ucan::Ucan::builder()
    //       .issuer(issuer_kp.public_key().to_did())
    //       .audience(audience_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .scoped_to_atrium(atrium_id)
    //       .sign(&issuer_kp).unwrap();
    //
    //   // Install + the grant cross-reference is encoded as a graph Edge:
    //   benten_id::ucan::DurableBackend::test_instance().install_proof(&ucan).unwrap();
    //
    //   // Find the grant Node:
    //   let grants = backend.query_nodes_by_label("id:ucan-grant").unwrap();
    //   let grant_node = grants.iter()
    //       .find(|n| n.cid() == ucan.cid()).unwrap();
    //
    //   // The Atrium-scope is an EDGE (label e.g. `GRANT_SCOPED_TO_ATRIUM`):
    //   let edges = backend.outgoing_edges(&grant_node.cid()).unwrap();
    //   let scope_edges: Vec<_> = edges.iter()
    //       .filter(|e| e.label() == "GRANT_SCOPED_TO_ATRIUM").collect();
    //   assert_eq!(scope_edges.len(), 1,
    //       "UCAN grant scoped to Atrium MUST emit exactly one \
    //        GRANT_SCOPED_TO_ATRIUM Edge per cag-r4-3");
    //   assert_eq!(scope_edges[0].dst_label(), Some("atrium"),
    //       "GRANT_SCOPED_TO_ATRIUM Edge MUST point to a Node with label `atrium`");
    //
    //   // Forbidden shape: a side-table KV entry with composite key
    //   // (NOT a graph Edge). The implementer cannot prove absence
    //   // of a private side-table directly; the contract is enforced
    //   // by asserting the FORWARD shape (Edge presence) which makes
    //   // a parallel side-table redundant + visibly wrong code.
    //
    // OBSERVABLE consequence: "list all grants scoped to Atrium X"
    // is a standard incoming-edge traversal at Atrium Node X,
    // NOT a KVBackend iterator. Defends against the architectural
    // drift toward "side-table state alongside the graph" per
    // Charter 4 + Compromise #11 closure floor.
    unimplemented!(
        "G16-A / G16-B wires grant-cross-reference graph-Edge pin: GRANT_SCOPED_TO_ATRIUM Edge \
         from UCAN grant Node to Atrium Node — NOT a side-table KV entry per cag-r4-3"
    );
}
