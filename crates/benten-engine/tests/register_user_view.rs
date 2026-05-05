//! R3-C RED-PHASE pins for `register_user_view` post-G15-A
//! generalization (G15-A wave-5a).
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
//!
//! ## RED-PHASE discipline
//!
//! Every test is `#[ignore]`'d with rationale
//! `"RED-PHASE: G15-A wave-5a generalizes register_user_view"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-major-5 — canonical_id + mismatched_label rejected"]
fn register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization()
 {
    // ivm-major-5 pin. Even after G15-A generalizes the kernel, the
    // engine's `register_user_view` still REJECTS a (canonical view
    // ID, mismatched label) registration with `E_VIEW_LABEL_MISMATCH`.
    //
    // Concrete shape:
    //   let mut engine = test_engine();
    //   let result = engine.register_user_view(
    //       "crud:post",
    //       LabelPattern::exact("user"),  // mismatch
    //       Projection::default(),
    //   );
    //   match result {
    //       Err(e) if e.code() == ErrorCode::E_VIEW_LABEL_MISMATCH => {}
    //       other => panic!("expected E_VIEW_LABEL_MISMATCH, got {other:?}"),
    //   }
    //
    // OBSERVABLE consequence: the post-G15-A engine still produces
    // the same typed error; generalization doesn't silently widen
    // acceptance.
    unimplemented!("G15-A wires register_user_view rejection for canonical_id + mismatched_label");
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-major-5 — user view + label_pattern under Strategy::B"]
fn register_user_view_with_label_pattern_succeeds_under_strategy_b() {
    // ivm-major-5 + D-PHASE-3-28 pin. User-defined view IDs MAY be
    // registered with arbitrary label patterns under Strategy::B
    // (the generalized Algorithm B kernel). The engine no longer
    // refuses these registrations as it did pre-G15-A.
    //
    // Concrete shape:
    //   let mut engine = test_engine();
    //   let view = engine.register_user_view(
    //       "custom:posts_by_author",
    //       LabelPattern::exact("post"),
    //       Projection::all_props(),
    //   ).expect("user view + matching label pattern succeeds");
    //   // Internal: the registered view runs under Strategy::B.
    //   assert_eq!(view.strategy(), benten_ivm::Strategy::B);
    //
    // OBSERVABLE consequence: registering a user-defined view with a
    // valid (label_pattern matches view-id semantics) registration
    // succeeds and produces a Strategy::B-backed view object.
    unimplemented!(
        "G15-A wires register_user_view success for user_id + label_pattern under Strategy::B"
    );
}
