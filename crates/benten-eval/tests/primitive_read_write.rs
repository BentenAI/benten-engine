//! READ + WRITE primitive happy-path + conflict tests (E3, G6-A — R2
//! landscape §2.5 rows 3 + 5).
//!
//! READ: `read_by_id_found` / `read_by_id_missing` / `read_by_query_empty_result`.
//! WRITE: create / update / delete / CAS; CAS wrong version → `ON_CONFLICT`
//! with `E_WRITE_CONFLICT`.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_WRITE_CONFLICT`.

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_eval::{EvalError, Evaluator, OperationNode, PrimitiveKind};

fn read_op(id: &str) -> OperationNode {
    OperationNode::new(id, PrimitiveKind::Read)
}

fn write_op(id: &str) -> OperationNode {
    OperationNode::new(id, PrimitiveKind::Write)
}

#[test]
fn read_primitive_step_returns_ok_on_happy_path() {
    let mut ev = Evaluator::new();
    let op = read_op("r1").with_property("target_cid", Value::text("found"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ok");
}

#[test]
fn read_primitive_missing_routes_to_on_not_found() {
    let mut ev = Evaluator::new();
    let op = read_op("r1").with_property("target_cid", Value::text("missing"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ON_NOT_FOUND");
}

#[test]
fn read_primitive_empty_query_routes_to_on_empty() {
    let mut ev = Evaluator::new();
    let op = read_op("r1").with_property("query_kind", Value::text("empty"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ON_EMPTY");
}

#[test]
fn write_primitive_create_returns_ok() {
    let mut ev = Evaluator::new();
    let op = write_op("w1").with_property("op", Value::text("create"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ok");
}

#[test]
fn write_primitive_update_returns_ok() {
    let mut ev = Evaluator::new();
    let op = write_op("w1").with_property("op", Value::text("update"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ok");
}

#[test]
fn write_primitive_delete_returns_ok() {
    let mut ev = Evaluator::new();
    let op = write_op("w1").with_property("op", Value::text("delete"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ok");
}

/// Covered by `covers_error_code[E_WRITE_CONFLICT]` entry
/// "write_cas_wrong_version_routes_on_conflict".
#[test]
fn write_cas_wrong_version_routes_on_conflict() {
    let mut ev = Evaluator::new();
    let op = write_op("w1")
        .with_property("op", Value::text("cas"))
        .with_property("expected_version", Value::Int(1))
        .with_property("actual_version", Value::Int(2));
    match ev.step(&op) {
        Ok(r) => assert_eq!(r.edge_label, "ON_CONFLICT"),
        Err(EvalError::WriteConflict) => {}
        Err(e) => panic!("expected ON_CONFLICT edge or E_WRITE_CONFLICT, got {e:?}"),
    }
}
