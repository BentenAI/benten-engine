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
//! Why a `cfg(feature = "phase_2b_landed")` aggregator: the contained
//! `subscribe_caps.rs` uses `#![cfg(feature = "phase_2b_landed")]` as its
//! file-level gate, so listing `pub mod subscribe_caps;` here unconditionally
//! would still produce a (gated-empty) module under default features and is
//! safe — but mirroring the gate at the aggregator keeps the discipline
//! visible at the discovery point.

#![cfg(feature = "phase_2b_landed")]

#[path = "security/subscribe_caps.rs"]
pub mod subscribe_caps;
