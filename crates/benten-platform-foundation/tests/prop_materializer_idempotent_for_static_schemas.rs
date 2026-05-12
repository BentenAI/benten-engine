//! R3 Family E RED-PHASE proptest pin: materializer idempotence for static
//! schemas (post-materializer safety).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 14.
//! - mat-r1-3 canonical-bytes determinism (companion proptest of the
//!   single-input determinism pin).
//! - cag-r1-1 + cag-r1-6 (12-primitive vocabulary irreducible across
//!   ARBITRARY schemas).
//!
//! ## What this proptest establishes
//!
//! For ARBITRARY structurally-valid static schemas (no SUBSCRIBE / no
//! external mutation), the materializer walk is idempotent: running it
//! twice on the same inputs produces byte-identical output. This is
//! stronger than mat-r1-3 because it covers a generated schema space
//! beyond the fixture set.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer proptest doesn't exist at HEAD; G23-B wave-5 wires the proptest harness \
    over arbitrary static schemas. Closes r2-test-landscape §2.5 row 14."]
fn prop_materializer_idempotent_for_static_schemas() {
    // G23-B implementer wires this with proptest:
    //
    //   use proptest::prelude::*;
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //   use benten_platform_foundation::schema_compiler;
    //
    //   proptest! {
    //       #![proptest_config(ProptestConfig {
    //           cases: 64, // MSRV 1.95 wall-clock budget (per Phase-3 precedent)
    //           ..ProptestConfig::default()
    //       })]
    //       #[test]
    //       fn idempotent_walk(schema_bytes in any_static_schema_strategy()) {
    //           let spec = match schema_compiler::compile(&schema_bytes) {
    //               Ok(s) => s,
    //               Err(_) => return Ok(()), // skip invalid generations
    //           };
    //           let mat = HtmlJsonMaterializer::default();
    //           let out1 = mat.materialize_with_gate(/* spec */ ..).unwrap();
    //           let out2 = mat.materialize_with_gate(/* spec */ ..).unwrap();
    //           prop_assert_eq!(out1.html_bytes(), out2.html_bytes());
    //           prop_assert_eq!(out1.json_bytes(), out2.json_bytes());
    //       }
    //   }
    //
    //   // any_static_schema_strategy() constructs arbitrary SchemaRoot-rooted
    //   // schemas over the 8-label vocab + 8-scalar set, optionally with
    //   // FieldRef cycles BLOCKED (negative-pin space is exercised elsewhere).
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = materializer_fixtures::sample_note_content_bytes();
    unimplemented!(
        "G23-B wave-5 wires proptest idempotence over arbitrary static schemas; \
         expected ~64 cases under MSRV 1.95 wall-clock budget"
    );
}
