//! Shared test helpers for the `benten-engine` integration test suite.
//!
//! This module is referenced from individual test binaries via `mod common;`
//! at the top of the test file. Cargo does NOT auto-discover files inside
//! `tests/<subdir>/` as integration test binaries (unlike top-level
//! `tests/*.rs`), so this layout cleanly partitions shared helpers from
//! the test entry-points.
//!
//! ## Submodules
//!
//! - [`ucan_fixtures`] — `Ucan::builder` + `Keypair::generate` +
//!   `audience_did` composition helpers used by the
//!   `ucan_validate_chain_returns_*` end-to-end pins (DX-3 closure
//!   per `docs/future/phase-3-backlog.md §2.5(f)`).

#![allow(dead_code)]

pub mod ucan_fixtures;
