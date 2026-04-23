//! Phase 2a R3 / G4-A test — READ primitive honours Option C symmetric-None
//! end-to-end via the evaluator path.
//!
//! Named compromise #2 (Option C): a capability-denied READ collapses to
//! `ON_NOT_FOUND` so an unauthorised reader cannot distinguish "denied" from
//! "never existed". Phase 1 shipped this at the engine-layer public API
//! (`Engine::get_node`); Phase 2a G4-A threads the same gate into the
//! evaluator-layer READ primitive via `PrimitiveHost::check_read_capability`.
//!
//! The contract pinned here: when a host's `check_read_capability` returns
//! `EvalError::Capability(_)`, the READ primitive executor's
//! `ON_NOT_FOUND` edge MUST fire — NOT the happy-path `ok` edge, and NOT
//! the `ON_DENIED` edge (that would leak existence). Covers both the
//! by-CID branch (bytes-encoded `target_cid`) and the by-query branch
//! (`query_kind="label"` + `label` property).
//!
//! Owner: G4-A (rust-implementation-developer). Red-phase would fail
//! because the primitive path did not consult `check_read_capability`
//! before Phase 2a.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Edge, Node, Value};
use benten_eval::host::ViewQuery;
use benten_eval::{EvalError, OperationNode, PrimitiveHost, PrimitiveKind};
use std::collections::BTreeMap;

/// Host that always denies reads — mirrors the shape of the engine's real
/// policy plumbing without requiring the engine to be instantiated.
struct DenyReadsHost;

impl PrimitiveHost for DenyReadsHost {
    fn read_node(&self, _cid: &Cid) -> Result<Option<Node>, EvalError> {
        // If the READ primitive's cap gate is wired correctly, we never
        // reach this method under a deny-reads host. If we do, panic —
        // the test explicitly wants to catch that regression.
        panic!(
            "read_node must not be called when check_read_capability denies \
             — the READ primitive should collapse to ON_NOT_FOUND before \
             touching the backend"
        );
    }
    fn get_by_label(&self, _label: &str) -> Result<Vec<Cid>, EvalError> {
        panic!(
            "get_by_label must not be called when check_read_capability denies \
             — the READ primitive should collapse to ON_EMPTY before \
             touching the backend"
        );
    }
    fn get_by_property(
        &self,
        _label: &str,
        _prop: &str,
        _value: &Value,
    ) -> Result<Vec<Cid>, EvalError> {
        Ok(Vec::new())
    }
    fn put_node(&self, _node: &Node) -> Result<Cid, EvalError> {
        Err(EvalError::Backend(
            "DenyReadsHost: put_node unsupported".into(),
        ))
    }
    fn put_edge(&self, _edge: &Edge) -> Result<Cid, EvalError> {
        Err(EvalError::Backend(
            "DenyReadsHost: put_edge unsupported".into(),
        ))
    }
    fn delete_node(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn delete_edge(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn call_handler(&self, _: &str, _: &str, _: Node) -> Result<Value, EvalError> {
        Err(EvalError::Backend("unsupported".into()))
    }
    fn emit_event(&self, _: &str, _: Value) {}
    fn check_capability(&self, _: &str, _: Option<&Cid>) -> Result<(), EvalError> {
        Ok(())
    }
    fn read_view(&self, _: &str, _: &ViewQuery) -> Result<Value, EvalError> {
        Err(EvalError::Backend("unsupported".into()))
    }
    fn check_read_capability(
        &self,
        _label: &str,
        _target_cid: Option<&Cid>,
    ) -> Result<(), EvalError> {
        // The Option-C DeniedRead shape. The READ primitive must see this
        // and collapse to ON_NOT_FOUND (symmetric with a miss).
        Err(EvalError::Capability(benten_caps::CapError::DeniedRead {
            required: "post:read".to_string(),
            entity: String::new(),
        }))
    }
}

#[test]
fn read_primitive_option_c_symmetric_by_cid_collapses_to_not_found() {
    let host = DenyReadsHost;
    // Fabricate a well-formed CID so the READ primitive parses it but
    // never reaches the backend (the cap gate must short-circuit).
    let cid = Cid::from_blake3_digest([7u8; 32]);
    let mut props = BTreeMap::new();
    props.insert(
        "target_cid".to_string(),
        Value::Bytes(cid.as_bytes().to_vec()),
    );
    let op = OperationNode {
        id: "read_1".into(),
        kind: PrimitiveKind::Read,
        properties: props,
    };
    let step = benten_eval::primitives::read::execute(&op, &host)
        .expect("READ primitive must route via typed edge under denial");
    assert_eq!(
        step.edge_label, "ON_NOT_FOUND",
        "Option C symmetric-None: a cap-denied by-CID READ must route \
         ON_NOT_FOUND, not ON_DENIED. Got `{}`",
        step.edge_label
    );
}

#[test]
fn read_primitive_option_c_symmetric_by_label_collapses_to_empty() {
    let host = DenyReadsHost;
    let mut props = BTreeMap::new();
    props.insert("query_kind".to_string(), Value::text("label"));
    props.insert("label".to_string(), Value::text("post"));
    let op = OperationNode {
        id: "read_by_label".into(),
        kind: PrimitiveKind::Read,
        properties: props,
    };
    let step = benten_eval::primitives::read::execute(&op, &host)
        .expect("READ by-label must route via typed edge under denial");
    assert_eq!(
        step.edge_label, "ON_EMPTY",
        "Option C symmetric-empty: a cap-denied by-label READ must route \
         ON_EMPTY, not ON_DENIED. Got `{}`",
        step.edge_label
    );
    // The payload is an empty list — symmetric with "no matching Nodes".
    match step.output {
        Value::List(items) => assert!(
            items.is_empty(),
            "Option C: by-label denial must produce an empty list, got {} items",
            items.len()
        ),
        other => panic!("expected empty list, got {other:?}"),
    }
}
