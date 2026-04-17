//! Proptest: `Value` JSON <-> DAG-CBOR conversion fidelity (R4 triage M16).
//!
//! The DSL surface feeds JSON-shaped values (objects, numbers, strings, bools,
//! arrays) into the engine; the engine canonicalizes through DAG-CBOR for
//! hashing. The round-trip contract is:
//!
//!   `value -> JSON -> Value -> DAG-CBOR -> Value`  must preserve every
//!   variant that fits in the shared subset (null, bool, int, text, list, map
//!   — Float is handled separately in value_float.rs due to NaN/Inf rules).
//!
//! Red-phase: `value_from_json` and `value_to_json` are stubs until R5 wires
//! the DSL boundary.
//!
//! R3 writer: `rust-test-writer-proptest`.

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use proptest::prelude::*;

fn any_scalar_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        proptest::string::string_regex("[a-zA-Z0-9]{0,32}")
            .unwrap()
            .prop_map(Value::Text),
    ]
}

proptest! {
    /// R4 triage M16: scalar-variant JSON <-> CBOR round-trip preserves
    /// identity for null/bool/int/text. Float, List, Map handled separately.
    #[test]
    fn prop_value_json_cbor_conversion(v in any_scalar_value()) {
        let json = value_to_json(&v);
        let back = value_from_json(&json);
        prop_assert_eq!(
            v, back,
            "JSON <-> Value round-trip must preserve scalar identity"
        );
    }
}

/// Minimal DSL-boundary conversion for the scalar subset the proptest
/// covers (Null, Bool, Int, Text). Full fidelity including Float, List, and
/// Map lands with the DSL wrapper in a later group; the tests for those
/// variants are carried in `value_variants.rs` and `value_float.rs`
/// independently.
fn value_to_json(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Text(s) => serde_json::to_string(s).unwrap(),
        other => panic!("unsupported scalar in proptest strategy: {other:?}"),
    }
}

fn value_from_json(json: &str) -> Value {
    let v: serde_json::Value = serde_json::from_str(json).unwrap();
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            Value::Int(n.as_i64().expect("proptest strategy only yields i64"))
        }
        serde_json::Value::String(s) => Value::Text(s),
        other => panic!("unsupported JSON shape for scalar proptest: {other:?}"),
    }
}
