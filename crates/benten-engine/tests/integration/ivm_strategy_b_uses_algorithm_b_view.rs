//! Phase 2b Wave-8h audit-gap fix #3 — `Strategy::B` user views actually
//! run through `AlgorithmBView` at runtime.
//!
//! Pin source:
//! `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
//! "IVM Algorithm B" DRIFT verdict.
//!
//! ## Pre-fix behaviour (the bug)
//!
//! `crates/benten-engine/src/engine_views.rs::create_user_view` at
//! lines 282-291 (pre-wave-8h) unconditionally constructed a
//! [`benten_ivm::views::ContentListingView`] regardless of the spec's
//! declared `Strategy::B`. The strategy declaration was persisted on
//! the `system:IVMView` Node (folded into the definition's CID per
//! D8) but ignored at runtime registration. The `AlgorithmBView`
//! shipped by wave-G8-A was wired into the type system + benchmarked
//! but NEVER USED IN PRODUCTION — `grep -rn 'Box::new(AlgorithmB'` on
//! `crates/` (excluding `tests/`) found zero hits before this fix.
//!
//! ## Post-fix behaviour (this test)
//!
//! Wave-8h routes Strategy::B user views through
//! [`benten_ivm::algorithm_b::AlgorithmBView::for_id`] when the spec
//! id matches one of the 5 canonical Phase-1 view ids. For ids
//! outside that set the ContentListingView shim is retained as a
//! fallback (the `for_id` rejection path) — Phase-3 user-supplied
//! dispatch removes that fallback. The test below registers a user
//! view with id `"content_listing"` (one of the 5 canonical ids) and
//! asserts the registered view reports `Strategy::B` — proving
//! `AlgorithmBView` is actually wired.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::outcome::UserViewInputPattern;
use benten_engine::{Engine, UserViewSpec};

/// Wave-8h audit-gap fix #3 — a user view registered with the
/// canonical id `"capability_grants"` and the default `Strategy::B`
/// MUST report `Strategy::B` at runtime. Pre-wave-8h this assertion
/// would FAIL because `ContentListingView` (the unconditional
/// fallback) reports `Strategy::A`.
///
/// We use `"capability_grants"` rather than `"content_listing"` because
/// the EngineBuilder auto-registers a `ContentListingView` for label
/// `"post"` at engine open (see `crates/benten-engine/src/builder.rs`
/// lines ~348-354), and the dedupe-by-id check inside
/// `create_user_view` would skip our registration. `"capability_grants"`
/// is one of the 5 canonical view ids (so AlgorithmBView::for_id
/// dispatches cleanly) but is NOT auto-registered by the builder.
#[test]
fn ivm_strategy_b_user_view_registers_as_algorithm_b() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = UserViewSpec::builder()
        .id("capability_grants")
        .input_pattern(UserViewInputPattern::Label("CapabilityGrant".to_string()))
        // Strategy::B is the builder default but spell it explicitly so
        // the test reads as the fix it pins.
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();

    engine
        .create_user_view(spec)
        .expect("create_user_view with Strategy::B + canonical id must succeed");

    let observed = engine.view_strategy("capability_grants").expect(
        "after create_user_view, the view MUST be queryable by id — \
             None here means the IVM subscriber didn't see the registration",
    );

    // The load-bearing assertion. Pre-wave-8h this would equal
    // `Strategy::A` because ContentListingView reports A. Post-fix the
    // AlgorithmBView wrapper is in place + reports B.
    assert_eq!(
        observed,
        benten_ivm::Strategy::B,
        "Strategy::B user view MUST run through AlgorithmBView at \
         runtime; got {observed:?}. Pre-wave-8h this would be \
         Strategy::A because the unconditional ContentListingView \
         fallback reports A — the wave-G8-A `AlgorithmBView` type \
         was wired into the type system but NEVER constructed in \
         production code."
    );
}

/// Companion regression test — registering a `Strategy::B` user view
/// with a different canonical id (`"version_current"`) ALSO routes
/// through `AlgorithmBView`. Protects against accidental hardcoded
/// dispatch on a single id.
#[test]
fn ivm_strategy_b_routes_through_algorithm_b_for_each_canonical_id() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = UserViewSpec::builder()
        .id("version_current")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();

    engine
        .create_user_view(spec)
        .expect("create_user_view with Strategy::B + canonical 'version_current' must succeed");

    let observed = engine.view_strategy("version_current").expect(
        "after create_user_view, the canonical-id view MUST be queryable; \
             None here means AlgorithmBView::for_id rejected a canonical id",
    );
    assert_eq!(
        observed,
        benten_ivm::Strategy::B,
        "Strategy::B user view with canonical id 'version_current' MUST \
         report Strategy::B at runtime; got {observed:?}"
    );
}
