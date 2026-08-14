//! S-3 closure — frozen const VALUE pins for `benten-membership-set`.
//!
//! The required `cargo-public-api` baseline records const TYPES only, so a
//! value-only edit to any constant below is an EMPTY baseline diff. Several of
//! these bytes feed key derivation and never reach the wire, so no round-trip
//! or golden-hex test observes a change either.

use benten_membership_set::aad::{AAD_VERSION, SELF_DESCRIBING_CID_LEN, SETID_COMMITMENT_LABEL};
use benten_membership_set::codepoints::{
    MEMBERSHIP_SET_BAND_HI, MEMBERSHIP_SET_BAND_LO, MEMBERSHIP_SET_ENCRYPTION,
    MEMBERSHIP_SET_GROUP_MULTI_STANZA, MEMBERSHIP_SET_RESERVED_0X6620,
};
use benten_membership_set::error::E_ROLE_STALE_AT_VERIFY;
use benten_membership_set::federation::{
    ENVELOPE_FORMAT_VERSION_V2, MEMBERSHIP_RECURSION_MAX_DEPTH,
};
use benten_membership_set::keying::{KN_DERIVE_CONTEXT, KV_DERIVE_CONTEXT};
use benten_membership_set::kind::MembershipSetKind;
use benten_membership_set::ucan::DEFAULT_EXP_BOUND_SECS;

// ---------------------------------------------------------------------------
// Group 1 — AAD framing.
//
// WHAT BREAKS: AAD_VERSION is committed into the AEAD tag; SETID_COMMITMENT_LABEL
// is committed into the set-id; SELF_DESCRIBING_CID_LEN is an AAD field WIDTH.
// Any change invalidates existing sealed group content.
// ---------------------------------------------------------------------------

#[test]
fn membership_set_aad_framing_is_frozen() {
    assert_eq!(AAD_VERSION, 0x01, "MembershipSet AAD version byte");
    assert_eq!(
        SETID_COMMITMENT_LABEL, b"benten:setid:v1",
        "set-id commitment label — a change forks every set identity"
    );
    assert_eq!(
        SELF_DESCRIBING_CID_LEN, 36,
        "self-describing CIDv1 length — an AAD field width"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — codepoint band.
//
// WHAT BREAKS: the 0x6600..=0x66FF band is reserved for MembershipSet in the
// central registry. Moving a member or the band edges collides with a
// neighbouring band.
// ---------------------------------------------------------------------------

#[test]
fn membership_set_codepoints_are_frozen() {
    assert_eq!(MEMBERSHIP_SET_BAND_LO, 0x6600, "band low edge");
    assert_eq!(MEMBERSHIP_SET_BAND_HI, 0x66FF, "band high edge");
    assert_eq!(MEMBERSHIP_SET_ENCRYPTION, 0x6600);
    assert_eq!(MEMBERSHIP_SET_GROUP_MULTI_STANZA, 0x6610);
    assert_eq!(MEMBERSHIP_SET_RESERVED_0X6620, 0x6620);
}

// ---------------------------------------------------------------------------
// Group 3 — K(V) / K(N) derivation contexts.
//
// WHAT BREAKS: these are BLAKE3 KDF contexts. A change silently forks the
// MembershipSet key schedule while every round-trip stays self-consistent —
// the S-6 hazard in its purest form.
// ---------------------------------------------------------------------------

#[test]
fn membership_set_kdf_contexts_are_frozen() {
    assert_eq!(
        KV_DERIVE_CONTEXT, "benten-membership-set:K(V):v1",
        "K(V) derivation context — a change makes all prior K(V)-sealed content undecryptable"
    );
    assert_eq!(
        KN_DERIVE_CONTEXT, "benten-membership-set:K(N):v1",
        "K(N) derivation context — a change makes all prior K(N)-sealed content undecryptable"
    );
}

// ---------------------------------------------------------------------------
// Group 4 — federation + shape bounds.
//
// WHAT BREAKS: ENVELOPE_FORMAT_VERSION_V2 is a wire version byte;
// MEMBERSHIP_RECURSION_MAX_DEPTH is the Inv-20-clause-k recursion bound applied
// to untrusted federated input; VARIANT_COUNT is the frozen
// EXACTLY-3-MembershipSetKind Rust-mechanism surface.
// ---------------------------------------------------------------------------

#[test]
fn membership_set_bounds_and_shape_are_frozen() {
    assert_eq!(
        ENVELOPE_FORMAT_VERSION_V2, 0x02,
        "federation envelope wire version byte"
    );
    assert_eq!(
        MEMBERSHIP_RECURSION_MAX_DEPTH, 4,
        "Inv-20 clause-k federated recursion bound over untrusted input — widening re-opens \
         unbounded recursion"
    );
    assert_eq!(
        MembershipSetKind::VARIANT_COUNT,
        3,
        "the frozen EXACTLY-3 MembershipSetKind mechanism surface"
    );
    assert_eq!(
        MembershipSetKind::ALL.len(),
        MembershipSetKind::VARIANT_COUNT,
        "ALL must enumerate exactly VARIANT_COUNT variants"
    );
    assert_eq!(
        DEFAULT_EXP_BOUND_SECS, 3_600,
        "default UCAN expiry bound (seconds)"
    );
    assert_eq!(
        E_ROLE_STALE_AT_VERIFY, "E_ROLE_STALE_AT_VERIFY",
        "stable error-code string — mirrored cross-language"
    );
}
