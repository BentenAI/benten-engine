//! Criterion benchmark: WAIT suspend/resume latency.
//!
//! Measures the end-to-end cost of serialising an `ExecutionState` via
//! `suspend_to_bytes`, discarding the in-memory engine state, then
//! deserialising via `resume_from_bytes`. I/O is excluded — the
//! measurement captures the pure serialisation + structural-validation
//! round-trip, which is the component the engine can actually control.
//! Disk I/O on top of this is the consumer's problem (WAL fsync, network
//! transmit, etc.) and varies by target by 2+ orders of magnitude.
//!
//! **Target source:** plan §4.4 — "≤ 50 µs for suspend_to_bytes +
//! resume_from_bytes round-trip excluding I/O."
//!
//! **Gate policy:** INFORMATIONAL baseline. The plan §4.4 row tags this
//! benchmark `CI-gated` at 50 µs, but the R3 brief narrows it to
//! informational for Phase 2a because the WAIT primitive's G3-A / G3-B
//! implementation has enough moving pieces (DAG-CBOR envelope shape,
//! frame-stack canonicalisation, pinned-subgraph-CID re-verification)
//! that a hard gate at 50 µs is premature. The number is reported; humans
//! read it at Phase-2a close and the R4b reviewer decides whether to
//! promote the bench to CI-gated.
//!
//! **Threshold encoding (machine-readable):**
//!
//! ```text
//! BENCH_ID = wait_suspend_resume_latency/round_trip_no_io
//! THRESHOLD_NS = 50000  // 50 µs per plan §4.4 — INFORMATIONAL in 2a
//! POLICY = informational
//! ```
//!
//! ## Red-phase TDD
//!
//! The `Engine::call_with_suspension` / `suspend_to_bytes` /
//! `resume_from_bytes` trio are G3-B deliverables. At R3 they return
//! `todo!()`. The bench compiles + links but panics on first iteration;
//! once G3-B lands, the measurement becomes real.
//!
//! ## Phase-3 forward-compat
//!
//! The `SuspendedHandle` shape is frozen at 2a close (§8 frozen-interfaces
//! list). When Phase-3 sync layers grow on top of this, the bench number
//! should NOT regress — if it does, the Phase-3 layer added
//! per-suspension overhead that violates the round-trip budget. The bench
//! therefore survives into Phase 3 unchanged.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Duration;

use benten_core::{Node, Value};
use benten_engine::{Engine, SuspensionOutcome};
use criterion::{Criterion, criterion_group, criterion_main};

/// Build a WAIT-containing handler: READ → WAIT(signal) → TRANSFORM →
/// RESPOND. This is the §9.1 reference handler shape that every
/// suspend/resume test exercises; using the same shape here means the
/// bench number tracks the real-world cost, not an artificial microbench.
fn register_wait_handler(engine: &Engine) -> benten_core::Cid {
    // The concrete subgraph construction happens inside
    // `engine.register_wait_reference_handler()` (a G3-A testing helper).
    // At R3 the helper returns `todo!()`; the bench panics on first
    // iteration until G3-A lands it.
    engine
        .register_wait_reference_handler()
        .expect("register wait reference handler")
}

fn bench_wait_round_trip_no_io(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = register_wait_handler(&engine);

    // Drive to suspension once so the rest of the bench measures the
    // pure serialisation round-trip, not the evaluator walk.
    let mut input = BTreeMap::new();
    input.insert("trigger".into(), Value::Text("bench".into()));
    let input_node = Node::new(vec![], input);

    let suspended = match engine
        .call_with_suspension(&handler_id, "wait:entry", input_node)
        .expect("call_with_suspension succeeds")
    {
        SuspensionOutcome::Suspended(handle) => handle,
        SuspensionOutcome::Complete(_) => panic!(
            "WAIT reference handler must suspend; G3-A is misconfigured if Complete is returned"
        ),
    };

    // Serialize once to capture the envelope bytes used by the resume leg.
    let envelope_bytes = engine
        .suspend_to_bytes(&suspended)
        .expect("suspend_to_bytes");

    let mut group = c.benchmark_group("wait_suspend_resume_latency");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // MACHINE-READABLE GATE: informational — the exit-criteria workflow
    // reports this number but does not fail on regression in Phase 2a.
    // THRESHOLD_NS=50000 policy=informational source=plan-§4.4-informational-in-2a

    group.bench_function("round_trip_no_io", |b| {
        let signal_value = Value::Text("signal-fired".into());
        b.iter(|| {
            // Round-trip: serialise the existing handle + deserialise +
            // resume once. The resume produces a terminal Outcome (the
            // WAIT reference handler ends in RESPOND after the signal).
            let bytes = engine
                .suspend_to_bytes(black_box(&suspended))
                .expect("suspend_to_bytes");
            // Sanity that round-trip shape is preserved across the loop.
            debug_assert_eq!(bytes.len(), envelope_bytes.len());
            let outcome = engine
                .resume_from_bytes_unauthenticated(
                    black_box(&bytes),
                    black_box(signal_value.clone()),
                )
                .expect("resume_from_bytes_unauthenticated");
            black_box(outcome);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_wait_round_trip_no_io);
criterion_main!(benches);
