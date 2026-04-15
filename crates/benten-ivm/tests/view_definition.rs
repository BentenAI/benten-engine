//! `ViewDefinition` content-addressing + Node serialization (I1 — R2 landscape
//! §2.3 row 2).
//!
//! ViewDefinitions are stored as Nodes with label `system:IVMView` so they
//! themselves are content-addressed and the five Phase 1 definitions can be
//! stably referenced by CID.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

#[test]
fn content_listing_definition_has_expected_view_id() {
    let def = ContentListingView::definition();
    assert_eq!(def.view_id, "content_listing");
}

#[test]
fn capability_grants_definition_has_expected_view_id() {
    let def = CapabilityGrantsView::definition();
    assert_eq!(def.view_id, "capability_grants");
}

#[test]
fn event_dispatch_definition_has_expected_view_id() {
    let def = EventDispatchView::definition();
    assert_eq!(def.view_id, "event_dispatch");
}

#[test]
fn governance_inheritance_definition_has_expected_view_id() {
    let def = GovernanceInheritanceView::definition();
    assert_eq!(def.view_id, "governance_inheritance");
}

#[test]
fn version_current_definition_has_expected_view_id() {
    let def = VersionCurrentView::definition();
    assert_eq!(def.view_id, "version_current");
}

#[test]
fn view_definition_as_node_label_is_system_ivmview() {
    let def = ContentListingView::definition();
    let node = def.as_node();
    assert_eq!(node.labels, vec!["system:IVMView".to_string()]);
}

#[test]
fn view_definition_cid_is_deterministic() {
    let def1 = ContentListingView::definition();
    let def2 = ContentListingView::definition();
    assert_eq!(def1.cid().unwrap(), def2.cid().unwrap());
}
