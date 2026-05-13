//! R3 Family D RED-PHASE pin for G23-A end-to-end cap-policy firing
//! (sec-3.5-r1-4 §3.6b end-to-end pin; LOAD-BEARING; would-FAIL-if-no-op'd).
//!
//! Pin source: r2-test-landscape §2.4 row 5.
//!
//! ## §3.6b end-to-end shape (per pim-2)
//!
//! - PRODUCTION RUNTIME ARM: register the schema-emitted SubgraphSpec via
//!   `Engine::register_subgraph`, then call the resulting handler via
//!   `Engine::call_as(principal, ...)`. The cap policy fires inside the
//!   evaluator's walk, not in the schema compiler.
//! - OBSERVABLE CONSEQUENCE: capability checks register at every primitive
//!   boundary the walk crosses; a recording cap-policy backend captures the
//!   trail.
//! - WOULD-FAIL-IF-NO-OP: if the schema-emitted SubgraphSpec carried EMPTY
//!   cap-scope annotations, the recording backend would observe zero
//!   checks at primitive boundaries → assertion fails.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    §3.6b end-to-end pin: schema-emitted SubgraphSpec walk through Engine fires \
    cap-policy at each primitive boundary. would-FAIL-if-cap-check-no-op'd per pim-2. \
    Closes r2 §2.4 row 5."]
fn schema_compiler_emitted_subgraph_walk_fires_cap_policy_at_each_primitive_boundary() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_engine::{Engine, EngineBuilder};
    //   use benten_engine::testing::RecordingCapPolicy;
    //
    //   // PRODUCTION RUNTIME ARM:
    //   let spec = compile(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    //   let recorder = RecordingCapPolicy::default();
    //   let mut engine = EngineBuilder::default()
    //       .with_cap_policy(recorder.clone())
    //       .open_in_memory().unwrap();
    //   engine.register_subgraph(spec.clone()).unwrap();
    //
    //   // Call the resulting handler with a test principal.
    //   let principal = benten_engine::testing::test_principal();
    //   let _ = engine.call_as(principal, /* handler-id */ "Note.read", /* input */ ());
    //
    //   // OBSERVABLE CONSEQUENCE: each primitive boundary must have surfaced
    //   // a recorded cap-check.
    //   let checks = recorder.checks();
    //   assert_eq!(checks.len(), spec.primitives().len(),
    //       "cap-policy MUST fire once per primitive boundary (would-FAIL-if-no-op)");
    //
    //   // WOULD-FAIL-IF-NO-OP: if cap-scope annotations were empty, checks.len()
    //   // would be 0 → this assertion explicitly catches the no-op gap.
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires end-to-end cap-policy firing pin (§3.6b shape)");
}
