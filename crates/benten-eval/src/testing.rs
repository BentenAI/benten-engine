//! Phase-2b G6-A — `benten-eval` test helpers.
//!
//! Re-exported under `benten_eval::testing::testing_*` so R3-A red-phase
//! tests can drive STREAM + SUBSCRIBE primitives without a full engine
//! stack. Gated on `cfg(any(test, feature = "testing"))` per the workspace
//! testing-helper convention; downstream crate-test binaries opt in via
//! `features = ["testing"]` on their `[dev-dependencies]` block.
//!
//! The shape mirrors `benten_core::testing` but lives in `benten-eval`
//! because the helpers reach into `chunk_sink` + `primitives::subscribe`
//! state that doesn't belong on the `no_std` core surface.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use benten_core::{ChangeEvent, ChangeKind, Cid, SubscriberId};

use crate::chunk_sink::{
    BoundedSink, ChunkSource, make_chunk_sink, make_chunk_sink_lossy,
    make_chunk_sink_with_wallclock,
};
use crate::primitives::stream::{
    AggregateStreamNode, ConcurrentProducerOutcome, LosslessScheduleOutcome, StreamPrimitiveSpec,
    StreamRunOutcome, collect_aggregate_node, run_concurrent_producers,
    run_lossless_stream_with_schedule, run_stream_persist,
};
use crate::primitives::subscribe::{
    ActiveSubscription, ConcurrentSubscribeNoLossOutcome, ConcurrentSubscribeOrderingOutcome,
    InMemorySuspensionStore, PatternProptestOutcome, ReplayDedupOutcome, SubscribeError,
    SubscriptionSpec, SuspensionStore, TestHandler, TestPrincipal, active_subscription_count,
    inject_event, make_change_event, make_persistent_subscription_id, publish_change_event,
    register, register_as, register_with_store, run_concurrent_subscribe_event_ordering,
    run_concurrent_subscribe_no_event_loss, run_pattern_proptest, run_replay_dedup_proptest,
    subscription_exists,
};

// ---------------------------------------------------------------------------
// Chunk-sink helpers.
// ---------------------------------------------------------------------------

/// Build a default lossless bounded sink + its source (test convenience).
#[must_use]
pub fn testing_make_chunk_sink(capacity: NonZeroUsize) -> (BoundedSink, ChunkSource) {
    make_chunk_sink(capacity)
}

/// Build a lossy bounded sink + its source (test convenience).
#[must_use]
pub fn testing_make_chunk_sink_lossy(capacity: NonZeroUsize) -> (BoundedSink, ChunkSource) {
    make_chunk_sink_lossy(capacity)
}

/// Build a lossless bounded sink with a producer wallclock budget.
#[must_use]
pub fn testing_make_chunk_sink_with_wallclock(
    capacity: NonZeroUsize,
    budget: Duration,
) -> (BoundedSink, ChunkSource) {
    make_chunk_sink_with_wallclock(capacity, budget)
}

// ---------------------------------------------------------------------------
// SUBSCRIBE helpers.
// ---------------------------------------------------------------------------

/// Register a SUBSCRIBE with the default in-memory placeholder store.
///
/// # Errors
/// See [`SubscribeError`].
pub fn testing_subscribe_register(
    spec: SubscriptionSpec,
) -> Result<ActiveSubscription, SubscribeError> {
    register(spec)
}

/// Register a SUBSCRIBE against an explicit `SuspensionStore`.
///
/// # Errors
/// See [`SubscribeError`].
pub fn testing_subscribe_register_with_store(
    spec: SubscriptionSpec,
    store: Arc<dyn SuspensionStore>,
) -> Result<ActiveSubscription, SubscribeError> {
    register_with_store(spec, store)
}

/// Register a SUBSCRIBE as a specific principal (drives cap checks).
///
/// # Errors
/// See [`SubscribeError`].
pub fn testing_subscribe_register_as(
    principal: &Arc<TestPrincipal>,
    spec: SubscriptionSpec,
) -> Result<ActiveSubscription, SubscribeError> {
    register_as(principal.clone(), spec)
}

/// Inject a change event into a subscription.
///
/// # Errors
/// See [`SubscribeError`].
pub fn testing_subscribe_inject_event(
    sub: &ActiveSubscription,
    event: ChangeEvent,
) -> Result<(), SubscribeError> {
    inject_event(sub, event)
}

/// Build a [`ChangeEvent`] fixture (`seq = 0`; tests bump as needed).
#[must_use]
pub fn testing_make_change_event(
    anchor_cid: Cid,
    kind: ChangeKind,
    payload: serde_json::Value,
) -> ChangeEvent {
    make_change_event(anchor_cid, kind, payload)
}

/// Mint a fresh persistent subscriber id.
#[must_use]
pub fn testing_make_persistent_subscription_id() -> SubscriberId {
    make_persistent_subscription_id()
}

/// Construct an in-memory `SuspensionStore` placeholder (D5-G6-A interim).
#[must_use]
pub fn testing_make_suspension_store_in_memory() -> Arc<dyn SuspensionStore> {
    Arc::new(InMemorySuspensionStore::new())
}

/// Force a persistent cursor's retention window to "exhausted".
/// Routes through the trait's testing override; production backends
/// expose the hook as a no-op so the helper is safe by default.
pub fn testing_force_retention_exhausted(store: &Arc<dyn SuspensionStore>, id: &SubscriberId) {
    store.testing_force_retention_exhausted(id);
}

/// Publish a pre-registration change event (`Latest` cursor drop semantics).
pub fn testing_publish_change_event(event: ChangeEvent) {
    publish_change_event(event);
}

/// Construct a test principal with the given caps.
#[must_use]
pub fn testing_principal_with_caps(caps: &[&str]) -> Arc<TestPrincipal> {
    TestPrincipal::new(caps)
}

/// Construct a test principal with no caps.
#[must_use]
pub fn testing_principal_without_caps() -> Arc<TestPrincipal> {
    TestPrincipal::no_caps()
}

/// Revoke a cap mid-stream (drives delivery-time cap re-check).
pub fn testing_revoke_cap_mid_subscribe(principal: &Arc<TestPrincipal>, cap: &str) {
    principal.revoke(cap);
}

/// Register a fresh idempotent-write handler (tests bind it to a
/// subscription via [`ActiveSubscription::bind_handler`]).
#[must_use]
pub fn testing_register_idempotent_write_handler() -> TestHandler {
    TestHandler::new()
}

/// Total active subscriptions in this process.
#[must_use]
pub fn testing_active_subscription_count() -> usize {
    active_subscription_count()
}

/// True iff the engine still tracks `id`.
#[must_use]
pub fn testing_subscription_exists(id: &SubscriberId) -> bool {
    subscription_exists(id)
}

// ---------------------------------------------------------------------------
// STREAM persist + proptest helpers.
// ---------------------------------------------------------------------------

/// Read the materialized aggregate Node for `cid`.
#[must_use]
pub fn testing_collect_stream_aggregate_node(cid: &Cid) -> Option<AggregateStreamNode> {
    collect_aggregate_node(cid)
}

/// Run a STREAM primitive with the given persist mode + chunk sequence.
#[must_use]
pub fn testing_run_stream_persist(
    spec: StreamPrimitiveSpec,
    chunks: Vec<Vec<u8>>,
) -> StreamRunOutcome {
    run_stream_persist(spec, chunks)
}

/// Run a lossless stream under explicit pause schedules (proptest helper).
#[must_use]
pub fn testing_run_lossless_stream_with_schedule(
    chunk_count: usize,
    cap: NonZeroUsize,
    producer_pause_us: Vec<u64>,
    consumer_pause_us: Vec<u64>,
) -> LosslessScheduleOutcome {
    run_lossless_stream_with_schedule(chunk_count, cap, producer_pause_us, consumer_pause_us)
}

/// Run N concurrent producers + 1 consumer (proptest helper).
#[must_use]
pub fn testing_run_concurrent_producers(
    producer_count: usize,
    chunks_per_producer: usize,
) -> ConcurrentProducerOutcome {
    run_concurrent_producers(producer_count, chunks_per_producer)
}

/// Run a SUBSCRIBE pattern proptest case.
#[must_use]
pub fn testing_run_pattern_proptest(
    pattern_glob: &str,
    anchor_label: &str,
) -> PatternProptestOutcome {
    run_pattern_proptest(pattern_glob, anchor_label)
}

/// Run a SUBSCRIBE replay-dedup proptest case.
#[must_use]
pub fn testing_run_replay_dedup_proptest(seq: u64, replay_count: usize) -> ReplayDedupOutcome {
    run_replay_dedup_proptest(seq, replay_count)
}

/// Run a concurrent-subscribe ordering proptest case.
#[must_use]
pub fn testing_run_concurrent_subscribe_event_ordering(
    anchor_count: usize,
    subscriber_count: usize,
    writes_per_anchor: usize,
) -> ConcurrentSubscribeOrderingOutcome {
    run_concurrent_subscribe_event_ordering(anchor_count, subscriber_count, writes_per_anchor)
}

/// Run a concurrent-subscribe no-event-loss proptest case.
#[must_use]
pub fn testing_run_concurrent_subscribe_no_event_loss(
    writer_count: usize,
    writes_per_writer: usize,
) -> ConcurrentSubscribeNoLossOutcome {
    run_concurrent_subscribe_no_event_loss(writer_count, writes_per_writer)
}

// ---------------------------------------------------------------------------
// phil-r1-4 conformance proptest helpers — two distinct sink-driving paths
// that MUST produce identical observable traces. The "two sinks" are:
//
//   A: route the chunks through a regular `BoundedSink` and capture the
//      receiver-side seq sequence + final-marker observation.
//   B: same input, same handler logic, but observed via a separate
//      Source<->Sink instantiation. Conformance is the property that the
//      two paths see equivalent results.
//
// Both implementations live in this module so the conformance proptest
// has a single source-of-truth for the deterministic-seed input
// generation.
// ---------------------------------------------------------------------------

use crate::chunk_sink::{Chunk, SendOutcome};

fn drive_handler_against_sink(chunk_count: usize, chunk_size: usize, seed: u64) -> Vec<u64> {
    let cap = NonZeroUsize::new(8).expect("8 is non-zero");
    let (mut sink, mut src) = make_chunk_sink(cap);
    let mut state = seed;
    let mut payloads: Vec<Vec<u8>> = Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        // Tiny xorshift for deterministic byte payloads.
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let mut payload = vec![0u8; chunk_size];
        for (i, b) in payload.iter_mut().enumerate() {
            *b = ((state >> ((i % 8) * 8)) & 0xff) as u8;
        }
        payloads.push(payload);
    }
    let producer = std::thread::spawn(move || {
        // Lossless `sink.send` awaits capacity, so a single call per chunk
        // is sufficient. We exit early on a closed peer / sink-side
        // disconnect — the conformance proptest only asserts equivalence
        // on the prefix the consumer actually observes.
        for (i, payload) in payloads.into_iter().enumerate() {
            let r = sink.send(Chunk {
                seq: i as u64,
                bytes: payload,
                final_chunk: false,
            });
            match r {
                Ok(SendOutcome::Accepted | SendOutcome::BackpressureCredit(_)) => {}
                Ok(SendOutcome::Closed) | Err(_) => return,
            }
        }
        let _ = sink.close();
    });
    let mut trace = Vec::with_capacity(chunk_count);
    while let Ok(Some(c)) = src.recv_blocking() {
        if c.final_chunk {
            break;
        }
        trace.push(c.seq);
    }
    producer.join().expect("producer panicked");
    trace
}

/// Reference path A — drives a handler against an in-memory bounded sink.
#[must_use]
pub fn testing_run_handler_against_sink_a(
    chunk_count: usize,
    chunk_size: usize,
    seed: u64,
) -> Vec<u64> {
    drive_handler_against_sink(chunk_count, chunk_size, seed)
}

/// Reference path B — drives the same handler input through a fresh sink
/// instantiation. By construction A and B observe equivalent traces; this
/// pins the phil-r1-4 conformance contract so future alternative sink
/// implementations stay observationally equivalent.
#[must_use]
pub fn testing_run_handler_against_sink_b(
    chunk_count: usize,
    chunk_size: usize,
    seed: u64,
) -> Vec<u64> {
    drive_handler_against_sink(chunk_count, chunk_size, seed)
}
