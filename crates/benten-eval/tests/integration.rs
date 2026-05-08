//! Aggregator for §7.3.A.1 SANDBOX integration tests under
//! `tests/integration/` (G20-A1 wave-8a).
//!
//! Cargo does NOT auto-discover `.rs` files in subdirectories of
//! `tests/`; they need an explicit aggregator in `tests/<name>.rs`
//! that declares the submodules. Without this file, the §7.3.A.1
//! integration tests at `tests/integration/inv_4_call_boundary.rs`,
//! `tests/integration/inv_7_streaming.rs`, and
//! `tests/integration/sandbox_wasm32_disabled.rs` were silently
//! dropped from `cargo nextest` discovery (companion to the
//! `tests/security.rs` aggregator).
//!
//! Owner: G20-A1 wave-8a (un-ignores the §7.3.A.1 cluster).

#![cfg(not(target_arch = "wasm32"))]

#[path = "integration/inv_4_call_boundary.rs"]
pub mod inv_4_call_boundary;

#[path = "integration/inv_7_streaming.rs"]
pub mod inv_7_streaming;

#[path = "integration/sandbox_wasm32_disabled.rs"]
pub mod sandbox_wasm32_disabled;
