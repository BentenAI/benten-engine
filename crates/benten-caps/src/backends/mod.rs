//! Capability backends (G14-B wave-4b).
//!
//! Phase-3 lights up the durable [`ucan::UCANBackend`] (replacing the
//! Phase-2b stub at `crate::ucan_stub::UcanBackend`). The submodule
//! tree exists for the backends-by-shape group of `CapabilityPolicy`
//! impls — rate-limit policies live alongside at
//! [`crate::rate_limit`] (separate trait surface so a single backend
//! can compose `CapabilityPolicy` + `RateLimitPolicy` without
//! conflating the two).
//!
//! See `crates/benten-caps/src/backends/ucan.rs` for the durable
//! UCAN backend that closes Phase-2b's `CapError::NotImplemented`
//! stub per `crypto-blocker-2` BLOCKER + CLR-2.

pub mod ucan;

pub use ucan::UCANBackend;
