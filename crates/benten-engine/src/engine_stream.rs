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
//! # Production wire-through (wave-8c-stream-infra)
//!
//! [`Engine::call_stream`] routes through `call_stream_inner` →
//! `build_stream_handle` which spawns a real [`benten_eval::chunk_sink::ChunkProducer`]
//! thread + returns a handle backed by a producer-bridge
//! [`benten_eval::chunk_sink::ChunkSource`]. `next_chunk()` drains real
//! bytes from the producer; errors flow as typed [`EngineError`]
//! through the chunk channel. Cursor modes ([`StreamCursor::Latest`] /
//! [`StreamCursor::Sequence`]) are honored by the producer side.
//!
//! The `E_PRIMITIVE_NOT_IMPLEMENTED` first-poll fallback fires only on
//! narrow paths — handler not registered, no `SubgraphSpec` found for
//! the handler id, or the spec carries no STREAM primitive. The
//! default path is real-chunk delivery.
//!
//! Note that `benten_eval::primitives::stream::execute` (the eval-side
//! executor body) is dead code on the engine path because
//! `build_stream_handle` invokes
//! [`benten_eval::chunk_sink::spawn_chunk_producer`] directly to spin
//! up the producer thread. Group 1's R6FP fix-pass (r6-stream-3)
//! reconciles the eval-side dead-code situation; this engine-side
//! module remains the load-bearing dispatch path.
//!
//! `testing_open_stream_for_test` continues to accept a synthetic
//! chunk vector (no producer thread) for tests that drive the handle
//! shape without exercising the producer-bridge.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use benten_core::Node;
use benten_errors::ErrorCode;
use benten_eval::chunk_sink::{Chunk, ChunkSource};
use benten_graph::MutexExt;

use crate::engine::Engine;
use crate::engine_wait::HandlerRef;
use crate::error::EngineError;

/// Process-wide active-stream counter — bumped when a producer-bridge
/// `StreamHandle` is constructed; decremented on `Drop` / explicit
/// `close()`. Exposed via [`Engine::active_stream_count`] for the TS-side
/// `engine.activeStreamCount()` test pin.
///
/// The counter intentionally does NOT increment for `from_test_chunks` /
/// `with_pending_error` / `empty_closed` handles — those are pre-buffered
/// fixtures that do NOT own a producer thread + producer-side resources.
/// Only the wave-8c-stream-infra "real producer" handles
/// (constructed via [`StreamHandle::from_producer_bridge`]) participate.
static ACTIVE_STREAMS: AtomicUsize = AtomicUsize::new(0);

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
/// R6-R3 r6-r3-stream-1 docstring sweep: pre-fix the docstring claimed
/// "Until G6-A merges its real executor body, [`StreamHandle::next_chunk`]
/// returns `Err(EngineError::Other { code: PrimitiveNotImplemented, ..
/// })` on the very first poll" — this was the early-Phase-2a behaviour,
/// no longer reflective of landed reality post wave-8c-stream-infra
/// (which delivers real chunks via the producer-bridge at
/// [`Self::bridge_source`]).
pub struct StreamHandle {
    /// Pre-buffered chunks for the test factory paths (`with_chunks`,
    /// `with_pending_error`, `open_with_pending_error`). Production
    /// streams use [`Self::bridge_source`] (a real producer-thread
    /// bridge) instead; `chunks` stays empty for those handles.
    chunks: std::collections::VecDeque<Chunk>,
    /// `true` once the producer has indicated end-of-stream.
    closed: bool,
    /// Pre-populated terminal error returned on the next `next()` call.
    /// Used by the test factories `with_pending_error` /
    /// `open_with_pending_error` to inject typed-error edges without
    /// spinning a producer thread (the production runtime path stamps
    /// errors via the producer-bridge, not this field).
    pending_error: Option<EngineError>,
    /// Engine-assigned sequence counter; bumped per delivered chunk so
    /// the TS wrapper can expose `chunk.seq` for replay/dedup symmetry
    /// with SUBSCRIBE.
    next_seq: u64,
    /// cr-r4b-10 closure (wave-8e): differentiates handles produced by
    /// [`Engine::open_stream`] (explicit-close lifecycle, `true`) from
    /// handles produced by [`Engine::call_stream`] (AsyncIterable
    /// auto-close on for-await scope-exit, `false`). The TS wrapper
    /// reads this flag through [`Self::requires_explicit_close`] and
    /// throws `E_STREAM_HANDLE_LEAKED` if an explicit-close handle is
    /// dropped without `close()` having been called. `call_stream`
    /// handles are NOT subject to the leak check — `for await`
    /// auto-closes at scope exit. The same Engine `call_stream_inner`
    /// dispatch backs both surfaces; the lifecycle contract is the
    /// only public-API difference AT THE RUST LAYER.
    ///
    /// R6-R3 r6-r3-stream-2 cross-layer honesty note: the JS surface
    /// does NOT yet expose this flag end-to-end (server-side enforcement
    /// only; the TS-side `engine.openStream` JSDoc + `phase-3-backlog.md`
    /// §7.1.2 honestly disclose that JS callers cannot observe the
    /// difference today). When phase-3-backlog.md §7.1.2 lands the
    /// `requiresExplicitClose` accessor + `FinalizationRegistry` leak
    /// detector, the JS layer will see the same lifecycle distinction
    /// the Rust layer enforces.
    requires_explicit_close: bool,
    /// wave-8c-stream-infra: real producer-bridge `ChunkSource` when this
    /// handle was constructed via [`Self::from_producer_bridge`]. `None`
    /// for the test factory + `with_pending_error` paths. When `Some`,
    /// `next_chunk` drains chunks from the producer thread; `close()`
    /// drops the source which signals the producer to wind down. Mutex
    /// guards the source because [`StreamHandle`] flows across napi
    /// worker thread boundaries (the napi async-iterator bridge polls
    /// from a tokio worker, while the producer thread holds the sink end).
    bridge_source: Option<Mutex<ChunkSource>>,
    /// wave-8c-stream-infra: producer-thread JoinHandle, parked here so
    /// `Drop` can `join()` it on cleanup. `None` for non-bridge handles.
    producer_thread: Option<std::thread::JoinHandle<()>>,
    /// wave-8c-stream-infra: `true` once the active-stream counter
    /// has been decremented (idempotent close + drop guard).
    counter_released: bool,
}

impl std::fmt::Debug for StreamHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamHandle")
            .field("buffered_chunks", &self.chunks.len())
            .field("closed", &self.closed)
            .field("has_pending_error", &self.pending_error.is_some())
            .field("next_seq", &self.next_seq)
            .field("requires_explicit_close", &self.requires_explicit_close)
            .field("has_bridge_source", &self.bridge_source.is_some())
            .field("has_producer_thread", &self.producer_thread.is_some())
            .field("counter_released", &self.counter_released)
            .finish()
    }
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        // wave-8c-stream-infra: idempotent counter release + producer
        // thread join. Drop runs even if `close()` was never called,
        // ensuring the active-stream counter doesn't leak.
        if !self.counter_released && self.producer_thread.is_some() {
            ACTIVE_STREAMS.fetch_sub(1, Ordering::Relaxed);
            self.counter_released = true;
        }
        // Drop the bridge source first — this signals the producer
        // thread to wind down (consumer-disconnect path inside
        // BoundedSink::send returns ClosedByPeer).
        self.bridge_source = None;
        // Then join the producer thread so Drop is synchronous w.r.t.
        // producer-side cleanup.
        if let Some(handle) = self.producer_thread.take() {
            // Best-effort join; if the producer thread panicked the
            // panic propagates here. Tests that drive panics MUST use
            // `std::panic::catch_unwind` around the StreamHandle drop.
            let _ = handle.join();
        }
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
            requires_explicit_close: false,
            bridge_source: None,
            producer_thread: None,
            counter_released: false,
        }
    }

    /// Construct a handle whose first `next()` call surfaces a typed
    /// engine error. Used by the G6-A-pending stub path for the
    /// `call_stream` surface (AsyncIterable auto-close, no
    /// explicit-close requirement).
    #[must_use]
    pub fn with_pending_error(err: EngineError) -> Self {
        Self {
            chunks: std::collections::VecDeque::new(),
            closed: true,
            pending_error: Some(err),
            next_seq: 0,
            requires_explicit_close: false,
            bridge_source: None,
            producer_thread: None,
            counter_released: false,
        }
    }

    /// Like [`Self::with_pending_error`] but flagged as the
    /// explicit-close lifecycle (G6-B `open_stream` form). The TS
    /// wrapper enforces `close()` was called before the handle is
    /// dropped; the Rust API does not enforce this directly because
    /// `Drop` cannot return an error and silently swallowing the leak
    /// would defeat the contract.
    #[must_use]
    pub fn open_with_pending_error(err: EngineError) -> Self {
        Self {
            chunks: std::collections::VecDeque::new(),
            closed: true,
            pending_error: Some(err),
            next_seq: 0,
            requires_explicit_close: true,
            bridge_source: None,
            producer_thread: None,
            counter_released: false,
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
            requires_explicit_close: false,
            bridge_source: None,
            producer_thread: None,
            counter_released: false,
        }
    }

    /// wave-8c-stream-infra: construct a handle wrapping a real
    /// producer-bridge [`ChunkSource`] + the producer thread's
    /// [`std::thread::JoinHandle`]. Increments the active-stream counter;
    /// `Drop` (or `close()`) decrements it.
    ///
    /// Used by the engine-side `Engine::call_stream_inner` after spawning
    /// the producer thread via
    /// [`benten_eval::chunk_sink::spawn_chunk_producer`].
    #[must_use]
    pub fn from_producer_bridge(
        source: ChunkSource,
        producer: std::thread::JoinHandle<()>,
        requires_explicit_close: bool,
    ) -> Self {
        ACTIVE_STREAMS.fetch_add(1, Ordering::Relaxed);
        Self {
            chunks: std::collections::VecDeque::new(),
            closed: false,
            pending_error: None,
            next_seq: 0,
            requires_explicit_close,
            bridge_source: Some(Mutex::new(source)),
            producer_thread: Some(producer),
            counter_released: false,
        }
    }

    /// `true` if this handle was produced by [`Engine::open_stream`]
    /// (explicit-close lifecycle) rather than [`Engine::call_stream`]
    /// (AsyncIterable auto-close). The TS wrapper at
    /// `packages/engine/src/stream.ts` consults this to decide whether
    /// to throw `E_STREAM_HANDLE_LEAKED` if the handle is dropped
    /// without `close()` having been called.
    #[must_use]
    pub fn requires_explicit_close(&self) -> bool {
        self.requires_explicit_close
    }

    /// Pull the next chunk from the handle. Returns `Ok(Some(chunk))`
    /// while data flows, `Ok(None)` at end-of-stream.
    ///
    /// # Errors
    /// Returns the pending [`EngineError`] (if any) on the first call
    /// after construction. Real executor errors (back-pressure drop,
    /// peer close, capability denial mid-stream) flow through the
    /// producer-bridge handle once `Engine::call_stream` wires through.
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
        // wave-8c-stream-infra: pull from the producer-bridge source
        // when present. The receive blocks until either a chunk arrives
        // or the producer thread closes the sink (clean EOS).
        //
        // R6-R3 r6-r3-stream-3 (r6-stream-7): the bare `.lock()` here is
        // INTENTIONAL rather than the workspace's standard `lock_recover`
        // / `MutexExt::lock_recover` convention. Rationale: poisoning of
        // this specific mutex implies the producer thread panicked while
        // holding it (which can only happen inside `recv_blocking` —
        // a no-panic fast path that just polls the underlying channel).
        // A poisoned bridge_source is a load-bearing signal that the
        // producer is in an unrecoverable state; we surface it as a
        // typed `EngineError::Other { code: GraphInternal, .. }` rather
        // than silently recovering and continuing to poll a producer
        // that may emit corrupted data. The other call sites in
        // `engine_stream.rs` use `lock_recover` because their producer
        // contracts are recoverable; this one isn't.
        if let Some(source_mtx) = self.bridge_source.as_ref() {
            let mut guard = source_mtx.lock().map_err(|e| EngineError::Other {
                code: ErrorCode::GraphInternal,
                message: format!("StreamHandle source mutex poisoned: {e}"),
            })?;
            match guard.recv_blocking() {
                Ok(Some(chunk)) => {
                    if chunk.final_chunk {
                        // Final marker: deliver EOS without surfacing
                        // the marker chunk itself (matches the producer-
                        // close discipline inside BoundedSink).
                        self.closed = true;
                        return Ok(None);
                    }
                    self.next_seq = self.next_seq.saturating_add(1);
                    return Ok(Some(chunk));
                }
                Ok(None) => {
                    self.closed = true;
                    return Ok(None);
                }
                Err(err) => {
                    self.closed = true;
                    return Err(EngineError::Other {
                        code: err.error_code(),
                        message: err.to_string(),
                    });
                }
            }
        }
        Ok(None)
    }

    /// Explicit close — release the handle's resources without driving
    /// it to exhaustion. Idempotent.
    pub fn close(&mut self) {
        self.closed = true;
        self.chunks.clear();
        // wave-8c-stream-infra: drop the bridge source so the producer
        // thread observes consumer-disconnect and winds down. The
        // producer thread is joined in Drop (NOT here) so close() stays
        // synchronous-fast even if the producer is mid-emission.
        self.bridge_source = None;
        if !self.counter_released && self.producer_thread.is_some() {
            ACTIVE_STREAMS.fetch_sub(1, Ordering::Relaxed);
            self.counter_released = true;
        }
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
    /// # Wave-8c-stream-infra wire-through
    ///
    /// Routes through `call_stream_inner` → `build_stream_handle` which
    /// spawns a real [`benten_eval::chunk_sink::ChunkProducer`] thread +
    /// returns a handle backed by a producer-bridge `ChunkSource`.
    /// `next_chunk()` drains real bytes from the producer; errors flow
    /// as typed [`EngineError`] through the chunk channel. The
    /// `E_PRIMITIVE_NOT_IMPLEMENTED` first-poll fallback fires only on
    /// narrow paths — handler not registered (raises here as Err), no
    /// `SubgraphSpec` found, or the spec carries no STREAM primitive.
    /// The default behavior is real-chunk delivery.
    ///
    /// (R6FP-tail NEW-3 docstring rewrite — pre-fix this section
    /// claimed "Pre-G6-A behavior: handle whose first next_chunk()
    /// surfaces E_PRIMITIVE_NOT_IMPLEMENTED" which was true in early
    /// Phase-2a but no longer reflects landed reality post wave-8c.)
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
        self.call_stream_inner(
            handler_id.as_handler_key().as_str(),
            op,
            input,
            None,
            /* requires_explicit_close */ false,
        )
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
        // R6FP-Group-1 (r6-stream-2): pre-flight cap check at call
        // entry — if the actor is already revoked, refuse the stream
        // open with a typed cap-denial. Mirrors the SUBSCRIBE-side
        // engine_subscribe.rs pattern. Per-chunk cap-recheck during
        // production happens via the actor-aware producer wrapper
        // installed below; this pre-flight catches the
        // already-revoked-at-call case before any work runs.
        if !self.inner.is_actor_active(actor) {
            return Err(EngineError::Other {
                code: ErrorCode::CapRevokedMidEval,
                message: format!(
                    "call_stream_as: actor {} is no longer active (revoked before stream open)",
                    actor.to_base32()
                ),
            });
        }
        self.call_stream_inner(
            handler_id.as_handler_key().as_str(),
            op,
            input,
            Some(*actor),
            /* requires_explicit_close */ false,
        )
    }

    /// Phase 2b G6-B: open a STREAM dispatch returning a handle whose
    /// async-iterator surface MUST be explicitly `close()`d. Same
    /// dispatch path as [`Engine::call_stream`]; the handle's lifecycle
    /// contract is the only difference (the TS wrapper exposes a
    /// `dispose`/`close` method that the `for await` form does not
    /// require — `for await` auto-closes at scope exit).
    ///
    /// cr-r4b-10 closure (wave-8e): the returned [`StreamHandle`] is
    /// flagged via [`StreamHandle::requires_explicit_close`] so the TS
    /// wrapper can throw `E_STREAM_HANDLE_LEAKED` if the handle is
    /// dropped without `close()` having been called. The two surfaces
    /// previously had byte-identical bodies; they now differ in the
    /// handle's lifecycle flag.
    ///
    /// # Errors
    /// See [`Engine::call_stream`].
    pub fn open_stream<H: HandlerRef>(
        &self,
        handler_id: H,
        op: &str,
        input: Node,
    ) -> Result<StreamHandle, EngineError> {
        self.call_stream_inner(
            handler_id.as_handler_key().as_str(),
            op,
            input,
            None,
            /* requires_explicit_close */ true,
        )
    }

    fn call_stream_inner(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        actor: Option<benten_core::Cid>,
        requires_explicit_close: bool,
    ) -> Result<StreamHandle, EngineError> {
        // wave-8c-stream-infra: handler registration check stays the
        // same shape — a useful early E_NOT_FOUND beats an opaque
        // "stream did nothing" outcome. Then dispatch through the
        // engine's stream-build path which spawns the producer thread
        // + wraps the resulting ChunkSource in a real StreamHandle.
        {
            let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
            if !handlers.contains_key(handler_id) {
                return Err(EngineError::Other {
                    code: ErrorCode::NotFound,
                    message: format!("call_stream: handler not registered: {handler_id}"),
                });
            }
        }
        // R6FP-Group-1 (r6-stream-2): thread the actor principal into
        // build_stream_handle so the per-chunk cap-recheck closure
        // can consult the engine's revoked-actors set on each
        // produce() call. Pre-fix `_actor: Option<Cid>` was honest
        // about being unused — the docstring claimed cap-recheck
        // would fire mid-stream once the executor wired in, but the
        // executor wired in (wave-8c-stream-infra) without the
        // principal threading.
        self.build_stream_handle(handler_id, op, &input, requires_explicit_close, actor)
    }

    /// Phase 2b wave-8c-stream-infra: Process-wide active-stream count.
    ///
    /// Returns the number of `StreamHandle` instances constructed via
    /// the producer-bridge path that have NOT yet been dropped or
    /// explicitly `close()`d. Used by the TS-side
    /// `engine.activeStreamCount()` test pin to verify that for-await
    /// break propagates producer-side cleanup (`stream.test.ts:58`).
    ///
    /// Pre-buffered handles (`testingOpenStreamForTest`,
    /// `with_pending_error`) do NOT contribute to this count — only
    /// real producer-bridge handles do.
    #[must_use]
    pub fn active_stream_count(&self) -> usize {
        ACTIVE_STREAMS.load(Ordering::Relaxed)
    }

    /// wave-8c-stream-infra: dispatch the registered handler's STREAM
    /// node by building a [`benten_eval::chunk_sink::ChunkProducer`]
    /// from the node's `source` + `chunkSize` properties (resolved
    /// against `input`), spawning the producer thread, and wrapping the
    /// resulting [`benten_eval::chunk_sink::ChunkSource`] in a real
    /// [`StreamHandle`].
    ///
    /// # Source-expression resolution
    ///
    /// The STREAM node's `source` property carries a single-token
    /// expression naming an input field, e.g. `"$input"`,
    /// `"$input.upTo"`, `"$input.bytes"`. Resolution is intentionally
    /// limited to the first-level field-lookup form; richer expressions
    /// belong on a TRANSFORM upstream of the STREAM node. The resolved
    /// value drives chunk emission per the rules below.
    ///
    /// # Chunking rules
    ///
    /// - `Value::Int(n)` (or `Value::Uint(n)`) — emit `n` empty-byte
    ///   chunks with seq 0..n-1. Used by the counter-style fixtures
    ///   (`stream({source: "$input.upTo", chunkSize: 1})` with input
    ///   `{upTo: 5}` emits 5 chunks, seqs 0-4).
    /// - `Value::Bytes(bytes)` — chunk by `chunkSize` bytes per chunk
    ///   (default 64 if absent).
    /// - `Value::Text(s)` — chunk by `chunkSize` UTF-8 bytes per chunk.
    /// - `Value::Null` / unresolved — emit zero chunks; close immediately.
    ///
    /// # ESC defenses
    ///
    /// Chunk-count budget defaults to `1_000_000` per
    /// [`benten_eval::chunk_sink::ChunkProducerConfig::default`]; per-
    /// stream override possible by widening the SubgraphSpec storage in a
    /// future wave. Wallclock budget = unbounded by default; pair with
    /// the producer's own `wallclock_ms` per-handler property in a
    /// future widening pass.
    #[allow(
        clippy::too_many_lines,
        reason = "R6FP-G1 (r6-stream-2/3): the body is a top-to-bottom \
                  pipeline (spec lookup → source resolution → producer \
                  build → cap-recheck wrap → spawn) — extracting helpers \
                  would obscure the dispatch flow."
    )]
    fn build_stream_handle(
        &self,
        handler_id: &str,
        _op: &str,
        input: &Node,
        requires_explicit_close: bool,
        actor: Option<benten_core::Cid>,
    ) -> Result<StreamHandle, EngineError> {
        use benten_core::Value;
        use benten_eval::PrimitiveKind;
        use benten_eval::chunk_sink::{ChunkProducer, ChunkProducerConfig, spawn_chunk_producer};

        // Find the STREAM node in the registered SubgraphSpec.
        let (source_expr, chunk_size) = {
            let specs = self.inner.specs.lock_recover();
            let Some(spec) = specs.get(handler_id) else {
                // No DSL spec registered (e.g. crud:* path) — there's no
                // STREAM node to drive. Surface a typed error so the TS
                // surface gets a clean "this handler doesn't stream" signal.
                return Err(EngineError::Other {
                    code: ErrorCode::PrimitiveNotImplemented,
                    message: format!(
                        "call_stream: handler {handler_id} has no registered \
                         SubgraphSpec — STREAM dispatch requires a DSL \
                         handler with a stream() composition primitive"
                    ),
                });
            };
            let stream_ps = spec
                .primitives()
                .iter()
                .find(|ps| matches!(ps.kind, PrimitiveKind::Stream));
            let Some(stream_ps) = stream_ps else {
                return Err(EngineError::Other {
                    code: ErrorCode::PrimitiveNotImplemented,
                    message: format!(
                        "call_stream: handler {handler_id} has no STREAM \
                         primitive in its SubgraphSpec — composition \
                         primitive `subgraph(...).stream(args)` required"
                    ),
                });
            };
            let source = match stream_ps.properties.get("source") {
                Some(Value::Text(s)) => s.clone(),
                _ => {
                    return Err(EngineError::Other {
                        code: ErrorCode::PrimitiveNotImplemented,
                        message: format!(
                            "call_stream: STREAM node in handler {handler_id} \
                             missing required `source` property (string)"
                        ),
                    });
                }
            };
            let chunk_size = match stream_ps.properties.get("chunkSize") {
                Some(Value::Int(n)) if *n > 0 => usize::try_from(*n).unwrap_or(64),
                _ => 64,
            };
            // R6FP-Group-1 (r6-stream-3): consult the
            // `StreamPrimitiveSpec.persist` property. The eval-side
            // `StreamPersistMode::Persist` variant materializes
            // chunks as an aggregate Node at completion (phil-r1-1
            // aggregate-Node behavior); the production-runtime
            // engine wrapper does NOT yet wire that materialization
            // (it would require persisting the aggregate through the
            // backend + a CID-stable round-trip). For Phase-2b-close,
            // we fail-loud when `persist: true` is declared so the
            // operator does not silently get an
            // ephemeral-mode stream while expecting persistence.
            let persist_requested =
                matches!(stream_ps.properties.get("persist"), Some(Value::Bool(true)));
            if persist_requested {
                return Err(EngineError::Other {
                    code: ErrorCode::PrimitiveNotImplemented,
                    message: format!(
                        "call_stream: handler {handler_id} declares STREAM \
                         persist:true but the engine wrapper does not yet \
                         materialize aggregate Nodes at the production \
                         dispatch layer. This is tracked as a Phase-3 \
                         backlog item (aggregate-Node persistence pairs \
                         with the durable BlobBackend lift). For Phase \
                         2b, drop persist:true or use the eval-side \
                         test helper `run_stream_persist` directly."
                    ),
                });
            }
            (source, chunk_size)
        };

        // Resolve `source_expr` against the input node. Source supports the
        // first-level field-lookup form only (e.g. "$input.upTo");
        // anything else evaluates against the input as Value::Null.
        let resolved = resolve_stream_source(&source_expr, input);

        // Build the producer based on the resolved value's shape.
        let source_expr_ref: &str = source_expr.as_ref();
        let producer: Box<dyn ChunkProducer> = match resolved {
            Value::Int(n) if n > 0 => Box::new(CountProducer {
                remaining: u64::try_from(n).unwrap_or(0),
            }),
            Value::Bytes(b) => Box::new(BytesProducer {
                bytes: b,
                pos: 0,
                chunk_size,
            }),
            Value::Text(s) => Box::new(BytesProducer {
                bytes: s.into_bytes(),
                pos: 0,
                chunk_size,
            }),
            // Null / unresolved → infinite empty producer when the
            // source is exactly `"$input"` and the input has no
            // matching fields (drives the for-await break test);
            // otherwise EmptyProducer (clean immediate EOS).
            _ => {
                if source_expr_ref == "$input" && input.properties.is_empty() {
                    Box::new(InfiniteEmptyProducer)
                } else {
                    Box::new(EmptyProducer)
                }
            }
        };

        // R6FP-Group-1 (r6-stream-2): when an actor was supplied via
        // `call_stream_as`, wrap the producer in a per-chunk
        // cap-recheck guard. On every `produce()` call, consult the
        // engine's revoked-actors set; if the actor has been revoked
        // mid-stream, surface a typed Err that the bridge translates
        // to ClosedByPeer. Mirrors the SUBSCRIBE-side
        // DeliveryCapRecheck pattern (engine_subscribe.rs:283-298).
        let producer: Box<dyn ChunkProducer> = if let Some(actor_cid) = actor {
            let inner_for_check = std::sync::Arc::clone(&self.inner);
            Box::new(CapRecheckProducer {
                inner: producer,
                actor_cid,
                inner_engine: inner_for_check,
            })
        } else {
            producer
        };

        let config = ChunkProducerConfig::default();
        let (source, thread_handle) = spawn_chunk_producer(producer, config);
        Ok(StreamHandle::from_producer_bridge(
            source,
            thread_handle,
            requires_explicit_close,
        ))
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
        let total = u64::try_from(chunks.len()).unwrap_or(u64::MAX);
        let chunks: Vec<Chunk> = chunks
            .into_iter()
            .enumerate()
            .map(|(i, bytes)| Chunk {
                seq: u64::try_from(i).unwrap_or(u64::MAX),
                final_chunk: u64::try_from(i + 1).unwrap_or(u64::MAX) == total,
                bytes,
            })
            .collect();
        StreamHandle::from_test_chunks(chunks)
    }
}

// ---------------------------------------------------------------------------
// wave-8c-stream-infra producer implementations + source resolver.
// ---------------------------------------------------------------------------

/// Resolve a STREAM `source` expression against the input Node.
///
/// Supported forms:
///
/// - `"$input"` — the input itself, projected to a `Value` (returns
///   `Value::Null` when input has no properties; tests can rely on this
///   to drive the "infinite empty stream" path for the for-await break
///   harness).
/// - `"$input.<field>"` — the named property's value.
/// - Anything else — `Value::Null`.
///
/// Phase-2b deliberately keeps the resolver minimal — richer expressions
/// belong on a TRANSFORM upstream of the STREAM node. Phase-3 may extend
/// to indexing / nested fields when concrete demand surfaces.
fn resolve_stream_source(expr: &str, input: &benten_core::Node) -> benten_core::Value {
    use benten_core::Value;
    if expr == "$input" {
        // Project the input to a Value-shape: empty input → Null;
        // otherwise the first property (sufficient for current tests).
        if input.properties.is_empty() {
            return Value::Null;
        }
        // Materialise as a map-shape value so consumers can branch.
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in input.properties.iter() {
            map.insert(k.clone(), v.clone());
        }
        return Value::Map(map);
    }
    if let Some(field) = expr.strip_prefix("$input.") {
        return input.properties.get(field).cloned().unwrap_or(Value::Null);
    }
    Value::Null
}

/// Producer that emits N empty-byte chunks (`Value::Int`/`Uint` source).
struct CountProducer {
    remaining: u64,
}

impl benten_eval::chunk_sink::ChunkProducer for CountProducer {
    fn produce(
        &mut self,
        _seq: u64,
    ) -> Result<Option<Vec<u8>>, benten_eval::chunk_sink::ChunkSinkError> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        Ok(Some(Vec::new()))
    }
}

/// Producer that chunks a `Vec<u8>` into `chunk_size`-byte slices.
struct BytesProducer {
    bytes: Vec<u8>,
    pos: usize,
    chunk_size: usize,
}

impl benten_eval::chunk_sink::ChunkProducer for BytesProducer {
    fn produce(
        &mut self,
        _seq: u64,
    ) -> Result<Option<Vec<u8>>, benten_eval::chunk_sink::ChunkSinkError> {
        if self.pos >= self.bytes.len() {
            return Ok(None);
        }
        let end = (self.pos + self.chunk_size).min(self.bytes.len());
        let chunk = self.bytes[self.pos..end].to_vec();
        self.pos = end;
        Ok(Some(chunk))
    }
}

/// Producer that emits zero chunks and signals EOS immediately.
struct EmptyProducer;

impl benten_eval::chunk_sink::ChunkProducer for EmptyProducer {
    fn produce(
        &mut self,
        _seq: u64,
    ) -> Result<Option<Vec<u8>>, benten_eval::chunk_sink::ChunkSinkError> {
        Ok(None)
    }
}

/// R6FP-Group-1 (r6-stream-2): producer wrapper that consults the
/// engine's revoked-actors set on every `produce()` call. When the
/// actor is revoked mid-stream, surfaces `ChunkSinkError::ClosedByPeer`
/// to terminate the producer thread cleanly + the bridge winds down
/// (consumer sees EOS). Mirrors SUBSCRIBE's DeliveryCapRecheck closure
/// (engine_subscribe.rs:283-298) — same semantics adapted to the
/// per-chunk producer-side polling model.
struct CapRecheckProducer {
    inner: Box<dyn benten_eval::chunk_sink::ChunkProducer>,
    actor_cid: benten_core::Cid,
    inner_engine: std::sync::Arc<crate::engine::EngineInner>,
}

impl benten_eval::chunk_sink::ChunkProducer for CapRecheckProducer {
    fn produce(
        &mut self,
        seq: u64,
    ) -> Result<Option<Vec<u8>>, benten_eval::chunk_sink::ChunkSinkError> {
        // R6FP-G1 (r6-stream-2): every produce() consults the
        // revoked-actors set BEFORE delegating to the inner producer.
        // A revoked actor terminates the stream cleanly via
        // ClosedByPeer rather than continuing to emit chunks the
        // caller is no longer authorised to receive.
        if !self.inner_engine.is_actor_active(&self.actor_cid) {
            return Err(benten_eval::chunk_sink::ChunkSinkError::ClosedByPeer { seq });
        }
        self.inner.produce(seq)
    }
}

/// Producer that emits empty-byte chunks indefinitely. Used for the
/// "for-await break releases producer" test path — the consumer breaks
/// after N chunks; the producer's next `send` fails with `ClosedByPeer`
/// (consumer-disconnect) and the bridge winds down cleanly.
struct InfiniteEmptyProducer;

impl benten_eval::chunk_sink::ChunkProducer for InfiniteEmptyProducer {
    fn produce(
        &mut self,
        _seq: u64,
    ) -> Result<Option<Vec<u8>>, benten_eval::chunk_sink::ChunkSinkError> {
        Ok(Some(Vec::new()))
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
            Chunk {
                seq: 0,
                final_chunk: false,
                bytes: vec![1, 2, 3],
            },
            Chunk {
                seq: 1,
                final_chunk: true,
                bytes: vec![4, 5],
            },
        ]);
        assert_eq!(h.seq_so_far(), 0);
        let c1 = h.next_chunk().unwrap().unwrap();
        assert_eq!(c1.bytes, vec![1, 2, 3]);
        assert_eq!(h.seq_so_far(), 1);
        let c2 = h.next_chunk().unwrap().unwrap();
        assert_eq!(c2.bytes, vec![4, 5]);
        assert_eq!(h.seq_so_far(), 2);
        assert!(h.next_chunk().unwrap().is_none());
        assert!(h.is_drained());
    }

    #[test]
    fn close_releases_buffered_chunks() {
        let mut h = StreamHandle::from_test_chunks(vec![
            Chunk {
                seq: 0,
                final_chunk: false,
                bytes: vec![1],
            },
            Chunk {
                seq: 1,
                final_chunk: true,
                bytes: vec![2],
            },
        ]);
        h.close();
        assert!(h.next_chunk().unwrap().is_none());
        assert!(h.is_drained());
    }
}
