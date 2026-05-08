//! Capability backends (G14-B wave-4b).
//!
//! Phase-3 lights up the durable [`ucan::UCANBackend`] (replacing the
//! Phase-2b stub at `crate::ucan_stub::LegacyUcanStubBackend`). The submodule
//! tree exists for the backends-by-shape group of `CapabilityPolicy`
//! impls — rate-limit policies live alongside at
//! [`crate::rate_limit`] (separate trait surface so a single backend
//! can compose `CapabilityPolicy` + `RateLimitPolicy` without
//! conflating the two).
//!
//! See `crates/benten-caps/src/backends/ucan.rs` for the durable
//! UCAN backend that closes Phase-2b's `CapError::NotImplemented`
//! stub per `crypto-blocker-2` BLOCKER + CLR-2.

// `ucan` durable backend gated to non-wasm32 targets — `benten-id`
// transitively pulls `getrandom` for OS CSPRNG which rejects
// `wasm32-unknown-unknown` without the `js` feature. Per CLAUDE.md
// baked-in #17, identity + capability work is full-peer-only; the
// thin-client doesn't need this backend. Mirrors the G13-C / G14-A1
// `bindings/napi` cfg-gating pattern; `benten-id` is in
// `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` in
// `Cargo.toml`.
#[cfg(not(target_arch = "wasm32"))]
pub mod ucan;

#[cfg(not(target_arch = "wasm32"))]
pub use ucan::UCANBackend;
