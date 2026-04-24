//! napi bridge for the WAIT primitive's suspend / resume surfaces
//! (Phase 2a G3-B).
//!
//! This module exposes thin adapters around [`benten_engine::Engine`]'s
//! suspension APIs so the TypeScript wrapper in `packages/engine/src/engine.ts`
//! can offer:
//!
//! ```text
//! engine.callWithSuspension(handler_id, op, input)
//!   -> { kind: "complete", outcome } | { kind: "suspended", handle: Buffer }
//! engine.resumeFromBytesUnauthenticated(bytes, signalValue)   -> Outcome
//! engine.resumeFromBytesAs(bytes, signalValue, principalCid) -> Outcome
//! ```
//!
//! The `Unauthenticated` variant skips step 2 (principal binding) of the
//! 4-step resume protocol by design — G11-A Decision 3 (2026-04-24). TS
//! callers should prefer `resumeFromBytesAs` unless they're in a
//! single-user / in-process context where no principal identity is
//! meaningful.
//!
//! Envelope bytes cross the boundary as `Buffer` (napi's `Vec<u8>` bridge).
//! Signal payloads cross as `serde_json::Value`, re-used from the existing
//! Node-input codec path.
//!
//! The `#[napi]` methods themselves live in `lib.rs::napi_surface::Engine`
//! — napi-rs v3 requires every `#[napi] impl` block to be in the same
//! translation unit as the struct declaration. This file exposes the
//! underlying adapters as plain Rust functions so the impl methods stay
//! thin.

#![cfg(feature = "napi-export")]

use benten_core::{Cid, Node as CoreNode, Value};
use benten_engine::{Engine as InnerEngine, SuspensionOutcome};
use napi::bindgen_prelude::*;

use crate::error::engine_err;
use crate::node::{json_to_props, parse_cid};
use crate::subgraph::outcome_to_json;

/// Discriminated-union payload mirroring the TS `SuspensionResult` shape.
/// Returned by [`call_with_suspension_adapter`]; the caller renders it as
/// JSON before handing it back across the napi boundary.
pub(crate) enum SuspensionBridge {
    Complete(serde_json::Value),
    Suspended { handle_bytes: Vec<u8> },
}

impl SuspensionBridge {
    /// Render to a `serde_json::Value` matching the TS `SuspensionResult`
    /// shape:
    ///
    /// - Complete: `{ kind: "complete", outcome: <Outcome-JSON> }`
    /// - Suspended: `{ kind: "suspended", handle: <base64 string> }` —
    ///   the napi layer wraps `handle` in `Buffer.from(handle, "base64")`
    ///   before exposing it to TS callers.
    pub(crate) fn into_json(self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        match self {
            SuspensionBridge::Complete(outcome) => {
                map.insert("kind".into(), serde_json::Value::String("complete".into()));
                map.insert("outcome".into(), outcome);
            }
            SuspensionBridge::Suspended { handle_bytes } => {
                map.insert("kind".into(), serde_json::Value::String("suspended".into()));
                // Base64-encode the bytes so the JSON surface is self-
                // contained. The TS wrapper decodes with `Buffer.from(s, 'base64')`
                // to produce the `Buffer` the public API promises.
                let b64 = base64_encode(&handle_bytes);
                map.insert("handle".into(), serde_json::Value::String(b64));
            }
        }
        serde_json::Value::Object(map)
    }
}

/// Internal: drive `Engine::call_with_suspension`, persist the envelope
/// via `suspend_to_bytes` if the handler suspended, and return a
/// discriminated bridge payload.
pub(crate) fn call_with_suspension_adapter(
    engine: &InnerEngine,
    handler_id: &str,
    op: &str,
    input: serde_json::Value,
) -> napi::Result<SuspensionBridge> {
    let input_node = json_to_node(input)?;
    let outcome = engine
        .call_with_suspension(handler_id, op, input_node)
        .map_err(engine_err)?;
    match outcome {
        SuspensionOutcome::Complete(o) => Ok(SuspensionBridge::Complete(outcome_to_json(&o))),
        SuspensionOutcome::Suspended(handle) => {
            let bytes = engine.suspend_to_bytes(&handle).map_err(engine_err)?;
            Ok(SuspensionBridge::Suspended {
                handle_bytes: bytes,
            })
        }
    }
}

/// Internal: drive `Engine::resume_from_bytes_unauthenticated`. The signal
/// payload is handed through as a structured `Value` (routed via the
/// existing `json_to_props` path so maps surface as `Value::Map` and
/// primitives as their matching `Value::*` variant).
///
/// This adapter is routed to TS as `resumeFromBytesUnauthenticated` —
/// callers who need step-2 principal binding must go through
/// [`resume_from_bytes_as_adapter`] instead. G11-A Decision 3 renamed the
/// Rust-side method from `resume_from_bytes` to
/// `resume_from_bytes_unauthenticated` so the name itself warns that
/// principal binding is skipped.
pub(crate) fn resume_from_bytes_unauthenticated_adapter(
    engine: &InnerEngine,
    bytes: &[u8],
    signal_value: serde_json::Value,
) -> napi::Result<serde_json::Value> {
    let signal = json_to_value(signal_value)?;
    let outcome = engine
        .resume_from_bytes_unauthenticated(bytes, signal)
        .map_err(engine_err)?;
    Ok(outcome_to_json(&outcome))
}

/// Internal: drive `Engine::resume_from_bytes_as` with a caller-supplied
/// principal CID. Principal arrives as a base32 CID string.
pub(crate) fn resume_from_bytes_as_adapter(
    engine: &InnerEngine,
    bytes: &[u8],
    signal_value: serde_json::Value,
    principal: &str,
) -> napi::Result<serde_json::Value> {
    let principal_cid = parse_cid(principal)?;
    let signal = json_to_value(signal_value)?;
    let outcome = engine
        .resume_from_bytes_as(bytes, signal, &principal_cid)
        .map_err(engine_err)?;
    Ok(outcome_to_json(&outcome))
}

// ---------------------------------------------------------------------------
// Internal helpers — JSON → Value / Node
// ---------------------------------------------------------------------------

fn json_to_node(input: serde_json::Value) -> napi::Result<CoreNode> {
    match input {
        serde_json::Value::Object(_) => {
            let props = json_to_props(input)?;
            Ok(CoreNode::new(Vec::new(), props))
        }
        serde_json::Value::Null => Ok(CoreNode::empty()),
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "call_with_suspension: input must be an object or null",
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
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(napi::Error::new(
                    Status::InvalidArg,
                    "signal value: unsupported JSON number",
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
        serde_json::Value::Object(_) => {
            // Reuse the node-props codec so nested maps produce the
            // same `Value::Map` shape as create_node inputs.
            let props = json_to_props(v)?;
            Ok(Value::Map(props))
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal base64 encoder (avoids a new direct dep on `base64`)
// ---------------------------------------------------------------------------

const B64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let n =
            (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8) | u32::from(input[i + 2]);
        out.push(B64_ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let n = u32::from(input[i]) << 16;
        out.push(B64_ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8);
        out.push(B64_ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}

/// Keep `Cid` imported for the `parse_cid` / principal-adapter path when
/// napi-rs re-runs the build against different feature flags. Removing
/// the import changes no behavior; retaining it documents intent.
#[allow(dead_code, reason = "exported to sibling napi-surface glue")]
pub(crate) fn _touch_cid(_c: &Cid) {}
