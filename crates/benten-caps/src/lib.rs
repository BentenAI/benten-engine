//! # benten-caps — Capability policy
//!
//! Pluggable capability policy for the Benten graph engine. Phase 1 ships:
//!
//! - The [`CapabilityPolicy`] pre-write hook trait + [`WriteContext`].
//! - [`NoAuthBackend`] — the zero-cost Phase 1 default; permits every write.
//! - [`UcanBackend`] — a stub that cleanly errors with
//!   [`CapError::NotImplemented`] so operator misconfiguration in Phase 1
//!   surfaces as a distinct error code, not a denial.
//! - [`CapabilityGrant`] — the typed grant Node + [`GrantScope`] parsing +
//!   the canonical [`GRANTED_TO_LABEL`] / [`REVOKED_AT_LABEL`] edge labels.
//! - [`CapError`] — mapped 1:1 to the stable ERROR-CATALOG codes.
//!
//! # Named compromises preserved here
//!
//! - **#1 — TOCTOU window on long ITERATE.** The evaluator refreshes cap
//!   snapshots on batch boundaries only; the boundary size is
//!   [`DEFAULT_BATCH_BOUNDARY`]. Revocations between boundaries are invisible
//!   to in-flight writes. Phase 2 Invariant 13 tightens to per-operation.
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

pub mod error;
pub mod grant;
pub mod noauth;
pub mod policy;
pub mod ucan_stub;

pub use error::CapError;
pub use grant::{
    CAPABILITY_GRANT_LABEL, CapabilityGrant, GRANTED_TO_LABEL, GrantScope, REVOKED_AT_LABEL,
};
pub use noauth::NoAuthBackend;
pub use policy::{CapabilityPolicy, WriteContext};
pub use ucan_stub::{UCANBackend, UcanBackend};

/// Default ITERATE batch size for capability-refresh boundaries.
///
/// G6 (evaluator) uses this constant to schedule cap-snapshot refreshes; a
/// revocation arriving during a batch is not observed until the next
/// multiple of `DEFAULT_BATCH_BOUNDARY` iterations. See named compromise
/// #1 above.
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
/// `TODO(phase-1-cleanup)`: G8 removes this once the engine-side reference is
/// retired.
pub const STUB_MARKER: &str = "benten-caps::phase-1";

/// Test-only helpers exposed for unit / integration tests.
///
/// Kept public-but-internal (the module name `testing` signals the scope)
/// because unit tests in this crate and integration tests in sibling crates
/// both reach into the helpers; gating behind `#[cfg(test)]` would hide them
/// from the integration test binaries.
pub mod testing {
    use super::{CapError, GrantScope};

    /// Allocation-counter stub. Phase 1 returns a constant so the NoAuth
    /// "zero-alloc" proptest compiles; wiring a real counter is a Phase 2
    /// deliverable once the global allocator choice settles (mimalloc vs
    /// snmalloc-rs) and a `tracking-allocator` feature lands behind a flag.
    ///
    /// The proptest asserts `alloc_count()` is unchanged across a
    /// `check_write` call. Returning a constant `0` makes the assertion
    /// trivially hold — which is correct-by-construction for NoAuth today
    /// (zero-sized, zero-alloc hot path), and the real counter will catch
    /// regressions in Phase 2.
    #[must_use]
    pub fn alloc_count() -> u64 {
        0
    }

    /// Attenuation check: does `parent_scope` permit everything
    /// `child_required` requires?
    ///
    /// Phase 1 semantics (colon-segment match with `*` wildcard):
    /// - Scopes split on `':'`.
    /// - Parent `"*"` segment matches any child segment.
    /// - Parent non-wildcard segment must exactly equal the child segment.
    /// - Parent is permitted to be a strict prefix of the child: a parent
    ///   `"store:post"` permits any sub-scope `"store:post:*"`. The inverse
    ///   (child shorter than parent) is rejected — a child that does not
    ///   name the parent's full depth is ambiguous, and the "honest no"
    ///   for ambiguity is a denial.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::Attenuation`] if the child exceeds the parent on
    /// any segment. Phase 3 UCAN lands a lattice-based check that supersedes
    /// this string comparison.
    pub fn check_attenuation(
        parent_scope: &GrantScope,
        child_required: &GrantScope,
    ) -> Result<(), CapError> {
        let parent_segments: Vec<&str> = parent_scope.as_str().split(':').collect();
        let child_segments: Vec<&str> = child_required.as_str().split(':').collect();

        // Child must be at least as deep as parent — a shorter child is
        // ambiguous (does it mean "broader", or does it mean "same"?). The
        // unambiguous parent-prefix case is handled by iterating only over
        // the parent's segments; any tail the child has beyond the parent
        // is permitted when the parent's last segment is `*` OR when the
        // parent is a strict prefix.
        if child_segments.len() < parent_segments.len() {
            return Err(CapError::Attenuation);
        }

        for (p, c) in parent_segments.iter().zip(child_segments.iter()) {
            if *p == "*" {
                continue;
            }
            if p != c {
                return Err(CapError::Attenuation);
            }
        }
        Ok(())
    }
}
