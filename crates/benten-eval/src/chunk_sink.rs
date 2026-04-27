//! Wave-4 chunk-sink-scaffold precondition (Phase-2b R5 §6.0.1).
//!
//! Stub trait + chunk type so G6-A and G7-A can compile-reference
//! `benten_eval::chunk_sink::ChunkSink` independently and merge in
//! either order without compile-fail. G6-A fills in the real trait
//! body (`fn send`, `fn try_send`, `fn close`, `fn capacity_remaining`)
//! + a `tokio::sync::mpsc`-backed default impl per D4-RESOLVED PULL-mpsc
//! semantics; G7-A's `CountedSink` references this trait.
//!
//! Until G6-A lands, this trait is empty and the `Chunk` newtype is the
//! only meaningful surface. Production STREAM behavior is NOT yet
//! exercised — G6-A is the wave-4 group that brings it online.

/// Sink for streaming chunks emitted by a STREAM primitive (and by
/// SANDBOX host functions writing back into the engine, per G7-A's
/// CountedSink wiring). Trait body intentionally empty in the
/// scaffold; G6-A adds the methods per plan §3 G6-A.
pub trait ChunkSink: Send {}

/// Single chunk of streaming output. Newtype wrapper around `Vec<u8>`
/// so the trait surface can name a stable type without leaking the
/// underlying byte representation.
pub struct Chunk(pub Vec<u8>);
