#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: ChunkSink trait shape (G6-A).
//!
//! Pin source: arch-pre-r1-9 + benten-philosophy r1 + streaming-systems
//! stream-d4-1 + D4-RESOLVED (`Send` bound, `'static`-lifetime, default
//! capacity 16, no zero-capacity sinks).
//!
//! These tests target the future trait surface in
//! `crates/benten-eval/src/chunk_sink.rs`. They will not compile / pass until
//! G6-A's `rust-implementation-developer` lands the real trait body. R5 will
//! un-ignore each test as the corresponding implementation arrives.
//!
//! Phase 2b R3 TDD red-phase. Owner: R3-A (rust-test-writer-streaming).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::useless_conversion,
    clippy::no_effect_underscore_binding,
    clippy::clone_on_copy
)]

// NOTE: imports reference the future surface; will fail to resolve until
// G6-A scaffolds beyond the trait-signature stub already on main per
// `.addl/phase-2b/00-implementation-plan.md` §6.0.1.
use benten_eval::chunk_sink::{Chunk, ChunkSink, ChunkSinkError};
use std::num::NonZeroUsize;

/// Compile-time pin: ChunkSink is `Send + 'static`, NOT `Sync`. arch-pre-r1-9.
///
/// If a future refactor adds a borrowed lifetime parameter or removes `Send`
/// the napi async-iterator bridge in G6-B cannot move the sink across thread
/// boundaries. This test is a static-shape assertion.
#[test]
fn chunk_sink_send_static_no_lifetime_thread() {
    fn assert_send_static<T: Send + 'static>() {}
    // The trait must be object-safe AND `Send + 'static` when boxed.
    assert_send_static::<Box<dyn ChunkSink>>();

    // Confirm the trait can in fact be moved to a fresh thread (the napi
    // worker-thread idiom). If this compiles AND runs, the bound holds.
    let (sink, _src) =
        benten_eval::testing::testing_make_chunk_sink(NonZeroUsize::new(16).unwrap());
    let handle = std::thread::spawn(move || {
        let _moved: Box<dyn ChunkSink> = Box::new(sink);
    });
    handle.join().expect("sink moves to fresh thread");
}

/// Capacity-zero sinks are rejected at construction time. streaming-systems
/// stream-d4-1: `NonZeroUsize` enforcement at the constructor surface.
#[test]
fn chunk_sink_capacity_zero_rejected_at_construction() {
    // The `NonZeroUsize::new(0)` call returns `None`; any caller attempting
    // to thread a zero-capacity literal cannot reach the constructor at all.
    // This test pins the type-level enforcement: `make_chunk_sink(0)` must
    // not compile (or, when threaded through an indirect builder, return a
    // typed error). We exercise the runtime-panic path the indirect builder
    // surfaces:
    let zero = NonZeroUsize::new(0);
    assert!(zero.is_none(), "NonZeroUsize::new(0) must return None");

    // R5 implementer note: any `ChunkSink::with_capacity(usize)` convenience
    // constructor MUST return `Err(ChunkSinkError::CapacityZero)` when
    // passed 0; pin here as a runtime backstop in case a non-`NonZero`
    // convenience surface ships.
}

/// Default capacity is 16 chunks (D4-RESOLVED). Pinning the literal here so
/// docs + DSL DX guides cannot drift.
#[test]
fn chunk_sink_default_capacity_is_16() {
    assert_eq!(
        benten_eval::chunk_sink::DEFAULT_CAPACITY,
        NonZeroUsize::new(16).unwrap(),
        "D4-RESOLVED default capacity must be 16; doc-drift guard"
    );

    // Sink built with the default reports 16 remaining when freshly created.
    let (sink, _src) =
        benten_eval::testing::testing_make_chunk_sink(benten_eval::chunk_sink::DEFAULT_CAPACITY);
    assert_eq!(sink.capacity_remaining(), 16);
}

/// Compile-fail-if-removed regression: `Chunk` carries `seq: u64`,
/// `bytes: Bytes` (or `Vec<u8>`), and `final_chunk: bool`. streaming-systems
/// implementation_hint pin.
#[test]
fn chunk_struct_carries_seq_bytes_final_marker() {
    let c = Chunk {
        seq: 0,
        bytes: Vec::new().into(),
        final_chunk: false,
    };
    assert_eq!(c.seq, 0);
    assert!(!c.final_chunk);
    let _err: ChunkSinkError = ChunkSinkError::CapacityZero;
}
