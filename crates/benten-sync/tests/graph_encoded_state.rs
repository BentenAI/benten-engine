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
