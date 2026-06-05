//! The MembershipSet codepoint family (the keying-mechanism's frozen wire
//! identifiers).
//!
//! The MembershipSet band is `0x6600..=0x66FF` (Inv-18 / NQ-W2 FROZEN-band
//! ownership). Three values are assigned at v1-beta:
//!
//! - `0x6600` `MEMBERSHIP_SET_ENCRYPTION` — the set-keying envelope codepoint
//!   (Sealed-Sender by default — F-LC-9 / BR-1 ruling 1; §4.0 RELOCATED from
//!   the M-CONS-FINAL `0x6380` that collided MLS-Application). Every
//!   [`crate::kind::MembershipSetKind`] binds to this set-keying value.
//! - `0x6610` `MEMBERSHIP_SET_GROUP_MULTI_STANZA` — the group multi-stanza
//!   per-stanza AAD codepoint (R0.7 §3.10/§4.1; also Sealed-Sender by
//!   default). The `0x6610` group per-stanza AAD assembler binds THIS value
//!   (NOT the `0x6600` set-keying value).
//! - `0x6620` — reserved for a future MembershipSet wire shape (additive over
//!   the crypto-agility framework; never a wire break).

/// The MembershipSet codepoint band lower bound (`0x6600`).
pub const MEMBERSHIP_SET_BAND_LO: u16 = 0x6600;

/// The MembershipSet codepoint band upper bound (`0x66FF`).
pub const MEMBERSHIP_SET_BAND_HI: u16 = 0x66FF;

/// The MembershipSet **set-keying** envelope codepoint (`0x6600`). Every
/// [`crate::kind::MembershipSetKind`] binds to this value (the keying axis).
/// Sealed-Sender by default (F-LC-9 / BR-1 ruling 1).
pub const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;

/// The MembershipSet **group multi-stanza** per-stanza AAD codepoint
/// (`0x6610`). The group per-stanza AAD assembler ([`crate::aad::assemble_group_aad`])
/// binds THIS value (R0.7 §3.10/§4.1) — distinct from the set-keying value.
/// Sealed-Sender by default.
pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;

/// The MembershipSet `0x6620` codepoint — RESERVED for a future wire shape
/// (additive; never a wire break).
pub const MEMBERSHIP_SET_RESERVED_0X6620: u16 = 0x6620;
