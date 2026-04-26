//! G12-B red-phase: devserver threads handler registration through
//! `Engine::register_subgraph` — NOT through the in-memory Phase-2a
//! `HandlerTable` stub.
//!
//! Per plan §3.2 G12-B: "swap in-memory `HandlerTable` for
//! `Engine::register_subgraph` calls; preserve the Phase-2a
//! `ReloadCoordinator` / `CallGuard` (concurrency coordination, not storage)."
//!
//! TDD red-phase. Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: devserver → Engine routing not yet wired"]
fn devserver_register_handler_calls_engine_register_subgraph_not_handler_table() {
    // Drive: devserver consumes a DSL source file; under the hood,
    // benten-dsl-compiler emits SubgraphSpec; devserver passes that into
    // Engine::register_subgraph (NOT into a process-local HandlerTable).
    //
    // Pin: post-registration, Engine::call(handler_id, ...) succeeds AND
    // the in-memory HandlerTable surface (if it still exists during transition)
    // does NOT contain the handler — proves the engine path took ownership.
    todo!(
        "R5 G12-B: build a temp DSL file; spawn devserver against it; \
           assert engine.call(handler_id, payload) returns the expected response; \
           assert the legacy in-memory HandlerTable doesn't carry the handler entry"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: end-to-end DSL compile path not yet wired"]
fn devserver_compiles_dsl_source_via_dsl_compiler_crate() {
    // Pin: devserver's compile entry point is `benten_dsl_compiler::compile_str`
    // / `compile_file`, NOT a private inline DSL parser inside benten-dev.
    todo!(
        "R5 G12-B: spawn devserver; trace compile call; assert it dispatches \
           through benten_dsl_compiler::compile_file (not an inline parser)"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: registration error propagation not yet wired"]
fn devserver_propagates_dsl_compile_error_to_user_facing_diagnostic() {
    // Pin: bad DSL → devserver renders Diagnostic.error_code + line/column
    // (NOT a generic "registration failed" string).
    todo!(
        "R5 G12-B: feed bad DSL; assert devserver surfaces Diagnostic.error_code = E_DSL_PARSE_ERROR"
    )
}
