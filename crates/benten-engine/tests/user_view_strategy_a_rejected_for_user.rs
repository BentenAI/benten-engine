#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G8-B
// `engine.create_view` Strategy::A rejection.
//
//! Phase 2b R4-FP (B-1) — D8-RESOLVED: user views REFUSE Strategy::A.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D8-RESOLVED
//!     ("REFUSES `'A'` with typed error since hand-written = Rust-only
//!     and not user-registerable from TS").
//!   - `.addl/phase-2b/r2-test-landscape.md` §7 row 463 (TS pin) +
//!     §8 D8 row.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04.
//!
//! Rust-side defense-in-depth pin: even if a future TS-bridge bug
//! permits `'A'` to slip through to the Rust surface, `engine.create_view`
//! MUST reject it with a typed `ViewRegistrationError`. The TS DSL
//! refusal is the front-line gate; this test is the backstop.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;

/// `user_view_strategy_a_refused_with_typed_error` — D8 + plan §3 G8-B.
///
/// Building a `UserViewSpec` with `.strategy(Strategy::A)` MUST yield
/// either a builder-time error OR a registration-time typed error
/// (`E_VIEW_STRATEGY_A_REFUSED_FOR_USER_VIEW` — exact code TBD by R5
/// G8-B, but the test asserts a recognizably typed error renders).
#[test]
#[ignore = "Phase 2b G8-B pending — Strategy::A rejection for user views unimplemented"]
fn user_view_strategy_a_refused_with_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo (one of two acceptable shapes):
    //
    // Shape A — builder rejects:
    //   let err = UserViewSpec::builder()
    //       .id("user_a_attempt")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::A)
    //       .build()
    //       .expect_err("builder MUST reject Strategy::A for user views");
    //
    // Shape B — registration rejects:
    //   let spec = UserViewSpec::builder_unchecked()
    //       .id("user_a_attempt")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::A)
    //       .build();
    //   let err = engine.create_view(spec).expect_err(
    //       "create_view MUST reject Strategy::A for user views"
    //   );
    //
    // Either way:
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_VIEW_STRATEGY_A_REFUSED")
    //           || rendered.contains("hand-written")
    //           || rendered.contains("Strategy::A"),
    //       "expected typed Strategy::A-refused error, got: {rendered}"
    //   );
    todo!("R5 G8-B — assert Strategy::A rejection for user views");
}

/// Companion: `Strategy::C` (reserved for Phase-3 Z-set cancellation
/// per g8-concern-3) MUST also be refused for user views in Phase 2b.
/// Different error code path than Strategy::A — A is "Rust-only",
/// C is "not yet implemented".
#[test]
#[ignore = "Phase 2b G8-B pending — Strategy::C reserved-for-phase-3 path"]
fn user_view_strategy_c_refused_as_reserved() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   let err = UserViewSpec::builder()
    //       .id("user_c_attempt")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::C)
    //       .build()
    //       .expect_err("Strategy::C MUST be rejected — Phase 3 reserved");
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_VIEW_STRATEGY_C_RESERVED")
    //           || rendered.contains("Phase 3")
    //           || rendered.contains("Z-set"),
    //       "expected Strategy::C-reserved error, got: {rendered}"
    //   );
    todo!("R5 G8-B — assert Strategy::C reserved-for-Phase-3 rejection");
}
