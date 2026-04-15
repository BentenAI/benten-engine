//! # benten-napi
//!
//! Node.js bindings for the Benten graph engine via napi-rs v3.
//!
//! The spike surface exposes just enough to validate the Rust → napi → Node
//! round-trip: `createNode` takes labels + a JSON-like properties object and
//! returns a CID string; `getNode` takes a CID string and returns the Node
//! (or `null` if missing).
//!
//! TypeScript types are generated from the `#[napi]` annotations at build
//! time (napi-rs v3's default workflow).

// napi-rs's `#[napi]` macro expansion contains `unsafe extern "C"` ctor
// registration shims and therefore cannot coexist with `#![forbid(unsafe_code)]`.
// We use `deny` instead: the effect is identical for hand-written code
// (unsafe is not allowed in this crate) but macro-expanded unsafe is
// permitted, which is appropriate for an FFI binding layer whose entire
// reason for existing is wrapping the Node.js C API.
#![deny(unsafe_code)]
// napi-rs generates code that triggers a number of pedantic lints we don't
// control; silence them only for the generated surface, not our own code.
#![allow(clippy::needless_pass_by_value, clippy::missing_safety_doc)]

// All napi-derive-using symbols are gated on `feature = "napi-export"`
// (default-on for the cdylib build, default-off when running `cargo test`
// because `cargo test --workspace` walks each crate without the default
// features that the cdylib enables). The in-process integration test in
// `tests/input_validation.rs` only touches `crate::testing::*`, so the
// `napi_surface` module is dead from the test binary's POV.
#[cfg(feature = "napi-export")]
mod napi_surface {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use benten_core::{Cid, Node, Value};
    use benten_engine::Engine;
    use napi::bindgen_prelude::*;
    use napi_derive::napi;

    // ---------------------------------------------------------------------------
    // Global engine handle
    // ---------------------------------------------------------------------------
    //
    // napi-rs v3 supports first-class class-style wrappers, but for the spike a
    // single process-wide engine handle is enough to exercise the round-trip.
    // Phase 1 proper will expose `Engine` as a `#[napi]` class so callers can
    // manage multiple databases.

    static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

    /// Initialize the engine against a local redb file. Must be called before
    /// any `createNode` / `getNode` call. Calling twice replaces the handle.
    ///
    /// # Errors
    /// Returns a JS error if the storage backend cannot be opened.
    #[napi(js_name = "initEngine")]
    pub fn init_engine(path: String) -> napi::Result<()> {
        let engine = Engine::open(&path)
            .map_err(|e| napi::Error::new(Status::GenericFailure, format!("open: {e}")))?;
        let mut slot = ENGINE
            .lock()
            .map_err(|_| napi::Error::new(Status::GenericFailure, "engine mutex poisoned"))?;
        *slot = Some(engine);
        Ok(())
    }

    /// Create a Node with the given labels and properties, and return its CID
    /// as a multibase-encoded string (base32, prefix `b`).
    ///
    /// Properties are a JSON-compatible object. The spike supports the same
    /// Value subset as `benten-core::Value` (null, bool, number, string, array,
    /// object). Floats are rejected per DAG-CBOR's deterministic-encoding rule
    /// that forbids float values that would round-trip as integers.
    ///
    /// # Errors
    /// Returns a JS error if the engine is not initialized or the Node cannot be
    /// hashed/persisted.
    #[napi(js_name = "createNode")]
    pub fn create_node(labels: Vec<String>, properties: serde_json::Value) -> napi::Result<String> {
        let props = json_to_props(properties)?;
        let node = Node::new(labels, props);

        let slot = ENGINE
            .lock()
            .map_err(|_| napi::Error::new(Status::GenericFailure, "engine mutex poisoned"))?;
        let engine = slot
            .as_ref()
            .ok_or_else(|| napi::Error::new(Status::GenericFailure, "engine not initialized"))?;

        let cid = engine
            .create_node(&node)
            .map_err(|e| napi::Error::new(Status::GenericFailure, format!("create: {e}")))?;
        Ok(cid.to_base32())
    }

    /// Retrieve a Node by CID string. Returns a JSON object or `null` on miss.
    ///
    /// # Errors
    /// Returns a JS error if the engine is not initialized, the CID cannot be
    /// parsed, or the storage backend fails.
    #[napi(js_name = "getNode")]
    pub fn get_node(cid: String) -> napi::Result<Option<serde_json::Value>> {
        let bytes = super::base32_lower_nopad_decode(cid.strip_prefix('b').unwrap_or(&cid))
            .ok_or_else(|| napi::Error::new(Status::InvalidArg, "cid: invalid base32"))?;
        let parsed = Cid::from_bytes(&bytes)
            .map_err(|e| napi::Error::new(Status::InvalidArg, format!("cid: {e}")))?;

        let slot = ENGINE
            .lock()
            .map_err(|_| napi::Error::new(Status::GenericFailure, "engine mutex poisoned"))?;
        let engine = slot
            .as_ref()
            .ok_or_else(|| napi::Error::new(Status::GenericFailure, "engine not initialized"))?;

        match engine
            .get_node(&parsed)
            .map_err(|e| napi::Error::new(Status::GenericFailure, format!("get: {e}")))?
        {
            Some(node) => Ok(Some(node_to_json(&node))),
            None => Ok(None),
        }
    }

    // ---------------------------------------------------------------------------
    // JSON <-> Value conversions
    // ---------------------------------------------------------------------------

    fn json_to_props(v: serde_json::Value) -> napi::Result<BTreeMap<String, Value>> {
        match v {
            serde_json::Value::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, val) in map {
                    out.insert(k, json_to_value(val)?);
                }
                Ok(out)
            }
            _ => Err(napi::Error::new(
                Status::InvalidArg,
                "properties: must be an object",
            )),
        }
    }

    fn json_to_value(v: serde_json::Value) -> napi::Result<Value> {
        match v {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i))
                } else {
                    Err(napi::Error::new(
                        Status::InvalidArg,
                        "numbers must be representable as i64 in the spike",
                    ))
                }
            }
            serde_json::Value::String(s) => Ok(Value::Text(s)),
            serde_json::Value::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(json_to_value(item)?);
                }
                Ok(Value::List(out))
            }
            serde_json::Value::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, val) in map {
                    out.insert(k, json_to_value(val)?);
                }
                Ok(Value::Map(out))
            }
        }
    }

    fn node_to_json(node: &Node) -> serde_json::Value {
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

    fn value_map_to_json(map: &BTreeMap<String, Value>) -> serde_json::Value {
        let mut out = serde_json::Map::new();
        for (k, v) in map {
            out.insert(k.clone(), value_to_json(v));
        }
        serde_json::Value::Object(out)
    }

    fn value_to_json(v: &Value) -> serde_json::Value {
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
            Value::List(items) => {
                serde_json::Value::Array(items.iter().map(value_to_json).collect())
            }
            Value::Map(map) => value_map_to_json(map),
        }
    }
} // end mod napi_surface (cfg(not(test)))

// ---------------------------------------------------------------------------
// Testing module — stubs for the B8 input-validation harness. R5 wires real
// size/depth enforcement; today every helper returns `todo!()` or a
// placeholder so `tests/input_validation.rs` compiles.
// ---------------------------------------------------------------------------

#[allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]
pub mod testing {
    //! Test-only surface for the napi input-validation harness.

    use benten_core::{Cid, CoreError, Value};

    /// Result type alias mirroring what the harness expects — the stub
    /// returns `CoreError` today.
    pub fn deserialize_value_from_js_like(_bytes: &[u8]) -> Result<Value, CoreError> {
        todo!("deserialize_value_from_js_like — B8 (Phase 1)")
    }

    /// CID deserialization from the TS side with size/shape validation.
    pub fn deserialize_cid_from_js_like(_bytes: &[u8]) -> Result<Cid, CoreError> {
        todo!("deserialize_cid_from_js_like — B8 (Phase 1)")
    }

    /// Generate a synthetic map payload with `keys` entries, meant to trip
    /// the 10K-key limit.
    pub fn make_giant_map(_keys: usize) -> Vec<u8> {
        todo!("make_giant_map — B8 (Phase 1)")
    }

    /// Generate a synthetic depth-`n` nested list payload.
    pub fn make_deep_list(_depth: usize) -> Vec<u8> {
        todo!("make_deep_list — B8 (Phase 1)")
    }

    /// Generate a synthetic `bytes`-long bytes payload.
    pub fn make_giant_bytes(_bytes: usize) -> Vec<u8> {
        todo!("make_giant_bytes — B8 (Phase 1)")
    }

    /// Generate a small on-the-wire CBOR payload that decodes to a deeply
    /// nested map (CBOR zip-bomb analog).
    pub fn make_cbor_bomb(_nominal_depth: usize) -> Vec<u8> {
        todo!("make_cbor_bomb — B8 (Phase 1)")
    }

    /// Process RSS in KB, or `None` if the platform doesn't provide a
    /// cheap reader. Used as a tripwire for allocation-before-rejection bugs.
    #[must_use]
    pub fn rss_kb() -> Option<u64> {
        None
    }
}

// ---------------------------------------------------------------------------
// base32 decode (multibase `b` / RFC 4648 lowercase, no padding)
// ---------------------------------------------------------------------------

#[cfg_attr(any(test, not(feature = "napi-export")), allow(dead_code))]
fn base32_lower_nopad_decode(s: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut out = Vec::with_capacity((s.len() * 5).div_ceil(8));
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for ch in s.bytes() {
        // Alphabet length is 32, so the index always fits in u32.
        let idx = u32::try_from(ALPHABET.iter().position(|c| *c == ch)?).ok()?;
        buffer = (buffer << 5) | idx;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            #[allow(
                clippy::cast_possible_truncation,
                reason = "we just masked the low 8 bits"
            )]
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}
