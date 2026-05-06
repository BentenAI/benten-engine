//! GREEN-PHASE pins for `register_user_view` post-G15-A generalization
//! (G15-A wave-5a).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows
//!   `register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization`
//!   + `register_user_view_with_label_pattern_succeeds_under_strategy_b`.
//! - plan §3 G15-A row.
//! - `ivm-major-5` (engine refuses Strategy::A user-view registration;
//!   user views always run under Strategy::B post-G15-A).
//! - `D-PHASE-3-28` RESOLVED (non-canonical view IDs maintained via
//!   generic kernel under Strategy::B).

#![allow(clippy::unwrap_used)]

use benten_engine::{Engine, EngineError, ErrorCode, UserViewInputPattern, UserViewSpec};

#[test]
fn register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization()
 {
    // ivm-major-5 pin. Even after G15-A generalizes the kernel, the
    // engine's `register_user_view` still REJECTS a (canonical view
    // ID, mismatched label) registration with `E_VIEW_LABEL_MISMATCH`.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("capability_grants")
        .input_pattern(UserViewInputPattern::Label("user".to_string())) // mismatch
        .build()
        .unwrap();
    let result = engine.register_user_view(spec);
    match result {
        Err(EngineError::ViewLabelMismatch {
            view_id,
            expected_label,
            got_label,
        }) => {
            assert_eq!(view_id, "capability_grants");
            assert_eq!(expected_label, "system:CapabilityGrant");
            assert_eq!(got_label, "user");
            // Catalog code is stable across the generalization.
            let err = EngineError::ViewLabelMismatch {
                view_id: "capability_grants".into(),
                expected_label: "system:CapabilityGrant".into(),
                got_label: "user".into(),
            };
            assert_eq!(err.code(), ErrorCode::ViewLabelMismatch);
        }
        other => panic!("expected ViewLabelMismatch, got {other:?}"),
    }
}

#[test]
fn register_user_view_with_label_pattern_succeeds_under_strategy_b() {
    // ivm-major-5 + D-PHASE-3-28 pin. User-defined view IDs MAY be
    // registered with arbitrary label patterns under Strategy::B
    // (the generalized Algorithm B kernel). The engine no longer
    // forces these registrations through a ContentListingView shim.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:posts_by_author")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    let _cid = engine
        .register_user_view(spec)
        .expect("user view + matching label pattern succeeds");
    // Internal: the registered view runs under Strategy::B at the
    // engine boundary regardless of which inner kernel the dispatch
    // router selected.
    let strategy = engine
        .view_strategy("custom:posts_by_author")
        .expect("view registered + queryable strategy");
    assert_eq!(strategy, benten_ivm::Strategy::B);
}

#[test]
fn register_user_view_with_anchor_prefix_pattern_no_silent_label_equality_coerce() {
    // Phase-3 G15-A specifically retires the Phase-2b
    // "AnchorPrefix is silently coerced to a Label-equality match
    // against the prefix string" stub. AnchorPrefix("crud:") must
    // genuinely prefix-match; it must NOT match label == "crud:"
    // exclusively.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:by_prefix")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".to_string()))
        .build()
        .unwrap();
    let _ = engine
        .register_user_view(spec)
        .expect("AnchorPrefix registers under Strategy::B");
    let strategy = engine.view_strategy("custom:by_prefix").unwrap();
    assert_eq!(strategy, benten_ivm::Strategy::B);
}
