//! [`LegacyUcanStubBackend`] — Phase 1 stub.
//!
//! Renamed from `UcanBackend` at G21-T2 (audit-6-1 closure) to
//! eliminate the import-order ambiguity that allowed `PolicyKind::Ucan`
//! to silently resolve to the stub even after the durable
//! [`crate::backends::UCANBackend`] shipped at G14-B wave-4b. The
//! stub is preserved for the small number of legacy tests that still
//! pin the `CapError::NotImplemented` routing contract; new code MUST
//! NOT use this type — the durable [`crate::backends::UCANBackend`]
//! is the production UCAN proof-chain validator + the durable
//! [`crate::GrantBackedPolicy`] is the production capability policy
//! hook (composed by `EngineBuilder::capability_policy_ucan_durable`).
//!
//! The error-routing contract (must surface as `ON_ERROR`, not
//! `ON_DENIED`) is tested in `tests/ucan_stub_messages.rs` — the
//! evaluator (G6) honors it.

use crate::error::CapError;
use crate::policy::{CapabilityPolicy, WriteContext};

/// Phase-1 UCAN capability backend stub. Every `check_write` call
/// returns [`CapError::NotImplemented`] with `backend = "UCANBackend"`
/// and `lands_in_phase = 3` so the displayed message names both the
/// backend and the scheduled landing phase.
///
/// Renamed from `UcanBackend` at G21-T2 to disambiguate from the
/// durable [`crate::backends::UCANBackend`]. The napi `PolicyKind::Ucan`
/// arm no longer routes through this type — it composes the durable
/// grant-backed policy via
/// `EngineBuilder::capability_policy_ucan_durable`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LegacyUcanStubBackend;

impl LegacyUcanStubBackend {
    /// Construct a Phase-1 UCAN backend stub. Reserved for the legacy
    /// tests that still pin the `CapError::NotImplemented` routing
    /// contract; production callers should use
    /// `EngineBuilder::capability_policy_ucan_durable` (G21-T2 closure
    /// of audit-6-1).
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl CapabilityPolicy for LegacyUcanStubBackend {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Err(CapError::NotImplemented {
            backend: "UCANBackend",
            lands_in_phase: 3,
        })
    }
}
