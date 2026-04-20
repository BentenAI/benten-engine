//! `PrimitiveHost::put_edge` / `delete_edge` fail loud (r6b-ce-2 regression).
//!
//! Prior to this fix, `PrimitiveHost::put_edge` / `delete_edge` buffered a
//! pending op AND returned `Ok` to the evaluator, but the replay arm in
//! `dispatch_call_inner` silently dropped the op — a torn-state hazard
//! that contradicted the "buffer+replay ALL-or-NONE" atomicity claim.
//!
//! The Phase-1 fix is to fail loud at the host boundary: both methods now
//! return `EvalError::Unsupported` (catalog code `E_NOT_IMPLEMENTED`).
//! A Phase-2 `EngineTransaction` edge API will wire the real replay path.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Edge, Node, Value};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use benten_eval::{EvalError, PrimitiveHost};

fn canonical_edge() -> Edge {
    // Two stand-in CIDs — endpoints don't need to exist for the host
    // method to refuse the op; the refusal is unconditional.
    let src_cid = {
        let mut p = BTreeMap::new();
        p.insert("x".into(), Value::Int(1));
        Node::new(vec!["A".into()], p).cid().unwrap()
    };
    let tgt_cid = {
        let mut p = BTreeMap::new();
        p.insert("x".into(), Value::Int(2));
        Node::new(vec!["B".into()], p).cid().unwrap()
    };
    Edge::new(src_cid, tgt_cid, "L".to_string(), None)
}

#[test]
fn primitive_host_put_edge_returns_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let edge = canonical_edge();
    // Invoke via the PrimitiveHost trait path so we exercise the shim the
    // evaluator would see — not the engine's own `create_edge` surface.
    let res = <Engine as PrimitiveHost>::put_edge(&engine, &edge);
    match res {
        Err(EvalError::Unsupported { operation }) => {
            assert_eq!(operation, "put_edge");
        }
        other => {
            panic!("expected EvalError::Unsupported {{ operation: \"put_edge\" }}; got {other:?}")
        }
    }
    // The catalog code must be `E_NOT_IMPLEMENTED` so the stable error
    // identity survives cross-crate boundaries.
    let err = <Engine as PrimitiveHost>::put_edge(&engine, &edge).unwrap_err();
    assert_eq!(err.code(), ErrorCode::NotImplemented);
}

#[test]
fn primitive_host_delete_edge_returns_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let edge = canonical_edge();
    let edge_cid = edge.cid().unwrap();
    let res = <Engine as PrimitiveHost>::delete_edge(&engine, &edge_cid);
    match res {
        Err(EvalError::Unsupported { operation }) => {
            assert_eq!(operation, "delete_edge");
        }
        other => panic!(
            "expected EvalError::Unsupported {{ operation: \"delete_edge\" }}; got {other:?}"
        ),
    }
    let err = <Engine as PrimitiveHost>::delete_edge(&engine, &edge_cid).unwrap_err();
    assert_eq!(err.code(), ErrorCode::NotImplemented);
}
