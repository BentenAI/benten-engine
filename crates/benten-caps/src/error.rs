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
/// `#[non_exhaustive]` (R6b bp-17) — Phase 3 UCAN backend introduces
/// `CapError::UcanInvalidProof`, `CapError::UcanExpired`, principal-identity
/// variants; downstream matchers must include `_ =>` so adding variants is
/// a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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

    /// Phase 2a ucca-7: `GrantScope::parse("*")` refused — the lone star is
    /// a root-scope footgun. Compound `*:<ns>` remains accepted.
    #[error("grant scope lone '*' rejected — use a compound scope like '*:ns'")]
    ScopeLoneStarRejected,

    /// Phase 2a ucca-6: attenuation chain exceeds `GrantReader::max_chain_depth`.
    /// Bounds resume-time CPU cost under adversarial deep chains.
    #[error("capability chain too deep (depth {depth}, limit {limit})")]
    ChainTooDeep {
        /// Actual depth observed on the chain.
        depth: usize,
        /// Configured maximum.
        limit: usize,
    },

    /// Phase 2a G9-A / §9.13 refresh-point-5: wall-clock refresh bound
    /// breached during a long-running ITERATE / CALL.
    #[error("capability wall-clock refresh ceiling exceeded")]
    WallclockExpired,

    // -----------------------------------------------------------------
    // Phase 3 G14-B (durable UCAN backend in
    // `crates/benten-caps/src/backends/ucan.rs::UCANBackend`).
    //
    // Replaces the Phase-2b stub `CapError::NotImplemented` with typed
    // chain-walk + delegation + revocation + nbf/exp time-window
    // surfaces. Each variant maps 1:1 to an `ErrorCode` in
    // `benten-errors::ErrorCode` (see `code()` below) and to the
    // `docs/ERROR-CATALOG.md` entries
    // `E_CAP_UCAN_*` / `E_CAP_BACKEND_STORAGE` /
    // `E_CAP_RATE_LIMIT_EXCEEDED` / `E_CAP_PEER_BANDWIDTH_EXCEEDED`.
    // -----------------------------------------------------------------
    /// G14-B: presented UCAN's `exp` window has elapsed at chain-walk
    /// time. Per `crypto-blocker-2` BLOCKER + CLR-2: every link in the
    /// chain has its `exp` checked, not just the leaf.
    #[error("UCAN expired (exp={exp}, now={now})")]
    UcanExpired {
        /// The UCAN's `exp` epoch second.
        exp: u64,
        /// The chain-walk's evaluation `now` epoch second.
        now: u64,
    },

    /// G14-B: presented UCAN's `nbf` window has not yet opened at
    /// chain-walk time.
    #[error("UCAN not yet valid (nbf={nbf}, now={now})")]
    UcanNotYetValid {
        /// The UCAN's `nbf` epoch second.
        nbf: u64,
        /// The chain-walk's evaluation `now` epoch second.
        now: u64,
    },

    /// G14-B: signature failed to verify against the issuer's public
    /// key. Per `crypto-major-4`, comparison goes through
    /// `subtle::ConstantTimeEq` at the chain-walk layer.
    #[error("UCAN signature failed verification (link_index={link_index})")]
    UcanBadSignature {
        /// The 0-based link index in the chain (leaf-first ordering).
        link_index: usize,
    },

    /// G14-B: child UCAN's capability widens its parent's authority.
    /// Per `crypto-blocker-2` + UCAN attenuation contract — the
    /// durable backend rejects at chain-walk so the structural
    /// delegation invariant survives across persistence.
    #[error(
        "UCAN attenuation violated: child cap '{child_cap}' is not subsumed by parent caps (link_index={link_index})"
    )]
    UcanAttenuationViolated {
        /// The 0-based link index of the offending child.
        link_index: usize,
        /// The widening capability formatted as `resource:ability`.
        child_cap: String,
    },

    /// G14-B mini-review fix-pass: the presented UCAN's audience DID
    /// does not match the validation context's expected audience.
    /// Defends against cross-atrium replay (a UCAN issued to
    /// atrium A persisted in atrium B's durable store and replayed)
    /// per CLR-2. Pinned at the durable chain-walk seam so audit
    /// pipelines can route on cross-atrium replay independently of
    /// the generic [`CapError::Denied`] family. Mirrors
    /// [`benten_id::errors::UcanError::AudienceMismatch`].
    #[error("UCAN audience mismatch: token aud '{actual}' != expected '{expected}'")]
    UcanAudienceMismatch {
        /// The audience the validator expected (the local atrium's DID).
        expected: String,
        /// The audience the token actually names (the cross-atrium
        /// replay source).
        actual: String,
    },

    /// G14-B: durable UCAN backend failed to read or write its grant
    /// store. Surfaces a layered backend I/O failure to the policy
    /// hook caller. Distinct from [`CapError::Denied`] — the backend
    /// cannot determine permitted-or-not when its store is unreadable.
    #[error("UCAN backend storage I/O failure: {reason}")]
    BackendStorage {
        /// Human-readable reason for the storage failure (the wrapped
        /// backend error rendered through `.to_string()`).
        reason: String,
    },

    /// G14-B: rate-limit policy plug rejected a write because the
    /// per-actor writes/sec/zone bucket exceeded its budget (per D-F
    /// + D-PHASE-3-26).
    #[error("rate-limit exceeded for actor {actor} on zone {zone}")]
    RateLimitExceeded {
        /// Actor DID / hint string.
        actor: String,
        /// Zone the write targeted.
        zone: String,
    },

    /// G14-B: rate-limit policy plug rejected an inbound chunk
    /// account because the per-peer bandwidth bytes/sec budget at the
    /// Atrium boundary exceeded its limit (per D-F + D-PHASE-3-26 +
    /// D-PHASE-3-30).
    #[error("peer bandwidth budget exceeded for peer {peer} ({bytes} bytes)")]
    PeerBandwidthExceeded {
        /// Peer DID / hint string.
        peer: String,
        /// Bytes the offending account-call attempted to push.
        bytes: usize,
    },
}

impl CapError {
    /// R4 tq-7 — structured-context accessor for `ChainTooDeep`. Returns
    /// `Some((depth, limit))` for that variant and `None` for every other,
    /// so tests can assert on the diagnostic payload without matching a
    /// human-readable message substring.
    #[must_use]
    pub fn chain_depth_context(&self) -> Option<(usize, usize)> {
        match self {
            CapError::ChainTooDeep { depth, limit } => Some((*depth, *limit)),
            _ => None,
        }
    }

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
            CapError::ScopeLoneStarRejected => ErrorCode::CapScopeLoneStarRejected,
            CapError::ChainTooDeep { .. } => ErrorCode::CapChainTooDeep,
            CapError::WallclockExpired => ErrorCode::CapWallclockExpired,
            // Phase-3 G14-B durable UCAN backend variants.
            CapError::UcanExpired { .. } => ErrorCode::CapUcanExpired,
            CapError::UcanNotYetValid { .. } => ErrorCode::CapUcanNotYetValid,
            CapError::UcanBadSignature { .. } => ErrorCode::CapUcanBadSignature,
            CapError::UcanAttenuationViolated { .. } => ErrorCode::CapUcanAttenuationViolated,
            CapError::UcanAudienceMismatch { .. } => ErrorCode::CapUcanAudienceMismatch,
            CapError::BackendStorage { .. } => ErrorCode::CapBackendStorage,
            CapError::RateLimitExceeded { .. } => ErrorCode::CapRateLimitExceeded,
            CapError::PeerBandwidthExceeded { .. } => ErrorCode::CapPeerBandwidthExceeded,
        }
    }
}
