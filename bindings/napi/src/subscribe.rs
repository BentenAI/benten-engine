//! napi bridge for the SUBSCRIBE primitive's `onChange` ad-hoc
//! consumer surface (Phase 2b G6-B).
//!
//! This module exposes thin adapters around [`benten_engine::Engine`]'s
//! SUBSCRIBE APIs so the TypeScript wrapper in
//! `packages/engine/src/subscribe.ts` can offer:
//!
//! ```text
//! engine.onChange(pattern, callback) -> Subscription
//! ```
//!
//! The TS surface receives a `pattern` glob + a JS callback; the napi
//! Engine impl method wraps the callback in a `napi::ThreadsafeFunction`
//! once G6-A's change-stream port lands and exposes a `Subscription`
//! handle whose `unsubscribe()` flips the engine-side active flag.
//! Pre-G6-A the handle is constructed but inactive; the round-trip
//! shape is exercisable end-to-end before the executor wires in.
//!
//! Renamed from `engine.subscribe` per dx-optimizer R1 finding to
//! avoid name-collision with the DSL `subgraph(...).subscribe`
//! builder method.
//!
//! The `#[napi]` methods themselves live in `lib.rs::napi_surface::Engine`
//! — napi-rs v3 requires every `#[napi] impl` block to be in the same
//! translation unit as the struct declaration. This file exposes the
//! underlying adapters as plain Rust functions so the impl methods
//! stay thin.

#![cfg(feature = "napi-export")]

use std::sync::Arc;

use benten_engine::{Chunk, Engine as InnerEngine, SubscribeCursor, Subscription};

use crate::error::engine_err;

/// Cursor mode JSON shape parsed by the napi adapter. Mirrors
/// [`SubscribeCursor`] one-to-one.
///
/// Wire shape:
/// - `null` / absent / `{ "kind": "latest" }` → `SubscribeCursor::Latest`
/// - `{ "kind": "sequence", "seq": <u64> }` → `SubscribeCursor::Sequence(seq)`
/// - `{ "kind": "persistent", "subscriberId": <string> }` →
///   `SubscribeCursor::Persistent(subscriber_id)`
pub(crate) fn parse_cursor(raw: &serde_json::Value) -> napi::Result<SubscribeCursor> {
    if raw.is_null() {
        return Ok(SubscribeCursor::Latest);
    }
    let obj = raw.as_object().ok_or_else(|| {
        napi::Error::new(
            napi::Status::InvalidArg,
            "on_change: cursor must be null or an object",
        )
    })?;
    let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or("latest");
    match kind {
        "latest" => Ok(SubscribeCursor::Latest),
        "sequence" => {
            let seq = obj
                .get("seq")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    napi::Error::new(
                        napi::Status::InvalidArg,
                        "on_change: cursor.kind=sequence requires `seq: number`",
                    )
                })?;
            Ok(SubscribeCursor::Sequence(seq))
        }
        "persistent" => {
            let id = obj
                .get("subscriberId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    napi::Error::new(
                        napi::Status::InvalidArg,
                        "on_change: cursor.kind=persistent requires `subscriberId: string`",
                    )
                })?;
            Ok(SubscribeCursor::Persistent(id.to_string()))
        }
        other => Err(napi::Error::new(
            napi::Status::InvalidArg,
            format!("on_change: unknown cursor.kind \"{other}\""),
        )),
    }
}

/// Internal: drive `Engine::on_change_with_cursor`. The callback is a
/// no-op closure today — once G6-A wires the change-stream port, this
/// adapter accepts a `napi::ThreadsafeFunction` parameter and bridges
/// per-event delivery into JS. The signature is shaped now so the TS
/// wrapper's compile-time contract doesn't shift when G6-A merges.
pub(crate) fn on_change_adapter(
    engine: &InnerEngine,
    pattern: &str,
    cursor_raw: &serde_json::Value,
) -> napi::Result<Subscription> {
    let cursor = parse_cursor(cursor_raw)?;
    // No-op callback shim: once G6-A's change-stream port lands, the
    // `#[napi]` impl method accepts a `JsFunction` that this adapter
    // wraps in a `ThreadsafeFunction` and threads through here. Until
    // then the callback is dropped (the handle is `is_active() == false`
    // pre-G6-A so no events would fire anyway).
    let cb = Arc::new(|_seq: u64, _chunk: &Chunk| {});
    engine
        .on_change_with_cursor(pattern, cursor, cb)
        .map_err(engine_err)
}

/// Render a [`Subscription`] handle as the JSON shape the TS wrapper
/// expects. Carries the active flag, pattern, and current
/// `max_delivered_seq` so JS-side code can verify the dedup state
/// machine without holding a raw rust handle reference.
pub(crate) fn subscription_to_json(sub: &Subscription) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("active".into(), serde_json::Value::Bool(sub.is_active()));
    map.insert(
        "pattern".into(),
        serde_json::Value::String(sub.pattern().to_string()),
    );
    map.insert(
        "maxDeliveredSeq".into(),
        serde_json::Value::Number(sub.max_delivered_seq().into()),
    );
    map.insert(
        "cursor".into(),
        match sub.cursor() {
            SubscribeCursor::Latest => {
                let mut m = serde_json::Map::new();
                m.insert("kind".into(), serde_json::Value::String("latest".into()));
                serde_json::Value::Object(m)
            }
            SubscribeCursor::Sequence(s) => {
                let mut m = serde_json::Map::new();
                m.insert("kind".into(), serde_json::Value::String("sequence".into()));
                m.insert("seq".into(), serde_json::Value::Number((*s).into()));
                serde_json::Value::Object(m)
            }
            SubscribeCursor::Persistent(id) => {
                let mut m = serde_json::Map::new();
                m.insert(
                    "kind".into(),
                    serde_json::Value::String("persistent".into()),
                );
                m.insert("subscriberId".into(), serde_json::Value::String(id.clone()));
                serde_json::Value::Object(m)
            }
        },
    );
    serde_json::Value::Object(map)
}
