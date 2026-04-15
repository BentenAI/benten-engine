//! `View` shared trait shape (I1, R1 architect — R2 landscape §2.3 row 1).
//!
//! The trait defines `update`, `read`, `rebuild`, `id`, `is_stale`. Every
//! one of the 5 Phase-1 views implements it.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_ivm::View;
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

fn assert_is_view<V: View>() {}

#[test]
fn capability_grants_view_implements_view_trait() {
    assert_is_view::<CapabilityGrantsView>();
}

#[test]
fn event_dispatch_view_implements_view_trait() {
    assert_is_view::<EventDispatchView>();
}

#[test]
fn content_listing_view_implements_view_trait() {
    assert_is_view::<ContentListingView>();
}

#[test]
fn governance_inheritance_view_implements_view_trait() {
    assert_is_view::<GovernanceInheritanceView>();
}

#[test]
fn version_current_view_implements_view_trait() {
    assert_is_view::<VersionCurrentView>();
}

#[test]
fn view_trait_is_object_safe() {
    // If this line compiles, View is object-safe.
    fn _accepts(_v: Box<dyn View>) {}
}
