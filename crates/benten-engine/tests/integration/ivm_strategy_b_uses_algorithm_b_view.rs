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
///
/// **R6-R3 r6-r3-ivm-1 label-update:** pre-fix this test supplied
/// `Label("CapabilityGrant")` which was silently coerced to the
/// hardcoded `system:CapabilityGrant` label. Post-fix the engine
/// rejects the mismatch with `E_VIEW_LABEL_MISMATCH`, so the test
/// must supply the matching hardcoded value.
#[test]
fn ivm_strategy_b_user_view_registers_as_algorithm_b() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = UserViewSpec::builder()
        .id("capability_grants")
        .input_pattern(UserViewInputPattern::Label(
            "system:CapabilityGrant".to_string(),
        ))
        // Strategy::B is the builder default but spell it explicitly so
        // the test reads as the fix it pins.
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();

    engine
        .register_user_view(spec)
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
/// through `AlgorithmBView` when the supplied label MATCHES the
/// hardcoded value `"NEXT_VERSION"`. Protects against accidental
/// hardcoded dispatch on a single id.
///
/// **R6-R3 r6-r3-ivm-1 test contract pivot:** pre-fix this test
/// registered with `Label("post")` and asserted SUCCESS — that pinned
/// the OPPOSITE of the contract the TS-DSL `validateUserViewSpec`
/// enforces. Post-fix the engine rejects the mismatched label with
/// `E_VIEW_LABEL_MISMATCH`, so the test now supplies the matching
/// hardcoded label `"NEXT_VERSION"` to keep the load-bearing
/// strategy-B-routes-through-AlgorithmBView assertion + ALSO covers
/// the new fail-loud branch in
/// `ivm_strategy_b_register_user_view_rejects_canonical_id_with_label_mismatch`.
#[test]
fn ivm_strategy_b_routes_through_algorithm_b_for_each_canonical_id() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = UserViewSpec::builder()
        .id("version_current")
        // The hand-written `VersionCurrentView` filters on the hardcoded
        // label `"NEXT_VERSION"` (see `crates/benten-ivm/src/algorithm_b.rs`
        // dispatch arm + `CANONICAL_HARDCODED_LABELS` table). Supplying
        // any other label is rejected post-r6-r3-ivm-1 with
        // `EngineError::ViewLabelMismatch`.
        .input_pattern(UserViewInputPattern::Label("NEXT_VERSION".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();

    engine.register_user_view(spec).expect(
        "register_user_view with Strategy::B + canonical 'version_current' \
         + matching label 'NEXT_VERSION' must succeed",
    );

    let observed = engine.view_strategy("version_current").expect(
        "after register_user_view, the canonical-id view MUST be queryable; \
             None here means AlgorithmBView::for_id rejected a canonical id",
    );
    assert_eq!(
        observed,
        benten_ivm::Strategy::B,
        "Strategy::B user view with canonical id 'version_current' MUST \
         report Strategy::B at runtime; got {observed:?}"
    );
}

/// R6-R3 r6-r3-ivm-1 — load-bearing end-to-end test pin (per
/// `dispatch-conventions.md` §3.6b). Drives the production
/// `Engine::register_user_view` entry point with a canonical view id
/// (`version_current`) paired with a Label that DISAGREES with the
/// hardcoded value (`NEXT_VERSION`). Asserts the call returns
/// `EngineError::ViewLabelMismatch` carrying the structured
/// {view_id, expected_label, got_label} bag — pre-fix this call
/// silently SUCCEEDED + the resulting view filtered on the wrong label.
///
/// Would FAIL if the new fail-loud arm were silently no-op'd back to
/// the pre-fix silent-accept behavior — exactly the §3.6b
/// "behavioral consequence of the arm firing" requirement.
#[test]
fn ivm_strategy_b_register_user_view_rejects_canonical_id_with_label_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = UserViewSpec::builder()
        .id("version_current")
        // Mismatch: the hand-written VersionCurrentView's hardcoded label
        // is "NEXT_VERSION"; supplying "post" pre-fix silently DISCARDED
        // the supplied label + registered a view filtering on the wrong
        // basis. Post-r6-r3-ivm-1 this surfaces typed
        // EngineError::ViewLabelMismatch.
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();

    let err = engine.register_user_view(spec).expect_err(
        "canonical id `version_current` paired with mismatched label `post` MUST be rejected",
    );

    use benten_engine::EngineError;
    match err {
        EngineError::ViewLabelMismatch {
            view_id,
            expected_label,
            got_label,
        } => {
            assert_eq!(view_id, "version_current");
            assert_eq!(expected_label, "NEXT_VERSION");
            assert_eq!(got_label, "post");
        }
        other => panic!(
            "expected EngineError::ViewLabelMismatch for canonical-id + \
             label-mismatch combo; got {other:?}"
        ),
    }

    // Defence-in-depth: the rejected registration must leave NO
    // observable residue — querying the view's strategy after rejection
    // should return None (the def_node was not written; the IVM
    // subscriber never saw the spec).
    assert!(
        engine.view_strategy("version_current").is_none(),
        "rejected registration must leave no observable residue: \
         view_strategy MUST return None for an id that was never accepted"
    );
}

/// Sibling regression — the four canonical view ids (capability_grants,
/// version_current, event_dispatch, governance_inheritance) all enforce
/// hardcoded-label matching. `content_listing` (the fifth canonical id)
/// is intentionally absent from the table because its dispatch arm
/// honors caller-supplied label.
#[test]
fn ivm_strategy_b_register_user_view_label_mismatch_fires_on_all_four_canonical_ids() {
    use benten_engine::EngineError;

    let canonical_cases = [
        ("capability_grants", "system:CapabilityGrant", "wrong"),
        ("version_current", "NEXT_VERSION", "post"),
        ("event_dispatch", "system:EventDispatch", "other"),
        (
            "governance_inheritance",
            "system:GovernanceInheritance",
            "different",
        ),
    ];

    for (view_id, expected, supplied) in canonical_cases {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

        let spec = UserViewSpec::builder()
            .id(view_id)
            .input_pattern(UserViewInputPattern::Label(supplied.to_string()))
            .strategy(benten_ivm::Strategy::B)
            .build()
            .unwrap();

        let err = engine.register_user_view(spec).expect_err(&format!(
            "expected Err for `{view_id}` + label `{supplied}` (≠ `{expected}`)"
        ));

        match err {
            EngineError::ViewLabelMismatch {
                view_id: id,
                expected_label,
                got_label,
            } => {
                assert_eq!(id, view_id);
                assert_eq!(expected_label, expected);
                assert_eq!(got_label, supplied);
            }
            other => panic!("expected ViewLabelMismatch for `{view_id}`; got {other:?}"),
        }
    }

    // content_listing is the canonical id whose dispatch arm honors the
    // supplied label; the engine MUST NOT fail-loud for a non-matching
    // label there.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("content_listing")
        .input_pattern(UserViewInputPattern::Label("any-label-is-fine".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();
    engine.register_user_view(spec).expect(
        "content_listing honors caller-supplied label; the label-mismatch \
         guard MUST NOT fire for it (dispatch arm reads input_pattern_label)",
    );
}
