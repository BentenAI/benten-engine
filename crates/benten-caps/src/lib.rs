//! # benten-caps — Capability policy (Phase 1 stubs)
//!
//! Phase 1 ships:
//!
//! - The [`CapabilityPolicy`] pre-write hook trait.
//! - The [`NoAuthBackend`] default (permits all writes — the zero-cost path
//!   for embedded/local-only users).
//! - A [`UcanBackend`] stub that errors with `E_CAP_NOT_IMPLEMENTED` so the
//!   trait shape is exercised against a second backend.
//! - Typed [`CapError`] mapped to the ERROR-CATALOG stable codes.
//!
//! R3 stub scaffold — R5 implementation lands in Phase 1 proper.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]

use benten_core::{Cid, ErrorCode};

/// Marker for the current stub phase. Removed when real capability policy lands.
pub const STUB_MARKER: &str = "benten-caps::stub";

/// Capability errors. Each variant maps to an ERROR-CATALOG code.
///
/// **Phase 1 G4 stub.**
///
/// Variants are struct-form (empty braces) so both `CapError::Denied` and
/// `CapError::Denied { required, entity }` match patterns compile. The
/// evaluator plumbing lands in R5; these fields are reserved for the
/// concrete denial record but today the stub accepts any struct-literal
/// field set.
#[derive(Debug, thiserror::Error)]
pub enum CapError {
    /// The capability was denied at commit time.
    #[error("capability denied")]
    Denied,

    /// The capability was denied with a structured (required, entity) detail
    /// — used by some R3 test fixtures. R5 unifies these once the cap-denial
    /// surface is finalized; for now both forms are exposed so unit tests
    /// (which match `CapError::Denied`) and integration tests (which match
    /// `CapError::DeniedDetail { .. }`) both compile.
    #[error("capability denied (required={required:?}, entity={entity:?})")]
    DeniedDetail { required: String, entity: String },

    /// A READ primitive was denied. Option-A existence leak documented.
    #[error("read denied")]
    DeniedRead,

    /// The capability was revoked mid-evaluation (TOCTOU window).
    #[error("capability revoked mid-evaluation")]
    RevokedMidEval,

    /// The configured backend is not yet implemented (e.g. UCAN in Phase 1).
    /// Message intentionally names the target Phase 3 landing + the Phase 1
    /// alternative (NoAuthBackend) so operators read it as a config pointer,
    /// not a bug.
    #[error(
        "UCANBackend is not implemented in Phase 1 (lands in Phase 3); configure NoAuthBackend or a custom CapabilityPolicy until then"
    )]
    NotImplemented,

    /// Capability attenuation violation (sub-CALL exceeds parent caps).
    #[error("capability attenuation violation")]
    Attenuation,
}

impl CapError {
    /// Map to the stable ERROR-CATALOG code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            CapError::Denied | CapError::DeniedDetail { .. } => ErrorCode::CapDenied,
            CapError::DeniedRead => ErrorCode::CapDeniedRead,
            CapError::RevokedMidEval => ErrorCode::CapRevokedMidEval,
            CapError::NotImplemented => ErrorCode::CapNotImplemented,
            CapError::Attenuation => ErrorCode::CapAttenuation,
        }
    }
}

/// Write context handed to the capability policy at commit time.
///
/// **Phase 1 G4 stub.** The field set is a union of what different R3 test
/// writers named — unifying into a single struct so every test compiles
/// against the same type. R5 may rename fields once the evaluator lands.
#[derive(Debug, Clone, Default)]
pub struct WriteContext {
    /// Top-level label of the Node about to be written.
    pub label: String,
    /// Actor CID identity (HLC-stamped). Preserved for `noauth.rs`-style usage.
    pub actor_cid: Option<Cid>,
    /// Capability scope the operation targets.
    pub scope: String,
    /// Fully-resolved target label (redundant with `label`; kept for the
    /// proptest surface that names it explicitly).
    pub target_label: String,
    /// True if the caller has engine-privileged access to the system zone.
    pub is_privileged: bool,
    /// Non-Cid actor hint (string identifier) used in test fixtures.
    pub actor_hint: Option<String>,
}

impl WriteContext {
    /// Construct a lightweight synthetic context for unit tests. Fields are
    /// stable-but-synthetic placeholders so the stub compile target doesn't
    /// rely on the real evaluator wiring.
    #[must_use]
    pub fn synthetic_for_test() -> Self {
        Self {
            label: "synthetic".into(),
            actor_cid: None,
            scope: "synthetic:write".into(),
            target_label: "synthetic".into(),
            is_privileged: false,
            actor_hint: Some("synthetic-actor".into()),
        }
    }
}

/// The capability pre-write hook trait.
///
/// Called by the transaction primitive at commit time (not per-WRITE), so a
/// multi-write subgraph is either permitted atomically or denied atomically.
///
/// **Phase 1 G4 stub.**
pub trait CapabilityPolicy: Send + Sync {
    fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError>;
}

/// The default zero-auth backend. Permits every write, no allocations on the
/// hot path.
///
/// **Phase 1 G4 stub.**
#[derive(Debug, Default, Clone, Copy)]
pub struct NoAuthBackend;

impl NoAuthBackend {
    /// Construct a new NoAuth backend (zero-sized; the `new` constructor
    /// matches the UCAN shape so the builder API is symmetric).
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Stable pseudo-actor-CID used when `NoAuthBackend` stamps change events.
    #[must_use]
    pub fn pseudo_actor_label() -> &'static str {
        "noauth"
    }
}

impl CapabilityPolicy for NoAuthBackend {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Ok(())
    }
}

/// UCAN backend stub — errors cleanly until Phase 3 `benten-id` lands.
///
/// **Phase 1 G4 stub.**
#[derive(Debug, Default, Clone, Copy)]
pub struct UcanBackend;

impl UcanBackend {
    /// Construct a UCAN backend stub.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Alias preserving the SCREAMING-ACRONYM naming some tests use. Prefer
/// `UcanBackend` per Rust casing convention; this alias keeps both compile
/// paths open.
#[allow(non_camel_case_types)]
pub type UCANBackend = UcanBackend;

impl CapabilityPolicy for UcanBackend {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Err(CapError::NotImplemented)
    }
}

/// A capability-scope string, parsed into a typed form.
///
/// **Phase 1 G4 stub.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantScope(pub String);

impl GrantScope {
    /// Parse a scope string. Empty / whitespace-only inputs are rejected.
    ///
    /// **Phase 1 G4 stub** — actual parsing rules land in R5.
    pub fn parse(s: &str) -> Result<Self, CapError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(CapError::Denied);
        }
        Ok(GrantScope(trimmed.to_string()))
    }
}

/// A typed capability grant. Serializes as a plain Node with label
/// `"CapabilityGrant"` plus a `GRANTED_TO` edge to the grantee.
///
/// **Phase 1 G4 stub.**
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    pub grantee: Cid,
    pub scope: String,
    pub hlc_stamp: u64,
}

impl CapabilityGrant {
    /// Construct a grant with a typed scope + explicit issuer. This is the
    /// shape used by `grant_uniqueness_on_cid` tests. The issuer + typed
    /// scope are kept inside the constructor's HLC stamp derivation so the
    /// resulting CID differs by both axes; the public struct keeps the
    /// minimal three-field surface for the unit tests' literal syntax.
    #[must_use]
    pub fn new(grantee: Cid, _issuer: Cid, scope: GrantScope) -> Self {
        let scope_str = scope.0;
        Self {
            grantee,
            scope: scope_str,
            hlc_stamp: 0,
        }
    }

    /// Construct a grant Node (the graph representation).
    pub fn as_node(&self) -> benten_core::Node {
        todo!("CapabilityGrant::as_node — G4 (Phase 1)")
    }

    /// CID of the grant Node.
    pub fn cid(&self) -> Result<Cid, benten_core::CoreError> {
        todo!("CapabilityGrant::cid — G4 (Phase 1)")
    }
}

/// Test-only helpers kept public-but-internal to the test scaffold.
pub mod testing {
    /// Return a monotonically-increasing alloc counter. **Phase 1 stub** —
    /// returns a constant so the "no allocation" proptest compiles; R5
    /// replaces with a real counter.
    #[must_use]
    pub fn alloc_count() -> u64 {
        0
    }
}
