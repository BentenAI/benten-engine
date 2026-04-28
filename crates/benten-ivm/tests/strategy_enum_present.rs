#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! Strategy enum presence + default tests (G8-A, D8-RESOLVED).
//!
//! R3-D TDD red-phase. Pins the EXPLICIT-OPT-IN strategy enum shape:
//! `pub enum Strategy { A, B, C }` lives in `crates/benten-ivm/src/strategy.rs`,
//! re-exported from `benten_ivm::Strategy`. The 5 Phase-1 hand-written views
//! default to `Strategy::A` per D8.
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A + §5 D8.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 rows 1-2.
//!
//! Implementation lands in G8-A.

#![allow(clippy::unwrap_used)]

use benten_ivm::Strategy;
use benten_ivm::View;
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

#[test]
fn strategy_enum_a_b_c_present() {
    // D8-RESOLVED: enum has exactly three variants — A (hand-written),
    // B (generalized Algorithm B), C (Z-set / DBSP cancellation, reserved
    // for Phase 3+). Equality + Debug round-trip pin the variant set so
    // adding/removing one breaks compile.
    let a = Strategy::A;
    let b = Strategy::B;
    let c = Strategy::C;

    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);

    // Debug-string pin so printing is stable across renames (matters for
    // error catalog wiring + view-definition CID stability).
    assert_eq!(format!("{a:?}"), "A");
    assert_eq!(format!("{b:?}"), "B");
    assert_eq!(format!("{c:?}"), "C");
}

#[test]
fn strategy_default_is_a_for_handwritten_views() {
    // D8-RESOLVED: every Phase-1 hand-written view returns `Strategy::A`
    // from the new `View::strategy()` default-method. This is the contract
    // the bench gate measures B against.
    let cap = CapabilityGrantsView::new();
    let evt = EventDispatchView::new();
    let content = ContentListingView::new("post");
    let gov = GovernanceInheritanceView::new();
    let ver = VersionCurrentView::new();

    assert_eq!(cap.strategy(), Strategy::A);
    assert_eq!(evt.strategy(), Strategy::A);
    assert_eq!(content.strategy(), Strategy::A);
    assert_eq!(gov.strategy(), Strategy::A);
    assert_eq!(ver.strategy(), Strategy::A);
}
