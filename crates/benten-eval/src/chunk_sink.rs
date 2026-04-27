//! Phase-2b G6-A — STREAM `ChunkSink` trait + lossless / lossy default impls
//! per D4-RESOLVED.
//!
//! ## Decisions baked in
//!
//! - **PULL-based bounded mpsc** is the default-lossless transport
//!   (D4-RESOLVED). The trait surface is intentionally synchronous
//!   (`fn send(&mut self, ...) -> Result<...>` and not `-> impl Future`)
//!   because every G6-A R3-A red-phase test exercises the sink without
//!   `.await`. The sync API is canonical: it's compatible with the napi
//!   async-iterator bridge in G6-B (the bridge wraps the blocking call on
//!   a worker thread per the napi convention) and avoids dragging tokio
//!   into `benten-eval`. Async-friendly variants stay reserved for the
//!   Phase-3 iroh transport boundary, where credit-based push protocols
//!   warrant the additional surface area.
//!
//! - **Default capacity 16** (`DEFAULT_CAPACITY`). Picked because (a) it's
//!   the same as `tokio::sync::mpsc::channel(16)` defaults the streaming-
//!   systems R1 reviewer cited and (b) it's small enough that adversarial
//!   slow consumers engage backpressure quickly during tests, large enough
//!   to absorb realistic burst patterns at the napi boundary. Doc-drift is
//!   pinned by `chunk_sink_default_capacity_is_16`.
//!
//! - **Capacity zero rejected at the type level** via `NonZeroUsize`. A
//!   zero-capacity sink would deadlock the very first send.
//!
//! - **Lossless mode is default; lossy is opt-in** via
//!   [`testing_make_chunk_sink_lossy`](crate::testing::testing_make_chunk_sink_lossy)
//!   in tests + the `lossy_mode` builder field at production call sites.
//!   Lossy mode emits `E_STREAM_BACKPRESSURE_DROPPED` to the trace per
//!   dropped chunk — never silent.
//!
//! - **Producer-wallclock budget** kills permanently-stalled lossless
//!   sends (streaming-systems implementation hint). Default disabled
//!   (`None`); enabled via the wallclock-budget builder.
//!
//! ## Trace-preservation pattern (D1 carry from G12-A)
//!
//! Both `E_STREAM_BACKPRESSURE_DROPPED` and `E_STREAM_CLOSED_BY_PEER`
//! mirror G12-A's `inv_8_iteration` trace-preservation flow: the typed
//! error captures `consumed` / `limit` / `path`-style context so the
//! evaluator can emit a `TraceStep::BudgetExhausted` row with
//! `budget_type = "stream_backpressure"` BEFORE propagating the typed
//! error up the call stack. The `BudgetExhausted` emission is the
//! evaluator's responsibility (G6-A doesn't reach into the evaluator
//! state from the sink); this module surfaces the typed error envelope
//! the evaluator inspects.

use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use benten_errors::ErrorCode;

/// Default sink capacity (D4-RESOLVED).
///
/// Pinned by `chunk_sink_default_capacity_is_16` so docs + DX guides can't
/// drift from the implementation.
pub const DEFAULT_CAPACITY: NonZeroUsize = match NonZeroUsize::new(16) {
    Some(n) => n,
    None => unreachable!(),
};

/// A single chunk on the stream wire.
///
/// `seq` is engine-assigned (monotonic per stream); `final_chunk = true`
/// signals the producer's intent that this chunk closes the stream (e.g.
/// the close-marker emitted by [`ChunkSink::close`]). Carries `bytes` as
/// `Vec<u8>` to avoid pulling in the `bytes` crate; `Vec::into()` from
/// existing call sites is identity-into for `Vec<u8>` so test idiom
/// `vec![..].into()` compiles either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Engine-assigned monotonic seq.
    pub seq: u64,
    /// Chunk payload bytes.
    pub bytes: Vec<u8>,
    /// True when this chunk closes the stream.
    pub final_chunk: bool,
}

/// Outcome of a successful `send` / `try_send`.
///
/// `Accepted` is the common case; `BackpressureCredit(remaining_credit)`
/// signals "accepted but the buffer is filling — back off"; `Closed`
/// signals "consumer disconnected before the producer noticed" (lossless
/// `send` will normally surface this as `Err(ChunkSinkError::ClosedByPeer)`,
/// but the variant is reserved on the success path so future credit-based
/// transports can express clean shutdown without an error).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOutcome {
    /// Chunk handed to the buffer with capacity to spare.
    Accepted,
    /// Chunk handed to the buffer; remaining capacity follows in the
    /// payload so the producer can pace itself.
    BackpressureCredit(usize),
    /// Sink already closed before the send arrived. Reserved for clean-
    /// shutdown idioms; lossless `send` surfaces this as
    /// `Err(ChunkSinkError::ClosedByPeer)` instead.
    Closed,
}

/// Typed error surface for `ChunkSink::send` / `try_send` / `close`.
///
/// `#[non_exhaustive]` because Phase-3 iroh credit-based transports may
/// add `RemoteBackpressureCredit` / `Disconnected` variants that downstream
/// `match` arms must opt into.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ChunkSinkError {
    /// Constructor refused a zero-capacity sink. `NonZeroUsize` makes this
    /// unreachable through the typed constructor; the variant exists as a
    /// runtime backstop for indirect builders that take `usize`.
    #[error("chunk sink capacity must be non-zero")]
    CapacityZero,
    /// Lossy `try_send` dropped a chunk because the buffer was full. Loud
    /// at the trace surface — never silent.
    #[error("backpressure dropped chunk seq={seq}; capacity={capacity}")]
    BackpressureDropped {
        /// Seq of the dropped chunk.
        seq: u64,
        /// Sink capacity (so the trace + diagnostic message is self-
        /// describing).
        capacity: usize,
    },
    /// Consumer dropped the source side; the producer's next send fails
    /// closed. D4-RESOLVED + tests/stream_basic.
    #[error("stream consumer disconnected; producer cannot deliver chunk seq={seq}")]
    ClosedByPeer {
        /// Seq of the chunk that could not be delivered.
        seq: u64,
    },
    /// Lossless producer's wallclock budget elapsed while awaiting
    /// available capacity. Kills permanently-stalled sends.
    #[error("stream producer wallclock budget elapsed after {elapsed_ms}ms (budget {budget_ms}ms)")]
    ProducerWallclockExceeded {
        /// Elapsed time before the budget fired.
        elapsed_ms: u64,
        /// Configured budget.
        budget_ms: u64,
    },
}

impl ChunkSinkError {
    /// Map the typed error to its stable catalog code.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ChunkSinkError::CapacityZero => ErrorCode::InputLimit,
            ChunkSinkError::BackpressureDropped { .. } => ErrorCode::StreamBackpressureDropped,
            ChunkSinkError::ClosedByPeer { .. } => ErrorCode::StreamClosedByPeer,
            ChunkSinkError::ProducerWallclockExceeded { .. } => {
                ErrorCode::StreamProducerWallclockExceeded
            }
        }
    }
}

/// Trace entry surfaced by the sink for the evaluator's tracing pipeline.
///
/// G6-A surfaces drop entries via this lightweight envelope so the lossy-
/// mode "drops are loud" contract holds without coupling `chunk_sink.rs`
/// to the evaluator's `TraceStep` enum directly. The evaluator + napi
/// layer (G12-F) consume these via [`ChunkSink::drain_trace`] and project
/// them into `TraceStep::BudgetExhausted { budget_type:
/// "stream_backpressure", ... }` rows per the trace-preservation pattern
/// (D1 carry from G12-A).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkTraceEntry {
    /// A chunk was dropped by lossy `try_send` on a saturated sink.
    BackpressureDropped {
        /// Seq of the dropped chunk.
        seq: u64,
        /// Sink capacity at the time of drop.
        capacity: usize,
    },
    /// The producer's wallclock budget elapsed.
    WallclockExceeded {
        /// Elapsed time before the budget fired.
        elapsed_ms: u64,
        /// Configured budget.
        budget_ms: u64,
    },
}

impl SinkTraceEntry {
    /// True iff this entry is a `BackpressureDropped` row.
    #[must_use]
    pub fn is_backpressure_dropped(&self) -> bool {
        matches!(self, SinkTraceEntry::BackpressureDropped { .. })
    }
}

/// The streaming sink trait.
///
/// `Send + 'static` (no `Sync`) by construction — chunks flow through a
/// single producer thread; the napi async-iterator bridge moves the sink
/// across worker thread boundaries via `std::thread::spawn` (pinned by
/// `chunk_sink_send_static_no_lifetime_thread`). The trait is object-safe
/// so a `Box<dyn ChunkSink>` is a viable storage shape for the engine's
/// evaluator-owned sink table.
pub trait ChunkSink: Send + 'static {
    /// Lossless send. Awaits available capacity until either the chunk is
    /// accepted, the consumer disconnects (`ClosedByPeer`), or the
    /// producer-wallclock budget fires (`ProducerWallclockExceeded`).
    ///
    /// # Errors
    ///
    /// See [`ChunkSinkError`].
    fn send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError>;

    /// Lossy send. Returns `Err(ChunkSinkError::BackpressureDropped)`
    /// immediately when the buffer is full instead of awaiting capacity.
    /// Always emits a `SinkTraceEntry::BackpressureDropped` row on drop.
    ///
    /// # Errors
    ///
    /// See [`ChunkSinkError`].
    fn try_send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError>;

    /// Close the sink. Idempotent. After close, sends fail with
    /// `ClosedByPeer` (or are no-ops if the sink was already closed).
    ///
    /// # Errors
    ///
    /// Returns the underlying transport error if shutdown signalling
    /// fails; for the in-process default impl, `close` is infallible.
    fn close(&mut self) -> Result<(), ChunkSinkError>;

    /// Remaining capacity. Cheap accessor for caller-side back-pressure
    /// hints; not load-bearing for correctness.
    fn capacity_remaining(&self) -> usize;

    /// Drain accumulated trace entries. Returns the entries in producer
    /// order and clears the buffer. Tests assert "drops are loud" via
    /// this surface; the evaluator + napi layer (G12-F) do the same to
    /// project into `TraceStep::BudgetExhausted` rows.
    fn drain_trace(&mut self) -> Vec<SinkTraceEntry>;
}

// ---------------------------------------------------------------------------
// In-process bounded sink (the lossless / lossy default impls).
// ---------------------------------------------------------------------------

/// Shared state between the producer-side `BoundedSink` and the consumer-
/// side `ChunkSource`. Bounded `VecDeque<Chunk>` + condvars for blocking.
struct SharedChannel {
    inner: Mutex<ChannelInner>,
    capacity: usize,
    not_full: Condvar,
    not_empty: Condvar,
    consumer_alive: AtomicBool,
}

struct ChannelInner {
    buffer: VecDeque<Chunk>,
    closed: bool,
}

/// Producer side of the in-process bounded channel.
///
/// `Send + 'static`; not `Sync` (single-producer). Lossy + wallclock
/// behavior is configured at construction time and stays fixed across the
/// sink's lifetime to keep the per-send hot path branch-light.
pub struct BoundedSink {
    shared: Arc<SharedChannel>,
    next_seq_hint: u64,
    lossy_mode: bool,
    wallclock_budget: Option<Duration>,
    trace: Vec<SinkTraceEntry>,
    closed_local: bool,
}

/// Consumer side of the in-process bounded channel.
///
/// Decoupled from `BoundedSink` so the napi async-iterator bridge can move
/// the source onto its own worker thread independently of the sink's
/// producer thread. Dropping the source signals "consumer disconnected"
/// to the producer (next send fails with `ClosedByPeer`).
pub struct ChunkSource {
    shared: Arc<SharedChannel>,
    consumer_alive_owned: bool,
}

impl Drop for ChunkSource {
    fn drop(&mut self) {
        if self.consumer_alive_owned {
            self.shared.consumer_alive.store(false, Ordering::SeqCst);
            // Wake any blocked producer so it can observe the close.
            self.shared.not_full.notify_all();
        }
    }
}

impl ChunkSource {
    /// Non-blocking receive. Returns `Ok(None)` on a clean miss and
    /// `Ok(Some(chunk))` when a chunk is available.
    ///
    /// # Errors
    ///
    /// Reserved for transport-layer failures; the in-process channel is
    /// infallible at receive time.
    pub fn try_recv(&mut self) -> Result<Option<Chunk>, ChunkSinkError> {
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        let chunk = guard.buffer.pop_front();
        if chunk.is_some() {
            self.shared.not_full.notify_one();
        }
        Ok(chunk)
    }

    /// Blocking receive — waits until a chunk is available or the producer
    /// closes. Returns `Ok(None)` on clean producer-side close.
    ///
    /// # Errors
    ///
    /// Reserved for transport-layer failures.
    pub fn recv_blocking(&mut self) -> Result<Option<Chunk>, ChunkSinkError> {
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        loop {
            if let Some(chunk) = guard.buffer.pop_front() {
                self.shared.not_full.notify_one();
                return Ok(Some(chunk));
            }
            if guard.closed {
                return Ok(None);
            }
            guard = self
                .shared
                .not_empty
                .wait(guard)
                .expect("not_empty condvar poisoned");
        }
    }

    /// Blocking receive with a timeout.
    ///
    /// # Errors
    ///
    /// Reserved for transport-layer failures.
    pub fn recv_blocking_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<Chunk>, ChunkSinkError> {
        let deadline = Instant::now() + timeout;
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        loop {
            if let Some(chunk) = guard.buffer.pop_front() {
                self.shared.not_full.notify_one();
                return Ok(Some(chunk));
            }
            if guard.closed {
                return Ok(None);
            }
            let remaining = match deadline.checked_duration_since(Instant::now()) {
                Some(r) => r,
                None => return Ok(None),
            };
            let (g, res) = self
                .shared
                .not_empty
                .wait_timeout(guard, remaining)
                .expect("not_empty condvar poisoned");
            guard = g;
            if res.timed_out() && guard.buffer.is_empty() && !guard.closed {
                return Ok(None);
            }
        }
    }
}

impl BoundedSink {
    fn new(
        capacity: NonZeroUsize,
        lossy_mode: bool,
        wallclock_budget: Option<Duration>,
    ) -> (Self, ChunkSource) {
        let shared = Arc::new(SharedChannel {
            inner: Mutex::new(ChannelInner {
                buffer: VecDeque::with_capacity(capacity.get()),
                closed: false,
            }),
            capacity: capacity.get(),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
            consumer_alive: AtomicBool::new(true),
        });
        let sink = Self {
            shared: shared.clone(),
            next_seq_hint: 0,
            lossy_mode,
            wallclock_budget,
            trace: Vec::new(),
            closed_local: false,
        };
        let source = ChunkSource {
            shared,
            consumer_alive_owned: true,
        };
        (sink, source)
    }

    fn current_capacity_remaining(&self) -> usize {
        let guard = self.shared.inner.lock().expect("sink mutex poisoned");
        self.shared.capacity.saturating_sub(guard.buffer.len())
    }
}

/// Inherent forwarders so call sites can spell `sink.send(...)` /
/// `sink.try_send(...)` / `sink.close()` / `sink.capacity_remaining()` /
/// `sink.drain_trace()` without importing the [`ChunkSink`] trait. The
/// trait stays the canonical surface for `Box<dyn ChunkSink>` storage; the
/// inherent methods cut three lines of `use` boilerplate per test file.
impl BoundedSink {
    /// Inherent forwarder for [`ChunkSink::send`].
    ///
    /// # Errors
    /// See [`ChunkSinkError`].
    pub fn send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError> {
        <Self as ChunkSink>::send(self, chunk)
    }

    /// Inherent forwarder for [`ChunkSink::try_send`].
    ///
    /// # Errors
    /// See [`ChunkSinkError`].
    pub fn try_send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError> {
        <Self as ChunkSink>::try_send(self, chunk)
    }

    /// Inherent forwarder for [`ChunkSink::close`].
    ///
    /// # Errors
    /// See [`ChunkSinkError`].
    pub fn close(&mut self) -> Result<(), ChunkSinkError> {
        <Self as ChunkSink>::close(self)
    }

    /// Inherent forwarder for [`ChunkSink::capacity_remaining`].
    #[must_use]
    pub fn capacity_remaining(&self) -> usize {
        <Self as ChunkSink>::capacity_remaining(self)
    }

    /// Inherent forwarder for [`ChunkSink::drain_trace`].
    pub fn drain_trace(&mut self) -> Vec<SinkTraceEntry> {
        <Self as ChunkSink>::drain_trace(self)
    }
}

impl ChunkSink for BoundedSink {
    fn send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError> {
        if self.closed_local {
            return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
        }
        // Fail-fast on consumer disconnect — without this, a lossless
        // `send` would block forever once the source drops.
        if !self.shared.consumer_alive.load(Ordering::SeqCst) {
            return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
        }
        let started = Instant::now();
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        loop {
            if !self.shared.consumer_alive.load(Ordering::SeqCst) {
                return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
            }
            if guard.closed {
                return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
            }
            if guard.buffer.len() < self.shared.capacity {
                self.next_seq_hint = self.next_seq_hint.max(chunk.seq.saturating_add(1));
                let final_marker = chunk.final_chunk;
                guard.buffer.push_back(chunk);
                let remaining = self.shared.capacity - guard.buffer.len();
                self.shared.not_empty.notify_one();
                // Producer-side close-on-final discipline: a chunk with
                // `final_chunk: true` immediately marks the channel
                // closed so the consumer drains and observes EOF.
                if final_marker {
                    guard.closed = true;
                    self.shared.not_empty.notify_all();
                    self.closed_local = true;
                }
                if remaining == 0 {
                    return Ok(SendOutcome::BackpressureCredit(0));
                } else if remaining < self.shared.capacity / 2 {
                    return Ok(SendOutcome::BackpressureCredit(remaining));
                }
                return Ok(SendOutcome::Accepted);
            }
            // Buffer full: lossless awaits, lossy drops.
            if self.lossy_mode {
                self.trace.push(SinkTraceEntry::BackpressureDropped {
                    seq: chunk.seq,
                    capacity: self.shared.capacity,
                });
                return Err(ChunkSinkError::BackpressureDropped {
                    seq: chunk.seq,
                    capacity: self.shared.capacity,
                });
            }
            // Lossless wait — apply wallclock budget if configured.
            match self.wallclock_budget {
                Some(budget) => {
                    let elapsed = started.elapsed();
                    if elapsed >= budget {
                        let elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
                        let budget_ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX);
                        self.trace.push(SinkTraceEntry::WallclockExceeded {
                            elapsed_ms,
                            budget_ms,
                        });
                        return Err(ChunkSinkError::ProducerWallclockExceeded {
                            elapsed_ms,
                            budget_ms,
                        });
                    }
                    let remaining = budget.saturating_sub(elapsed);
                    let (g, res) = self
                        .shared
                        .not_full
                        .wait_timeout(guard, remaining)
                        .expect("not_full condvar poisoned");
                    guard = g;
                    if res.timed_out() {
                        let elapsed_ms =
                            u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
                        let budget_ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX);
                        self.trace.push(SinkTraceEntry::WallclockExceeded {
                            elapsed_ms,
                            budget_ms,
                        });
                        return Err(ChunkSinkError::ProducerWallclockExceeded {
                            elapsed_ms,
                            budget_ms,
                        });
                    }
                }
                None => {
                    guard = self
                        .shared
                        .not_full
                        .wait(guard)
                        .expect("not_full condvar poisoned");
                }
            }
        }
    }

    fn try_send(&mut self, chunk: Chunk) -> Result<SendOutcome, ChunkSinkError> {
        if self.closed_local {
            return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
        }
        if !self.shared.consumer_alive.load(Ordering::SeqCst) {
            return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
        }
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        if guard.closed {
            return Err(ChunkSinkError::ClosedByPeer { seq: chunk.seq });
        }
        if guard.buffer.len() < self.shared.capacity {
            let final_marker = chunk.final_chunk;
            guard.buffer.push_back(chunk);
            let remaining = self.shared.capacity - guard.buffer.len();
            self.shared.not_empty.notify_one();
            if final_marker {
                guard.closed = true;
                self.shared.not_empty.notify_all();
                self.closed_local = true;
            }
            if remaining == 0 {
                return Ok(SendOutcome::BackpressureCredit(0));
            } else if remaining < self.shared.capacity / 2 {
                return Ok(SendOutcome::BackpressureCredit(remaining));
            }
            return Ok(SendOutcome::Accepted);
        }
        // Buffer full.
        if self.lossy_mode {
            self.trace.push(SinkTraceEntry::BackpressureDropped {
                seq: chunk.seq,
                capacity: self.shared.capacity,
            });
            return Err(ChunkSinkError::BackpressureDropped {
                seq: chunk.seq,
                capacity: self.shared.capacity,
            });
        }
        // Lossless try_send on full → also a typed drop, but with the
        // BackpressureDropped variant since try_send semantically asks
        // "deliver-or-fail-now".
        Err(ChunkSinkError::BackpressureDropped {
            seq: chunk.seq,
            capacity: self.shared.capacity,
        })
    }

    fn close(&mut self) -> Result<(), ChunkSinkError> {
        if self.closed_local {
            return Ok(());
        }
        let close_seq = self.next_seq_hint;
        // Push a final-marker chunk so the consumer observes a single
        // close event then EOF (matches `stream_close_propagates`).
        let marker = Chunk {
            seq: close_seq,
            bytes: Vec::new(),
            final_chunk: true,
        };
        // Honour consumer-disconnect: if the source already dropped, the
        // close is still locally idempotent; we just don't bother
        // pushing the marker.
        if !self.shared.consumer_alive.load(Ordering::SeqCst) {
            self.closed_local = true;
            return Ok(());
        }
        let mut guard = self.shared.inner.lock().expect("sink mutex poisoned");
        // Wait until the marker has room — the close marker is part of
        // the deliverable stream.
        loop {
            if guard.closed {
                self.closed_local = true;
                return Ok(());
            }
            if guard.buffer.len() < self.shared.capacity {
                guard.buffer.push_back(marker);
                guard.closed = true;
                self.shared.not_empty.notify_all();
                self.closed_local = true;
                return Ok(());
            }
            guard = self
                .shared
                .not_full
                .wait(guard)
                .expect("not_full condvar poisoned");
            if !self.shared.consumer_alive.load(Ordering::SeqCst) {
                self.closed_local = true;
                return Ok(());
            }
        }
    }

    fn capacity_remaining(&self) -> usize {
        self.current_capacity_remaining()
    }

    fn drain_trace(&mut self) -> Vec<SinkTraceEntry> {
        std::mem::take(&mut self.trace)
    }
}

// ---------------------------------------------------------------------------
// Reserved counter-style accessor for G7-A's CountedSink to wrap the
// sink with byte-accumulating accounting (D17 streaming-sink output-bytes
// path). Phase-2b G6-A leaves the counter unimplemented; G7-A composes
// over the trait, not over a sibling counter.
// ---------------------------------------------------------------------------

/// Process-wide active-sink counter — used by tests that want to assert
/// that Drop releases sink resources. Increments on construction;
/// decrements when the producer side is dropped. Cheap atomic.
static ACTIVE_SINKS: AtomicUsize = AtomicUsize::new(0);

impl Drop for BoundedSink {
    fn drop(&mut self) {
        ACTIVE_SINKS.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Accessor for the active-sink count. Visible to tests via the
/// `testing` module.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn active_sink_count() -> usize {
    ACTIVE_SINKS.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Constructor surface — exposed both as a typed public constructor for
// production-side callers and re-exported from the `testing` module.
// ---------------------------------------------------------------------------

/// Build a default lossless bounded sink + its source.
///
/// # Panics
/// Never panics; the constructor accepts a `NonZeroUsize` so the zero-
/// capacity edge case is impossible at the type level.
#[must_use]
pub fn make_chunk_sink(capacity: NonZeroUsize) -> (BoundedSink, ChunkSource) {
    ACTIVE_SINKS.fetch_add(1, Ordering::Relaxed);
    BoundedSink::new(capacity, false, None)
}

/// Build a lossy bounded sink + its source. `try_send` on a saturated
/// buffer drops the chunk and emits a typed-loud trace entry.
#[must_use]
pub fn make_chunk_sink_lossy(capacity: NonZeroUsize) -> (BoundedSink, ChunkSource) {
    ACTIVE_SINKS.fetch_add(1, Ordering::Relaxed);
    BoundedSink::new(capacity, true, None)
}

/// Build a lossless bounded sink with a producer wallclock budget. A
/// blocked `send` past the budget surfaces
/// `ChunkSinkError::ProducerWallclockExceeded` rather than blocking forever.
#[must_use]
pub fn make_chunk_sink_with_wallclock(
    capacity: NonZeroUsize,
    budget: Duration,
) -> (BoundedSink, ChunkSource) {
    ACTIVE_SINKS.fetch_add(1, Ordering::Relaxed);
    BoundedSink::new(capacity, false, Some(budget))
}
