//! Aggregator for security-class subdirectory tests under `tests/security/`.
//!
//! Cargo does NOT auto-discover `.rs` files in subdirectories of `tests/`;
//! they need an explicit aggregator in `tests/<name>.rs` that declares the
//! submodules. Without this file, `tests/security/subscribe_caps.rs` was
//! silently dropped from `cargo nextest` discovery (rust-test-coverage A2 +
//! security-test-reviewer.json sec-r4-1 + r3-consolidation.md §5 item 9).
//!
//! Owner: R4-FP Bucket A fix-pass (orchestrator).
//!
//! The `phase_2b_landed` feature gate that originally protected this
//! aggregator + its submodule was retired at G20-B Phase-3-close per
//! audit-3-mr-1 — Phase 2b shipped at `phase-2b-close` 2026-05-03 and
//! the contained tests run cleanly under default features.

#[path = "security/subscribe_caps.rs"]
pub mod subscribe_caps;
