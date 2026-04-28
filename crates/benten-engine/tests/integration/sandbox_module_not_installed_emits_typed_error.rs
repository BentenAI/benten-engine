//! Phase 2b Wave-8d-types acceptance test.
//!
//! Pin: a SANDBOX dispatch whose declared module CID has no bytes
//! registered MUST surface the typed
//! [`benten_eval::EvalError::Sandbox`] variant carrying
//! [`benten_eval::sandbox::SandboxError::ModuleNotInstalled`] (NOT the
//! placeholder `EvalError::Backend(format!("..."))` shape used through
//! Wave-8b). The catalog code on the resulting `EngineError` MUST
//! be [`ErrorCode::SandboxModuleNotInstalled`] (`E_SANDBOX_MODULE_NOT_INSTALLED`).
//!
//! Companion to `tests/sandbox_execute_via_engine_dispatch_invokes_executor.rs`
//! (the Wave-8b acceptance test which used string-matching on the prior
//! placeholder shape). This test asserts the Wave-8d-types typed-error
//! refactor: the override emits the typed `EvalError::Sandbox` variant
//! and the stable `E_SANDBOX_MODULE_NOT_INSTALLED` discriminant survives
//! the `EvalError → EngineError → Outcome.error_code` boundary.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;

/// Build a 2-node SubgraphSpec (SANDBOX -> RESPOND) carrying the
/// SANDBOX node's `module` (base32 CID) + `caps` (inline manifest)
/// properties. Mirrors the Wave-8b acceptance test's helper.
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

/// **Load-bearing acceptance test (Wave-8d-types).** A SANDBOX
/// dispatch whose declared module CID has no bytes registered MUST
/// surface the typed `EvalError::Sandbox(SandboxError::ModuleNotInstalled(_))`
/// variant + the resulting EngineError's catalog code MUST be
/// `ErrorCode::SandboxModuleNotInstalled` (`E_SANDBOX_MODULE_NOT_INSTALLED`).
///
/// This test would FAIL against the pre-Wave-8d-types tree because:
///
/// 1. `EvalError::Sandbox(_)` did not exist as a variant; the override
///    surfaced `EvalError::Backend(format!("...E_SANDBOX_MODULE_NOT_INSTALLED..."))`
///    instead.
/// 2. The resulting `EngineError`'s catalog code was
///    `Unknown("E_EVAL_BACKEND")`, NOT `SandboxModuleNotInstalled`.
///
/// Both halves of the post-refactor reality are asserted.
#[test]
fn sandbox_module_not_installed_emits_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Construct a deterministic CID that we DO NOT register bytes for.
    // The test exercises the missing-bytes path specifically.
    let unregistered_module_cid = Cid::from_blake3_digest(
        *blake3::hash(b"sandbox_module_not_installed_emits_typed_error::unregistered").as_bytes(),
    );
    let module_cid_str = unregistered_module_cid.to_base32();

    let spec = sandbox_spec(
        "sandbox.module_not_installed_emits_typed_error",
        &module_cid_str,
    );
    let handler_id = engine
        .register_subgraph(spec)
        .expect("SANDBOX-bearing SubgraphSpec must register cleanly on native targets");

    // Dispatch through the production path. The override at
    // `crates/benten-engine/src/primitive_host.rs::execute_sandbox`
    // must hit the missing-bytes branch and surface the typed variant.
    let err = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect_err(
            "with no bytes registered for the declared module CID, the SANDBOX \
             dispatch MUST surface a typed engine error",
        );

    // Half 1: the resulting EngineError's stable catalog code is
    // `E_SANDBOX_MODULE_NOT_INSTALLED`. This is the load-bearing
    // assertion — it would fail with `E_EVAL_BACKEND` against the
    // pre-Wave-8d-types tree where the variant didn't exist.
    let code = err.code();
    assert_eq!(
        code,
        ErrorCode::SandboxModuleNotInstalled,
        "EngineError.code() must surface E_SANDBOX_MODULE_NOT_INSTALLED \
         after the Wave-8d-types typed-error refactor; got {code:?} \
         (full error: {err:?})"
    );
    assert_eq!(
        code.as_str(),
        "E_SANDBOX_MODULE_NOT_INSTALLED",
        "stable catalog string must match the docs/ERROR-CATALOG.md \
         row added in Wave-8d-types"
    );

    // Half 2: the error message names the missing CID for operator
    // triage. The exact CID base32 string is part of the actionable
    // text per the SandboxError::ModuleNotInstalled Display impl.
    let err_string = format!("{err}");
    assert!(
        err_string.contains(&module_cid_str) || format!("{err:?}").contains(&module_cid_str),
        "error rendering MUST include the missing module CID for \
         operator triage; got message {err_string:?} debug {err:?}"
    );

    // Half 3: ANTI-REGRESSION — the Wave-8b placeholder `Backend` shape
    // would have stuffed the actionable text into a `Backend(String)`
    // variant whose code() returned `Unknown("E_EVAL_BACKEND")`. The
    // post-refactor world MUST NOT route through that placeholder.
    assert_ne!(
        code,
        ErrorCode::Unknown(String::from("E_EVAL_BACKEND")),
        "regression guard: the typed `EvalError::Sandbox` route MUST NOT \
         collapse back into the Wave-8b `EvalError::Backend(String)` \
         placeholder; got {err:?}"
    );
}
