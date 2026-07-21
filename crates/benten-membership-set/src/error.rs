//! Error types for the MembershipSet keying primitive.
//!
//! The crate surfaces two error families:
//!
//! - [`MembershipSetError`] — the crate-local typed error enum for the
//!   keying-mechanism surface (construction cardinality, members-table
//!   coupling, Kind dispatch, role staleness).
//! - The stable catalog mirror lives in `benten_errors::ErrorCode`. The
//!   role-staleness rejection is the one variant that crosses the
//!   public/wire surface (it is observed at verify time across engines),
//!   so it has a first-class catalog home:
//!   [`benten_errors::ErrorCode::RoleStaleAtVerify`] →
//!   `E_ROLE_STALE_AT_VERIFY` (the §3.5g atomic mint at F4-031). The
//!   crate-local [`MembershipSetError::RoleStaleAtVerify`] carries the same
//!   canonical wire string via [`MembershipSetError::error_code`].

use benten_errors::ErrorCode;

/// The canonical wire string for the role-staleness verify rejection — the
/// §3.5g catalog mirror constant (kept in lock-step with
/// `benten_errors::ErrorCode::RoleStaleAtVerify.as_static_str()` +
/// `packages/engine/src/errors.generated.ts` + `docs/ERROR-CATALOG.md`).
pub const E_ROLE_STALE_AT_VERIFY: &str = "E_ROLE_STALE_AT_VERIFY";

/// The typed error surface for the MembershipSet keying primitive.
///
/// Every variant that crosses the public/wire surface carries a stable
/// `benten_errors::ErrorCode` via [`MembershipSetError::error_code`] so the
/// Rust ↔ TS ↔ catalog mirror stays in sync (§3.5g).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MembershipSetError {
    /// Per-Kind admin cardinality violated: Atrium needs ≥1 admin;
    /// DeviceMesh / SingleDevice need exactly-1.
    #[error("admin cardinality violated for the requested Kind")]
    AdminCardinality,

    /// SingleDevice is exactly-1 member.
    #[error("member cardinality violated (SingleDevice is exactly-1 member)")]
    MemberCardinality,

    /// A `MemberRef` variant not valid for the requested Kind (e.g. a
    /// `DeviceDid` inside an Atrium) — the Kind↔MemberRef coupling
    /// (m-15 GNC-7 / Inv-22 boundary).
    #[error("MemberRef is not valid for the requested Kind")]
    MemberRefKindMismatch,

    /// The per-Kind member-count ceiling (Compromise #46) was exceeded
    /// (Atrium 32 / DeviceMesh 5 / SingleDevice 1).
    #[error("per-Kind wire-cost ceiling (Compromise #46) exceeded")]
    WireCostCeilingExceeded,

    /// `is_authority = true` but `sig_pubkey` is absent — the fused
    /// authorities coupling rule (`is_authority ⟹ sig_pubkey.is_some()`).
    #[error("an authority member must carry a sig_pubkey")]
    AuthorityMissingPubkey,

    /// A stanza sealed under a stale `role_assignments_generation` was
    /// rejected at verify (BC-5; §3.10). Maps to the
    /// [`E_ROLE_STALE_AT_VERIFY`] catalog code.
    #[error("a stanza sealed under a stale role_assignments_generation was rejected at verify")]
    RoleStaleAtVerify,
}

impl MembershipSetError {
    /// The stable `benten_errors::ErrorCode` for the variants that cross the
    /// public/wire surface. The role-staleness rejection is the §3.5g-minted
    /// catalog code (`E_ROLE_STALE_AT_VERIFY`); the construction-time
    /// cardinality/coupling errors route through the existing capability /
    /// invariant family codes.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            MembershipSetError::RoleStaleAtVerify => ErrorCode::RoleStaleAtVerify,
            // Construction-time cardinality / coupling errors are
            // registration-time rejections; they share the generic
            // capability-denied disposition until/unless a dedicated catalog
            // code is minted for each (none is required by the R5 corpus).
            _ => ErrorCode::CapDenied,
        }
    }

    /// The canonical wire string for this error.
    #[must_use]
    pub fn as_wire_str(&self) -> &'static str {
        self.error_code().as_static_str()
    }
}
