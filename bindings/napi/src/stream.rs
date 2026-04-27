//! napi bridge for the STREAM primitive's `callStream` / `openStream`
//! / `testingOpenStreamForTest` surfaces (Phase 2b G6-B).
//!
//! This module exposes thin adapters around [`benten_engine::Engine`]'s
//! STREAM APIs so the TypeScript wrapper in
//! `packages/engine/src/stream.ts` can offer:
//!
//! ```text
//! engine.callStream(handlerId, action, input)
//!   -> AsyncIterable<Chunk>
//! engine.openStream(handlerId, action, input)
//!   -> StreamHandle (AsyncIterable + explicit close())
//! engine.testingOpenStreamForTest(chunks)  // cfg-gated
//!   -> StreamHandle pre-populated with synthetic chunks
//! ```
//!
//! Chunks cross the boundary as `Buffer` (napi's `Vec<u8>` bridge).
//! Per ts-r4-2 R4 finding, the test-harness factory
//! `testingOpenStreamForTest` is exposed as a cfg-gated symbol so
//! vitest fixtures can construct a handle without going through the
//! production async-iterator setup. The symbol presence is pinned by
//! `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`.
//!
//! The `#[napi]` methods themselves live in `lib.rs::napi_surface::Engine`
//! — napi-rs v3 requires every `#[napi] impl` block to be in the same
//! translation unit as the struct declaration. This file exposes the
//! underlying adapters as plain Rust functions so the impl methods stay
//! thin.

#![cfg(feature = "napi-export")]

use benten_core::Node as CoreNode;
use benten_engine::{Engine as InnerEngine, StreamHandle};
use napi::bindgen_prelude::*;

use crate::error::engine_err;
use crate::node::json_to_props;

/// Internal: drive `Engine::call_stream`, return a typed
/// [`StreamHandle`] the napi `Engine` impl wraps in a JS-side
/// async-iterable. Pre-G6-A the handle's first `next()` surfaces
/// `E_PRIMITIVE_NOT_IMPLEMENTED`; once G6-A's executor merges, the
/// handle is populated by the `tokio::sync::mpsc::Receiver<Chunk>` of
/// the running STREAM executor.
pub(crate) fn call_stream_adapter(
    engine: &InnerEngine,
    handler_id: &str,
    op: &str,
    input: serde_json::Value,
) -> napi::Result<StreamHandle> {
    let input_node = json_to_node(input)?;
    engine
        .call_stream(handler_id, op, input_node)
        .map_err(engine_err)
}

/// Internal: drive `Engine::open_stream`. Same dispatch path as
/// [`call_stream_adapter`]; the explicit-close lifecycle is enforced
/// on the TS-wrapper side by exposing a `dispose()` / `close()`
/// method on the JS handle.
pub(crate) fn open_stream_adapter(
    engine: &InnerEngine,
    handler_id: &str,
    op: &str,
    input: serde_json::Value,
) -> napi::Result<StreamHandle> {
    let input_node = json_to_node(input)?;
    engine
        .open_stream(handler_id, op, input_node)
        .map_err(engine_err)
}

/// ts-r4-2 R4: synchronous stream-handle factory for vitest harnesses.
/// Accepts a vector of `Buffer`s; returns a [`StreamHandle`] that
/// drains them in insertion order without going through the production
/// async-iterator setup.
///
/// cfg-gated under `cfg(any(test, feature = "test-helpers"))` per
/// Phase-2a sec-r6r2-02 discipline so the production cdylib does NOT
/// compile this surface in. The napi `Engine` impl method that calls
/// this adapter is cfg-gated identically.
#[cfg(any(test, feature = "test-helpers"))]
pub(crate) fn testing_open_stream_for_test_adapter(
    engine: &InnerEngine,
    chunks: Vec<Vec<u8>>,
) -> StreamHandle {
    engine.testing_open_stream_for_test(chunks)
}

/// Drain the next chunk from a [`StreamHandle`]. The napi `Engine` impl
/// method routes per-iteration `next()` calls through here. Returns:
///
/// - `Ok(Some(Buffer))` — chunk available.
/// - `Ok(None)` — end-of-stream (the JS async-iterable returns
///   `{ done: true }`).
/// - `Err(napi::Error)` — typed terminal error (drives the JS
///   async-iterable to throw).
pub(crate) fn next_chunk_adapter(handle: &mut StreamHandle) -> napi::Result<Option<Vec<u8>>> {
    handle
        .next_chunk()
        .map(|opt| opt.map(|c| c.0))
        .map_err(engine_err)
}

/// Idempotent close on a [`StreamHandle`]. Releases buffered chunks
/// and flips the handle's drained state.
pub(crate) fn close_handle_adapter(handle: &mut StreamHandle) {
    handle.close();
}

// ---------------------------------------------------------------------------
// Internal helpers — JSON → Node
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
            "call_stream: input must be an object or null",
        )),
    }
}
