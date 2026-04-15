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

/// Phase 1 red-phase shim. R5 wires the real DSL-boundary conversion.
fn value_to_json(_v: &Value) -> String {
    todo!("value_to_json: R5 must wire the DSL-boundary conversion")
}

/// Phase 1 red-phase shim.
fn value_from_json(_json: &str) -> Value {
    todo!("value_from_json: R5 must wire the DSL-boundary conversion")
}
