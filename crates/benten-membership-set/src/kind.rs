//! The EXACTLY-3 [`MembershipSetKind`] enum + its codepoint family.
//!
//! `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` is the SCALE /
//! keying axis (R0 §1.3.A / §3.6.A). It is **EXACTLY 3** ordinal-stable
//! variants — a 4th arm is a *missing-pattern compile error*, NOT a
//! `#[non_exhaustive]` wildcard (§15.c HALT-AND-SURFACE). Garden/Grove are
//! NOT Kinds; they are `GovernanceConfig` tiers (graph Nodes, F-GOV-1).
//!
//! The two keying-reserves (`AtriumWithRotatingGroupKey` / `EphemeralLobby`)
//! are modeled on a SEPARATE axis ([`RequestedReserveKind`]) so the real Kind
//! enum stays EXACTLY-3; selecting a reserve at v1-beta is a typed-reject
//! ([`KindDispatchError::ReserveTypedReject`]).

use crate::codepoints::MEMBERSHIP_SET_ENCRYPTION;

/// The MembershipSet keying/scale Kind — **EXACTLY 3** ordinal-stable
/// variants (the discriminants are wire-keying-load-bearing).
///
/// There is deliberately **no** `#[non_exhaustive]`: every `match` over a
/// `MembershipSetKind` must be exhaustive at compile time, so adding a 4th
/// arm is a compile error at every dispatch site (the §15.c HALT-AND-SURFACE
/// guarantee). Garden/Grove live on the orthogonal GOVERNANCE axis, never as
/// Kinds.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum MembershipSetKind {
    /// A community of user-DIDs (≥1 admin).
    Atrium = 0,
    /// A user's own device set (exactly-1 user-DID admin).
    DeviceMesh = 1,
    /// A single local device (exactly-1 self-admin member).
    SingleDevice = 2,
}

impl MembershipSetKind {
    /// The number of active `MembershipSetKind` variants — **EXACTLY 3**.
    /// A 4th arm is a §15.c HALT-AND-SURFACE compile error, not a
    /// `#[non_exhaustive]` wildcard.
    pub const VARIANT_COUNT: usize = 3;

    /// All 3 Kinds in ordinal order.
    pub const ALL: [MembershipSetKind; 3] = [
        MembershipSetKind::Atrium,
        MembershipSetKind::DeviceMesh,
        MembershipSetKind::SingleDevice,
    ];

    /// The frozen codepoint a Kind binds to (the keying axis). All 3 Kinds
    /// bind to the single `0x6600` MembershipSet set-keying codepoint band.
    ///
    /// The body is an **exhaustive match with no wildcard** — adding a 4th
    /// `MembershipSetKind` variant breaks THIS compile (the HALT-AND-SURFACE
    /// guarantee).
    #[must_use]
    pub fn codepoint(self) -> u16 {
        match self {
            MembershipSetKind::Atrium => MEMBERSHIP_SET_ENCRYPTION,
            MembershipSetKind::DeviceMesh => MEMBERSHIP_SET_ENCRYPTION,
            MembershipSetKind::SingleDevice => MEMBERSHIP_SET_ENCRYPTION,
            // NO wildcard arm. A new Kind = a compile error here.
        }
    }

    /// The stable ordinal byte (the discriminant) — wire-keying-load-bearing.
    #[must_use]
    pub fn ordinal(self) -> u8 {
        self as u8
    }
}

/// The two keying-reserves the R0 §3.6.A names. They are NOT Kinds — they are
/// a *selector* the dispatch rejects at v1-beta (typed-reject, never a silent
/// 4th Kind). Modeled as a separate "requested reserve" axis so the real
/// [`MembershipSetKind`] enum stays EXACTLY-3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequestedReserveKind {
    /// Reserved: an Atrium with a rotating group key (v1-GM codepoint-reserve).
    AtriumWithRotatingGroupKey,
    /// Reserved: an ephemeral pre-acceptance lobby (v1-GM codepoint-reserve).
    EphemeralLobby,
}

/// The typed error for the Kind-dispatch surface.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future dispatch-failure mode
/// lands ADDITIVELY without a breaking SemVer bump. Mirrors its 5 sibling
/// error enums (`MembershipSetError`, `AuditChainError`, `AcquisitionError`,
/// `FederationError`, `KvError`), which already carry the attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum KindDispatchError {
    /// A reserved keying-Kind was selected at v1-beta (reserved-not-selectable).
    #[error("a reserved keying-Kind is not selectable at v1-beta")]
    ReserveTypedReject,
}

/// Dispatch a requested reserve-Kind: at v1-beta either reserve is a
/// typed-reject (never silently promoted to a real Kind).
///
/// The body is an exhaustive no-wildcard match — adding a reserve variant
/// breaks this compile.
///
/// # Errors
///
/// Always returns [`KindDispatchError::ReserveTypedReject`] at v1-beta — the
/// reserves are reserved-not-selectable.
pub fn dispatch_reserve(r: RequestedReserveKind) -> Result<MembershipSetKind, KindDispatchError> {
    match r {
        RequestedReserveKind::AtriumWithRotatingGroupKey | RequestedReserveKind::EphemeralLobby => {
            Err(KindDispatchError::ReserveTypedReject)
        }
    }
}
