//! TRANSFORM primitive expression evaluator — built-in coverage (E4, G6-B —
//! R2 landscape §2.5 row 7).
//!
//! Exercises a sample of the 50+ built-ins: arithmetic, Math.min/max/round,
//! string lowercase/uppercase/truncate, array map/filter/reduce/length,
//! object construction, property access, ternary, comparison, logical.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_eval::{Evaluator, NullHost, OperationNode, PrimitiveKind};
use std::collections::BTreeMap;

fn transform(expr: &str, input: Value) -> Value {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("t", PrimitiveKind::Transform)
        .with_property("expr", Value::text(expr))
        .with_property("input", input);
    ev.step(&op, &NullHost).unwrap().output
}

#[test]
fn transform_arithmetic_addition() {
    assert_eq!(transform("1 + 2", Value::Null), Value::Int(3));
}

#[test]
fn transform_arithmetic_subtraction() {
    assert_eq!(transform("10 - 4", Value::Null), Value::Int(6));
}

#[test]
fn transform_arithmetic_multiplication() {
    assert_eq!(transform("6 * 7", Value::Null), Value::Int(42));
}

#[test]
fn transform_comparison_returns_bool() {
    assert_eq!(transform("1 < 2", Value::Null), Value::Bool(true));
    assert_eq!(transform("2 < 1", Value::Null), Value::Bool(false));
}

#[test]
fn transform_logical_and() {
    assert_eq!(transform("true && false", Value::Null), Value::Bool(false));
}

#[test]
fn transform_logical_or() {
    assert_eq!(transform("false || true", Value::Null), Value::Bool(true));
}

#[test]
fn transform_ternary_picks_true_branch() {
    assert_eq!(transform("true ? 1 : 2", Value::Null), Value::Int(1));
}

#[test]
fn transform_ternary_picks_false_branch() {
    assert_eq!(transform("false ? 1 : 2", Value::Null), Value::Int(2));
}

#[test]
fn transform_string_lowercase_returns_lowercased() {
    assert_eq!(
        transform("$input.toLowerCase()", Value::text("HELLO")),
        Value::text("hello")
    );
}

#[test]
fn transform_string_uppercase_returns_uppercased() {
    assert_eq!(
        transform("$input.toUpperCase()", Value::text("hello")),
        Value::text("HELLO")
    );
}

#[test]
fn transform_math_min_returns_smaller() {
    assert_eq!(transform("Math.min(3, 7)", Value::Null), Value::Int(3));
}

#[test]
fn transform_math_max_returns_larger() {
    assert_eq!(transform("Math.max(3, 7)", Value::Null), Value::Int(7));
}

#[test]
fn transform_math_abs_negative_returns_positive() {
    assert_eq!(transform("Math.abs(-5)", Value::Null), Value::Int(5));
}

#[test]
fn transform_array_length_returns_count() {
    let input = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(transform("$input.length", input), Value::Int(3));
}

#[test]
fn transform_array_map_projects_each_element() {
    let input = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let r = transform("$input.map(x => x * 2)", input);
    assert_eq!(
        r,
        Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)])
    );
}

#[test]
fn transform_array_filter_retains_truthy() {
    let input = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let r = transform("$input.filter(x => x > 1)", input);
    assert_eq!(r, Value::List(vec![Value::Int(2), Value::Int(3)]));
}

#[test]
fn transform_array_reduce_folds_sum() {
    let input = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let r = transform("$input.reduce((a, b) => a + b, 0)", input);
    assert_eq!(r, Value::Int(6));
}

#[test]
fn transform_object_construction_builds_map() {
    let r = transform("({ x: 1, y: 2 })", Value::Null);
    let mut expected = BTreeMap::new();
    expected.insert("x".to_string(), Value::Int(1));
    expected.insert("y".to_string(), Value::Int(2));
    assert_eq!(r, Value::Map(expected));
}

#[test]
fn transform_property_access_reads_from_input_map() {
    let mut m = BTreeMap::new();
    m.insert("name".to_string(), Value::text("bob"));
    assert_eq!(transform("$input.name", Value::Map(m)), Value::text("bob"));
}
