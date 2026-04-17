//! Capability errors + stable `ErrorCode` mapping.
//!
//! Each variant maps 1:1 to an entry in `docs/ERROR-CATALOG.md`. Two variants
//! exist for the "denied" case — [`CapError::Denied`] (the bare form used by
//! unit-test fixtures) and [`CapError::DeniedDetail`] (the structured form the
//! evaluator emits at commit time). Both map to `E_CAP_DENIED`; the variant
//! split lets integration tests match on the structured payload without
//! forcing every call site to populate `required` / `entity` strings.
//!
//! See `docs/ENGINE-SPEC.md` §9 for the capability posture and the R1
//! triage for the TOCTOU window named compromise.

use benten_core::ErrorCode;

/// Typed capability errors.
///
/// Every variant carries a stable [`ErrorCode`] via [`CapError::code`]. The
/// evaluator pipes these through `ON_ERROR` typed edges; the `NotImplemented`
/// branch in particular must NOT route to `ON_DENIED` — see
/// `tests/ucan_stub_messages.rs` for the routing contract.
#[derive(Debug, thiserror::Error)]
pub enum CapError {
    /// The capability was denied at commit time. Bare form used by tests
    /// that don't care about the structured payload.
    #[error("capability denied")]
    Denied,

    /// The capability was denied at commit time with a structured
    /// `(required, entity)` pair. Emitted by the evaluator's commit path; the
    /// pair is preserved so observability surfaces can render the operator-
    /// actionable message without re-deriving it from trace context.
    #[error("capability denied (required={required:?}, entity={entity:?})")]
    DeniedDetail {
        /// The capability scope the write required (e.g. `"store:post:write"`).
        required: String,
        /// The entity / actor identifier the write targeted.
        entity: String,
    },

    /// A READ primitive was denied. Phase 1 named compromise #2 (Option A):
    /// returning this error leaks "this CID exists" to an unauthorized reader.
    /// Phase 3 revisits once the identity / principal surface lands.
    #[error("read denied")]
    DeniedRead,

    /// Capability was revoked. Phase 3 sync-revocation code — distinct from
    /// [`CapError::RevokedMidEval`] so evaluator-visible revocation during a
    /// long-running ITERATE can be told apart from a cross-peer revocation
    /// arriving over the sync protocol.
    #[error("capability revoked")]
    Revoked,

    /// Capability was revoked mid-evaluation. Phase 1 TOCTOU window named
    /// compromise #1: the evaluator snapshots caps at batch boundaries (every
    /// [`crate::DEFAULT_BATCH_BOUNDARY`] iterations), so a revocation between
    /// boundaries is invisible to in-flight writes; writes in the next batch
    /// see this error.
    #[error("capability revoked mid-evaluation")]
    RevokedMidEval,

    /// The configured capability backend is not implemented. Emitted by
    /// [`crate::UcanBackend`] in Phase 1 (UCAN lands with `benten-id` in
    /// Phase 3). The message intentionally names the phase AND the interim
    /// alternative so operators read it as a config pointer, not a bug.
    #[error(
        "UCANBackend is not implemented in Phase 1 (lands in Phase 3); configure NoAuthBackend or a custom CapabilityPolicy until then"
    )]
    NotImplemented,

    /// Capability attenuation violation on chained CALL. Child's required
    /// capability is not a subset of the parent's held capability.
    #[error("capability attenuation violation")]
    Attenuation,
}

impl CapError {
    /// Map to the stable ERROR-CATALOG code. Kept as an associated `fn` (not
    /// a `From` impl) so the mapping is introspectable from diagnostics
    /// without pulling in the `CapError` value.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            CapError::Denied | CapError::DeniedDetail { .. } => ErrorCode::CapDenied,
            CapError::DeniedRead => ErrorCode::CapDeniedRead,
            CapError::Revoked => ErrorCode::CapRevoked,
            CapError::RevokedMidEval => ErrorCode::CapRevokedMidEval,
            CapError::NotImplemented => ErrorCode::CapNotImplemented,
            CapError::Attenuation => ErrorCode::CapAttenuation,
        }
    }
}
