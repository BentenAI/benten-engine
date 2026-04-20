//! Capability errors + stable `ErrorCode` mapping.
//!
//! Each variant maps 1:1 to an entry in `docs/ERROR-CATALOG.md`. The denial
//! surface is a single [`CapError::Denied`] struct variant carrying
//! `required` and `entity` strings (both allowed to be empty for test
//! fixtures that don't need the structured payload) — the earlier dual
//! `Denied` / `DeniedDetail` split was a hazard for audit pipelines that
//! only matched the structured arm.
//!
//! See `docs/ENGINE-SPEC.md` §9 for the capability posture and the R1
//! triage for the TOCTOU window named compromise.

use benten_errors::ErrorCode;

/// Typed capability errors.
///
/// Every variant carries a stable [`ErrorCode`] via [`CapError::code`]. The
/// evaluator pipes these through `ON_ERROR` typed edges; the `NotImplemented`
/// branch in particular must NOT route to `ON_DENIED` — see
/// `tests/ucan_stub_messages.rs` for the routing contract.
#[derive(Debug, thiserror::Error)]
pub enum CapError {
    /// The capability was denied.
    ///
    /// Carries a `(required, entity)` pair for audit pipelines; both may be
    /// empty strings when the refusal is at construction time (e.g.
    /// [`crate::GrantScope::parse`] on empty input) and no structured
    /// payload is available. An empty pair is still a denial — the
    /// `ErrorCode` is `E_CAP_DENIED` regardless.
    ///
    /// The Display format uses `{required}` / `{entity}` (not `{:?}`) —
    /// `:?` on `String` wraps the payload in escaped quotes that travel
    /// through to the JS `Error.message` (r6-err-6).
    #[error("capability denied (required={required}, entity={entity})")]
    Denied {
        /// The capability scope the write required (e.g. `"store:post:write"`).
        required: String,
        /// The entity / actor identifier the write targeted.
        entity: String,
    },

    /// A READ primitive was denied. Phase 1 named compromise #2 (Option A):
    /// returning this error leaks "this CID exists" to an unauthorized reader.
    /// Phase 3 revisits once the identity / principal surface lands.
    ///
    /// Carries the same `(required, entity)` shape as [`CapError::Denied`]
    /// so audit pipelines can log the denied scope + actor on a read the
    /// same way they do on a write (r6-err-9). Both fields may be empty
    /// when the policy's denial path does not have structured context
    /// available — the `E_CAP_DENIED_READ` code is unchanged in that case.
    #[error("read denied (required={required}, entity={entity})")]
    DeniedRead {
        /// The capability scope the read required (e.g. `"store:post:read"`).
        required: String,
        /// The entity / actor identifier the read targeted.
        entity: String,
    },

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
    /// Phase 3). The message intentionally names the backend AND the phase
    /// AND the interim alternative so operators read it as a config pointer,
    /// not a bug.
    #[error(
        "{backend} is not implemented in Phase 1 (lands in Phase {lands_in_phase}); configure NoAuthBackend or a custom CapabilityPolicy until then"
    )]
    NotImplemented {
        /// The backend name the operator configured (e.g. `"UCANBackend"`).
        backend: &'static str,
        /// The phase in which this backend's full implementation lands.
        lands_in_phase: u8,
    },

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
            CapError::Denied { .. } => ErrorCode::CapDenied,
            CapError::DeniedRead { .. } => ErrorCode::CapDeniedRead,
            CapError::Revoked => ErrorCode::CapRevoked,
            CapError::RevokedMidEval => ErrorCode::CapRevokedMidEval,
            CapError::NotImplemented { .. } => ErrorCode::CapNotImplemented,
            CapError::Attenuation => ErrorCode::CapAttenuation,
        }
    }
}
