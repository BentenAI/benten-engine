#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! All-5-handwritten-views-remain-live enforcement (g8-clarity-1).
//!
//! HYBRID 5-view fate per `r1-ivm-algorithm.json`: retire NONE in Phase 2b;
//! Algorithm B ships as ADDITIVE code. The 5 hand-written views stay frozen
//! as `Strategy::A` baselines. Retirement is Phase-3+ requiring 3 named
//! conditions (B passes A's perf gate, B has shipped 6 months, opt-in user
//! adoption signal).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A
//! (g8-clarity-1).
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 row 8.
//!
//! This test asserts the 5 hand-written types are still constructable, still
//! report their stable view-ids, and still default to `Strategy::A`. If any
//! of the 5 is removed/renamed, this test fails to compile or assert.

#![allow(clippy::unwrap_used)]

use benten_ivm::Strategy;
use benten_ivm::View;
use benten_ivm::views::{
    CapabilityGrantsView, ContentListingView, EventDispatchView, GovernanceInheritanceView,
    VersionCurrentView,
};

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn all_5_handwritten_views_remain_live_as_strategy_a() {
    // Constructability — each of the 5 is still a real type at the
    // documented re-export path.
    let cap: Box<dyn View> = Box::new(CapabilityGrantsView::new());
    let evt: Box<dyn View> = Box::new(EventDispatchView::new());
    let content: Box<dyn View> = Box::new(ContentListingView::new("post"));
    let gov: Box<dyn View> = Box::new(GovernanceInheritanceView::new());
    let ver: Box<dyn View> = Box::new(VersionCurrentView::new());

    // Stable view-id contract — these strings are part of the public API
    // surface (used in error messages, view-definition CIDs, telemetry).
    assert_eq!(cap.id(), "capability_grants");
    assert_eq!(evt.id(), "event_dispatch");
    assert_eq!(content.id(), "content_listing");
    assert_eq!(gov.id(), "governance_inheritance");
    assert_eq!(ver.id(), "version_current");

    // All 5 default to Strategy::A — this is the line that distinguishes
    // "still live" from "lifted into Algorithm B": if any of these flips
    // to Strategy::B in Phase 2b, the additive-only contract is breached.
    assert_eq!(cap.strategy(), Strategy::A);
    assert_eq!(evt.strategy(), Strategy::A);
    assert_eq!(content.strategy(), Strategy::A);
    assert_eq!(gov.strategy(), Strategy::A);
    assert_eq!(ver.strategy(), Strategy::A);
}
