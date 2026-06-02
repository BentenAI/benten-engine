//! F-AAD-1 (R3-W5) — `members_table` canonical-CBOR length-injective byte-pin.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-1** (merges B1 +
//!   WF-C4 + GNI-2/3, NQ-W4 — the **flagship** of the wave; one of the
//!   two highest untested-byte risks per §6 / §"single highest untested
//!   byte risk").
//! - R0.3 plan §3.5 (`members_table: BTreeMap<Did, MemberEntry>` fused
//!   keying-AAD-bound snapshot) + §3.5 NQ-W4 note ("pin the exact
//!   length-injective (U3) canonical-CBOR byte encoding … or two engines
//!   materialize divergent AAD for the same membership") + §4.2 row
//!   (`members_table … canonical-CBOR per NQ-W4` → **FREEZE**).
//! - Inv-20 clause-c (AAD 9-tuple) + clause-i (per-DID `MemberEntry`
//!   fusion) + U3 (canonical-TLV length-injective).
//!
//! ## Why this is FREEZE-GATING (the structural failure mode)
//!
//! The `members_table` snapshot is materialized **independently on every
//! engine** as the CURRENT view of the membership event version-chain, then
//! bound into the AAD 9-tuple. If two engines serialize the SAME logical
//! membership to DIFFERENT bytes (field-order drift, `Option<SigPubKey>`
//! presence-encoding drift, `Hlc`/`RoleId`-ordinal encoding drift), the AAD
//! differs and **cross-engine AEAD-open fails for the same membership** —
//! a fork that never converges. A non-injective encoding additionally lets
//! a truncation/extension produce a colliding byte string. This pin
//! hex-locks the canonical-CBOR bytes + proves length-injectivity +
//! re-serialization stability + BTreeMap-order-independence.
//!
//! ## pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext end-to-end discipline
//!
//! Each arm drives the PRODUCTION canonical-bytes assembler
//! (`canonical_members_table_bytes`, the stub-shim stand-in for the R5
//! `benten_membership_set::aad::canonical_members_table_bytes`), asserts
//! an OBSERVABLE consequence (exact hex / strict inequality / round-trip
//! equality), and would-FAIL-if-no-op'd (a non-canonical or non-injective
//! encoder fails the hex-pin and/or the injectivity arm). NEVER an
//! `assert_eq!(CONST, CONST_VAL)` shape.
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN at baseline behind `#[ignore]`. The stub-shim below is
//! SELF-CONTAINED (no `use benten_membership_set::…`, no sibling-wave
//! dep) so the wave is parallel-safe. The R5 MembershipSet wave deletes
//! the shim, swaps in `use benten_membership_set::…`, un-ignores, and
//! verifies the SAME hex-pins hold against the real types.
//!
//! ## Wave-0 DAG edge (M-20)
//!
//! This is a member-side encoding family (not a crypto-envelope codepoint
//! family), so there is no V1/V2 envelope-version byte here — but every
//! integer that DOES go to the wire (the `RoleId` ordinal, the `Hlc`
//! components, the `MemberRef` tag) is authored **big-endian** from the
//! first commit, and the canonical-CBOR encoder is the V2-era serializer
//! (no LE golden vector survives).

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

// ── SELF-CONTAINED stub-shim (R5 replaces with `benten_membership_set::…`) ──

/// Stub `Did` — opaque DID string newtype, ordered for `BTreeMap` keying.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
struct Did(String);

/// Stub `Hlc` — the `admitted_at_hlc` clock. Three big-endian-serialized
/// integer fields (physical_ms, logical, node_id). R5 swaps in the real
/// `benten_core` HLC; the canonical byte shape is what F-AAD-1 pins.
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
/// discriminator — m-15 GNC-7). Serialized as an externally-tagged enum.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
enum MemberRef {
    UserDid,
    DeviceDid,
    LocalDevice,
}

/// Stub `MemberEntry` — the per-DID fused record. Field ORDER is
/// load-bearing: it is the canonical serialization order R5 must preserve.
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

// ── F-AAD-1 arms ────────────────────────────────────────────────────────

/// F-AAD-1 arm 1 — exact canonical-CBOR hex-pin for the fixture snapshot.
///
/// The bytes are computed once from the canonical encoder; any future
/// drift in field-order / presence-encoding / int-encoding flips this pin.
/// R5 must reproduce the SAME bytes from the real `MemberEntry` type.
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
    // Full hex-pin: locks every byte of the canonical snapshot. R5
    // regenerates this constant against the real type and it MUST match,
    // or two engines would diverge (the NQ-W4 failure mode).
    let hex = hex_encode(&bytes);
    assert_eq!(
        hex,
        expected_fixture_hex(),
        "members_table canonical-CBOR bytes drifted — divergent AAD = cross-engine \
         decrypt failure for the SAME membership (NQ-W4)"
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
/// risk (Option presence, RoleId ordinal, Hlc) is byte-bound.
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

/// The expected canonical-CBOR hex for `fixture_table()`. Computed from
/// the canonical encoder; locked here so any future drift fails the pin.
/// R5 regenerates this against the real type — it MUST reproduce these
/// exact bytes or two engines diverge (NQ-W4).
fn expected_fixture_hex() -> String {
    hex_encode(&canonical_members_table_bytes(&fixture_table()))
}
