//! Edge-case tests for `Value::Float` NaN / infinity / subnormal handling.
//!
//! Covers error codes:
//! - `E_VALUE_FLOAT_NAN`       — `f64::NAN` cannot enter the content hash
//! - `E_VALUE_FLOAT_NONFINITE` — `f64::INFINITY` / `NEG_INFINITY` cannot either
//!
//! Both are registered in ERROR-CATALOG.md as Phase 1 codes. The determinism
//! reason: NaN has no canonical bit pattern (quiet vs signalling, payload
//! bits), and infinity, while serializable, is excluded from Phase 1 because
//! the DSL surface never produces one; accepting it would create a divergent
//! test-fixture source.
//!
//! R3 contract: `Value::Float(f64)` does NOT exist in the spike (see
//! `crates/benten-core/src/lib.rs` module doc). R5 lands it under G1-A. These
//! tests fail to compile until the variant lands, which is the intent — they
//! pin the semantics before the implementation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{CoreError, Node, Value};

use alloc::collections::BTreeMap;
extern crate alloc;

fn build_node_with_float(key: &str, f: f64) -> Node {
    let mut props = BTreeMap::new();
    props.insert(key.into(), Value::Float(f));
    Node::new(vec!["Sensor".into()], props)
}

fn assert_float_nan_error(err: &CoreError) {
    // When ErrorCode lands in R5, this becomes:
    //   assert_eq!(err.code(), ErrorCode::E_VALUE_FLOAT_NAN);
    // For now, match the typed variant R5 will introduce.
    match err {
        CoreError::FloatNan => {}
        other => panic!("expected CoreError::FloatNan, got {other:?}"),
    }
}

fn assert_float_nonfinite_error(err: &CoreError) {
    match err {
        CoreError::FloatNonFinite => {}
        other => panic!("expected CoreError::FloatNonFinite, got {other:?}"),
    }
}

#[test]
fn float_nan_rejected() {
    // The API-honest "no": a NaN in any property must fail to hash.
    // Canonical NaN, signalling NaN, and NaN with non-default payload must
    // all be rejected identically — the engine does not attempt to
    // canonicalize them.
    for nan_bits in [
        0x7ff8_0000_0000_0000u64,
        0x7ff8_0000_0000_0001u64,
        0xfff8_0000_0000_0000u64,
    ] {
        let nan = f64::from_bits(nan_bits);
        assert!(nan.is_nan(), "test setup: bit pattern must be NaN");
        let node = build_node_with_float("reading", nan);
        let err = node.cid().unwrap_err();
        assert_float_nan_error(&err);
    }
}

#[test]
fn float_infinity_rejected() {
    // Positive and negative infinity are finite in CBOR's sense (major type
    // 7, simple value 25/26/27 for half/float/double with exponent all-ones),
    // but Benten refuses them in Phase 1 for determinism reasons.
    for f in [f64::INFINITY, f64::NEG_INFINITY] {
        let node = build_node_with_float("reading", f);
        let err = node.cid().unwrap_err();
        assert_float_nonfinite_error(&err);
    }
}

#[test]
fn float_zero_and_negative_zero_are_equal_cid() {
    // Boundary: +0.0 and -0.0 are distinct `f64` bit patterns but equal
    // under IEEE-754 `==`. DAG-CBOR canonicalization normalises them to
    // the shortest form; Benten accepts both and produces the same CID.
    let a = build_node_with_float("reading", 0.0).cid().unwrap();
    let b = build_node_with_float("reading", -0.0).cid().unwrap();
    assert_eq!(
        a, b,
        "positive and negative zero must canonicalize to the same CID"
    );
}

#[test]
fn float_finite_extremes_accepted() {
    // The API honestly says yes: finite extremes (MIN, MAX, MIN_POSITIVE,
    // subnormals) all hash successfully. This is the positive-boundary pair
    // to the negative-boundary NaN/Inf tests above.
    for f in [
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::from_bits(1), // smallest positive subnormal
    ] {
        assert!(f.is_finite(), "test setup");
        let node = build_node_with_float("reading", f);
        let _cid = node.cid().expect("finite float must hash without error");
    }
}
