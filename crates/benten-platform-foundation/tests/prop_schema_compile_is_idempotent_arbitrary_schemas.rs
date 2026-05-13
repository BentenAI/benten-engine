//! R3 Family D RED-PHASE proptest pin for G23-A idempotency
//! (canonical-bytes round-trip).
//!
//! Pin source: r2-test-landscape §2.4 row 11.
//!
//! ## Invariant
//!
//! For any valid schema bytes, `compile(bytes)` must yield a SubgraphSpec
//! whose canonical-bytes encoding is identical across repeat compiles —
//! schema-compiler is a pure function.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D) — DEFERRED PER docs/future/phase-4-backlog.md §4.6. \
    G23-A canary covers fixed-fixture round-trip idempotency via schema_compiler_round_trip_canonical_bytes_stable.rs (un-ignored, PASS); \
    the arbitrary-schema generator arbitrary_valid_schema_bytes(seed) needs the strict 4-of-4 input-dialect grammar finalized (per §4.6 carry-criterion) before it can generate property-test inputs that exercise the dialect boundary. \
    Un-ignores when strict 4-of-4 input-dialect lands. Pin source: r2 §2.4 row 11. Named destination: phase-4-backlog §4.6 per HARD RULE 12 BELONGS-NAMED-NOW."]
fn prop_schema_compile_is_idempotent_arbitrary_schemas() {
    // G23-A implementer wires this with proptest:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_core::canonical_subgraph_bytes;
    //   use proptest::prelude::*;
    //
    //   proptest!(|(seed in 0u64..1000)| {
    //       // R5 wires arbitrary_valid_schema_bytes(seed) helper.
    //       let bytes = arbitrary_valid_schema_bytes(seed);
    //       let spec1 = compile(&bytes).unwrap();
    //       let spec2 = compile(&bytes).unwrap();
    //       prop_assert_eq!(
    //           canonical_subgraph_bytes(spec1.as_subgraph()),
    //           canonical_subgraph_bytes(spec2.as_subgraph()),
    //       );
    //   });
    //
    // For the RED-PHASE landing, the fixed-fixture round-trip placeholder:
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = schema_fixtures::minimal_schema_bytes();
    let _ = schema_fixtures::benign_schema_round_trip_bytes();
    unimplemented!("G23-A wave-4 wires schema-compile idempotency proptest");
}
