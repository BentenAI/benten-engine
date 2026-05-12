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
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    schema-compile idempotency proptest requires schema_compiler. \
    Canonical-bytes round-trip invariant per pure-function commitment. \
    Closes r2 §2.4 row 11 prop pin."]
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
