//! R3 Family D RED-PHASE pin for G23-A grep-assert: `register_subgraph`
//! signature unchanged (arch-r1-15).
//!
//! Pin source: r2-test-landscape §2.4 row 7.
//!
//! ## Defense
//!
//! Grep-asserts the public-API surface of `Engine::register_subgraph` is the
//! same shape post-G23-A as pre-G23-A: signature
//! `pub fn register_subgraph(&mut self, spec: impl IntoSubgraphSpec) -> Result<...>`
//! (or equivalent). Catches the failure mode where G23-A adds a schema-only
//! parallel registration surface that would fork the registration path.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    grep-asserts the public Engine::register_subgraph signature is unchanged post-G23-A. \
    arch-r1-15 sibling pin. Closes r2 §2.4 row 7."]
fn schema_compiler_does_not_widen_register_subgraph_signature() {
    // G23-A implementer wires this:
    //
    //   let baseline_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("public-api")
    //       .join("benten-engine.json");
    //   let baseline = std::fs::read_to_string(&baseline_path).unwrap();
    //
    //   // The cargo-public-api baseline contains every `register_subgraph`
    //   // method signature. Assert the only fn with that name is the canonical
    //   // pre-G23-A shape.
    //   let count = baseline.matches("register_subgraph").count();
    //   assert!(count >= 1, "register_subgraph must remain in public API");
    //
    //   // Negative pin: must not have introduced a parallel `register_subgraph_from_schema`
    //   // or `register_schema_subgraph` surface.
    //   assert!(!baseline.contains("register_subgraph_from_schema"),
    //       "schema compiler must not introduce parallel registration surface");
    //   assert!(!baseline.contains("register_schema_subgraph"),
    //       "schema compiler must not introduce parallel registration surface");
    unimplemented!(
        "G23-A wave-4 wires public-API baseline grep-assert for register_subgraph signature"
    );
}
