//! Phase 2b G6-B: STREAM engine wrappers — `callStream` / `openStream`
//! / `testing_open_stream_for_test` surfaces.
//!
//! Sibling module to [`engine_wait`](crate::engine_wait) following the
//! Phase-2a 5d-K pattern. Companion to G6-A which owns the
//! `benten_eval::primitives::stream` executor + the real
//! [`benten_eval::chunk_sink::ChunkSink`] trait body. The wrappers in
//! this file plumb the engine-side public API and the cross-language
//! boundary; the production STREAM executor itself lives in `benten-eval`.
//!
//! # Dual surface (dx-optimizer corrected)
//!
//! Per plan §3 G6-B (R1 dx-optimizer):
//!
//! - `subgraph(...).stream(args)` — DSL composition primitive (lives in
//!   `packages/engine/src/dsl.ts`; the Rust side just receives it as a
//!   `PrimitiveKind::Stream` Node in the registered SubgraphSpec).
//! - [`Engine::call_stream`] — `engine.callStream(handler_id, action,
//!   input) -> AsyncIterable<Chunk>`. Mirrors `Engine::call` /
//!   `Engine::call_as` / `Engine::call_with_suspension` naming.
//! - [`Engine::open_stream`] — `engine.openStream(...) -> StreamHandle`.
//!   Same dispatch path as `call_stream`; the handle exposes an
//!   explicit-close method on the TS side so callers can release the
//!   underlying chunk-sink resources without driving the iterator to
//!   exhaustion.
//!
//! # `testing_open_stream_for_test` (ts-r4-2)
//!
//! Vitest harnesses need a stream-handle factory that does NOT drag the
//! production async-iterator setup. Per ts-r4-2 R4 finding,
//! `Engine::testing_open_stream_for_test` returns a typed
//! [`StreamHandle`] with a pre-populated chunk vec the harness drives
//! synchronously. cfg-gated under `cfg(any(test, feature =
//! "test-helpers"))` per Phase-2a sec-r6r2-02 discipline so the
//! production cdylib does not compile this surface in. (No intra-doc
//! link: the cfg-gated method isn't compiled in default `cargo doc`,
//! so a `[link]` wrap fails `RUSTDOCFLAGS=-D warnings` — keep the
//! reference as plain prose.)
//!
//! # G6-A coordination
//!
//! Until G6-A lands its real [`benten_eval::chunk_sink::ChunkSink`]
//! trait body + tokio-mpsc backed STREAM executor, [`Engine::call_stream`]
//! returns a [`StreamHandle`] that yields the typed
//! `E_PRIMITIVE_NOT_IMPLEMENTED` error on first poll. The handle's shape
//! is locked here; once G6-A merges, its executor populates the handle's
//! chunk source with real bytes. `testing_open_stream_for_test` does NOT
//! depend on G6-A — the test factory accepts a synthetic chunk vector.

use benten_core::Node;
use benten_errors::ErrorCode;
use benten_eval::chunk_sink::Chunk;

use crate::engine::Engine;
use crate::engine_wait::HandlerRef;
use crate::error::EngineError;

/// Cursor mode for STREAM consumers.
///
/// Locked-shape per plan §3 G6-B / G6-A D5 cursor surface symmetry.
/// `Latest` and `Sequence` mirror the SUBSCRIBE cursor surface for
/// consistency; STREAM does not yet expose the `Persistent` mode
/// because per-stream resumption is Phase 3 (iroh transport boundary).
#[derive(Debug, Clone)]
pub enum StreamCursor {
    /// Start from the next chunk produced after this call.
    Latest,
    /// Start from the chunk at engine-assigned sequence number `seq`.
    Sequence(u64),
}

/// Handle to an open STREAM dispatch. Returned by
/// [`Engine::open_stream`] + [`Engine::call_stream`].
///
/// On the napi side this surface presents as `AsyncIterable<Chunk>`
/// with an optional explicit `close()` method. Inside this crate the
/// handle is a simple state machine: each [`StreamHandle::next_chunk`] call
/// pulls the next ready chunk from the underlying sink, returning
/// `Ok(Some(chunk))` while data flows, `Ok(None)` at end-of-stream, or
/// `Err(EngineError)` on a typed error edge.
///
/// Until G6-A merges its real executor body, [`StreamHandle::next_chunk`]
/// returns `Err(EngineError::Other { code: PrimitiveNotImplemented, ..
/// })` on the very first poll. The handle's TS-facing surface is
/// already wired so the wrapper code in `packages/engine/src/stream.ts`
/// compiles and exercises the round-trip shape before the executor
/// lands.
pub struct StreamHandle {
    /// Pre-buffered chunks (test factory + future eager-buffered modes).
    /// G6-A will replace this with a `tokio::sync::mpsc::Receiver<Chunk>`
    /// once the real executor lands; the public surface (`next` /
    /// `close`) does not change.
    chunks: std::collections::VecDeque<Chunk>,
    /// `true` once the producer has indicated end-of-stream.
    closed: bool,
    /// Pre-populated terminal error returned on the next `next()` call.
    /// Used by the G6-A-pending stub path to surface
    /// `E_PRIMITIVE_NOT_IMPLEMENTED` typed.
    pending_error: Option<EngineError>,
    /// Engine-assigned sequence counter; bumped per delivered chunk so
    /// the TS wrapper can expose `chunk.seq` for replay/dedup symmetry
    /// with SUBSCRIBE.
    next_seq: u64,
}

impl std::fmt::Debug for StreamHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamHandle")
            .field("buffered_chunks", &self.chunks.len())
            .field("closed", &self.closed)
            .field("has_pending_error", &self.pending_error.is_some())
            .field("next_seq", &self.next_seq)
            .finish()
    }
}

impl StreamHandle {
    /// Construct an empty closed handle. Useful for the test factory's
    /// "no chunks, immediate end-of-stream" fixture.
    #[must_use]
    pub fn empty_closed() -> Self {
        Self {
            chunks: std::collections::VecDeque::new(),
            closed: true,
            pending_error: None,
            next_seq: 0,
        }
    }

    /// Construct a handle whose first `next()` call surfaces a typed
    /// engine error. Used by the G6-A-pending stub path.
    #[must_use]
    pub fn with_pending_error(err: EngineError) -> Self {
        Self {
            chunks: std::collections::VecDeque::new(),
            closed: true,
            pending_error: Some(err),
            next_seq: 0,
        }
    }

    /// Construct a handle pre-populated with the given chunks. The
    /// handle is closed (no further chunks will arrive) once the
    /// vector drains. Test-factory entry point used by the napi
    /// `testing_open_stream_for_test` symbol per ts-r4-2 R4 finding.
    #[must_use]
    pub fn from_test_chunks(chunks: Vec<Chunk>) -> Self {
        Self {
            chunks: chunks.into_iter().collect(),
            closed: true,
            pending_error: None,
            next_seq: 0,
        }
    }

    /// Pull the next chunk from the handle. Returns `Ok(Some(chunk))`
    /// while data flows, `Ok(None)` at end-of-stream.
    ///
    /// # Errors
    /// Returns the pending [`EngineError`] (if any) on the first call
    /// after construction. Real executor errors (back-pressure drop,
    /// peer close, capability denial mid-stream) flow through the
    /// G6-A executor body once it lands.
    pub fn next_chunk(&mut self) -> Result<Option<Chunk>, EngineError> {
        if let Some(err) = self.pending_error.take() {
            return Err(err);
        }
        if let Some(chunk) = self.chunks.pop_front() {
            self.next_seq = self.next_seq.saturating_add(1);
            return Ok(Some(chunk));
        }
        if self.closed {
            return Ok(None);
        }
        // Pre-G6-A: no executor wired yet. The handle is constructed
        // closed for the call_stream/open_stream stub paths so this
        // branch is currently unreachable from the public API; it
        // exists for the G6-A executor to plug in its mpsc receiver.
        Ok(None)
    }

    /// Explicit close — release the handle's resources without driving
    /// it to exhaustion. Idempotent.
    pub fn close(&mut self) {
        self.closed = true;
        self.chunks.clear();
    }

    /// Current engine-assigned sequence counter. Bumped per delivered
    /// chunk; `0` before the first `next_chunk()` returns `Some`.
    #[must_use]
    pub fn seq_so_far(&self) -> u64 {
        self.next_seq
    }

    /// `true` if the handle has been explicitly closed AND its buffered
    /// chunks are drained.
    #[must_use]
    pub fn is_drained(&self) -> bool {
        self.closed && self.chunks.is_empty() && self.pending_error.is_none()
    }
}

impl Engine {
    /// Phase 2b G6-B: invoke a registered handler whose subgraph
    /// produces STREAM chunks. Returns a [`StreamHandle`] the caller
    /// drives by repeated [`StreamHandle::next_chunk`] calls.
    ///
    /// Mirrors [`Engine::call`] naming. The TS wrapper exposes this as
    /// `engine.callStream(handlerId, action, input)` and renders the
    /// handle as an `AsyncIterable<Chunk>` so consumers can write
    /// `for await (const chunk of engine.callStream(...))`.
    ///
    /// # Pre-G6-A behavior
    ///
    /// Until G6-A's real executor body lands, this method returns a
    /// handle whose first `next_chunk()` surfaces
    /// `E_PRIMITIVE_NOT_IMPLEMENTED`. Once G6-A merges, the handle is
    /// populated by the executor's `tokio::sync::mpsc::Receiver<Chunk>`
    /// per D4-RESOLVED.
    ///
    /// # Errors
    /// Returns [`EngineError`] if the handler isn't registered. Streaming
    /// errors surface through subsequent [`StreamHandle::next_chunk`]
    /// calls.
    pub fn call_stream<H: HandlerRef>(
        &self,
        handler_id: H,
        op: &str,
        input: Node,
    ) -> Result<StreamHandle, EngineError> {
        self.call_stream_inner(handler_id.as_handler_key().as_str(), op, input, None)
    }

    /// Phase 2b G6-B: `call_stream` with an explicit actor principal.
    /// Mirrors [`Engine::call_as`] naming.
    ///
    /// # Errors
    /// See [`Engine::call_stream`].
    pub fn call_stream_as<H: HandlerRef>(
        &self,
        handler_id: H,
        op: &str,
        input: Node,
        actor: &benten_core::Cid,
    ) -> Result<StreamHandle, EngineError> {
        self.call_stream_inner(
            handler_id.as_handler_key().as_str(),
            op,
            input,
            Some(*actor),
        )
    }

    /// Phase 2b G6-B: open a STREAM dispatch returning a handle whose
    /// async-iterator surface MUST be explicitly `close()`d. Same
    /// dispatch path as [`Engine::call_stream`]; the handle's lifecycle
    /// contract is the only difference (the TS wrapper exposes a
    /// `dispose`/`close` method that the `for await` form does not
    /// require — `for await` auto-closes at scope exit).
    ///
    /// # Errors
    /// See [`Engine::call_stream`].
    pub fn open_stream<H: HandlerRef>(
        &self,
        handler_id: H,
        op: &str,
        input: Node,
    ) -> Result<StreamHandle, EngineError> {
        self.call_stream_inner(handler_id.as_handler_key().as_str(), op, input, None)
    }

    fn call_stream_inner(
        &self,
        handler_id: &str,
        _op: &str,
        _input: Node,
        _actor: Option<benten_core::Cid>,
    ) -> Result<StreamHandle, EngineError> {
        // Pre-G6-A: verify the handler is registered (so callers get a
        // useful E_NOT_FOUND error early instead of an opaque
        // "stream did nothing" outcome) but defer real execution to
        // G6-A's executor body. Once G6-A lands, this method spins up
        // a `tokio::sync::mpsc::channel(16)` (default capacity per
        // D4-RESOLVED), hands the producer end to the STREAM executor
        // running on a tokio runtime, and wraps the consumer end in a
        // StreamHandle. The pending-error stub keeps the surface honest
        // (typed code, not a panic) until the executor lands.
        {
            let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
            if !handlers.contains_key(handler_id) {
                return Err(EngineError::Other {
                    code: ErrorCode::NotFound,
                    message: format!("call_stream: handler not registered: {handler_id}"),
                });
            }
        }
        Ok(StreamHandle::with_pending_error(EngineError::Other {
            code: ErrorCode::PrimitiveNotImplemented,
            message: "call_stream: STREAM executor lands with G6-A; \
                      use testing_open_stream_for_test for harness fixtures \
                      until G6-A merges."
                .into(),
        }))
    }

    /// ts-r4-2 R4: napi-side helper exposing a synchronous stream-
    /// handle factory for vitest harnesses. The harness supplies a
    /// vector of canned chunks; the returned handle drains them one
    /// per `next_chunk` call without touching the production
    /// async-iterator setup or requiring G6-A's executor to be on this
    /// branch.
    ///
    /// cfg-gated under `cfg(any(test, feature = "test-helpers"))` per
    /// Phase-2a sec-r6r2-02 discipline so the production cdylib does
    /// NOT compile this surface in. The narrower
    /// `envelope-cache-test-grade` / `iteration-budget-test-grade`
    /// gates do NOT light this up — only the broader `test-helpers`
    /// gate does. The test fixture
    /// `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`
    /// pins the symbol presence by asserting
    /// `typeof engine.testingOpenStreamForTest === "function"`.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_open_stream_for_test(&self, chunks: Vec<Vec<u8>>) -> StreamHandle {
        let chunks: Vec<Chunk> = chunks.into_iter().map(Chunk).collect();
        StreamHandle::from_test_chunks(chunks)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests may use unwrap per workspace policy"
)]
mod tests {
    use super::*;

    #[test]
    fn empty_closed_handle_drains_immediately() {
        let mut h = StreamHandle::empty_closed();
        assert!(h.next_chunk().unwrap().is_none());
        assert!(h.is_drained());
    }

    #[test]
    fn pending_error_surfaces_once_then_drains() {
        let mut h = StreamHandle::with_pending_error(EngineError::Other {
            code: ErrorCode::PrimitiveNotImplemented,
            message: "stub".into(),
        });
        let err = h.next_chunk().unwrap_err();
        match err {
            EngineError::Other { code, .. } => {
                assert_eq!(code, ErrorCode::PrimitiveNotImplemented);
            }
            _ => panic!("unexpected error variant"),
        }
        assert!(h.next_chunk().unwrap().is_none());
    }

    #[test]
    fn from_test_chunks_drains_in_order_with_seq_bump() {
        // Post-G6-A merge: Chunk struct shape is `{ seq, final_chunk, bytes }`
        // (was `Chunk(Vec<u8>)` newtype against the empty scaffold). Adapted
        // to the canonical struct shape; `seq_so_far()` continues to track
        // engine-side sequence stamping independent of the chunk's own seq.
        let mut h = StreamHandle::from_test_chunks(vec![
            Chunk { seq: 0, final_chunk: false, bytes: vec![1, 2, 3].into() },
            Chunk { seq: 1, final_chunk: true, bytes: vec![4, 5].into() },
        ]);
        assert_eq!(h.seq_so_far(), 0);
        let c1 = h.next_chunk().unwrap().unwrap();
        assert_eq!(c1.bytes.as_ref(), &[1, 2, 3][..]);
        assert_eq!(h.seq_so_far(), 1);
        let c2 = h.next_chunk().unwrap().unwrap();
        assert_eq!(c2.bytes.as_ref(), &[4, 5][..]);
        assert_eq!(h.seq_so_far(), 2);
        assert!(h.next_chunk().unwrap().is_none());
        assert!(h.is_drained());
    }

    #[test]
    fn close_releases_buffered_chunks() {
        let mut h = StreamHandle::from_test_chunks(vec![
            Chunk { seq: 0, final_chunk: false, bytes: vec![1].into() },
            Chunk { seq: 1, final_chunk: true, bytes: vec![2].into() },
        ]);
        h.close();
        assert!(h.next_chunk().unwrap().is_none());
        assert!(h.is_drained());
    }
}
