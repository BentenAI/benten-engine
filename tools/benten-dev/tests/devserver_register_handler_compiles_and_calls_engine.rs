//! G12-B green-phase: devserver threads handler registration through
//! `Engine::register_subgraph` — NOT only through the in-memory Phase-2a
//! `HandlerTable` stub.
//!
//! Per plan §3.2 G12-B: "swap in-memory `HandlerTable` for
//! `Engine::register_subgraph` calls; preserve the Phase-2a
//! `ReloadCoordinator` / `CallGuard` (concurrency coordination, not storage)."
//!
//! Lifted from red-phase 2026-04-28 (R5 G12-B implementer).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_dev::DevServer;
use tempfile::tempdir;

#[test]
fn devserver_register_handler_calls_engine_register_subgraph_not_handler_table() {
    let dir = tempdir().unwrap();
    let dev = DevServer::builder()
        .workspace(dir.path())
        .enable_engine(true)
        .build()
        .unwrap();
    let src = "handler 'h1' { read('post') -> respond }";
    let _hid = dev
        .register_handler_from_dsl("h1", "run", src)
        .expect("DSL register must succeed");
    // Engine MUST exist + carry the registered handler. The dev server's
    // engine() accessor returns the same Arc the dev server registered into.
    let engine = dev.engine().expect("engine routing enabled");
    // Re-register identical content via the engine — idempotent path returns
    // the same handler id without DuplicateHandler.
    let compiled = benten_dsl_compiler::compile_str(src).unwrap();
    let h2 = engine
        .register_subgraph(compiled.subgraph)
        .expect("re-register identical content is idempotent");
    assert_eq!(h2, "h1");
}

#[test]
fn devserver_compiles_dsl_source_via_dsl_compiler_crate() {
    // Pin: `register_handler_from_dsl` re-exports `compile_str` from the
    // dsl-compiler crate (NOT an inline parser).
    let dir = tempdir().unwrap();
    let dev = DevServer::builder()
        .workspace(dir.path())
        .enable_engine(true)
        .build()
        .unwrap();
    let src = "handler 'h2' { write('post', { title: $title }) -> respond }";
    let hid = dev
        .register_handler_from_dsl("h2", "run", src)
        .expect("must compile");
    assert_eq!(hid, "h2");
    // The same source must round-trip through the standalone compile entry
    // point — the devserver MUST NOT be using a private inline parser.
    let direct = benten_dev::compile_str(src).unwrap();
    assert_eq!(direct.subgraph.handler_id(), "h2");
}

#[test]
fn devserver_propagates_dsl_compile_error_to_user_facing_diagnostic() {
    let dir = tempdir().unwrap();
    let dev = DevServer::builder()
        .workspace(dir.path())
        .enable_engine(true)
        .build()
        .unwrap();
    let bad = "handler 'oops' { teleport -> respond }"; // teleport is not a primitive
    let err = dev
        .register_handler_from_dsl("oops", "run", bad)
        .expect_err("must fail with typed Diagnostic");
    let d = err.diagnostic().expect("typed diagnostic present");
    assert_eq!(d.error_code, "E_DSL_UNKNOWN_PRIMITIVE");
}
