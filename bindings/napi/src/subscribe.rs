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
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;

use crate::error::engine_err;

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
    cb: napi::bindgen_prelude::Function<'_, (u32, Buffer), ()>,
) -> napi::Result<EngineOnChangeCallback> {
    // `weak: false` (the default) keeps the process alive while
    // subscriptions are live — matches the dx contract that
    // `engine.onChange` returns a handle whose Drop controls cleanup.
    // `callee_handled: false` means we don't pass an error first-arg
    // into the JS callback.
    let tsfn = cb
        .build_threadsafe_function::<OnChangeTsfnPayload>()
        .callee_handled::<false>()
        .build_callback(
            |ctx: napi::threadsafe_function::ThreadsafeCallContext<OnChangeTsfnPayload>| {
                let (seq, bytes) = ctx.value;
                let buf = Buffer::from(bytes);
                Ok((seq, buf))
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
    cb: Option<napi::bindgen_prelude::Function<'_, (u32, Buffer), ()>>,
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
    cb: Option<napi::bindgen_prelude::Function<'_, (u32, Buffer), ()>>,
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
