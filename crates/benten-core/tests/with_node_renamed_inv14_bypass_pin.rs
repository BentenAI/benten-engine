//! #843 closure-pin (Surf-1): `Subgraph::with_node`/`with_edge` renamed to
//! `push_node_raw`/`push_edge_raw` with an unmissable Inv-14-bypass
//! docstring. The rename is byte-for-byte behaviour-preserving — **zero
//! canonical-byte / CID churn** (the stamping "fork" the brief flagged was
//! never the resolution; #843 option-(b) rename is).
//!
//! These assertions prove:
//!  1. The renamed raw path still does NOT stamp Inv-14 attribution (the
//!     reject-path construction surface is preserved — load-bearing for the
//!     Inv-14 negative test, which must be able to build un-attributed
//!     nodes that `validate_registration` rejects).
//!  2. The canonical bytes of a representative subgraph are unaffected by
//!     the rename (would FAIL if a stray attribution stamp leaked into the
//!     raw path, i.e. if someone "fixed" #843 by stamping).
//!
//! Note: the `with_node`/`with_edge` names are gone entirely (single
//! canonical rename, no alias kept — CLAUDE.md rule 5). Their absence is
//! enforced by the workspace compiling: every former caller was migrated
//! in the same change, and the names no longer resolve.

#![allow(clippy::unwrap_used)]

use benten_core::{ATTRIBUTION_PROPERTY_KEY, OperationNode, PrimitiveKind, Subgraph, Value};

#[test]
fn push_node_raw_does_not_stamp_inv14_attribution() {
    // The raw path appends verbatim — no `attribution: true` default.
    let sg = Subgraph::new("raw:no-stamp")
        .push_node_raw(OperationNode::new("re0", PrimitiveKind::Read))
        .push_node_raw(OperationNode::new("rs1", PrimitiveKind::Respond))
        .push_edge_raw("re0", "rs1", "next");

    for node in sg.nodes() {
        assert!(
            node.property(ATTRIBUTION_PROPERTY_KEY).is_none(),
            "push_node_raw must NOT stamp Inv-14 attribution — the \
             reject-path construction surface depends on un-stamped nodes; \
             stamping here was the false 'fork' #843 did NOT take"
        );
    }
}

#[test]
fn rename_is_zero_canonical_byte_churn() {
    // The canonical bytes of a representative raw-constructed subgraph are
    // a pure function of (handler_id, nodes, edges, deterministic). The
    // rename changed no field and added no stamp, so the bytes are exactly
    // what the pre-rename `with_node`/`with_edge` produced. A hard-coded
    // pin proves no canonical drift sneaked in with the refactor.
    let sg = Subgraph::new("rename:zero-churn")
        .push_node_raw(
            OperationNode::new("re0", PrimitiveKind::Read)
                .with_property("label", Value::Text("post".into())),
        )
        .push_node_raw(OperationNode::new("rs1", PrimitiveKind::Respond))
        .push_edge_raw("re0", "rs1", "next");

    let cid = sg.cid().unwrap();
    // Re-construct in a different order → identical CID (order-independent
    // canonical view; the rename did not perturb it).
    let reordered = Subgraph::new("rename:zero-churn")
        .push_node_raw(OperationNode::new("rs1", PrimitiveKind::Respond))
        .push_node_raw(
            OperationNode::new("re0", PrimitiveKind::Read)
                .with_property("label", Value::Text("post".into())),
        )
        .push_edge_raw("re0", "rs1", "next");
    assert_eq!(reordered.cid().unwrap(), cid);

    const PINNED_CID: &str = "bafyr4icl4umfqvsu7awtnvg2iwt3bxebuywb5tp7wkejvufgp2xstgao5m";
    assert_eq!(
        cid.to_string(),
        PINNED_CID,
        "canonical CID drifted — the #843 rename must be byte-for-byte \
         behaviour-preserving (zero churn). A change here means a stamp \
         or shape change leaked into the raw path."
    );
}
