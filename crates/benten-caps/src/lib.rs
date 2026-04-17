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

pub mod attenuation;
pub mod error;
pub mod grant;
pub mod noauth;
pub mod policy;
pub mod ucan_stub;

pub use attenuation::check_attenuation;
pub use error::CapError;
pub use grant::{
    CAPABILITY_GRANT_LABEL, CapabilityGrant, GRANTED_TO_LABEL, GrantScope, REVOKED_AT_LABEL,
};
pub use noauth::NoAuthBackend;
pub use policy::{CapabilityPolicy, ReadContext, WriteContext};
pub use ucan_stub::UcanBackend;

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

/// Legacy stub marker. Kept as a compile-time cross-crate link anchor because
/// `benten-engine` references it in a `const _:` assertion pinned at R3 — the
/// assertion exists so a fresh-eyes reviewer can tell, at a glance, which
/// crate modules are still pre-implementation. The constant's value is
/// cosmetic; only its presence (and `&str` type) are load-bearing for the
/// cross-crate assertion.
///
/// `TODO(phase-1-cleanup, G8)`: remove both this constant and the mirrored
/// reference at `benten-engine/src/lib.rs` (near the `const _: &str =
/// benten_caps::STUB_MARKER;` line) together — both sides of the cross-crate
/// assertion retire at the same commit.
pub const STUB_MARKER: &str = "benten-caps::phase-1";

/// Test-only back-compat surface.
///
/// The real [`check_attenuation`] lives at the crate root (see the
/// [`attenuation`] module). This `testing::` alias is preserved so the
/// integration tests in `tests/call_attenuation.rs` that wrote
/// `benten_caps::testing::check_attenuation` continue to resolve. New code
/// should call [`benten_caps::check_attenuation`](crate::check_attenuation).
pub mod testing {
    /// Back-compat re-export of [`super::check_attenuation`].
    pub use super::attenuation::check_attenuation;
}
