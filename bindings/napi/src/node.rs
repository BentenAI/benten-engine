//! JSON <-> `benten_core::Node` conversion for the napi surface.
//!
//! Shape exposed to TypeScript:
//! ```json
//! { "labels": ["Post"], "properties": { "title": "Hello" } }
//! ```
//!
//! Numbers round-trip through `i64` when they have no fractional component;
//! floats (with fractional part or when the number exceeds `i64::MAX`) land
//! in `Value::Float`. Arrays become `Value::List`, objects become
//! `Value::Map`, Uint8Array-shaped arrays-of-integers land as `Value::Bytes`
//! when every element is `0..=255`.

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use napi::bindgen_prelude::*;

use crate::error::core_err;

/// Depth cap enforced while walking a JSON tree into `Value`.
///
/// Each level of nested object/array counts as one unit of depth. Strictly
/// shallower than the napi boundary's theoretical limit — B8 wires a harder
/// `E_INPUT_LIMIT` check in its in-process-test surface; the Phase-1 class
/// binding only needs a DoS tripwire so a pathological JS input doesn't
/// blow the Rust stack.
const JSON_MAX_DEPTH: usize = 128;

/// Map key count ceiling applied to every nested object in the JSON tree.
const JSON_MAX_MAP_KEYS: usize = 10_000;

/// Byte payload ceiling (1 MiB) applied to any `Uint8Array`-shaped property.
const JSON_MAX_BYTES: usize = 1024 * 1024;

/// Decode a JSON object into the property map of a Benten Node.
///
/// # Errors
///
/// * `E_INPUT_LIMIT` — depth, map-key, or bytes limit exceeded.
/// * `InvalidArg` — the root JSON value is not an object.
pub(crate) fn json_to_props(v: serde_json::Value) -> napi::Result<BTreeMap<String, Value>> {
    match v {
        serde_json::Value::Object(map) => {
            if map.len() > JSON_MAX_MAP_KEYS {
                return Err(input_limit("properties: map exceeds 10000-key limit"));
            }
            let mut out = BTreeMap::new();
            for (k, val) in map {
                out.insert(k, json_to_value(val, 1)?);
            }
            Ok(out)
        }
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "properties: must be an object",
        )),
    }
}

/// Walk a `serde_json::Value` into a `benten_core::Value`.
fn json_to_value(v: serde_json::Value, depth: usize) -> napi::Result<Value> {
    if depth > JSON_MAX_DEPTH {
        return Err(input_limit("value tree exceeds 128-level depth limit"));
    }
    match v {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() {
                    Ok(Value::Float(f))
                } else {
                    Err(napi::Error::new(
                        Status::InvalidArg,
                        "numbers must be finite",
                    ))
                }
            } else {
                Err(napi::Error::new(
                    Status::InvalidArg,
                    "unsupported numeric shape",
                ))
            }
        }
        serde_json::Value::String(s) => Ok(Value::Text(s)),
        serde_json::Value::Array(items) => {
            // Detect the Uint8Array-shaped payload (every element is an integer
            // in `0..=255`) and route those into `Value::Bytes`. Otherwise a
            // plain `Value::List`.
            if items.iter().all(is_byte_number) {
                if items.len() > JSON_MAX_BYTES {
                    return Err(input_limit("bytes: exceeds 1MiB limit"));
                }
                // Already validated by `is_byte_number` — every entry fits in
                // `0..=255`; the truncating cast is safe.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let bytes: Vec<u8> = items
                    .into_iter()
                    .map(|i| i.as_i64().unwrap_or(0).clamp(0, 255) as u8)
                    .collect();
                return Ok(Value::Bytes(bytes));
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(json_to_value(item, depth + 1)?);
            }
            Ok(Value::List(out))
        }
        serde_json::Value::Object(map) => {
            if map.len() > JSON_MAX_MAP_KEYS {
                return Err(input_limit("object exceeds 10000-key limit"));
            }
            let mut out = BTreeMap::new();
            for (k, val) in map {
                out.insert(k, json_to_value(val, depth + 1)?);
            }
            Ok(Value::Map(out))
        }
    }
}

/// True if a JSON number is an integer in the `0..=255` byte range.
fn is_byte_number(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Number(n) => n.as_i64().is_some_and(|i| (0..=255).contains(&i)),
        _ => false,
    }
}

fn input_limit(msg: &str) -> napi::Error {
    napi::Error::new(Status::GenericFailure, format!("E_INPUT_LIMIT: {msg}"))
}

/// Render a `benten_core::Node` as a JSON object matching `{ labels, properties }`.
pub(crate) fn node_to_json(node: &Node) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    out.insert(
        "labels".to_string(),
        serde_json::Value::Array(
            node.labels
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    out.insert(
        "properties".to_string(),
        value_map_to_json(&node.properties),
    );
    serde_json::Value::Object(out)
}

pub(crate) fn value_map_to_json(map: &BTreeMap<String, Value>) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for (k, v) in map {
        out.insert(k.clone(), value_to_json(v));
    }
    serde_json::Value::Object(out)
}

pub(crate) fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::Bytes(b) => serde_json::Value::Array(
            b.iter()
                .copied()
                .map(|byte| serde_json::Value::Number(u64::from(byte).into()))
                .collect(),
        ),
        Value::List(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
        Value::Map(map) => value_map_to_json(map),
    }
}

/// Parse the `{ labels: string[], properties: object }` shape into a Node.
///
/// Loose: a top-level call of `{ title: ... }` with no `labels` key is treated
/// as a property-only input (empty label vec) for CRUD create inputs.
pub(crate) fn node_json_to_node(v: serde_json::Value) -> napi::Result<Node> {
    match v {
        serde_json::Value::Object(mut obj) => {
            let labels: Vec<String> = match obj.remove("labels") {
                Some(serde_json::Value::Array(arr)) => arr
                    .into_iter()
                    .filter_map(|x| match x {
                        serde_json::Value::String(s) => Some(s),
                        _ => None,
                    })
                    .collect(),
                Some(_) => {
                    return Err(napi::Error::new(
                        Status::InvalidArg,
                        "node.labels: must be an array of strings",
                    ));
                }
                None => Vec::new(),
            };
            let props = match obj.remove("properties") {
                Some(p @ serde_json::Value::Object(_)) => json_to_props(p)?,
                // Fall through: treat the whole object as the property bag.
                Some(_) => {
                    return Err(napi::Error::new(
                        Status::InvalidArg,
                        "node.properties: must be an object",
                    ));
                }
                None => {
                    // Remaining fields are the property bag.
                    json_to_props(serde_json::Value::Object(obj))?
                }
            };
            Ok(Node::new(labels, props))
        }
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "node: must be an object",
        )),
    }
}

/// Parse a base32 CID string (multibase `b` prefix) back into a `Cid`.
pub(crate) fn parse_cid(s: &str) -> napi::Result<benten_core::Cid> {
    let stripped = s.strip_prefix('b').unwrap_or(s);
    let bytes = crate::base32_lower_nopad_decode(stripped).ok_or_else(|| {
        napi::Error::new(Status::InvalidArg, "E_INPUT_LIMIT: cid: invalid base32")
    })?;
    benten_core::Cid::from_bytes(&bytes).map_err(core_err)
}
