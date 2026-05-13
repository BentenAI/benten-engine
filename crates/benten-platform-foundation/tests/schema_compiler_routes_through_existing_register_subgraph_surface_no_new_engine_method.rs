//! R3 Family D RED-PHASE pin for G23-A architecture commitment
//! (arch-r1-15; no signature widening of `register_subgraph`).
//!
//! Pin source: r2-test-landscape §2.4 row 6 + plan §3 G23-A.
//!
//! ## Architecture commitment
//!
//! Schema-driven rendering MUST route through the existing
//! `Engine::register_subgraph(spec: SubgraphSpec)` public API surface. No
//! new engine method is introduced. This pin verifies the routing path
//! exists end-to-end without widening the engine API.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    requires benten_platform_foundation::schema_compiler::compile + engine round-trip via \
    existing register_subgraph surface. arch-r1-15. Closes r2 §2.4 row 6."]
fn schema_compiler_routes_through_existing_register_subgraph_surface_no_new_engine_method() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_engine::{Engine, EngineBuilder};
    //
    //   let spec = compile(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    //   let mut engine = EngineBuilder::default().open_in_memory().unwrap();
    //
    //   // Routes through EXISTING Engine::register_subgraph. No new engine
    //   // method introduced. Compile-time pin: if the signature widens, this
    //   // line will not compile.
    //   engine.register_subgraph(spec).expect("register_subgraph(SubgraphSpec) round-trip");
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires register_subgraph round-trip pin (arch-r1-15)");
}
