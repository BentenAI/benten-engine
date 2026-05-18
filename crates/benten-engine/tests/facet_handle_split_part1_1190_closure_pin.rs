//! Wave-E HELD #1190 / #834 closure-pin — facet-handle split part 1
//! (RATIFIED #4, Atrium-pattern).
//!
//! ## What this pins (would-FAIL-if-reverted)
//!
//! #834 (Surf-1): the engine's view subsystem had no structural API
//! seam — view methods sat in the flat `impl Engine` namespace next to
//! caps / CRUD / install with no signal which compose against the view
//! subsystem. The established `engine.caps()` facet
//! (`EngineCapsHandle`) gave the capability subsystem that seam;
//! #1190 part-1 introduces the SIBLING `engine.views()` facet
//! (`EngineViewsHandle`) so the view subsystem has the same cohesive
//! namespace.
//!
//! This pin asserts:
//! 1. `engine.views()` returns the `EngineViewsHandle` facet and its
//!    delegated read path is behaviorally identical to the direct
//!    `engine.read_view(..)` entry point (the handle is a true facet,
//!    not a divergent surface). Removing the `views()` accessor or the
//!    handle delegation fails compilation / this assertion.
//! 2. The facet PAIR is present — `engine.caps()` AND `engine.views()`
//!    both resolve — proving the part-1 split landed the views sibling
//!    alongside the existing caps handle (the #834 cohesion seam).
//!
//! Part-1 is additive (RATIFIED P-II): the direct `impl Engine` view
//! methods stay; the `#[deprecated]` shims + napi JS-mirror +
//! workspace call-site migration are the RATIFIED Phase-4-Meta
//! part-2/3 (orchestrator-serialized P-II sweep).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

#[test]
fn views_facet_delegates_identically_to_direct_read_view_entry_point() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("facet.redb"))
        .expect("engine opens");

    // A non-registered view id surfaces the same Result shape through
    // both the direct entry point and the facet handle — proving the
    // facet is a true delegating seam, not a divergent surface. (We
    // assert error-equivalence rather than a specific Ok payload so
    // the pin is robust to view-registration setup; the load-bearing
    // property is "facet == direct entry point".)
    let direct = engine.read_view("system:ivm:nonexistent-view");
    let via_facet = engine.views().read_view("system:ivm:nonexistent-view");

    assert_eq!(
        direct.is_ok(),
        via_facet.is_ok(),
        "#1190 part-1: engine.views().read_view(..) MUST delegate \
         identically to engine.read_view(..) — the facet handle is a \
         cohesion seam over the SAME entry point, not a second \
         behavior. A divergence here means the facet split regressed."
    );
    if let (Err(d), Err(f)) = (&direct, &via_facet) {
        assert_eq!(
            format!("{d}"),
            format!("{f}"),
            "facet + direct error surfaces must be byte-identical"
        );
    }

    // The view_strategy + change-offset delegations resolve through the
    // facet (compile + run proof the handle exposes the view subsystem
    // surface, not just one method).
    assert_eq!(
        engine.views().view_strategy("system:ivm:nonexistent-view"),
        engine.view_strategy("system:ivm:nonexistent-view"),
        "view_strategy must delegate identically through the facet"
    );
    assert_eq!(
        engine.views().user_view_change_offset(),
        engine.user_view_change_offset(),
        "user_view_change_offset must delegate identically through the facet"
    );
}

#[test]
fn caps_and_views_facet_pair_both_present() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy_grant_backed()
        .open(dir.path().join("facet-pair.redb"))
        .expect("engine opens");

    // #834 cohesion seam: the part-1 split landed the views sibling
    // ALONGSIDE the pre-existing caps handle. Both accessors resolving
    // (compile + run) is the structural proof the facet PAIR exists —
    // the JS consumer's `engine.caps()` / `engine.views()` grouping
    // the Surf-1 finding asked for (the JS-side factory mirror is the
    // RATIFIED Phase-4-Meta part-2).
    let _caps = engine.caps();
    let _views = engine.views();
    // Exercise one method on each to prove they are live facets.
    let _ = engine
        .caps()
        .create_principal("facet-pin-principal")
        .expect("caps facet live");
    let _ = engine.views().user_view_change_offset();
}
