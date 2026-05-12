//! R3 Family E RED-PHASE pin: materializer source uses `read_node_as` ONLY
//! (never `read_node`) — grep-assert + runtime-trace pair per cag-r1-9 +
//! CLAUDE.md baked-in #18 Class B β (LOAD-BEARING).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 12 + §5
//!   smell-test note pairing grep-assert with runtime trace (cag-r1-9).
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-B
//!   "Materializer USES ONLY `read_node_as`, never `read_node`".
//! - CLAUDE.md baked-in #18 Class B β: `Engine::read_node_as(principal, cid)`
//!   is the public surface for any read attributed to a non-trusted principal
//!   (the materializer's walk-time principal qualifies); `Engine::read_node`
//!   is `pub(crate)` for engine internals only.
//! - mat-r1-7 (materializer-correctness-reviewer R1): the materializer's
//!   walk threads cap-recheck through `read_node_as` at every READ fanout.
//!
//! ## SHAPE + SUBSTANCE pairing (per R2 §5)
//!
//! - **GREP arm (SHAPE):** materializer.rs source contains zero `.read_node(`
//!   call sites; contains ≥1 `.read_node_as(` call site.
//! - **RUNTIME arm (SUBSTANCE):** a real materializer walk records the
//!   engine entry-point invocations via a trace subscriber; the trace
//!   carries `read_node_as` events with the correct walk_principal, not
//!   `read_node` events.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer.rs doesn't exist at HEAD; G23-B wave-5 wires the Class B β read seam. \
    Closes r2-test-landscape §2.5 row 12 + §5 substance pairing + cag-r1-9 + CLAUDE.md #18."]
fn materializer_uses_read_node_as_only_never_read_node() {
    // GREP-ASSERT arm.
    //
    //   let src = std::fs::read_to_string(
    //       "../../crates/benten-platform-foundation/src/materializer.rs"
    //   ).expect("materializer.rs source readable from tests/");
    //
    //   // No `.read_node(` call sites. (Match leading `.` so we don't pick up
    //   // the substring inside `.read_node_as(`.)
    //   let bare_calls: Vec<_> = src.match_indices(".read_node(").collect();
    //   assert!(
    //       bare_calls.is_empty(),
    //       "materializer.rs MUST NOT call Engine::read_node directly — found {} call \
    //        sites at offsets {:?}; route ALL reads through Engine::read_node_as per \
    //        CLAUDE.md #18 Class B β + cag-r1-9",
    //       bare_calls.len(),
    //       bare_calls.iter().map(|(o, _)| *o).collect::<Vec<_>>(),
    //   );
    //
    //   // At least one `.read_node_as(` call site (the materializer DOES
    //   // read content).
    //   let as_calls: Vec<_> = src.match_indices(".read_node_as(").collect();
    //   assert!(
    //       !as_calls.is_empty(),
    //       "materializer.rs MUST call Engine::read_node_as at least once (its read seam)"
    //   );
    //
    // RUNTIME-TRACE arm (SUBSTANCE per R2 §5).
    //
    //   use benten_engine::Engine;
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //
    //   // Trace subscriber that records engine read entry-point invocations.
    //   let trace = ReadSeamTraceRecorder::install();
    //
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    //   // ... write a Note through transaction ...
    //   let alice = materializer_fixtures::actor_principal_alice_cid();
    //   let mat = HtmlJsonMaterializer::default();
    //   let _ = mat.materialize_with_gate(&engine, /* spec */ ..).unwrap();
    //
    //   // Trace MUST contain `read_node_as` events.
    //   assert!(
    //       !trace.read_node_as_events().is_empty(),
    //       "runtime trace MUST contain read_node_as events for the walk"
    //   );
    //
    //   // Trace MUST NOT contain `read_node` events (the engine-internal
    //   // pub(crate) variant; the materializer is OUT-OF-CRATE).
    //   assert!(
    //       trace.bare_read_node_events().is_empty(),
    //       "runtime trace MUST NOT contain bare read_node events — found {}; \
    //        any such event proves the walk bypassed Class B β cap-recheck",
    //       trace.bare_read_node_events().len()
    //   );
    //
    //   // SUBSTANCE: assert read_node_as events carry the walk_principal (alice)
    //   // — not some hardcoded privileged principal.
    //   for evt in trace.read_node_as_events() {
    //       assert_eq!(evt.principal_cid, alice, "every read attributed to walk_principal");
    //   }
    let _ = materializer_fixtures::actor_principal_alice_cid();
    unimplemented!(
        "G23-B wave-5 lands materializer.rs; this pin combines GREP-arm (source-text \
         assert) + RUNTIME-arm (trace recorder asserts read_node_as events with \
         walk_principal; zero bare read_node events) per cag-r1-9 + R2 §5 substance"
    );
}
