//! # benten-caps — Capability policy
//!
//! Pluggable capability policy for the Benten graph engine. Phase 1 ships:
//!
//! - The [`CapabilityPolicy`] pre-write hook trait + [`CapWriteContext`] +
//!   [`ReadContext`].
//! - [`NoAuthBackend`] — the zero-cost Phase 1 default; permits every write
//!   and every read.
//! - [`LegacyUcanStubBackend`] (renamed from `UcanBackend` at G21-T2
//!   audit-6-1 closure) — a stub that cleanly errors with
//!   [`CapError::NotImplemented`] so legacy callsites that still
//!   reach for the Phase-1 stub surface a distinct error. Production
//!   code uses `EngineBuilder::capability_policy_ucan_durable` which
//!   composes the durable [`backends::UCANBackend`] + Phase-3
//!   [`GrantBackedPolicy`].
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
pub mod backends;
pub mod error;
pub mod grant;
pub mod grant_backed;
pub mod noauth;
pub mod policy;
pub mod rate_limit;
pub mod typed_cap_mapping;
#[cfg(not(target_arch = "wasm32"))]
pub mod ucan_grounded;
pub mod ucan_stub;

// G24-D — runtime UCAN delegation gate (Layer 3 of the three-layer
// plugin trust model per CLAUDE.md baked-in #18). Native-only; thin
// clients don't run delegation checks (CLAUDE.md #17).
#[cfg(not(target_arch = "wasm32"))]
pub mod plugin_delegation;

// G27-D — manifest-aware scope derivation. Pure functions mapping
// `PluginManifest::requires` / `shares` halves to canonical cap-scope
// strings + audience-side envelope check. Native-only (depends on
// `benten-platform-foundation::PluginManifest` which is full-peer-
// only per CLAUDE.md #17). See `docs/future/phase-4-backlog.md` §4.4
// for the cap-r1-3 closure narrative.
#[cfg(not(target_arch = "wasm32"))]
pub mod manifest_scope;

// G24-D-FP-2 — Layer 2 ↔ Layer 3 chain validator. Composes the
// single-step `plugin_delegation` gate across an entire UCAN
// delegation chain + asserts the user-DID root anchor per
// CLAUDE.md #18 clause-(a). Native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod manifest_envelope_chain_validation;

// COLLAPSE P2 CONSOLIDATE — policy-bearing UCAN chain-authority
// consultation (rotation-log-as-authority + the single generalized
// envelope-ceiling seam). Moved from `benten_id::ucan` per
// impl-design-COLLAPSE.md §2 (RATIFIED DECISION-RECORD §4); the pure
// crypto/structural chain primitive stays in `benten-id`. Native-only
// (depends on the native-only `benten-id` device-attestation +
// rotation-log primitive types). COLLAPSE P5 (#669) extends
// `validate_chain_with_envelope_ceiling`'s factored
// `envelope_ceiling_rejects_cap` predicate with the plugin-manifest
// ceiling as a second caller — ONE code path (build-constraint iii).
#[cfg(not(target_arch = "wasm32"))]
pub mod chain_authority;

pub use attenuation::check_attenuation;
// COLLAPSE P2 — the single moved authority seam (native-only).
#[cfg(not(target_arch = "wasm32"))]
pub use chain_authority::{
    FrameReplayMarker, envelope_ceiling_first_rejected_resource, envelope_ceiling_rejects_cap,
    manifest_to_envelope_ceiling, validate_chain_with_envelope_ceiling,
    validate_chain_with_manifest_ceiling, validate_chain_with_rotation_log,
};
// G14-B durable UCAN backend is native-only (see `backends/mod.rs`).
#[cfg(not(target_arch = "wasm32"))]
pub use backends::UCANBackend;
pub use error::CapError;
pub use grant::{
    CAPABILITY_GRANT_LABEL, CapabilityGrant, GRANTED_TO_LABEL, GrantScope, REVOKED_AT_LABEL,
};
pub use grant_backed::{GrantBackedPolicy, GrantReader, GrantReaderChain, GrantReaderConfig};
pub use noauth::NoAuthBackend;
pub use policy::{CapWriteContext, CapabilityPolicy, PendingOp, ReadContext, WriteAuthority};
pub use rate_limit::{
    InMemoryRateLimitPolicy, InMemoryRateLimitPolicyBuilder, NullRateLimitPolicy, RateLimitPolicy,
};
pub use typed_cap_mapping::{TypedCapGroup, typed_cap_for_ucan_claim};
#[cfg(not(target_arch = "wasm32"))]
pub use ucan_grounded::UcanGroundedPolicy;
pub use ucan_stub::LegacyUcanStubBackend;

// Surf-1 #884 (v1-API-stabilization): the three plugin-trust modules
// (Layer 2 + Layer 3 of CLAUDE.md #18) now follow the same crate-root
// re-export convention as every other public composable surface above.
// Consumers write `benten_caps::check_delegation_within_envelope`
// rather than the asymmetric `benten_caps::plugin_delegation::*`
// long-path import. The module-local `Did` newtypes are intentionally
// NOT re-exported (they collide across the two modules + the canonical
// identity type is `benten_id::did::Did`); `ChainAnchor` is excluded
// per the #816 zero-consumer disposition. Native-only, mirroring the
// `#[cfg(not(target_arch = "wasm32"))]` gate on the modules themselves.
#[cfg(not(target_arch = "wasm32"))]
pub use manifest_envelope_chain_validation::{
    ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
    validate_chain_with_manifest_envelope,
};
#[cfg(not(target_arch = "wasm32"))]
pub use manifest_scope::{
    PRIVATE_PREFIX, REQUIRES_PREFIX, SHARES_PREFIX, check_scope_within_envelope,
    manifest_requires_to_scope, manifest_shares_to_scope, private_namespace_scope_admits_actor,
};
#[cfg(not(target_arch = "wasm32"))]
pub use plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
    is_private_namespace_cap,
};

/// Phase 2a G9-A / P2 test-harness: helper surface the evaluator consults
/// for iteration-batch + wall-clock refresh delegation. Split into its own
/// module so the test pins that the engine's `PrimitiveHost` routes through
/// this helper rather than consulting a constant.
///
/// TODO(phase-3 — evaluator-delegation wire-up): wire the real
/// evaluator path through this helper. Carried from Phase-2a G9-A;
/// pairs with §2.1 Durable UCAN backend.
pub mod evaluator_delegation {
    use crate::policy::CapabilityPolicy;

    /// Consult the policy's iteration-batch boundary override. The engine's
    /// evaluator calls this helper once per batch; test mocks can count
    /// invocations to assert the delegation path is active.
    ///
    /// Qual-1 #674 (umbrella #1154): DISAGREE-WITH-EVIDENCE on the
    /// "delete zero-value wrapper" recommendation for THIS function —
    /// it is a LIVE production accessor consumed by
    /// `benten_engine::primitive_host` (dep-direction-safe seam: the
    /// engine cannot name `CapabilityPolicy::iterate_batch_boundary`
    /// across the trait-object boundary as ergonomically). The sibling
    /// `wallclock_refresh_ceiling_for` WAS a true zero-consumer wrapper
    /// and is deleted per #674.
    pub fn iterate_batch_boundary_for<P: CapabilityPolicy + ?Sized>(policy: &P) -> usize {
        policy.iterate_batch_boundary()
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
/// TODO(phase-3 — emit_refresh_event_for_test → real evaluator
/// refresh-path wire-up): wire this synthesis path into the real
/// evaluator refresh-event emission. Carried from Phase-2a G9-A;
/// pairs with §2.1 Durable UCAN backend.
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
/// - `crates/benten-caps/tests/toctou_iteration.rs::DEFAULT_BATCH_BOUNDARY`,
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
