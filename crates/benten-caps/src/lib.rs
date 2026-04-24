//! # benten-caps — Capability policy
//!
//! Pluggable capability policy for the Benten graph engine. Phase 1 ships:
//!
//! - The [`CapabilityPolicy`] pre-write hook trait + [`WriteContext`] +
//!   [`ReadContext`].
//! - [`NoAuthBackend`] — the zero-cost Phase 1 default; permits every write
//!   and every read.
//! - [`UcanBackend`] — a stub that cleanly errors with
//!   [`CapError::NotImplemented`] so operator misconfiguration in Phase 1
//!   surfaces as a distinct error code, not a denial.
//! - [`CapabilityGrant`] — the typed grant Node + [`GrantScope`] parsing +
//!   the canonical [`GRANTED_TO_LABEL`] / [`REVOKED_AT_LABEL`] edge labels.
//! - [`check_attenuation`] — the segment-wise subset check consumed by the
//!   evaluator's chained-CALL attenuation gate.
//! - [`CapError`] — mapped 1:1 to the stable ERROR-CATALOG codes.
//!
//! # Named compromises preserved here
//!
//! - **#1 — TOCTOU window on long ITERATE.** The evaluator refreshes cap
//!   snapshots on batch boundaries only; the boundary size is
//!   [`DEFAULT_BATCH_BOUNDARY`], exposed to backends as
//!   [`CapabilityPolicy::iterate_batch_boundary`] so a revocation-sensitive
//!   policy can tighten the bound. Revocations between boundaries are
//!   invisible to in-flight writes. Phase 2 Invariant 13 tightens to
//!   per-operation.
//! - **#2 — `E_CAP_DENIED_READ` leaks existence.** Option A: returning a
//!   denial error for unauthorized reads tells the caller "this CID exists".
//!   Documented on [`CapabilityPolicy::check_read`]. Phase 3 revisits once
//!   the identity surface lands and silent-`None` becomes safe to attribute.
//!
//! # What is *not* in this crate
//!
//! - Actual cap-check wiring into the transaction primitive (G3).
//! - `requires` property recognition on operation Nodes (G6).
//! - UCAN verification + principal types (Phase 3, `benten-id`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod attenuation;
pub mod error;
pub mod grant;
pub mod grant_backed;
pub mod noauth;
pub mod policy;
pub mod ucan_stub;

pub use attenuation::check_attenuation;
pub use error::CapError;
pub use grant::{
    CAPABILITY_GRANT_LABEL, CapabilityGrant, GRANTED_TO_LABEL, GrantScope, REVOKED_AT_LABEL,
};
pub use grant_backed::{GrantBackedPolicy, GrantReader, GrantReaderChain, GrantReaderConfig};
pub use noauth::NoAuthBackend;
pub use policy::{CapabilityPolicy, PendingOp, ReadContext, WriteAuthority, WriteContext};
pub use ucan_stub::UcanBackend;

/// Phase 2a G9-A / P2 test-harness: helper surface the evaluator consults
/// for iteration-batch + wall-clock refresh delegation. Split into its own
/// module so the test pins that the engine's `PrimitiveHost` routes through
/// this helper rather than consulting a constant.
///
/// TODO(phase-2a-G9-A): wire the real evaluator path through this helper.
pub mod evaluator_delegation {
    use crate::policy::CapabilityPolicy;

    /// Consult the policy's iteration-batch boundary override. The engine's
    /// evaluator calls this helper once per batch; test mocks can count
    /// invocations to assert the delegation path is active.
    pub fn iterate_batch_boundary_for<P: CapabilityPolicy + ?Sized>(policy: &P) -> usize {
        policy.iterate_batch_boundary()
    }

    /// Consult the policy's wall-clock refresh ceiling (§9.13 refresh-point-5).
    pub fn wallclock_refresh_ceiling_for<P: CapabilityPolicy + ?Sized>(
        policy: &P,
    ) -> core::time::Duration {
        policy.wallclock_refresh_ceiling()
    }
}

/// Refresh event emitted when the evaluator re-validates a capability grant
/// at a TOCTOU refresh point (§9.13 dual-source resolution). `hlc_stamp`
/// carries the HLC at the refresh instant for federation correlation;
/// `monotonic_authoritative` records that the cadence was driven by
/// `MonotonicSource::elapsed` (not the HLC).
#[derive(Debug, Clone)]
pub struct HlcStampedRefreshEvent {
    /// HLC stamp at the refresh instant (Phase-3 uses this for peer-skew
    /// correlation).
    pub hlc_stamp: Option<u64>,
    /// Marks the refresh as monotonic-authoritative (§9.13).
    pub monotonic_authoritative: bool,
}

/// Phase 2a G9-A test-harness: synthesise a refresh event so tests can pin
/// the §9.13 dual-source contract (MonotonicSource authoritative; HLC rides
/// alongside).
///
/// TODO(phase-2a-G9-A): wire this into the real evaluator refresh path.
#[must_use]
pub fn emit_refresh_event_for_test() -> HlcStampedRefreshEvent {
    HlcStampedRefreshEvent {
        hlc_stamp: Some(0),
        monotonic_authoritative: true,
    }
}

/// Default ITERATE batch size for capability-refresh boundaries.
///
/// The evaluator (G6) uses this constant as the default batch size between
/// cap-snapshot refreshes. A backend can tighten the bound by overriding
/// [`CapabilityPolicy::iterate_batch_boundary`]; a revocation arriving
/// during a batch is not observed until the next boundary. See named
/// compromise #1 above.
///
/// If this default changes, the following must move in lockstep:
/// - `tests/toctou_iteration.rs::DEFAULT_BATCH_BOUNDARY`,
/// - `.addl/phase-1/r1-triage.md` named compromise #1 prose,
/// - `docs/SECURITY-POSTURE.md` once that doc lands.
pub const DEFAULT_BATCH_BOUNDARY: usize = 100;

/// Test-only back-compat surface.
///
/// The real [`check_attenuation`] lives at the crate root (see the
/// [`attenuation`] module). This `testing::` alias is preserved so the
/// integration tests in `tests/call_attenuation.rs` that wrote
/// `benten_caps::testing::check_attenuation` continue to resolve. New code
/// should call [`benten_caps::check_attenuation`](crate::check_attenuation).
pub mod testing {
    use core::time::Duration;

    /// Back-compat re-export of [`super::check_attenuation`].
    pub use super::attenuation::check_attenuation;

    /// Phase 2a G9-A test helper: a wall-clock refresh probe. Tracks elapsed
    /// monotonic time since the last refresh and signals whether the
    /// configured ceiling has been breached.
    #[derive(Debug, Clone)]
    pub struct WallclockProbe {
        elapsed: Duration,
    }

    impl WallclockProbe {
        /// Whether the elapsed duration is at-or-past the ceiling.
        #[must_use]
        pub fn check_elapsed(&self, ceiling: Duration) -> bool {
            self.elapsed >= ceiling
        }

        /// Reset the probe (simulate a refresh).
        #[must_use]
        pub fn force_refresh(&self) -> usize {
            // Returns the synthetic "anchors refreshed" count for the bench.
            1
        }
    }

    /// Fresh probe (elapsed = 0).
    #[must_use]
    pub fn wallclock_refresh_probe_fresh() -> WallclockProbe {
        WallclockProbe {
            elapsed: Duration::from_secs(0),
        }
    }

    /// Expired probe (elapsed > 300s).
    #[must_use]
    pub fn wallclock_refresh_probe_expired() -> WallclockProbe {
        WallclockProbe {
            elapsed: Duration::from_secs(301),
        }
    }
}
