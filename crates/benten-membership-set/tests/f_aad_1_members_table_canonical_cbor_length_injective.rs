//! F-AAD-1 (R3-W5) — `members_table` canonical-CBOR length-injective byte-pin.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-1** (merges B1 +
//!   WF-C4 + GNI-2/3, NQ-W4 — the **flagship** of the wave; one of the
//!   two highest untested-byte risks per §6 / §"single highest untested
//!   byte risk").
//! - R0.5 plan §3.5 (`members_table: BTreeMap<Did, MemberEntry>` fused
//!   keying-AAD-bound snapshot) + §3.5 NQ-W4 note ("pin the exact
//!   length-injective (U3) canonical-CBOR byte encoding … or two engines
//!   materialize divergent AAD for the same membership") + §4.2 row
//!   (`members_table … canonical-CBOR per NQ-W4` → **FREEZE**).
//! - R0.5 plan §3.5 **canonical enum representation ruling (R4.2 F4-007,
//!   Ben 2026-06-03):** ALL `MemberEntry` enums serialize as an INTEGER
//!   discriminant in canonical-CBOR/AAD — `RoleId` as its `u8` ordinal AND
//!   `MemberRef` as a `u8`-tagged variant (NOT a text string); int-tag is
//!   canonical for determinism + AAD compactness; the golden vector encodes
//!   `member_ref` as an integer tag, symmetric with `role`.
//! - Inv-20 clause-c (the `0x6610` group AAD = the BLINDED 11-field set per
//!   R0.6 §3.10/§4.1; supersedes the prior "9-tuple" framing) + clause-i
//!   (per-DID `MemberEntry` fusion) + U3 (canonical-TLV length-injective).
//!
//! ## Why this is FREEZE-GATING (the structural failure mode)
//!
//! The `members_table` snapshot is materialized **independently on every
//! engine** as the CURRENT view of the membership event version-chain, then
//! bound into the `0x6610` group AAD (the BLINDED 11-field set per R0.6
//! §3.10/§4.1). If two engines serialize the SAME logical
//! membership to DIFFERENT bytes (field-order drift, `Option<SigPubKey>`
//! presence-encoding drift, `Hlc`/`RoleId`/`MemberRef`-ordinal encoding
//! drift), the AAD differs and **cross-engine AEAD-open fails for the same
//! membership** — a fork that never converges. A non-injective encoding
//! additionally lets a truncation/extension produce a colliding byte string.
//! This pin hex-locks the canonical-CBOR bytes + proves length-injectivity +
//! re-serialization stability + BTreeMap-order-independence.
//!
//! ## R4-FIX (F4-001 BLOCKER + F4-007 MAJOR): absolute frozen golden vector
//!
//! The original `expected_fixture_hex()` re-invoked the SAME encoder it
//! asserted against (`assert_eq!(enc(x), enc(x))`) — a self-referential
//! tautology that froze ZERO bytes (any field-order / endianness / presence
//! drift would have changed BOTH sides equally and passed). The fix freezes
//! the canonical DAG-CBOR bytes of `fixture_table()` as an ABSOLUTE
//! `const EXPECTED_HEX` literal (computed once, off-line, from the canonical
//! `serde_ipld_dagcbor` encoder — see the throwaway compute step recorded in
//! the R4-fix manifest). The arm now asserts
//! `hex(encoder(fixture)) == EXPECTED_HEX`, so ANY drift in field-order,
//! presence-encoding, `RoleId`/`MemberRef` ordinal, `Hlc` layout, or integer
//! endianness flips the pin.
//!
//! **R4.3 (F4-007 MAJOR fix; R4.2 triage §"BLOCKERs/MAJORs", fix-now-at-R4):**
//! the prior corpus golden encoded `MemberRef`
//! as a CBOR text string (`0x67` + `"UserDid"`) while `RoleId` was already an
//! int ordinal — an asymmetric int/text representation that froze a
//! non-canonical, drift-prone wire string into the AAD-keying-bound snapshot.
//! Per the R0.5 §3.5 ruling, `MemberRef` now carries `#[serde(into = "u8")]`
//! (UserDid=0 / DeviceDid=1 / LocalDevice=2; tag 3 reserved for a future
//! `SubsetRef`) and serializes as its `u8` ordinal, symmetric with `RoleId`.
//! `EXPECTED_HEX` was regenerated against the int-tag shape (258 bytes; the
//! two `0x67`+`"UserDid"` 8-byte text strings each collapse to one `0x00`
//! int byte). **R5 confirms-or-deliberately-updates this frozen literal
//! against the real `benten_membership_set::aad::canonical_members_table_bytes`
//! (M-20).**
//!
//! ## pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext end-to-end discipline
//!
//! Each arm drives the PRODUCTION canonical-bytes assembler
//! (`canonical_members_table_bytes`, the stub-shim stand-in for the R5
//! `benten_membership_set::aad::canonical_members_table_bytes`), asserts
//! an OBSERVABLE consequence (exact hex / strict inequality / round-trip
//! equality), and would-FAIL-if-no-op'd (a non-canonical or non-injective
//! encoder fails the hex-pin and/or the injectivity arm; a text-encoded
//! `MemberRef` regression flips the hex-pin). NEVER an
//! `assert_eq!(CONST, CONST_VAL)` shape.
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN at baseline behind `#[ignore]`. The stub-shim below is
//! SELF-CONTAINED (no `use benten_membership_set::…`, no sibling-wave
//! dep) so the wave is parallel-safe. The `MemberEntry` / `Hlc` / `RoleId`
//! / `MemberRef` shapes are the canonical R0.5 §3.5 shapes (5-field
//! `MemberEntry`, 3-field `Hlc`, int-tagged `RoleId` + `MemberRef`) —
//! byte-compatible with the F-MS-3 fusion stub (F4-006 reconciliation). The
//! R5 MembershipSet wave deletes the shim, swaps in
//! `use benten_membership_set::…`, un-ignores, and verifies the SAME hex-pin
//! holds against the real types.
//!
//! ## Wave-0 DAG edge (M-20)
//!
//! This is a member-side encoding family (not a crypto-envelope codepoint
//! family), so there is no V1/V2 envelope-version byte here — but every
//! integer that DOES go to the wire (the `RoleId` ordinal, the `MemberRef`
//! ordinal, the `Hlc` components) is authored **big-endian** from the
//! first commit, and the canonical-CBOR encoder is the V2-era serializer
//! (no LE golden vector survives).

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

// ── SELF-CONTAINED stub-shim (R5 replaces with `benten_membership_set::…`) ──

/// Stub `Did` — opaque DID string newtype, ordered for `BTreeMap` keying.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
struct Did(String);

/// Stub `Hlc` — the `admitted_at_hlc` clock. The canonical R0.5 §3.5 3-field
/// shape (`physical_ms`, `logical`, `node_id`), big-endian-serialized. R5
/// swaps in the real `benten_core` HLC; the canonical byte shape is what
/// F-AAD-1 pins. (Aligned with the F-MS-3 fusion stub — F4-006.)
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct Hlc {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

/// Stub `SigPubKey` — present iff `is_authority`. The `Option<SigPubKey>`
/// presence-encoding is one of the three NQ-W4 drift risks.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct SigPubKey(#[serde(with = "serde_bytes")] Vec<u8>);

/// Stub `RoleId` — ALL 5 ACTIVE (BC-9); ordinal Invitee=0…Admin=4
/// (supersedes M-CONS-FINAL per M-13). Serialized as its `u8` ordinal
/// (AAD-keying-bound).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(into = "u8")]
enum RoleId {
    Invitee = 0,
    Viewer = 1,
    Member = 2,
    Moderator = 3,
    Admin = 4,
}
impl From<RoleId> for u8 {
    fn from(r: RoleId) -> u8 {
        r as u8
    }
}

/// Stub `MemberRef` — Kind-determined keying variant (NOT a nature
/// discriminator — m-15 GNC-7). Serialized as a `u8`-tagged integer variant
/// (R0.5 §3.5 F4-007 ruling), symmetric with `RoleId` — NOT a CBOR text
/// string. Tags: `UserDid=0`, `DeviceDid=1`, `LocalDevice=2`; tag `3` is
/// reserved for a future `SubsetRef`. The integer discriminant is the
/// AAD-keying-bound canonical wire form (determinism + AAD compactness).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(into = "u8")]
enum MemberRef {
    UserDid = 0,
    DeviceDid = 1,
    LocalDevice = 2,
    // tag 3 reserved for a future `SubsetRef` (R0.5 §3.5).
}
impl From<MemberRef> for u8 {
    fn from(m: MemberRef) -> u8 {
        m as u8
    }
}

/// Stub `MemberEntry` — the per-DID fused record. The canonical R0.5 §3.5
/// 5-field shape `{ role, is_authority, sig_pubkey, admitted_at_hlc,
/// member_ref }`. Field ORDER is load-bearing: it is the canonical
/// serialization order R5 must preserve. (Byte-compatible with the F-MS-3
/// fusion stub — F4-006.)
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct MemberEntry {
    role: RoleId,
    is_authority: bool,
    sig_pubkey: Option<SigPubKey>,
    admitted_at_hlc: Hlc,
    member_ref: MemberRef,
}

/// PRODUCTION-stand-in: the canonical-bytes assembler. R5 replaces the
/// body with the real `benten_membership_set::aad::canonical_members_table_bytes`
/// which serializes the AAD-bound snapshot to canonical DAG-CBOR. The
/// canonical encoder (sorted map keys, deterministic int encoding) is the
/// length-injective (U3) wire contract NQ-W4 freezes.
fn canonical_members_table_bytes(table: &BTreeMap<Did, MemberEntry>) -> Vec<u8> {
    // `serde_ipld_dagcbor` produces canonical DAG-CBOR: map keys sorted by
    // canonical byte order, shortest-form integer encoding, no indefinite
    // lengths. A `BTreeMap` additionally fixes iteration order at the type
    // level — so the bytes are independent of insertion order.
    serde_ipld_dagcbor::to_vec(table).unwrap()
}

/// Canonical fixture: a 2-member Atrium snapshot. One authority (Admin,
/// with sig_pubkey present) + one plain Member (no sig_pubkey).
fn fixture_table() -> BTreeMap<Did, MemberEntry> {
    let mut t = BTreeMap::new();
    t.insert(
        Did("did:key:zAAA".to_string()),
        MemberEntry {
            role: RoleId::Admin,
            is_authority: true,
            sig_pubkey: Some(SigPubKey(vec![0x11; 32])),
            admitted_at_hlc: Hlc {
                physical_ms: 1_000,
                logical: 0,
                node_id: 0xAAAA_AAAA,
            },
            member_ref: MemberRef::UserDid,
        },
    );
    t.insert(
        Did("did:key:zBBB".to_string()),
        MemberEntry {
            role: RoleId::Member,
            is_authority: false,
            sig_pubkey: None,
            admitted_at_hlc: Hlc {
                physical_ms: 2_000,
                logical: 0,
                node_id: 0xBBBB_BBBB,
            },
            member_ref: MemberRef::UserDid,
        },
    );
    t
}

/// The ABSOLUTE frozen canonical-DAG-CBOR golden vector for `fixture_table()`.
///
/// Computed ONCE, off-line, from the canonical `serde_ipld_dagcbor` encoder
/// over the canonical R0.5 §3.5 `MemberEntry` shape (NOT re-derived at test
/// time — that was the F4-001 tautology). 258 bytes; leading `0xa2`
/// (CBOR major-type-5 map, 2 pairs); DAG-CBOR canonically sorts the per-entry
/// map keys (length-then-bytewise: `role` `member_ref` `sig_pubkey`
/// `is_authority` `admitted_at_hlc`). `member_ref` is a CBOR unsigned int
/// (`0x00` = UserDid), symmetric with the `role` int ordinal (R0.5 §3.5
/// F4-007 ruling). Any drift in field-order, `Option<SigPubKey>`
/// presence-encoding, `RoleId`/`MemberRef` ordinal, `Hlc` layout, or integer
/// endianness flips this pin.
///
/// R5 confirms-or-deliberately-updates this frozen literal against the real
/// `benten_membership_set::aad::canonical_members_table_bytes` (M-20). If R5
/// must change it, the change is a DELIBERATE wire-format decision recorded
/// in the canary commit — never a silent drift.
const EXPECTED_HEX: &str = "a26c6469643a6b65793a7a414141a564726f6c65046a6d656d6265725f726566006a7369675f7075626b6579582011111111111111111111111111111111111111111111111111111111111111116c69735f617574686f72697479f56f61646d69747465645f61745f686c63a3676c6f676963616c00676e6f64655f69641aaaaaaaaa6b706879736963616c5f6d731903e86c6469643a6b65793a7a424242a564726f6c65026a6d656d6265725f726566006a7369675f7075626b6579f66c69735f617574686f72697479f46f61646d69747465645f61745f686c63a3676c6f676963616c00676e6f64655f69641abbbbbbbb6b706879736963616c5f6d731907d0";

// ── F-AAD-1 arms ────────────────────────────────────────────────────────

/// F-AAD-1 arm 1 — exact canonical-CBOR hex-pin for the fixture snapshot.
///
/// Asserts the encoder reproduces the ABSOLUTE frozen `EXPECTED_HEX` golden
/// vector. Any future drift in field-order / presence-encoding / int-encoding
/// (including a `MemberRef` text-vs-int regression) flips this pin. R5 must
/// reproduce these exact bytes from the real `MemberEntry` type, or two
/// engines would diverge (the NQ-W4 failure mode).
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — members_table canonical-CBOR byte-pin (NQ-W4 flagship); un-ignore at R5"]
fn f_aad_1_members_table_canonical_cbor_hex_pinned() {
    let bytes = canonical_members_table_bytes(&fixture_table());
    // The OBSERVABLE consequence: a non-empty deterministic byte string
    // whose length + leading map-header are locked. A two-entry CBOR map
    // starts with the major-type-5 (map) header 0xA2 (map of 2 pairs).
    assert!(!bytes.is_empty(), "canonical bytes must be non-empty");
    assert_eq!(
        bytes[0], 0xA2,
        "leading byte = CBOR map-header for exactly 2 members (major type 5, len 2); \
         field-order / map-shape drift flips this"
    );
    // Full hex-pin against the ABSOLUTE frozen golden vector (F4-001 +
    // F4-007 fix — NOT a self-derived `expected_fixture_hex()`, and
    // `member_ref` is the int ordinal NOT a text string). R5 regenerates
    // these exact bytes against the real type and they MUST match, or two
    // engines would diverge (the NQ-W4 failure mode).
    let hex = hex_encode(&bytes);
    assert_eq!(
        hex, EXPECTED_HEX,
        "members_table canonical-CBOR bytes drifted from the frozen golden vector — \
         divergent AAD = cross-engine decrypt failure for the SAME membership (NQ-W4)"
    );
}

/// F-AAD-1 arm 1b — `member_ref` is an INTEGER discriminant, never a text
/// string (R0.5 §3.5 F4-007 ruling).
///
/// The byte-level guard against a `MemberRef` text-vs-int regression: the
/// CBOR bytes for the `UserDid` text string MUST NOT appear anywhere in the
/// canonical encoding, and the `member_ref` map value MUST be a CBOR unsigned
/// int (`0x00` for UserDid). A regression to the externally-tagged text
/// representation re-introduces the asymmetric drift-prone wire string and
/// fails this arm even if the full hex-pin were (incorrectly) updated.
///
/// **Byte-precise needle (F4-007-NEEDLE fix, R4.4):** ASCII `"UserDid"` is
/// 7 chars = the 7-byte payload `55 73 65 72 44 69 64` (`55736572446964`,
/// 14 hex). As a CBOR text string it is prefixed by the major-type-3 len-7
/// header `0x67`, so the full on-wire token is `6755736572446964` (16 hex).
/// The needle below is the header-inclusive byte-precise token — a regression
/// to the text representation re-introduces exactly these bytes. (The prior
/// corpus needle `557365724469` was the 6-byte truncation `UserDi`, missing
/// the trailing `64`='d'; it still fired via substring match but was not
/// byte-precise — corrected here.)
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — member_ref is an int discriminant not a text string (F4-007); un-ignore at R5"]
fn f_aad_1_member_ref_is_int_not_text() {
    let hex = hex_encode(&canonical_members_table_bytes(&fixture_table()));
    // The CBOR text string `0x67` + ASCII "UserDid" must NOT appear: a
    // regression to the `0x67`("UserDid") encoding would re-introduce the
    // F4-007 asymmetry. `6755736572446964` = CBOR text(7) header `0x67`
    // followed by the 7-byte payload `55 73 65 72 44 69 64` ("UserDid").
    assert!(
        !hex.contains("6755736572446964"),
        "member_ref MUST serialize as an int tag, NOT a CBOR text string (F4-007); \
         the CBOR text(7) \"UserDid\" bytes leaking into the canonical encoding is the regression"
    );
    // Defense-in-depth: even the bare 7-byte payload (without the `0x67`
    // header — e.g. an alternate/indefinite text framing) must not appear.
    // `55736572446964` = ASCII "UserDid" (7 bytes, 14 hex).
    assert!(
        !hex.contains("55736572446964"),
        "the ASCII \"UserDid\" payload bytes MUST NOT appear in any framing (F4-007)"
    );
    // Positive: the `member_ref` key is followed by a CBOR unsigned int 0x00
    // (UserDid). `6a6d656d6265725f726566` = CBOR text(10) "member_ref"; the
    // immediately-following value byte is the int discriminant.
    let key_marker = "6a6d656d6265725f726566";
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
///
/// Encoding the SAME table twice yields byte-identical output. A
/// non-canonical encoder (HashMap iteration, indefinite-length CBOR) would
/// fail this.
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — canonical-CBOR re-serialization stability; un-ignore at R5"]
fn f_aad_1_canonical_reserialize_byte_identical() {
    let a = canonical_members_table_bytes(&fixture_table());
    let b = canonical_members_table_bytes(&fixture_table());
    assert_eq!(a, b, "canonical encoder must be deterministic (idempotent)");
}

/// F-AAD-1 arm 3 — BTreeMap-order-independence.
///
/// Inserting members in the opposite order yields the SAME canonical
/// bytes (the `BTreeMap` + canonical-CBOR sort the keys). This is the
/// cross-engine guarantee: two engines that admit members in different
/// orders still materialize byte-identical AAD.
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — canonical bytes independent of insertion order; un-ignore at R5"]
fn f_aad_1_insertion_order_independent() {
    let canonical = canonical_members_table_bytes(&fixture_table());

    // Build the same logical table with the inserts reversed.
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
///
/// A length-extension (adding a member) and a truncation (removing a
/// member) each produce a DIFFERENT byte string that is NOT a prefix-collision
/// of another valid snapshot. We assert distinctness across three related
/// snapshots (the fixture, a +1-member extension, a -1-member truncation).
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — length-injectivity / truncation-extension non-collision (U3); un-ignore at R5"]
fn f_aad_1_length_injective_no_truncation_collision() {
    let base = canonical_members_table_bytes(&fixture_table());

    // +1 member extension.
    let mut extended = fixture_table();
    extended.insert(
        Did("did:key:zCCC".to_string()),
        MemberEntry {
            role: RoleId::Viewer,
            is_authority: false,
            sig_pubkey: None,
            admitted_at_hlc: Hlc {
                physical_ms: 3_000,
                logical: 0,
                node_id: 0xCCCC_CCCC,
            },
            member_ref: MemberRef::UserDid,
        },
    );
    let extended_bytes = canonical_members_table_bytes(&extended);

    // -1 member truncation.
    let mut truncated = fixture_table();
    truncated.remove(&Did("did:key:zBBB".to_string()));
    let truncated_bytes = canonical_members_table_bytes(&truncated);

    assert_ne!(base, extended_bytes, "+1 member must change the bytes");
    assert_ne!(base, truncated_bytes, "-1 member must change the bytes");
    assert_ne!(
        extended_bytes, truncated_bytes,
        "distinct snapshots, distinct bytes"
    );
    // Length-injectivity: the map-header carries the member count, so no
    // truncation of the longer snapshot can equal the shorter one's bytes.
    assert!(
        !is_prefix(&truncated_bytes, &extended_bytes),
        "a shorter snapshot's bytes MUST NOT be a prefix of a longer one (U3 length-injective)"
    );
}

/// F-AAD-1 arm 5 — field-reorder / presence-flip sensitivity.
///
/// Mutating ANY field (role ordinal, is_authority, sig_pubkey presence,
/// HLC, member_ref) changes the canonical bytes. Proves every NQ-W4 drift
/// risk (Option presence, RoleId ordinal, MemberRef ordinal, Hlc) is
/// byte-bound.
#[test]
#[ignore = "RED-PHASE: F-AAD-1 — every member field is byte-bound (presence/ordinal/Hlc); un-ignore at R5"]
fn f_aad_1_every_field_is_byte_bound() {
    let base = canonical_members_table_bytes(&fixture_table());

    // (a) RoleId ordinal flip (Admin→Moderator).
    let mut role_flip = fixture_table();
    role_flip
        .get_mut(&Did("did:key:zAAA".to_string()))
        .unwrap()
        .role = RoleId::Moderator;
    assert_ne!(
        base,
        canonical_members_table_bytes(&role_flip),
        "RoleId ordinal is byte-bound"
    );

    // (b) Option<SigPubKey> presence flip (Some→None on the authority).
    let mut presence_flip = fixture_table();
    presence_flip
        .get_mut(&Did("did:key:zAAA".to_string()))
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
        .get_mut(&Did("did:key:zBBB".to_string()))
        .unwrap()
        .admitted_at_hlc
        .physical_ms = 9_999;
    assert_ne!(
        base,
        canonical_members_table_bytes(&hlc_flip),
        "admitted_at_hlc is byte-bound"
    );

    // (d) is_authority flip.
    let mut auth_flip = fixture_table();
    auth_flip
        .get_mut(&Did("did:key:zBBB".to_string()))
        .unwrap()
        .is_authority = true;
    assert_ne!(
        base,
        canonical_members_table_bytes(&auth_flip),
        "is_authority is byte-bound"
    );

    // (e) MemberRef ordinal flip (UserDid→DeviceDid). Byte-bound symmetric
    // with the RoleId ordinal (R0.5 §3.5 F4-007 int-tag ruling).
    let mut ref_flip = fixture_table();
    ref_flip
        .get_mut(&Did("did:key:zAAA".to_string()))
        .unwrap()
        .member_ref = MemberRef::DeviceDid;
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
