//! F-AAD-1 (R5) — `members_table` canonical-CBOR length-injective byte-pin.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-1** (merges B1 +
//!   WF-C4 + GNI-2/3, NQ-W4 — the **flagship** of the wave).
//! - R0.5 plan §3.5 (`members_table: BTreeMap<Did, MemberEntry>` fused
//!   keying-AAD-bound snapshot) + §4.2 row (`members_table … canonical-CBOR
//!   per NQ-W4` → **FREEZE**).
//! - R0.5 plan §3.5 canonical enum representation ruling (R4.2 F4-007): ALL
//!   `MemberEntry` enums serialize as an INTEGER discriminant in
//!   canonical-CBOR/AAD — `RoleId` as its `u8` ordinal AND `MemberRef` as a
//!   `u8`-tagged variant (NOT a text string).
//! - Inv-20 clause-c (the `0x6610` group AAD = the BLINDED 11-field set) +
//!   clause-i (per-DID `MemberEntry` fusion) + U3 (canonical-TLV
//!   length-injective).
//!
//! ## M-20 RECONCILIATION (R5 — real encoder confirms the frozen golden)
//!
//! The W5 stub froze `EXPECTED_HEX` against a self-contained stub-shim encoder.
//! At R5 this is recomputed via the REAL
//! `benten_membership_set::aad::canonical_members_table_bytes` over the REAL
//! `MemberEntry` / `BentenHlc` / `RoleId` / `MemberRef` / `Did` types — and it
//! reproduces the frozen 258-byte literal BYTE-FOR-BYTE (confirmed via the
//! M-20 throwaway recompute step; no golden update needed). `BentenHlc`
//! (which does not derive `Serialize`) is serialized through the crate's
//! `HlcWire` mirror so the 3-key (`logical`/`node_id`/`physical_ms`) HLC map
//! is byte-identical. Any future drift in field-order, presence-encoding,
//! ordinal, `Hlc` layout, or integer endianness flips this pin.
//!
//! ## RED-PHASE → R5
//!
//! The W5 self-contained stub-shim is deleted; the arms drive the REAL
//! `canonical_members_table_bytes`. The hex-pin against the ABSOLUTE frozen
//! golden vector holds against the real types.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::hlc::BentenHlc;
use benten_id::did::Did;
use benten_membership_set::aad::canonical_members_table_bytes;
use benten_membership_set::member::{MemberEntry, MemberRef, SigPubKey};
use benten_membership_set::role::RoleId;

fn did(s: &str) -> Did {
    Did::from_string_for_test_fixture(s.to_string())
}

/// Canonical fixture: a 2-member Atrium snapshot. One authority (Admin, with
/// sig_pubkey present) + one plain Member (no sig_pubkey).
fn fixture_table() -> BTreeMap<Did, MemberEntry> {
    let mut t = BTreeMap::new();
    t.insert(
        did("did:key:zAAA"),
        MemberEntry {
            role: RoleId::Admin,
            is_authority: true,
            sig_pubkey: Some(SigPubKey(vec![0x11; 32])),
            admitted_at_hlc: BentenHlc::new(1_000, 0, 0xAAAA_AAAA),
            member_ref: MemberRef::UserDid,
        },
    );
    t.insert(
        did("did:key:zBBB"),
        MemberEntry {
            role: RoleId::Member,
            is_authority: false,
            sig_pubkey: None,
            admitted_at_hlc: BentenHlc::new(2_000, 0, 0xBBBB_BBBB),
            member_ref: MemberRef::UserDid,
        },
    );
    t
}

/// The ABSOLUTE frozen canonical-DAG-CBOR golden vector for `fixture_table()`.
///
/// 258 bytes; leading `0xa2` (CBOR major-type-5 map, 2 pairs); DAG-CBOR
/// canonically sorts the per-entry map keys (length-then-bytewise: `role`
/// `member_ref` `sig_pubkey` `is_authority` `admitted_at_hlc`). `member_ref`
/// is a CBOR unsigned int (`0x00` = UserDid), symmetric with the `role` int
/// ordinal (R0.5 §3.5 F4-007). **R5 (M-20) confirms this byte-for-byte against
/// the real `benten_membership_set::aad::canonical_members_table_bytes`.**
const EXPECTED_HEX: &str = "a26c6469643a6b65793a7a414141a564726f6c65046a6d656d6265725f726566006a7369675f7075626b6579582011111111111111111111111111111111111111111111111111111111111111116c69735f617574686f72697479f56f61646d69747465645f61745f686c63a3676c6f676963616c00676e6f64655f69641aaaaaaaaa6b706879736963616c5f6d731903e86c6469643a6b65793a7a424242a564726f6c65026a6d656d6265725f726566006a7369675f7075626b6579f66c69735f617574686f72697479f46f61646d69747465645f61745f686c63a3676c6f676963616c00676e6f64655f69641abbbbbbbb6b706879736963616c5f6d731907d0";

// ── F-AAD-1 arms ────────────────────────────────────────────────────────

#[test]
fn f_aad_1_members_table_canonical_cbor_hex_pinned() {
    let bytes = canonical_members_table_bytes(&fixture_table());
    assert!(!bytes.is_empty(), "canonical bytes must be non-empty");
    assert_eq!(
        bytes[0], 0xA2,
        "leading byte = CBOR map-header for exactly 2 members (major type 5, len 2); \
         field-order / map-shape drift flips this"
    );
    let hex = hex_encode(&bytes);
    assert_eq!(
        hex, EXPECTED_HEX,
        "members_table canonical-CBOR bytes drifted from the frozen golden vector — \
         divergent AAD = cross-engine decrypt failure for the SAME membership (NQ-W4)"
    );
}

/// F-AAD-1 arm 1b — `member_ref` is an INTEGER discriminant, never a text
/// string (R0.5 §3.5 F4-007 ruling).
#[test]
fn f_aad_1_member_ref_is_int_not_text() {
    let hex = hex_encode(&canonical_members_table_bytes(&fixture_table()));
    // `6755736572446964` = CBOR text(7) header `0x67` + the 7-byte payload
    // `55 73 65 72 44 69 64` ("UserDid"). Must NOT appear.
    assert!(
        !hex.contains("6755736572446964"),
        "member_ref MUST serialize as an int tag, NOT a CBOR text string (F4-007)"
    );
    // Defense-in-depth: even the bare 7-byte payload must not appear.
    assert!(
        !hex.contains("55736572446964"),
        "the ASCII \"UserDid\" payload bytes MUST NOT appear in any framing (F4-007)"
    );
    // Positive: the `member_ref` key is followed by a CBOR unsigned int 0x00.
    let key_marker = "6a6d656d6265725f726566"; // CBOR text(10) "member_ref"
    let idx = hex
        .find(key_marker)
        .expect("member_ref key must be present in the canonical encoding");
    let value_byte = &hex[idx + key_marker.len()..idx + key_marker.len() + 2];
    assert_eq!(
        value_byte, "00",
        "member_ref value MUST be CBOR unsigned int 0x00 (UserDid) — symmetric with the role int ordinal (F4-007)"
    );
}

/// F-AAD-1 arm 2 — re-serialization stability (idempotent canonical form).
#[test]
fn f_aad_1_canonical_reserialize_byte_identical() {
    let a = canonical_members_table_bytes(&fixture_table());
    let b = canonical_members_table_bytes(&fixture_table());
    assert_eq!(a, b, "canonical encoder must be deterministic (idempotent)");
}

/// F-AAD-1 arm 3 — BTreeMap-order-independence.
#[test]
fn f_aad_1_insertion_order_independent() {
    let canonical = canonical_members_table_bytes(&fixture_table());

    let mut reversed = BTreeMap::new();
    let f = fixture_table();
    for (k, v) in f.iter().rev() {
        reversed.insert(k.clone(), v.clone());
    }
    let reversed_bytes = canonical_members_table_bytes(&reversed);
    assert_eq!(
        canonical, reversed_bytes,
        "canonical bytes MUST be insertion-order-independent (cross-engine convergence)"
    );
}

/// F-AAD-1 arm 4 — length-injectivity / non-collision (U3).
#[test]
fn f_aad_1_length_injective_no_truncation_collision() {
    let base = canonical_members_table_bytes(&fixture_table());

    let mut extended = fixture_table();
    extended.insert(
        did("did:key:zCCC"),
        MemberEntry {
            role: RoleId::Viewer,
            is_authority: false,
            sig_pubkey: None,
            admitted_at_hlc: BentenHlc::new(3_000, 0, 0xCCCC_CCCC),
            member_ref: MemberRef::UserDid,
        },
    );
    let extended_bytes = canonical_members_table_bytes(&extended);

    let mut truncated = fixture_table();
    truncated.remove(&did("did:key:zBBB"));
    let truncated_bytes = canonical_members_table_bytes(&truncated);

    assert_ne!(base, extended_bytes, "+1 member must change the bytes");
    assert_ne!(base, truncated_bytes, "-1 member must change the bytes");
    assert_ne!(
        extended_bytes, truncated_bytes,
        "distinct snapshots, distinct bytes"
    );
    assert!(
        !is_prefix(&truncated_bytes, &extended_bytes),
        "a shorter snapshot's bytes MUST NOT be a prefix of a longer one (U3 length-injective)"
    );
}

/// F-AAD-1 arm 5 — field-reorder / presence-flip sensitivity.
#[test]
fn f_aad_1_every_field_is_byte_bound() {
    let base = canonical_members_table_bytes(&fixture_table());

    // (a) RoleId ordinal flip (Admin→Moderator).
    let mut role_flip = fixture_table();
    role_flip.get_mut(&did("did:key:zAAA")).unwrap().role = RoleId::Moderator;
    assert_ne!(
        base,
        canonical_members_table_bytes(&role_flip),
        "RoleId ordinal is byte-bound"
    );

    // (b) Option<SigPubKey> presence flip (Some→None on the authority).
    let mut presence_flip = fixture_table();
    presence_flip
        .get_mut(&did("did:key:zAAA"))
        .unwrap()
        .sig_pubkey = None;
    assert_ne!(
        base,
        canonical_members_table_bytes(&presence_flip),
        "Option<SigPubKey> presence-encoding is byte-bound (NQ-W4)"
    );

    // (c) Hlc mutation.
    let mut hlc_flip = fixture_table();
    hlc_flip
        .get_mut(&did("did:key:zBBB"))
        .unwrap()
        .admitted_at_hlc = BentenHlc::new(9_999, 0, 0xBBBB_BBBB);
    assert_ne!(
        base,
        canonical_members_table_bytes(&hlc_flip),
        "admitted_at_hlc is byte-bound"
    );

    // (d) is_authority flip.
    let mut auth_flip = fixture_table();
    auth_flip
        .get_mut(&did("did:key:zBBB"))
        .unwrap()
        .is_authority = true;
    assert_ne!(
        base,
        canonical_members_table_bytes(&auth_flip),
        "is_authority is byte-bound"
    );

    // (e) MemberRef ordinal flip (UserDid→DeviceDid).
    let mut ref_flip = fixture_table();
    ref_flip.get_mut(&did("did:key:zAAA")).unwrap().member_ref = MemberRef::DeviceDid;
    assert_ne!(
        base,
        canonical_members_table_bytes(&ref_flip),
        "MemberRef ordinal is byte-bound (F4-007)"
    );
}

// ── helpers ─────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn is_prefix(short: &[u8], long: &[u8]) -> bool {
    short.len() <= long.len() && &long[..short.len()] == short
}
