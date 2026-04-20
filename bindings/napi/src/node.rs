//! JSON <-> `benten_core::Node` conversion for the napi surface.
//!
//! Shape exposed to TypeScript:
//! ```json
//! { "labels": ["Post"], "properties": { "title": "Hello" } }
//! ```
//!
//! Numbers round-trip through `i64` when they have no fractional component;
//! floats (with fractional part or when the number exceeds `i64::MAX`) land
//! in `Value::Float`. Plain JS arrays become `Value::List`; plain JS
//! objects become `Value::Map`; `Buffer` / `Uint8Array` / `ArrayBuffer`
//! — which napi-rs v3 converts to `Object` with numeric-string keys —
//! become `Value::Bytes`. The three input shapes therefore produce three
//! distinct `Value` variants (and three distinct CIDs), closing the
//! r6-perf-6 content-addressing bug where `Uint8Array([0,1,2])` and
//! plain `[0, 1, 2]` collapsed onto the same hash.

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

/// Per-string byte ceiling applied to any `Value::Text` decoded from JSON.
///
/// A single JSON string longer than this is rejected with `E_INPUT_LIMIT`
/// before the `Value::Text` lands in the tree — the attacker-cost vector
/// closed here is a multi-gigabyte single-key input that would otherwise
/// allocate before any downstream check fires. See r6-sec-7 +
/// `docs/SECURITY-POSTURE.md` "napi input-limit enforcement".
const JSON_MAX_BYTES: usize = 1024 * 1024;

/// Aggregate-bytes ceiling applied across an entire JSON tree (16 MiB).
///
/// Walk-time accumulator threaded through `json_to_value` so a tree that
/// evades the per-string cap by fragmenting across many small values still
/// trips the overall DoS tripwire. See r6-sec-7.
const JSON_MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;

/// Running byte-size accumulator threaded through `json_to_value` so the
/// aggregate-bytes cap (`JSON_MAX_TOTAL_BYTES`) is enforced across the whole
/// tree rather than per-leaf only. See r6-sec-7.
#[derive(Default)]
struct ByteBudget {
    consumed: usize,
}

impl ByteBudget {
    fn charge(&mut self, bytes: usize) -> napi::Result<()> {
        self.consumed = self.consumed.saturating_add(bytes);
        if self.consumed > JSON_MAX_TOTAL_BYTES {
            return Err(input_limit(
                "value tree exceeds 16 MiB aggregate-bytes limit",
            ));
        }
        Ok(())
    }
}

/// Decode a JSON object into the property map of a Benten Node.
///
/// # Errors
///
/// * `E_INPUT_LIMIT` — depth, map-key, per-string, or aggregate-bytes limit
///   exceeded.
/// * `InvalidArg` — the root JSON value is not an object.
pub(crate) fn json_to_props(v: serde_json::Value) -> napi::Result<BTreeMap<String, Value>> {
    match v {
        serde_json::Value::Object(map) => {
            if map.len() > JSON_MAX_MAP_KEYS {
                return Err(input_limit("properties: map exceeds 10000-key limit"));
            }
            let mut budget = ByteBudget::default();
            let mut out = BTreeMap::new();
            for (k, val) in map {
                budget.charge(k.len())?;
                out.insert(k, json_to_value(val, 1, &mut budget)?);
            }
            Ok(out)
        }
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "properties: must be an object",
        )),
    }
}

/// Walk a `serde_json::Value` into a `benten_core::Value`, charging each
/// textual leaf against `budget` so the aggregate-bytes cap applies.
fn json_to_value(
    v: serde_json::Value,
    depth: usize,
    budget: &mut ByteBudget,
) -> napi::Result<Value> {
    if depth > JSON_MAX_DEPTH {
        return Err(input_limit("value tree exceeds 128-level depth limit"));
    }
    match v {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            // Prefer Int when the number is integer-representable (even
            // when serde_json internally stores it as f64 — the JS
            // boundary doesn't distinguish `1` from `1.0`). This keeps
            // CRUD timestamps (`createdAt`, HLC stamps) in `Value::Int`
            // where sort-by-key and downstream consumers expect them.
            if let Some(i) = n.as_i64() {
                return Ok(Value::Int(i));
            }
            if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err(napi::Error::new(
                        Status::InvalidArg,
                        "numbers must be finite",
                    ));
                }
                // Integer-valued float within i64 range → Int.
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "comparing against exact conversion bounds"
                )]
                if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    #[allow(clippy::cast_possible_truncation, reason = "bounds-checked above")]
                    return Ok(Value::Int(f as i64));
                }
                return Ok(Value::Float(f));
            }
            Err(napi::Error::new(
                Status::InvalidArg,
                "unsupported numeric shape",
            ))
        }
        serde_json::Value::String(s) => {
            // r6-sec-7: per-string cap first so we reject before the
            // aggregate accumulator overflow message muddies the error
            // attribution. The napi boundary is the last place we can
            // bound allocation before downstream Rust code commits to the
            // String.
            if s.len() > JSON_MAX_BYTES {
                return Err(input_limit(
                    "string value exceeds 1 MiB per-string byte limit",
                ));
            }
            budget.charge(s.len())?;
            Ok(Value::Text(s))
        }
        serde_json::Value::Array(items) => {
            // JS arrays are always `Value::List`. The previous heuristic
            // collapsed numeric arrays into `Value::Bytes` when every
            // element fit in `0..=255`, but that broke content-addressing:
            // a user-constructed `List([Int(0), Int(1), Int(2)])` and a
            // `Bytes([0,1,2])` hashed to the same CID. Bytes must come
            // across the boundary as a typed-array / Buffer shape
            // (handled by napi-rs's `serde_json::Value::Object` with
            // numeric-string keys — not a plain Array), so we can
            // preserve List-vs-Bytes intent unambiguously.
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(json_to_value(item, depth + 1, budget)?);
            }
            Ok(Value::List(out))
        }
        serde_json::Value::Object(map) => {
            if map.len() > JSON_MAX_MAP_KEYS {
                return Err(input_limit("object exceeds 10000-key limit"));
            }
            // r6-perf-6: napi-rs v3 (with serde-json) converts Buffer /
            // Uint8Array / ArrayBuffer into `Value::Object` with numeric-
            // string keys, because `napi_is_array` returns false for
            // TypedArrays. We detect that canonical shape here —
            // sequential keys "0".."n-1", every value an integer in
            // 0..=255 — and route it to `Value::Bytes`. Plain JS objects
            // with arbitrary keys fall through to `Value::Map`, and plain
            // JS arrays arrive as `serde_json::Value::Array` → `Value::
            // List`. The three shapes now have unambiguous Rust-side
            // mappings and therefore distinct CIDs; the prior behavior
            // (plain array of bytes → Value::Bytes via heuristic) broke
            // content-addressing by collapsing List([Int]) and
            // Bytes([u8]) onto the same hash.
            if let Some(bytes) = detect_typed_array_bytes(&map) {
                if bytes.len() > JSON_MAX_BYTES {
                    return Err(input_limit("typed-array value exceeds 1 MiB byte limit"));
                }
                budget.charge(bytes.len())?;
                return Ok(Value::Bytes(bytes));
            }
            let mut out = BTreeMap::new();
            for (k, val) in map {
                budget.charge(k.len())?;
                out.insert(k, json_to_value(val, depth + 1, budget)?);
            }
            Ok(Value::Map(out))
        }
    }
}

/// Return `Some(bytes)` iff `map` has the canonical shape that napi-rs v3
/// produces for a JS `Buffer` / `Uint8Array` / `ArrayBuffer` under the
/// `serde-json` feature: keys are the decimal-string integers `"0"`,
/// `"1"`, ..., `"n-1"` (in some order, no gaps, no duplicates), and every
/// value is a `Value::Number` representing an integer in `0..=255`.
///
/// Returns `None` for any other shape so the caller can fall through to
/// `Value::Map`. The check is O(n) in `map.len()` and allocates only the
/// returned `Vec<u8>`; a `serde_json::Map` is backed by a BTreeMap (or
/// IndexMap with preserve-order), so the numeric-key iteration order does
/// not affect detection because we reassemble via explicit indexing.
fn detect_typed_array_bytes(map: &serde_json::Map<String, serde_json::Value>) -> Option<Vec<u8>> {
    let n = map.len();
    if n == 0 {
        // Empty object: `{}` is an empty Map on the JS side, not an
        // empty Uint8Array (Uint8Array(0) round-trips through napi to
        // the same empty-object shape, but treating `{}` as bytes would
        // reclassify legitimately-empty JS object property bags). We
        // favor the Map interpretation — callers who want an empty
        // Bytes can explicitly emit a `Value::Bytes(vec![])` on the
        // Rust side.
        return None;
    }
    let mut out = vec![0u8; n];
    let mut seen = vec![false; n];
    for (k, v) in map {
        // Reject any non-decimal-integer key — a plain Map with mixed
        // numeric/alphabetic keys is not a TypedArray.
        let idx: usize = k.parse().ok()?;
        if idx >= n || seen[idx] {
            return None;
        }
        // Reject any value that isn't an integer 0..=255 — matches the
        // Uint8Array JS type contract. napi-rs renders each slot as a
        // `serde_json::Value::Number`; `as_u64` returns the low
        // 64 bits of the numeric slot.
        let byte_u64 = v.as_u64()?;
        if byte_u64 > 255 {
            return None;
        }
        // `as u8` is safe after the bounds check; the allow is to satisfy
        // clippy's `cast_possible_truncation` without relaxing the lint.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "bounded 0..=255 by the check above"
        )]
        {
            out[idx] = byte_u64 as u8;
        }
        seen[idx] = true;
    }
    Some(out)
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
        Value::Bytes(b) => {
            // r6-perf-6: emit `Value::Bytes` as an Object with numeric-
            // string keys so it symmetrically round-trips through
            // `json_to_value` → `Value::Bytes`. This matches the shape
            // napi-rs uses for Uint8Array / Buffer on the inbound side;
            // the two directions therefore agree on "this is bytes, not
            // a list of ints" and content-addressing stays stable.
            //
            // Phase-2 note: when the napi surface grows a dedicated
            // typed-array return path (via `Env::create_buffer` or
            // equivalent), replace this shape with a native Buffer so
            // callers get a `Uint8Array` on the JS side rather than an
            // object with integer-string keys. Until then, the object
            // shape is the only Rust-side representation that preserves
            // the List-vs-Bytes distinction across the boundary.
            let mut obj = serde_json::Map::with_capacity(b.len());
            for (i, byte) in b.iter().enumerate() {
                obj.insert(
                    i.to_string(),
                    serde_json::Value::Number(u64::from(*byte).into()),
                );
            }
            serde_json::Value::Object(obj)
        }
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

/// Parse a string as a CID, falling back to a deterministic synthesized
/// CID when the string isn't a valid multibase-base32 CID.
///
/// Used by `Engine::callAs` so a QUICKSTART caller can pass a friendly
/// principal string (`"alice"`) without first minting a Node whose CID
/// they then thread back in. The synthesized CID hashes the bytes
/// `"benten-napi-synthetic-principal-v1\0<input>"` so the same input
/// string always maps to the same CID process-wide — enough to make the
/// NoAuthBackend path happy and keep audit attribution stable for the
/// given friendly name. Phase 3 replaces this with a typed principal
/// from `benten-id` (r6b-dx-C5).
pub(crate) fn parse_actor_cid_or_derive(s: &str) -> benten_core::Cid {
    if let Ok(cid) = parse_cid(s) {
        return cid;
    }
    let mut material = Vec::with_capacity(40 + s.len());
    material.extend_from_slice(b"benten-napi-synthetic-principal-v1\0");
    material.extend_from_slice(s.as_bytes());
    let digest: [u8; 32] = *blake3::hash(&material).as_bytes();
    benten_core::Cid::from_blake3_digest(digest)
}
