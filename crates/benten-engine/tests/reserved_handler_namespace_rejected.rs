//! Phase-3 G21-T3 §2.5(d) — `engine:typed:` handler_id namespace
//! registration-time hard reject (corr-minor-3 fold-in from G21-T1
//! fp-mini-review).
//!
//! Per phase-3-backlog §2.5(d): `Engine::register_subgraph` +
//! `register_subgraph_replace` MUST reject any handler_id that
//! starts with the reserved `engine:typed:` namespace prefix. The
//! eval-side dispatch fork pre-empts user-handler routing for this
//! prefix (the typed-CALL registry is closed; extension is a
//! Rust-only engine concern per CLAUDE.md baked-in commitment #16),
//! so a user registration in this namespace would be silent dead
//! code without this guard.
//!
//! 4-surface §3.5g atomic update:
//! 1. Rust enum: `ErrorCode::ReservedHandlerNamespace` at
//!    `crates/benten-errors/src/lib.rs`.
//! 2. ERROR-CATALOG row: `### E_RESERVED_HANDLER_NAMESPACE` at
//!    `docs/ERROR-CATALOG.md`.
//! 3. Codegen: `EReservedHandlerNamespace` at
//!    `packages/engine/src/errors.generated.ts` (regenerated via
//!    `npx tsx scripts/codegen-errors.ts`).
//! 4. Test catalog (this file).

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{OperationNode, Subgraph};
use benten_engine::{Engine, EngineError};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// Build a minimal subgraph with a custom handler_id (single
/// RESPOND node — sufficient to trigger the namespace guard at
/// registration time, which fires BEFORE invariant validation).
fn subgraph_with_handler_id(handler_id: &str) -> Subgraph {
    Subgraph::new(handler_id).with_node(OperationNode::new("r0", PrimitiveKind::Respond))
}

#[test]
fn register_subgraph_rejects_engine_typed_prefix() {
    let (_dir, engine) = fresh_engine();
    let sg = subgraph_with_handler_id("engine:typed:custom_op");

    let err = engine
        .register_subgraph(sg)
        .expect_err("registration MUST be rejected for reserved namespace");

    match err {
        EngineError::Other { code, message } => {
            assert_eq!(
                code,
                ErrorCode::ReservedHandlerNamespace,
                "MUST surface E_RESERVED_HANDLER_NAMESPACE; got {code:?}"
            );
            assert!(
                message.contains("engine:typed:"),
                "error message MUST cite the reserved prefix; got {message}"
            );
            assert!(
                message.contains("custom_op"),
                "error message MUST echo the offending handler_id; got {message}"
            );
        }
        other => panic!("expected EngineError::Other; got {other:?}"),
    }
}

#[test]
fn register_subgraph_rejects_engine_typed_with_known_op_name() {
    // Even using a known typed-CALL op name as handler_id MUST be
    // rejected — the engine-internal typed-CALL registry is the
    // sole owner of the prefix.
    let (_dir, engine) = fresh_engine();
    let sg = subgraph_with_handler_id("engine:typed:ed25519_sign");

    let err = engine
        .register_subgraph(sg)
        .expect_err("user registration of a typed-CALL op name MUST be rejected");
    match err {
        EngineError::Other { code, .. } => assert_eq!(
            code,
            ErrorCode::ReservedHandlerNamespace,
            "user registration of `engine:typed:ed25519_sign` MUST surface \
             E_RESERVED_HANDLER_NAMESPACE (not E_TYPED_CALL_UNKNOWN_OP — \
             that fires at eval time; this fires at registration time)"
        ),
        other => panic!("expected EngineError::Other; got {other:?}"),
    }
}

#[test]
fn register_subgraph_accepts_engine_other_prefix() {
    // The reject is namespace-bound to `engine:typed:` exactly —
    // `engine:other:foo` (different reserved sub-namespace) MUST
    // pass the typed-CALL guard. (Whether other reserved prefixes
    // need their own guards is a separate question.)
    let (_dir, engine) = fresh_engine();
    let sg = subgraph_with_handler_id("engine:other:foo");

    // The handler may still fail validation for unrelated reasons
    // (no actual registered subgraph shape matters here), but the
    // failure MUST NOT be E_RESERVED_HANDLER_NAMESPACE.
    match engine.register_subgraph(sg) {
        Ok(_) => {} // accepted — fine
        Err(EngineError::Other { code, .. }) => assert_ne!(
            code,
            ErrorCode::ReservedHandlerNamespace,
            "non-`engine:typed:` prefix MUST NOT trip the namespace guard"
        ),
        Err(_) => {} // unrelated registration failure is acceptable
    }
}

#[test]
fn register_subgraph_rejects_exact_prefix_match() {
    // The prefix is `engine:typed:`. A handler_id of just
    // `engine:typed:` (zero op name) should also reject; the
    // dispatch fork would also surface E_TYPED_CALL_UNKNOWN_OP for
    // an empty op name.
    let (_dir, engine) = fresh_engine();
    let sg = subgraph_with_handler_id("engine:typed:");

    let err = engine
        .register_subgraph(sg)
        .expect_err("zero-op-name reserved-prefix MUST be rejected");
    if let EngineError::Other { code, .. } = err {
        assert_eq!(code, ErrorCode::ReservedHandlerNamespace);
    } else {
        panic!("expected EngineError::Other");
    }
}

#[test]
fn register_subgraph_replace_also_rejects_reserved_prefix() {
    // Defense-in-depth: even if a future change accidentally
    // bypassed the `register_subgraph` check, the replace path
    // must still reject.
    let (_dir, engine) = fresh_engine();
    let sg = subgraph_with_handler_id("engine:typed:custom_op_replace");

    let err = engine
        .register_subgraph_replace(sg)
        .expect_err("replace MUST also reject reserved-prefix handler_id");
    if let EngineError::Other { code, .. } = err {
        assert_eq!(
            code,
            ErrorCode::ReservedHandlerNamespace,
            "replace path MUST surface E_RESERVED_HANDLER_NAMESPACE; got {code:?}"
        );
    } else {
        panic!("expected EngineError::Other");
    }
}

#[test]
fn error_code_routed_edge_label_is_none_for_reserved_namespace() {
    // Registration-time refusals do not route along primitive
    // edges (same disposition as ViewStrategyARefused +
    // DuplicateHandler).
    assert_eq!(
        ErrorCode::ReservedHandlerNamespace.routed_edge_label(),
        None,
        "registration-time refusals MUST have no edge-label routing"
    );
}

#[test]
fn error_code_string_round_trip() {
    assert_eq!(
        ErrorCode::ReservedHandlerNamespace.as_str(),
        "E_RESERVED_HANDLER_NAMESPACE"
    );
    assert_eq!(
        ErrorCode::from_str("E_RESERVED_HANDLER_NAMESPACE"),
        ErrorCode::ReservedHandlerNamespace
    );
}
