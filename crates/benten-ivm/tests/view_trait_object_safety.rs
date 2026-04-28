#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! `View` trait object-safety regression with the new `strategy()` default-method
//! (g8-clarity-2 + ivm-r6-8).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A; D8-RESOLVED
//! adds `fn strategy(&self) -> Strategy { Strategy::A }` to the `View` trait.
//! That default-method must NOT break object-safety because the subscriber
//! stores `Box<dyn View>` and the registry hands out `dyn View` references.
//!
//! Compile-only assertion: if this file builds, the trait remains object-safe.

#![allow(clippy::unwrap_used)]

use benten_ivm::View;
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

#[test]
fn view_trait_strategy_default_method_preserves_object_safety() {
    // The cast itself is the assertion: if `View` were not object-safe,
    // `Box<dyn View>` would not type-check.
    let cap: Box<dyn View> = Box::new(CapabilityGrantsView::new());
    let evt: Box<dyn View> = Box::new(EventDispatchView::new());
    let content: Box<dyn View> = Box::new(ContentListingView::new("post"));
    let gov: Box<dyn View> = Box::new(GovernanceInheritanceView::new());
    let ver: Box<dyn View> = Box::new(VersionCurrentView::new());

    let views: Vec<Box<dyn View>> = vec![cap, evt, content, gov, ver];

    // Strategy is reachable through the trait object (not just a concrete
    // method). If `strategy()` were generic or violated object-safety, the
    // call-through-dyn below would refuse to compile.
    for v in &views {
        let _ = v.strategy();
        let _ = v.id();
    }

    assert_eq!(views.len(), 5);
}
