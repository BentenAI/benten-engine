//! Phase 2b Wave-8b BLOCKER acceptance test (wsa-w8b-1).
//!
//! Pin: the production engine dispatch path for the SANDBOX primitive
//! MUST reach the wasmtime executor. Prior to the wsa-w8b-1 fix-pass,
//! `crates/benten-eval/src/primitives/mod.rs:96` routed
//! `PrimitiveKind::Sandbox => host.execute_sandbox(op)` but
//! `impl PrimitiveHost for Engine` did NOT override `execute_sandbox` —
//! so production SANDBOX dispatch hit the trait default's
//! `Err(EvalError::PrimitiveNotImplemented(Sandbox))`, NEVER reaching
//! the actual wasmtime invocation pipeline.
//!
//! This test builds a SANDBOX-bearing subgraph, registers wasm bytes,
//! and dispatches through `Engine::call`. It asserts that:
//!
//!   1. With NO bytes registered for the declared module CID, the call
//!      surfaces the typed `module bytes not registered` error from the
//!      engine override — NOT `E_PRIMITIVE_NOT_IMPLEMENTED` from the
//!      trait default. The error string is the load-bearing assertion
//!      because it could ONLY appear if the override fired.
//!
//!   2. With bytes registered, the call succeeds + the outcome routes
//!      through the OK edge. Success is observable because the trait
//!      default would have returned `Err(PrimitiveNotImplemented)` and
//!      collapsed to a non-OK outcome.
//!
//! These two assertions together prove the production path reaches the
//! executor — they cannot both pass against the pre-fix-pass tree.
//!
//! Closure narrative: this test is the regression gate for Compromise
//! #4 ("WASM runtime is compile-check only") at the engine boundary.
//! Adding it to the suite means a future refactor that drops the
//! `execute_sandbox` override on `impl PrimitiveHost for Engine` is
//! caught immediately.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

/// Build a 2-node SubgraphSpec (SANDBOX -> RESPOND) carrying the
/// SANDBOX node's `module` (base32 CID) + `caps` (inline manifest)
/// properties on the primitive's properties bag.
///
/// SubgraphSpec is the registration path that survives the
/// `register_subgraph → dispatch_call` round-trip — registering a raw
/// `benten_eval::Subgraph` directly does NOT populate the
/// `inner.specs` lookup the dispatcher consults at call time, so the
/// handler would surface as "unknown" at dispatch.
fn sandbox_spec(handler_id: &str, module_cid_str: &str) -> SubgraphSpec {
    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str.to_string()));
    sandbox_props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );

    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .respond()
        .build()
}

/// Compile a trivial WAT module that exports a `run` function. Mirrors
/// the eval-side test corpus pattern (see
/// `crates/benten-eval/src/primitives/sandbox.rs::tests::inline_manifest_resolves_without_registry_entry`).
fn trivial_run_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))")
        .expect("trivial run module compiles")
}

/// Compute the BLAKE3-CID of the wasm bytes — the canonical CID that
/// identifies the module under the project's content-addressing
/// discipline (matches the convention `Engine::install_module` uses for
/// manifest CIDs).
fn cid_for_bytes(bytes: &[u8]) -> Cid {
    let digest = *blake3::hash(bytes).as_bytes();
    Cid::from_blake3_digest(digest)
}

/// **Load-bearing acceptance test.** Production SANDBOX dispatch
/// reaches the wasmtime executor end-to-end via `Engine::call`.
///
/// HALF 1: with no bytes registered, the engine override surfaces the
///         typed `module bytes not registered` error string. This proves
///         the override IS being called (the trait default would have
///         surfaced `E_PRIMITIVE_NOT_IMPLEMENTED` instead).
///
/// HALF 2: with bytes registered, the call succeeds and routes through
///         the OK edge — the executor actually invoked the wasmtime
///         instance and returned a SandboxResult. The trait-default
///         path could not reach this state.
#[test]
fn sandbox_execute_via_engine_dispatch_invokes_executor() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();

    // Register the SANDBOX subgraph BEFORE registering bytes so we can
    // exercise the missing-bytes branch first.
    let spec = sandbox_spec("sandbox.test_invokes_executor", &module_cid_str);
    let handler_id = engine
        .register_subgraph(spec)
        .expect("SANDBOX-bearing SubgraphSpec must register cleanly on native targets");
    assert_eq!(
        handler_id, "sandbox.test_invokes_executor",
        "register_subgraph must return the same handler_id we passed to SubgraphSpec::builder().handler_id(...)"
    );

    // ------ HALF 1: no bytes registered -------------------------------
    let err = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect_err(
            "with no bytes registered for the declared module CID, the SANDBOX \
             dispatch MUST surface the engine override's typed error — NOT \
             the trait-default PrimitiveNotImplemented error",
        );
    let err_string = format!("{err:?}");
    assert!(
        err_string.contains("module bytes not registered")
            || err_string.contains("E_SANDBOX_MODULE_NOT_INSTALLED"),
        "HALF 1: error string must come from the engine `execute_sandbox` \
         override (the load-bearing proof that the override was called); \
         got: {err_string}"
    );
    // Companion assertion: the trait-default error would mention
    // PrimitiveNotImplemented. Its absence is corroborating evidence.
    assert!(
        !err_string.contains("PrimitiveNotImplemented"),
        "HALF 1: error string MUST NOT come from the trait-default \
         PrimitiveNotImplemented arm — that would prove the override is \
         NOT being called; got: {err_string}"
    );

    // ------ HALF 2: register bytes, call succeeds ---------------------
    engine.register_module_bytes(module_cid, module_bytes);

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect(
            "with bytes registered, SANDBOX dispatch MUST succeed end-to-end \
             through the wasmtime executor",
        );
    assert!(
        outcome.is_ok_edge(),
        "HALF 2: outcome must route through the OK edge after the \
         executor returns SandboxResult cleanly; got edge {:?} error_code {:?} \
         error_message {:?}",
        outcome.edge_taken(),
        outcome.error_code(),
        outcome.error_message(),
    );
}
