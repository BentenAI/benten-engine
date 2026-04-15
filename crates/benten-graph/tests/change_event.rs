//! `ChangeEvent` schema + attribution fields (G3-A, R1 architect — R2
//! landscape §2.2 row 9).
//!
//! Phase 1 G3-A stub — tests drive the public schema of ChangeEvent. R5
//! replaces `todo!()` bodies.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};

#[test]
fn change_event_exposes_cid_label_kind_and_tx_id() {
    // Construct an event using the public schema fields. This test compiles
    // exactly when the public shape matches the spec.
    let cid = canonical_test_node().cid().unwrap();
    let e = ChangeEvent {
        cid: cid.clone(),
        label: "Post".to_string(),
        kind: ChangeKind::Created,
        tx_id: 42,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    };
    assert_eq!(e.cid, cid);
    assert_eq!(e.label, "Post");
    assert_eq!(e.kind, ChangeKind::Created);
    assert_eq!(e.tx_id, 42);
}

#[test]
fn change_event_supports_attribution_fields() {
    // R1 attribution: `actor_cid`, `handler_cid`, `capability_grant_cid` are
    // optional but exposed on the event.
    let cid = canonical_test_node().cid().unwrap();
    let actor = cid.clone();
    let handler = cid.clone();
    let grant = cid.clone();
    let e = ChangeEvent {
        cid: cid.clone(),
        label: "Post".to_string(),
        kind: ChangeKind::Updated,
        tx_id: 7,
        actor_cid: Some(actor.clone()),
        handler_cid: Some(handler.clone()),
        capability_grant_cid: Some(grant.clone()),
    };
    assert_eq!(e.actor_cid, Some(actor));
    assert_eq!(e.handler_cid, Some(handler));
    assert_eq!(e.capability_grant_cid, Some(grant));
}

#[test]
fn change_kind_discriminates_create_update_delete() {
    assert_ne!(ChangeKind::Created, ChangeKind::Updated);
    assert_ne!(ChangeKind::Updated, ChangeKind::Deleted);
    assert_ne!(ChangeKind::Deleted, ChangeKind::Created);
}
