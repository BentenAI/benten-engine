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

// Un-ignored at G23-A wave-4 (2026-05-12 canary). The compile-time arch-grep
// against `engine.register_subgraph(...)` is load-bearing — if a future
// wave widened the registration surface, this test would fail to compile.
#[test]
fn schema_compiler_routes_through_existing_register_subgraph_surface_no_new_engine_method() {
    use benten_engine::EngineBuilder;
    use benten_platform_foundation::schema_compiler::compile;

    let spec = compile(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let engine = EngineBuilder::new()
        .path(":memory:")
        .build()
        .expect("engine build");
    // Routes through EXISTING Engine::register_subgraph(IntoSubgraphSpec).
    // `IntoSubgraphSpec for benten_eval::Subgraph` (== benten_core::Subgraph)
    // is the seam; no new engine method introduced.
    engine
        .register_subgraph(spec.into_subgraph())
        .expect("register_subgraph(Subgraph) round-trip — arch-r1-15");
}
