//! # benten-eval — Operation primitives + evaluator (STUB)
//!
//! Phase 1 stub. Full implementation lands in Phase 1 proper.
//!
//! Responsibilities (Phase 1 proper):
//!
//! - The 12 operation primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE,
//!   WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM).
//! - Iterative evaluator with an explicit execution stack (no recursion).
//! - Registration-time structural validation (14 invariants).
//! - Transaction primitive (begin/commit/rollback).
//! - TRANSFORM expression evaluator (arithmetic, built-ins, object construction).
//! - `wasmtime`-based SANDBOX host with fuel metering.
//!
//! See [`docs/ENGINE-SPEC.md`](../../../docs/ENGINE-SPEC.md) Sections 3–5 and 10.
//!
//! The spike uses this crate only to validate that the 6-crate workspace
//! compiles cleanly and that `benten-engine` can depend on it.

#![forbid(unsafe_code)]

/// Marker for the current stub phase. Removed when the evaluator lands.
pub const STUB_MARKER: &str = "benten-eval::stub";
