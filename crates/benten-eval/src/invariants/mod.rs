//! Structural- and runtime-invariant enforcement.
//!
//! Phase 1 shipped invariants 1/2/3/5/6/8-stopgap/9/10/12 inside a single
//! `invariants.rs` file. Phase 2a G1-A (plan §3) splits that file into a
//! directory so each downstream group (G4-A, G5-A, G5-B) can fill in its
//! Phase-2 invariant body without contention over a shared file.
//!
//! Layout:
//! - `structural` — invariants 1/2/3/5/6/9/10/12 (pure code-move from
//!   Phase-1 `invariants.rs`, unchanged semantics). Also owns
//!   `validate_transform_expressions` and the canonical DAG-CBOR encoder.
//! - `budget` — invariant 8 multiplicative cumulative budget (G4-A).
//! - `system_zone` — invariant 11 system-zone registration-time check
//!   (G5-B-i). Runtime enforcement lives in `benten-engine`.
//! - `immutability` — invariant 13 registration-time helper (G5-A). Runtime
//!   firing lives in `benten-graph`.
//! - `attribution` — invariant 14 structural declaration-time check
//!   (G5-B-ii). Runtime threading lives in `evaluator/attribution.rs`.
//! - `sandbox_depth` — invariant 4 SANDBOX nest-depth ceiling (G7-B).
//!   Both registration-time static analysis (`validate_registration`) and
//!   the runtime depth-check helper invoked by the SANDBOX primitive
//!   executor live here; the counter rides on
//!   `AttributionFrame.sandbox_depth: u8` per D20-RESOLVED.
//! - `sandbox_output` — invariant 7 SANDBOX cumulative output ceiling
//!   (G7-B). Provides the runtime check helper that the streaming
//!   `CountedSink` PRIMARY path (D17-RESOLVED) calls before accepting
//!   host-fn bytes; D15 trap-loudly default — no silent truncation.
//!
//! `mod.rs` is deliberately thin: downstream call-sites
//! (`benten_eval::invariants::validate_subgraph`,
//! `benten_eval::invariants::validate_transform_expressions`,
//! `benten_eval::invariants::canonical_subgraph_bytes`) are preserved as
//! re-exports so no engine-side or test-side import path changes.

pub mod attribution;
pub mod budget;
pub mod immutability;
pub mod sandbox_depth;
pub mod sandbox_output;
pub mod structural;
pub mod system_zone;

// Re-exports — preserve every call-site path from the pre-split single-file
// `invariants.rs` (plan §3 G1-A: "identical public surface; import paths
// preserved via `mod.rs` re-exports"). Downstream crates and tests continue
// to call `benten_eval::invariants::validate_subgraph` / `validate_builder`
// / `canonical_subgraph_bytes` / `validate_transform_expressions` without
// knowing the body moved into `structural.rs`.
pub(crate) use structural::{canonical_subgraph_bytes, validate_builder};
pub use structural::{validate_subgraph, validate_transform_expressions};
