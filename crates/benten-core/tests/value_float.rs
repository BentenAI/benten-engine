//! `Value::Float(f64)` contract tests (C3, R2 landscape §2.1 row 2).
//!
//! Canonical location for float-related tests. Merged at R4 triage (M10) —
//! the former `float_nan_inf.rs` contained overlapping tests with divergent
//! contracts; its unique coverage (three NaN bit patterns, finite extremes
//! boundary) is folded into this file.
//!
//! Phase 1 G1-A ships NaN + ±Inf rejection plus shortest-form encoding. These
//! tests are the R3 TDD spec — they fail until G1-A lands.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Error codes fired: `E_VALUE_FLOAT_NAN`, `E_VALUE_FLOAT_NONFINITE`.

#![allow(clippy::unwrap_used)]

use benten_core::{CoreError, ErrorCode, Node, Value};
use std::collections::BTreeMap;

fn node_with_float(f: f64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("f".to_string(), Value::Float(f));
    Node::new(vec!["T".to_string()], p)
}

/// Covered by `covers_error_code[E_VALUE_FLOAT_NAN]` entry in r3-coverage-stub.
#[test]
fn float_nan_rejected() {
    let n = node_with_float(f64::NAN);
    let err = n.canonical_bytes().expect_err("NaN must be rejected");
    assert!(
        matches!(err, CoreError::FloatNan),
        "expected CoreError::FloatNan, got {err:?}"
    );
    assert_eq!(err.code(), ErrorCode::ValueFloatNan);
}

/// Covered by `covers_error_code[E_VALUE_FLOAT_NONFINITE]`.
#[test]
fn float_infinity_rejected() {
    let n = node_with_float(f64::INFINITY);
    let err = n.canonical_bytes().expect_err("+Inf must be rejected");
    assert!(matches!(err, CoreError::FloatNonFinite));
    assert_eq!(err.code(), ErrorCode::ValueFloatNonFinite);
}

#[test]
fn float_negative_infinity_rejected() {
    let n = node_with_float(f64::NEG_INFINITY);
    let err = n.canonical_bytes().expect_err("-Inf must be rejected");
    assert!(matches!(err, CoreError::FloatNonFinite));
}

#[test]
fn float_finite_one_encodes_shortest_form() {
    // 1.0 must encode as the CBOR smallest representation (16-bit half if
    // lossless, else 32-bit, else 64). Assert the CID stays stable across
    // re-encode — the shortest-form contract is that decoding `1.0` back and
    // re-encoding produces identical bytes.
    let n = node_with_float(1.0);
    let bytes = n.canonical_bytes().unwrap();
    let decoded: Node = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
    let rebytes = decoded.canonical_bytes().unwrap();
    assert_eq!(bytes, rebytes, "shortest-form encoding must be idempotent");
}

#[test]
fn float_zero_positive_and_negative_have_same_cid() {
    // DAG-CBOR normalizes -0.0 to 0.0 for determinism.
    let pos = node_with_float(0.0);
    let neg = node_with_float(-0.0);
    assert_eq!(
        pos.cid().unwrap(),
        neg.cid().unwrap(),
        "+0.0 and -0.0 must hash identically"
    );
}

#[test]
fn float_nested_in_map_roundtrips() {
    let mut inner = BTreeMap::new();
    inner.insert("v".to_string(), Value::Float(3.14));
    let mut p = BTreeMap::new();
    p.insert("m".to_string(), Value::Map(inner.clone()));
    let n = Node::new(vec!["T".to_string()], p);
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("m").unwrap(), &Value::Map(inner));
}

// -- Folded in from float_nan_inf.rs at R4 triage (M10) --------------------

/// Canonical NaN, signalling NaN, and NaN with non-default payload must all
/// be rejected identically — the engine does not canonicalize NaN payloads.
#[test]
fn float_nan_multiple_bit_patterns_all_rejected() {
    for nan_bits in [
        0x7ff8_0000_0000_0000u64,
        0x7ff8_0000_0000_0001u64,
        0xfff8_0000_0000_0000u64,
    ] {
        let nan = f64::from_bits(nan_bits);
        assert!(nan.is_nan(), "test setup: bit pattern must be NaN");
        let node = node_with_float(nan);
        let err = node.cid().unwrap_err();
        assert!(
            matches!(err, CoreError::FloatNan),
            "all NaN payloads must reject with FloatNan; got {err:?}"
        );
    }
}

/// Positive-boundary pair to the NaN/Inf rejection: finite extremes
/// (MIN, MAX, MIN_POSITIVE, EPSILON, smallest subnormal) all hash
/// successfully.
#[test]
fn float_finite_extremes_accepted() {
    for f in [
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::from_bits(1),
    ] {
        assert!(f.is_finite(), "test setup");
        let node = node_with_float(f);
        let _cid = node.cid().expect("finite float must hash without error");
    }
}
