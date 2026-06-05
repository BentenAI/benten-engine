//! `MembershipSetKind` — the EXACTLY-3 keying-axis enum + its codepoint family.
//!
//! The keying axis of a MembershipSet is `{ Atrium, DeviceMesh, SingleDevice }`
//! — **EXACTLY 3**, ordinal-stable, NO `#[non_exhaustive]` wildcard (a 4th arm
//! is a §15.c missing-pattern compile error at every exhaustive `match` site,
//! the HALT-AND-SURFACE guarantee). Garden/Grove are **NOT** Kinds — they live
//! on the orthogonal GOVERNANCE axis (`GovernanceConfig` tiers; graph Nodes).
//!
//! # Codepoint family (the mechanism-half's frozen wire band)
//!
//! - `0x6600` `MEMBERSHIP_SET_ENCRYPTION` — the set-keying envelope codepoint
//!   (every Kind binds to it; Kind→codepoint is the keying axis).
//! - `0x6610` `MEMBERSHIP_SET_GROUP_MULTI_STANZA` — the group multi-stanza
//!   per-stanza AAD codepoint (`aad::assemble_group_aad`).
//! - `0x6620` reserved (future-additive).
//!
//! # Keying reserves (typed-reject at v1-beta)
//!
//! `AtriumWithRotatingGroupKey` / `EphemeralLobby` are reserved keying
//! selectors — NEVER promoted to a real Kind; selecting one at v1-beta is a
//! [`KindDispatchError::ReserveTypedReject`] (the #5 typed-reject / fail-closed
//! discipline — never a silent 4th Kind).

/// The MembershipSet **set-keying** envelope codepoint (`0x6600`).
///
/// Every [`MembershipSetKind`] binds to this codepoint (the keying axis). The
/// group multi-stanza per-stanza AAD uses the distinct `0x6610`
/// [`MEMBERSHIP_SET_GROUP_MULTI_STANZA`] codepoint.
pub const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;

/// The MembershipSet **group multi-stanza** per-stanza AAD codepoint
/// (`0x6610`). Bound BIG-ENDIAN into the group-AAD by
/// [`crate::aad::assemble_group_aad`].
pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;

/// The MembershipSet `0x6620`-band reserve (future-additive; typed-reject at
/// v1-beta via the crypto-suite codepoint dispatch — surfaced here only so the
/// band's upper-reserve is a named constant).
pub const MEMBERSHIP_SET_BAND_RESERVE: u16 = 0x6620;

/// The MembershipSet codepoint band lower bound (`0x6600`).
pub const MEMBERSHIP_SET_BAND_LO: u16 = 0x6600;

/// The MembershipSet codepoint band upper bound (`0x66FF`).
pub const MEMBERSHIP_SET_BAND_HI: u16 = 0x66FF;

/// The number of [`MembershipSetKind`] variants — **EXACTLY 3** (the keying
/// axis). A 4th arm is a §15.c HALT-AND-SURFACE compile error, NOT a
/// `#[non_exhaustive]` wildcard.
pub const KIND_VARIANT_COUNT: usize = 3;

/// The keying axis of a MembershipSet — **EXACTLY 3** ordinal-stable variants.
///
/// The discriminants are wire-keying-load-bearing (`#[repr(u8)]`, fixed). NO
/// `#[non_exhaustive]`: a 4th variant must break every exhaustive `match` site
/// (the HALT-AND-SURFACE guarantee), never be silently tolerated by a wildcard.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MembershipSetKind {
    /// A community / shared space — members are user-DIDs.
    Atrium = 0,
    /// A user's device mesh — members are device-DIDs (one user, N devices).
    DeviceMesh = 1,
    /// A lone device — the sole member is the local device.
    SingleDevice = 2,
}

impl MembershipSetKind {
    /// Map this Kind to its frozen set-keying codepoint (`0x6600`).
    ///
    /// The body is an **exhaustive match with no wildcard** — adding a 4th
    /// `MembershipSetKind` variant breaks THIS compile (the HALT-AND-SURFACE
    /// guarantee). All three Kinds bind to `0x6600` (Kind→codepoint is the
    /// keying axis; the Kind itself is carried separately in the set state).
    #[must_use]
    pub const fn codepoint(self) -> u16 {
        match self {
            MembershipSetKind::Atrium
            | MembershipSetKind::DeviceMesh
            | MembershipSetKind::SingleDevice => MEMBERSHIP_SET_ENCRYPTION,
            // NO wildcard arm. A new Kind = a compile error here.
        }
    }

    /// The stable wire ordinal (`#[repr(u8)]` discriminant) — keying-bound.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    /// All three Kinds, in ordinal order. The cardinality of this slice IS the
    /// EXACTLY-3 guarantee at the value level.
    #[must_use]
    pub const fn all() -> [MembershipSetKind; KIND_VARIANT_COUNT] {
        [
            MembershipSetKind::Atrium,
            MembershipSetKind::DeviceMesh,
            MembershipSetKind::SingleDevice,
        ]
    }
}

/// A reserved keying selector the dispatch typed-rejects at v1-beta. These are
/// NOT [`MembershipSetKind`] variants — they are a separate "requested reserve"
/// axis so the real Kind enum stays EXACTLY-3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequestedReserveKind {
    /// Atrium with a rotating group key (reserved; v1-GM additive).
    AtriumWithRotatingGroupKey,
    /// An ephemeral lobby (reserved; v1-GM additive).
    EphemeralLobby,
}

/// A Kind-dispatch error (the #5 typed-reject / fail-closed arm).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KindDispatchError {
    /// A reserved keying-Kind was selected at v1-beta (reserved-not-selectable).
    ReserveTypedReject,
}

/// Dispatch a requested keying reserve. At v1-beta every reserve is a
/// **typed-reject** — NEVER silently promoted to a real [`MembershipSetKind`].
///
/// # Errors
///
/// Always returns [`KindDispatchError::ReserveTypedReject`] at v1-beta — the
/// reserves are named but not selectable (the additive-codepoint discipline:
/// they become selectable only when a future codepoint lights them up).
pub fn dispatch_reserve(r: RequestedReserveKind) -> Result<MembershipSetKind, KindDispatchError> {
    match r {
        RequestedReserveKind::AtriumWithRotatingGroupKey | RequestedReserveKind::EphemeralLobby => {
            Err(KindDispatchError::ReserveTypedReject)
        }
    }
}
