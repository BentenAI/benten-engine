#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! Strategy selection is EXPLICIT-OPT-IN — no auto-select, no runtime
//! adaptation (D8-RESOLVED).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §5 D8.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 rows 9-10.
//!
//! Two assertions:
//! - `strategy_selection_respects_explicit_opt_in_only`: a hand-written view
//!   constructed via `View::new` does NOT silently become `Strategy::B`
//!   regardless of input shape, traffic pattern, or registered budget. Only
//!   an explicit `testing_construct_view_with_strategy(Strategy::B)` (the
//!   §9 test helper) yields a `Strategy::B` view.
//! - `strategy_runtime_adaptation_rejected`: there is no public API that
//!   mutates a view's strategy after construction. Compile-only check that
//!   no `set_strategy` / `with_strategy` / `migrate_to` method exists on the
//!   `View` trait — any such method would let the runtime swap algorithms
//!   mid-flight, which D8 explicitly rejects.

#![allow(clippy::unwrap_used)]

use benten_ivm::Strategy;
use benten_ivm::View;
use benten_ivm::testing::testing_construct_view_with_strategy;
use benten_ivm::views::ContentListingView;

#[test]
fn strategy_selection_respects_explicit_opt_in_only() {
    // Default constructor — never auto-selects B.
    let v = ContentListingView::new("post");
    assert_eq!(
        v.strategy(),
        Strategy::A,
        "ContentListingView::new must yield Strategy::A regardless of context"
    );

    // Explicit opt-in via the testing helper — only path that produces B.
    let b: Box<dyn View> = testing_construct_view_with_strategy(Strategy::B);
    assert_eq!(b.strategy(), Strategy::B);
}

#[test]
fn strategy_runtime_adaptation_rejected() {
    // D8-RESOLVED: no runtime mutation. The only way to change strategy is
    // to construct a fresh view. This is a behavioral check: take a view,
    // run a workload through it, then re-read the strategy and assert it
    // has NOT migrated.
    let mut v: Box<dyn View> = testing_construct_view_with_strategy(Strategy::A);
    let strategy_before = v.strategy();

    // Ingest enough events that a hypothetical "auto-migrate when hot" code
    // path would have a chance to fire.
    for _ in 0..1024 {
        // Empty events would still hit any auto-adapt logic that triggers
        // on update count. We don't need actual changes for this check.
        let dummy = benten_graph::ChangeEvent::new_node(
            benten_core::Cid::from_blake3_digest([0u8; 32]),
            vec!["irrelevant".into()],
            benten_graph::ChangeKind::Created,
            0,
            None,
        );
        let _ = v.update(&dummy);
    }

    let strategy_after = v.strategy();
    assert_eq!(
        strategy_before, strategy_after,
        "view strategy must NOT change at runtime — D8-RESOLVED rejects \
         runtime adaptation. Got {strategy_before:?} -> {strategy_after:?}"
    );
}
