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

use benten_engine::{
    Chunk, EmitSubscription, Engine as InnerEngine, SubscribeCursor, Subscription,
};
use napi::bindgen_prelude::FnArgs;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;

use crate::error::engine_err;
use crate::node::value_to_json;

/// Wave-8c-subscribe-infra: payload carried across the napi
/// `ThreadsafeFunction` boundary. A pair of `(seq, payload_bytes)` —
/// the receiver-side closure (registered via `build_callback`) maps it
/// onto the JS callback signature `(seq: number, payload: Buffer) =>
/// void`.
type OnChangeTsfnPayload = (u32, Vec<u8>);

/// Wave-8c-subscribe-infra: shorthand for the engine-side
/// `OnChangeCallback` shape — `Arc<dyn Fn(u64, &Chunk) + Send + Sync +
/// 'static>`. Mirrors `benten_engine::OnChangeCallback`. Defined here
/// so the type-position uses are self-explanatory + clippy's
/// `type_complexity` lint stays quiet on the napi-side trampoline.
type EngineOnChangeCallback = Arc<dyn Fn(u64, &Chunk) + Send + Sync + 'static>;

/// Wave-8c-subscribe-infra: build an engine-side `OnChangeCallback`
/// (`Arc<dyn Fn(u64, &Chunk) + Send + Sync>`) that fires the supplied
/// JS function on the libuv main loop via `napi::ThreadsafeFunction`.
///
/// The trampoline:
///   1. Engine publishes a change event on whichever thread committed
///      the WRITE (the broadcast walks subscribers synchronously).
///   2. The Rust closure built here projects `(seq, payload_bytes)` and
///      calls `tsfn.call(...)` with `NonBlocking` semantics — napi-rs
///      enqueues onto the libuv main loop.
///   3. The `build_callback` closure (registered at TSFN construction)
///      translates the tuple into `(JS number, JS Buffer)` and invokes
///      the user's JS callback.
pub(crate) fn build_on_change_tsfn(
    cb: napi::bindgen_prelude::Function<'_, FnArgs<(u32, Buffer)>, ()>,
) -> napi::Result<EngineOnChangeCallback> {
    // `weak: false` (the default) keeps the process alive while
    // subscriptions are live — matches the dx contract that
    // `engine.onChange` returns a handle whose Drop controls cleanup.
    // `callee_handled: false` means we don't pass an error first-arg
    // into the JS callback.
    //
    // FnArgs<(u32, Buffer)> is the napi-rs 3 idiom for "splat the tuple
    // into N JS arguments" — without the FnArgs wrapper the tuple's
    // blanket `JsValuesTupleIntoVec for T where T: ToNapiValue` impl
    // matches and the tuple is delivered as a single JS Array argument
    // (see napi-3.8.5/src/bindgen_runtime/js_values/function.rs:19 +
    // array.rs:388 — tuples impl ToNapiValue as Array). The
    // `JsValuesTupleIntoVec for FnArgs<tuple>` impl at function.rs:55
    // is the unpacker that delivers `(seq: number, payload: Buffer)`
    // as two distinct JS args. Pre-fix the
    // `subscribe.test.ts::LOAD-BEARING — onChange callback fires`
    // pin failed because `payload` was undefined on the JS side.
    let tsfn = cb
        .build_threadsafe_function::<OnChangeTsfnPayload>()
        .callee_handled::<false>()
        .build_callback(
            |ctx: napi::threadsafe_function::ThreadsafeCallContext<OnChangeTsfnPayload>| {
                let (seq, bytes) = ctx.value;
                let buf = Buffer::from(bytes);
                Ok(FnArgs::from((seq, buf)))
            },
        )?;
    let tsfn_arc = Arc::new(tsfn);
    let engine_cb: EngineOnChangeCallback = {
        let tsfn = Arc::clone(&tsfn_arc);
        Arc::new(move |seq: u64, chunk: &Chunk| {
            // u64 → u32 narrowing: ThreadsafeFunction tuples to JS
            // already encode u32 cleanly; events past 2^32 saturate
            // (operationally beyond the bounded retention window).
            let seq32 = u32::try_from(seq).unwrap_or(u32::MAX);
            let payload_bytes = chunk.bytes.clone();
            let _status = tsfn.call(
                (seq32, payload_bytes),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        })
    };
    Ok(engine_cb)
}

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

/// Drive `Engine::on_change_with_cursor`. The supplied JS callback is
/// wrapped in a `napi::ThreadsafeFunction` via [`build_on_change_tsfn`]
/// so deliveries from arbitrary publishing threads land on the libuv
/// main loop. Callers without a JS callback (synchronous TS wrappers
/// that just want the inactive-handle JSON shape) can pass `None`.
pub(crate) fn on_change_adapter(
    engine: &InnerEngine,
    pattern: &str,
    cursor_raw: &serde_json::Value,
    cb: Option<napi::bindgen_prelude::Function<'_, FnArgs<(u32, Buffer)>, ()>>,
) -> napi::Result<Subscription> {
    let cursor = parse_cursor(cursor_raw)?;
    let engine_cb = match cb {
        Some(jscb) => build_on_change_tsfn(jscb)?,
        None => Arc::new(|_seq: u64, _chunk: &Chunk| {}) as Arc<_>,
    };
    engine
        .on_change_with_cursor(pattern, cursor, engine_cb)
        .map_err(engine_err)
}

/// Phase 2b wave-8c-cont: drive `Engine::on_change_as_with_cursor`
/// with an explicit actor principal CID. The supplied JS callback is
/// wrapped in a `napi::ThreadsafeFunction`; the principal is captured
/// on the registered ad-hoc onChange entry's delivery-time cap-recheck
/// closure so D5 cap-recheck-at-delivery fires the named principal's
/// grants on every event.
pub(crate) fn on_change_as_adapter(
    engine: &InnerEngine,
    pattern: &str,
    cursor_raw: &serde_json::Value,
    actor: &benten_core::Cid,
    cb: Option<napi::bindgen_prelude::Function<'_, FnArgs<(u32, Buffer)>, ()>>,
) -> napi::Result<Subscription> {
    let cursor = parse_cursor(cursor_raw)?;
    let engine_cb = match cb {
        Some(jscb) => build_on_change_tsfn(jscb)?,
        None => Arc::new(|_seq: u64, _chunk: &Chunk| {}) as Arc<_>,
    };
    engine
        .on_change_as_with_cursor(pattern, cursor, engine_cb, actor)
        .map_err(engine_err)
}

/// ts-r4-2 mirror for SUBSCRIBE (mini-review cr-g6b-mr-5): synthetic
/// subscription factory for vitest harnesses verifying the unsubscribe
/// + dedup state machinery without depending on G6-A's change-stream
/// port. Mirrors `testing_open_stream_for_test_adapter` for STREAM.
///
/// cfg-gated under `cfg(feature = "test-helpers")` (D-NS-OBS-3
/// closure, wave-8e). See identical rationale in
/// [`crate::stream::testing_open_stream_for_test_adapter`].
#[cfg(feature = "test-helpers")]
pub(crate) fn testing_open_subscription_for_test_adapter(
    engine: &InnerEngine,
    pattern: &str,
    cursor_raw: &serde_json::Value,
) -> napi::Result<Subscription> {
    let cursor = parse_cursor(cursor_raw)?;
    Ok(engine.testing_open_subscription_for_test(pattern, cursor))
}

/// ts-r4-2 mirror (mini-review cr-g6b-mr-5): synthetic event delivery
/// path used by harness tests to exercise the dedup state machine
/// without a real change-stream port. Returns `true` if the synthetic
/// delivery was applied, `false` if it was deduped.
#[cfg(feature = "test-helpers")]
pub(crate) fn testing_deliver_synthetic_event_for_test_adapter(
    engine: &InnerEngine,
    sub: &Subscription,
    seq: u64,
) -> bool {
    engine.testing_deliver_synthetic_event_for_test(sub, seq)
}

// `subscription_to_json` removed in wave-8c fix-pass cr-w8c-fp-1: the
// JSON-shape return path dropped the underlying Subscription at end of
// method scope, releasing the `napi::ThreadsafeFunction` Arc that holds
// the JS callback alive — JS callbacks could never fire. Production
// consumers + test helpers now route through `SubscriptionJs` which
// holds the Subscription alive for the lifetime of the JS handle.

// =====================================================================
// EMIT broadcast adapter (R6 Round-2 r6-r2-mpc-1 closure of r6-mpc-2)
// =====================================================================

/// Payload carried across the napi `ThreadsafeFunction` boundary for
/// EMIT subscribers. A pair of `(channel, payload_json)` — the
/// payload-side `Value` is rendered to a JSON string via
/// [`crate::node::value_to_json`] before crossing because the TS-side
/// `engine.onEmit` wrapper parses JSON once on receive (matching the
/// pre-existing `(string, string)` shape declared at
/// `packages/engine/src/engine.ts::NativeEngine.onEmit`).
type OnEmitTsfnPayload = (String, String);

/// Build an engine-side `EmitCallback` (`Arc<dyn Fn(&EmitEvent) + Send +
/// Sync + 'static>`) that fires the supplied JS function on the libuv
/// main loop via `napi::ThreadsafeFunction`. Mirrors
/// [`build_on_change_tsfn`] but for the EMIT broadcast.
pub(crate) fn build_on_emit_tsfn(
    cb: napi::bindgen_prelude::Function<'_, FnArgs<(String, String)>, ()>,
) -> napi::Result<benten_engine::emit_broadcast::EmitCallback> {
    // Match SUBSCRIBE's defaults: keep the process alive while
    // EMIT subscriptions are live (`weak: false`) + don't pass an
    // error first-arg into the JS callback (`callee_handled: false`).
    //
    // FnArgs<(String, String)> for the same reason as
    // `build_on_change_tsfn` — without the wrapper napi-rs 3 delivers
    // the tuple as a single JS Array, so the JS callback's second
    // parameter (`payload`) is `undefined`. See the explanation on
    // `build_on_change_tsfn` for the full mechanic. The codebase
    // previously had a workaround comment on the
    // `emit_subscribe.test.ts::LOAD-BEARING` test ("napi-rs v3's
    // `Function<(String, String), ()>` callback-shape delivers the
    // tuple as a single Array argument rather than splatting to 2
    // args"); FnArgs IS the production splat mechanic.
    let tsfn = cb
        .build_threadsafe_function::<OnEmitTsfnPayload>()
        .callee_handled::<false>()
        .build_callback(
            |ctx: napi::threadsafe_function::ThreadsafeCallContext<OnEmitTsfnPayload>| {
                Ok(FnArgs::from(ctx.value))
            },
        )?;
    let tsfn_arc = Arc::new(tsfn);
    let engine_cb: benten_engine::emit_broadcast::EmitCallback = {
        let tsfn = Arc::clone(&tsfn_arc);
        Arc::new(move |event: &benten_engine::EmitEvent| {
            // Render the `Value` payload to its JSON shape before
            // crossing the TSFN boundary so the TS wrapper at
            // `engine.ts::onEmit` can `JSON.parse` it once on the JS
            // side. Matches the pre-existing `(string, string)`
            // signature declared on the optional native method type.
            let payload_json = serde_json::to_string(&value_to_json(&event.payload))
                .unwrap_or_else(|_| "null".to_string());
            let _status = tsfn.call(
                (event.channel.clone(), payload_json),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        })
    };
    Ok(engine_cb)
}

/// Drive `Engine::subscribe_emit_events_with_handle` against the
/// supplied JS callback. The callback is wrapped in a
/// `napi::ThreadsafeFunction` via [`build_on_emit_tsfn`] so EMIT
/// publishes from arbitrary dispatch threads land on the libuv main
/// loop.
///
/// Channel filtering: the engine-side EMIT broadcast fans every
/// published event out to every subscriber; this adapter applies the
/// channel filter on the engine-side closure so JS receives only events
/// whose `channel` matches `channel` exactly (string equality; no glob
/// matching at the engine surface in Phase 2b — matches the doc
/// contract at `engine.ts::onEmit`).
pub(crate) fn on_emit_adapter(
    engine: &InnerEngine,
    channel: &str,
    cb: napi::bindgen_prelude::Function<'_, FnArgs<(String, String)>, ()>,
) -> napi::Result<EmitSubscription> {
    let engine_cb = build_on_emit_tsfn(cb)?;
    let want_channel = channel.to_string();
    let sub = engine.subscribe_emit_events_with_handle(move |event| {
        if event.channel == want_channel {
            engine_cb(event);
        }
    });
    Ok(sub)
}
