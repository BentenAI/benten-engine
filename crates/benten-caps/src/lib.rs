//! # benten-caps — Capability policy (STUB)
//!
//! Phase 1 stub. Full implementation lands in Phase 1 proper.
//!
//! Responsibilities (Phase 1 proper):
//!
//! - Define the `CapabilityPolicy` pre-write hook trait.
//! - Provide the `NoAuthBackend` default (allows all writes — embedded/local-only
//!   users pay zero cost for capability enforcement).
//! - Expose capability types (grants, scopes, attenuation) as plain data that
//!   `benten-engine` wires into the write path.
//! - UCAN backend stub (full implementation deferred to `benten-id` in Phase 3).
//!
//! See [`docs/ENGINE-SPEC.md`](../../../docs/ENGINE-SPEC.md) Section 9.
//!
//! The spike uses this crate only to validate that the 6-crate workspace
//! compiles cleanly and that `benten-engine` can depend on it.

#![forbid(unsafe_code)]

/// Marker for the current stub phase. Removed when real capability policy lands.
pub const STUB_MARKER: &str = "benten-caps::stub";
