//! SANDBOX memory ResourceLimiter (Phase 2b Wave-8b).
//!
//! D21 priority 1 enforcement — wasmtime [`ResourceLimiter`] implementation
//! that bounds linear-memory growth per-call. Attached to the per-call
//! [`wasmtime::Store`]; the wasmtime runtime consults this limiter on
//! every `memory.grow` request inside the guest.
//!
//! Wave-8b owns the wire-through: the limiter rejects any growth that
//! would exceed `SandboxConfig::memory_bytes`. Rejection surfaces as a
//! wasmtime trap (or, on some host paths, the limiter returning
//! `Ok(false)` causes wasmtime to trap with the equivalent out-of-memory
//! shape). Either way the executor maps to `SandboxError::MemoryExhausted`
//! via [`super::trap_to_typed`].
//!
//! ESC-2 (linmem grow to limit) defense lands here: the fixture loops
//! `memory.grow(1)` until the cap is reached; this limiter caps the
//! cumulative byte count and forces the trap deterministically before
//! host OOM.
//!
//! This module is `#[cfg(not(target_arch = "wasm32"))]`-gated per
//! sec-pre-r1-05; the wasm32 build cuts SANDBOX entirely.

#![cfg(not(target_arch = "wasm32"))]

use wasmtime::ResourceLimiter;

/// Marker error the [`SandboxResourceLimiter`] raises when a memory-grow
/// request exceeds the per-call cap. Recognised by
/// [`super::trap_to_typed::map_call_error`] and routed to
/// `SandboxError::MemoryExhausted`.
#[derive(Debug, thiserror::Error)]
#[error(
    "SANDBOX memory cap exceeded: requested {requested_bytes} bytes, limit {limit_bytes} bytes"
)]
pub struct MemoryCapExceededMarker {
    /// Configured per-call cap.
    pub limit_bytes: u64,
    /// Bytes the guest tried to grow to.
    pub requested_bytes: u64,
}

/// Per-call wasmtime [`ResourceLimiter`] bounding linear-memory growth.
///
/// Constructed once per primitive call by the SANDBOX executor and
/// attached to the [`wasmtime::Store`] via `Store::limiter`. The
/// `current_size` field tracks the largest memory size accepted so far;
/// `limit_bytes` is the configured hard cap. A `memory_growing(...)`
/// callback returning `Ok(false)` causes wasmtime to deny the growth +
/// trap inside the guest.
///
/// Table growth is also gated through [`Self::table_growing`] but with a
/// permissive default (no per-call table-size cap exposed in 2b — added
/// when a future fixture exercises the surface).
#[derive(Debug)]
pub struct SandboxResourceLimiter {
    /// Configured per-call hard cap on linear-memory bytes.
    limit_bytes: u64,
    /// Largest accepted memory size observed so far (high-water mark).
    /// Diagnostic; not used to reject (the desired check uses `desired`).
    high_water_bytes: u64,
}

impl SandboxResourceLimiter {
    /// Construct a fresh limiter for one primitive call.
    #[must_use]
    pub fn new(limit_bytes: u64) -> Self {
        Self {
            limit_bytes,
            high_water_bytes: 0,
        }
    }

    /// Configured byte cap.
    #[must_use]
    pub fn limit_bytes(&self) -> u64 {
        self.limit_bytes
    }

    /// High-water mark of largest memory observed so far.
    #[must_use]
    pub fn high_water_bytes(&self) -> u64 {
        self.high_water_bytes
    }
}

impl ResourceLimiter for SandboxResourceLimiter {
    /// Called by wasmtime BEFORE any `memory.grow` request. Returning
    /// `Ok(false)` denies the request; wasmtime then returns -1 from
    /// `memory.grow` (per the WASM semantics). For Wave-8b's
    /// E_SANDBOX_MEMORY_EXHAUSTED routing we WANT cap-breach to surface
    /// as a typed error (not as a -1 return that the module could
    /// silently absorb). Returning `Err(MemoryCapExceededMarker)` on
    /// cap-breach causes wasmtime to abort the call with that error
    /// preserved through the `Instance::call` path; the executor's
    /// `trap_to_typed::map_call_error` recognises the marker and routes
    /// to `SandboxError::MemoryExhausted`.
    ///
    /// The check is on the desired POST-growth size (`desired`), bounded
    /// against [`Self::limit_bytes`].
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let desired_u64 = u64::try_from(desired).unwrap_or(u64::MAX);
        if desired_u64 > self.limit_bytes {
            return Err(wasmtime::Error::from(MemoryCapExceededMarker {
                limit_bytes: self.limit_bytes,
                requested_bytes: desired_u64,
            }));
        }
        if desired_u64 > self.high_water_bytes {
            self.high_water_bytes = desired_u64;
        }
        Ok(true)
    }

    /// Permissive default for table growth. Phase-2b doesn't ship a
    /// per-call table-size cap; if a future fixture exercises the
    /// surface, gate here.
    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limiter_accepts_growth_within_cap() {
        let mut lim = SandboxResourceLimiter::new(1024 * 1024);
        let ok = lim.memory_growing(0, 64 * 1024, None).unwrap();
        assert!(ok);
        assert_eq!(lim.high_water_bytes(), 64 * 1024);
    }

    #[test]
    fn limiter_rejects_growth_above_cap_with_typed_marker() {
        let mut lim = SandboxResourceLimiter::new(64 * 1024);
        let res = lim.memory_growing(0, 128 * 1024, None);
        let err = res.expect_err("growth above cap MUST raise typed marker");
        assert!(
            err.downcast_ref::<MemoryCapExceededMarker>().is_some(),
            "marker MUST be discoverable for trap mapping"
        );
    }

    #[test]
    fn limiter_high_water_tracks_max_accepted() {
        let mut lim = SandboxResourceLimiter::new(1024 * 1024);
        let _ = lim.memory_growing(0, 32 * 1024, None).unwrap();
        let _ = lim.memory_growing(0, 128 * 1024, None).unwrap();
        let _ = lim.memory_growing(0, 64 * 1024, None).unwrap();
        assert_eq!(lim.high_water_bytes(), 128 * 1024);
    }

    #[test]
    fn table_growing_permissive_default() {
        let mut lim = SandboxResourceLimiter::new(1024);
        assert!(lim.table_growing(0, 100_000, None).unwrap());
    }
}
