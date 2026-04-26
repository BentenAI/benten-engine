#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G8-B API
// (`engine.create_view`, `UserViewSpec`, user-registered IVM views).
//
//! Phase 2b R4-FP (B-1) — G8-B user-registered views unit tests.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §3 G8-B (`create_view`
//!     goes live; removes the Phase-1 `TODO(phase-2-view-id-registry)`
//!     at `crates/benten-engine/src/engine.rs:1817` — formerly `:1549`).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.7 rows 182-184.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (zero R3-landed coverage).
//!
//! Three assertions:
//!
//!   1. `user_registered_view_end_to_end` — register a user-defined view
//!      via the public `engine.create_view(spec)` API; emit synthetic
//!      events that match the view's input pattern; assert the view
//!      materializes against those events with the expected projection.
//!
//!   2. `user_view_pattern_mismatch_fires_typed_error` — a user view
//!      whose `input_pattern` references a non-existent label (or a
//!      malformed pattern) MUST be rejected at registration with a
//!      typed error (NOT silently accepted as an empty view).
//!
//!   3. `engine_create_view_removes_phase_1_todo` — the
//!      `TODO(phase-2-view-id-registry)` at engine.rs:1817 is GONE
//!      after G8-B lands; `engine.create_view` is the per-view
//!      registration path replacing the hard-coded 5-name whitelist.
//!      Compile-time / source-grep pin.
//!
//! Status: RED-PHASE — `engine.create_view`, `UserViewSpec`,
//! `ViewRegistrationError`, and `testing_emit_n_synthetic_events` for
//! the user-view path are R5 G8-B pending.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;

/// `user_registered_view_end_to_end` — R2 §1.7 + plan §3 G8-B.
///
/// Round-trip: build a `UserViewSpec` describing a label-targeted
/// projection; pass it to `engine.create_view(spec)`; emit N events
/// matching the input pattern; read the view back and assert the
/// materialized projection contains the expected rows.
#[test]
#[ignore = "Phase 2b G8-B pending — engine.create_view + UserViewSpec unimplemented"]
fn user_registered_view_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   let spec = UserViewSpec::builder()
    //       .id("user_posts_by_author")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::B)
    //       .project(|evt| (evt.actor_cid(), evt.label().to_string()))
    //       .build()
    //       .unwrap();
    //   engine.create_view(spec).expect("create_view succeeds");
    //
    //   benten_engine::testing::testing_emit_n_synthetic_events(
    //       &mut engine, "post", 5,
    //   ).unwrap();
    //
    //   let outcome = engine
    //       .read_view("user_posts_by_author", Default::default())
    //       .expect("read_view materializes user-registered view");
    //   assert_eq!(outcome.list.unwrap().len(), 5,
    //       "user view MUST reflect the 5 emitted events");
    todo!("R5 G8-B — engine.create_view round-trip + materialization assertion");
}

/// `user_view_pattern_mismatch_fires_typed_error` — R2 §1.7 + plan §3 G8-B.
///
/// A user view spec whose `input_pattern` either references an unknown
/// label or fails the pattern grammar MUST be rejected at
/// `create_view` time with a typed `ViewRegistrationError` — never
/// silently accepted (which would yield a perpetually-empty view +
/// hide the typo from the developer).
#[test]
#[ignore = "Phase 2b G8-B pending — UserViewSpec validation + ViewRegistrationError unimplemented"]
fn user_view_pattern_mismatch_fires_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   let bad_spec = UserViewSpec::builder()
    //       .id("user_bad_pattern")
    //       .input_pattern_raw("**::malformed::**")
    //       .strategy(Strategy::B)
    //       .build()
    //       .unwrap();
    //   let err = engine.create_view(bad_spec).expect_err(
    //       "malformed pattern MUST be rejected — no silent accept of \
    //        a perpetually-empty user view"
    //   );
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_VIEW_PATTERN_INVALID")
    //           || rendered.contains("E_VIEW_REGISTRATION_REJECTED"),
    //       "expected typed pattern-validation error, got: {rendered}"
    //   );
    todo!("R5 G8-B — assert ViewRegistrationError fires on bad pattern");
}

/// `engine_create_view_removes_phase_1_todo` — R2 §1.7 + plan §3 G8-B.
///
/// The Phase-1 `TODO(phase-2-view-id-registry)` at
/// `crates/benten-engine/src/engine.rs:1817` MUST be removed when G8-B
/// lands: the per-view-definition registry replaces the hard-coded
/// 5-name whitelist (`benten_ivm::views::*`).
///
/// Drift detector: source-grep the engine.rs for the TODO marker; the
/// test fails if the marker is still present.
#[test]
#[ignore = "Phase 2b G8-B pending — TODO marker removal verified at green-phase"]
fn engine_create_view_removes_phase_1_todo() {
    // R5 G8-B pseudo:
    //   let src = std::fs::read_to_string(concat!(
    //       env!("CARGO_MANIFEST_DIR"),
    //       "/src/engine.rs",
    //   )).expect("engine.rs readable");
    //   assert!(
    //       !src.contains("TODO(phase-2-view-id-registry)"),
    //       "the Phase-1 view-id-registry TODO at engine.rs:1817 MUST be \
    //        removed by G8-B (per-view definition registration replaces \
    //        the 5-name whitelist)"
    //   );
    //
    //   // Affirmative: the public create_view surface MUST exist.
    //   // Compile-time pin via fully-qualified type reference.
    //   fn _compile_pin(e: &mut Engine, s: benten_engine::UserViewSpec) {
    //       let _ = e.create_view(s);
    //   }
    todo!("R5 G8-B — assert TODO removed + create_view surface present");
}
