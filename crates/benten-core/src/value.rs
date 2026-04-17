//! Graph value type — the DAG-CBOR subset exposed to user operations.
//!
//! The load-bearing property of [`Value`] is *hashability*: every variant must
//! serialize deterministically through `serde_ipld_dagcbor`, because
//! [`Node::canonical_bytes`](crate::Node::canonical_bytes) is the sole input to
//! content addressing. The one non-obvious wrinkle is [`Value::Float`]:
//!
//! - **NaN is rejected.** IEEE-754 allows roughly 2^53 distinct NaN payloads.
//!   Without a canonicalization rule two semantically-equal Nodes could hash
//!   differently, breaking determinism. Rather than pick a canonical payload we
//!   refuse to hash any NaN at all and surface
//!   [`CoreError::FloatNan`] / `E_VALUE_FLOAT_NAN`.
//! - **±Infinity is rejected.** Infinities have a canonical encoding, but they
//!   are not carried forward by TRANSFORM's expression evaluator without
//!   explicit handling, so Phase 1 simply blocks them and surfaces
//!   [`CoreError::FloatNonFinite`] /
//!   `E_VALUE_FLOAT_NONFINITE`.
//! - **`-0.0` hashes identically to `+0.0`.** The DAG-CBOR canonical form does
//!   not distinguish the two (they compare equal, and the shortest-form encoder
//!   emits the same bytes), so the CID is stable across the sign of zero.
//!
//! Validation is performed up-front in
//! [`Value::validate_no_nonfinite`] before the value is handed to the CBOR
//! encoder, which keeps the rejection path out of `serde`'s error channel.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::CoreError;

/// A graph Value. This is the subset of DAG-CBOR we expose to user operations.
///
/// Maps use [`BTreeMap`] so in-memory iteration order is deterministic;
/// the on-wire canonical form is separately enforced by `serde_ipld_dagcbor`
/// at encode time (DAG-CBOR length-first key sort).
///
/// `#[serde(untagged)]` is safe here because DAG-CBOR's major-type tagging
/// makes each variant's wire encoding unambiguous: a boolean cannot
/// deserialize as an integer because CBOR major type 7 (simple) and major
/// types 0/1 (unsigned/negative integer) are distinct.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// CBOR null.
    Null,
    /// CBOR boolean.
    Bool(bool),
    /// CBOR signed integer (-2^63 .. 2^63-1).
    Int(i64),
    /// CBOR 64-bit float. NaN and ±Infinity are rejected at hash time — see
    /// [`Value::validate_no_nonfinite`] and the module docs.
    Float(f64),
    /// CBOR text string (UTF-8).
    Text(String),
    /// CBOR byte string. `#[serde(with = "serde_bytes")]` keeps this on major
    /// type 2 (byte string) on serialization rather than being encoded as a
    /// CBOR array of small integers.
    Bytes(#[serde(with = "serde_bytes")] Vec<u8>),
    /// CBOR array.
    List(Vec<Value>),
    /// CBOR map with text keys (DAG-CBOR restricts map keys to strings).
    Map(BTreeMap<String, Value>),
}

// Custom `Deserialize` impl. `#[serde(untagged)]` alone is ambiguous for CBOR
// because the `serde` data model collapses CBOR byte strings and text strings
// into the same `visit_bytes`/`visit_str` channels — control-byte payloads
// round-trip as `Text`, and small-integer arrays round-trip as `Bytes`. The
// visitor below dispatches on the actual data-model type the CBOR decoder
// surfaces, preserving the variant identity.
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a DAG-CBOR value")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_none<E: serde::de::Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
                Value::deserialize(d)
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Int(v))
            }

            fn visit_i128<E: serde::de::Error>(self, v: i128) -> Result<Value, E> {
                i64::try_from(v).map(Value::Int).map_err(E::custom)
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
                i64::try_from(v).map(Value::Int).map_err(E::custom)
            }

            fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Value, E> {
                i64::try_from(v).map(Value::Int).map_err(E::custom)
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
                Ok(Value::Float(v))
            }

            fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<Value, E> {
                Ok(Value::Float(f64::from(v)))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
                Ok(Value::Text(v.into()))
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
                Ok(Value::Text(v))
            }

            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Value, E> {
                Ok(Value::Bytes(v.to_vec()))
            }

            fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Value, E> {
                Ok(Value::Bytes(v))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(v) = seq.next_element()? {
                    out.push(v);
                }
                Ok(Value::List(out))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
                let mut out = BTreeMap::new();
                while let Some((k, v)) = map.next_entry()? {
                    out.insert(k, v);
                }
                Ok(Value::Map(out))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Value {
    /// Convenience constructor for text values.
    pub fn text(s: impl Into<String>) -> Self {
        Value::Text(s.into())
    }

    /// Recursively walk the value tree and reject any non-finite
    /// [`Value::Float`]. Called from
    /// [`Node::canonical_bytes`](crate::Node::canonical_bytes) before the
    /// DAG-CBOR encoder runs, so the error surface is a real
    /// [`CoreError`] rather than a `serde` error wrapped in a `Serialize`
    /// variant.
    ///
    /// # Errors
    ///
    /// - [`CoreError::FloatNan`] — any encountered float is `NaN` (all
    ///   payloads, including signalling and non-default).
    /// - [`CoreError::FloatNonFinite`] — any encountered float is `+∞` or
    ///   `-∞`.
    pub fn validate_no_nonfinite(&self) -> Result<(), CoreError> {
        match self {
            Value::Null | Value::Bool(_) | Value::Int(_) | Value::Text(_) | Value::Bytes(_) => {
                Ok(())
            }
            Value::Float(f) => {
                if f.is_nan() {
                    Err(CoreError::FloatNan)
                } else if !f.is_finite() {
                    Err(CoreError::FloatNonFinite)
                } else {
                    Ok(())
                }
            }
            Value::List(items) => {
                for item in items {
                    item.validate_no_nonfinite()?;
                }
                Ok(())
            }
            Value::Map(entries) => {
                for v in entries.values() {
                    v.validate_no_nonfinite()?;
                }
                Ok(())
            }
        }
    }

    /// Produce a hashing-canonical clone of this value: validates that no
    /// non-finite floats are present, and normalizes `-0.0` to `+0.0` so the
    /// CID is stable across the sign of zero. Used by
    /// [`Node::canonical_bytes`](crate::Node::canonical_bytes).
    ///
    /// # Errors
    ///
    /// Propagates [`CoreError::FloatNan`] / [`CoreError::FloatNonFinite`]
    /// from [`Value::validate_no_nonfinite`].
    pub fn to_canonical(&self) -> Result<Value, CoreError> {
        match self {
            Value::Null => Ok(Value::Null),
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => {
                if f.is_nan() {
                    Err(CoreError::FloatNan)
                } else if !f.is_finite() {
                    Err(CoreError::FloatNonFinite)
                } else if *f == 0.0 {
                    // Collapse `-0.0` and `+0.0` to a single bit pattern so
                    // downstream hashing can't distinguish them.
                    Ok(Value::Float(0.0))
                } else {
                    Ok(Value::Float(*f))
                }
            }
            Value::Text(s) => Ok(Value::Text(s.clone())),
            Value::Bytes(b) => Ok(Value::Bytes(b.clone())),
            Value::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(item.to_canonical()?);
                }
                Ok(Value::List(out))
            }
            Value::Map(entries) => {
                let mut out = BTreeMap::new();
                for (k, v) in entries {
                    out.insert(k.clone(), v.to_canonical()?);
                }
                Ok(Value::Map(out))
            }
        }
    }
}
