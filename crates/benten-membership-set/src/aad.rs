//! The AAD-bound keying minimum: the `members_table` canonical-CBOR snapshot
//! + the `0x6610` group multi-stanza per-stanza AAD assembler.
//!
//! Both produce **OPAQUE bytes** handed to `benten-crypto-suite` (m-15 GNC-5)
//! — the crypto-suite has NO reverse dependency on this crate (the AAD is
//! opaque to it).
//!
//! ## `members_table` canonical-CBOR (NQ-W4 — the flagship freeze)
//!
//! [`canonical_members_table_bytes`] serializes the AAD-bound members-table
//! snapshot to canonical DAG-CBOR (sorted map keys, deterministic shortest-form
//! int encoding, no indefinite lengths) — the length-injective (U3) wire
//! contract NQ-W4 freezes. If two engines serialize the SAME logical
//! membership to DIFFERENT bytes, the AAD differs and cross-engine AEAD-open
//! fails for the same membership (a fork that never converges).
//!
//! ## `0x6610` group per-stanza AAD = the BLINDED 11-field set
//!
//! [`assemble_group_aad`] is the canonical-TLV encoder for the R0.7
//! §3.10/§4.1 BLINDED 11-field set (big-endian, length-injective). It HONORS
//! Sealed-Sender by default (F-LC-9 / BR-1 ruling 1): the inner-sender-DID is
//! bound INSIDE the sealed per-stanza payload (recovered post-decrypt), NOT in
//! the plaintext AAD. The roster + raw set-id are BLINDED into two 32-byte
//! commitments (§3.9 / Compromise #61 blinding posture):
//!
//! - `audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)`
//!   over the CANONICAL SORTED recipient-DID list.
//! - `membership_set_id_commitment = blake3::keyed_hash(K_Set,
//!   "benten:setid:v1" || membership_set_id)` (R0.7 §4.1: `HMAC` =
//!   `blake3::keyed_hash`; native BLAKE3 keyed MAC, no hmac/sha2 dep — the SAME
//!   §3.9 gossip-topic keyed-MAC primitive).
//!
//! All crypto here uses BLAKE3 (the project hash + keyed-MAC primitive, a
//! vetted upstream) — never a forked construction (#5).

use crate::codepoints::MEMBERSHIP_SET_GROUP_MULTI_STANZA;
use crate::member::{Did, MemberEntry};
use std::collections::BTreeMap;

/// The frozen AAD version prefix byte (R0.7 §4.1: `aad_version: u8` prefix).
/// V2-era; bumped only on a deliberate AAD wire-format change.
pub const AAD_VERSION: u8 = 0x01;

/// The §3.9 / setid-commitment domain-separation label (R0.6 §3.10):
/// `membership_set_id_commitment = blake3::keyed_hash(K_Set,
/// "benten:setid:v1" || id)`.
pub const SETID_COMMITMENT_LABEL: &[u8] = b"benten:setid:v1";

/// The frozen byte width of the inline self-describing CIDv1 the group AAD
/// binds: `0x01 0x71 0x1e 0x20 || 32-byte BLAKE3` = **36 bytes**. The
/// `body_cid` field is encoded INLINE (no external length prefix) and sits
/// before fixed-width integer fields, so a malformed (wrong-width) CID would
/// shift every following field boundary. [`assemble_group_aad`] asserts this
/// exact width at the assembly site (U3 length-injectivity). Mirrors the
/// drop-side `benten_drop::layer_c::group_posture::SELF_DESCRIBING_CID_LEN`.
pub const SELF_DESCRIBING_CID_LEN: usize = 36;

/// Serialize the AAD-bound `members_table` snapshot to canonical DAG-CBOR (the
/// NQ-W4 length-injective (U3) wire contract).
///
/// `serde_ipld_dagcbor` produces canonical DAG-CBOR: map keys sorted by
/// canonical byte order, shortest-form integer encoding, no indefinite
/// lengths. A `BTreeMap` additionally fixes iteration order at the type level —
/// so the bytes are independent of insertion order.
///
/// The bytes are OPAQUE to `benten-crypto-suite` (m-15 GNC-5).
///
/// # Panics
///
/// Panics only if the canonical CBOR encoder fails on the in-memory snapshot,
/// which cannot happen for the well-formed `MemberEntry` shape (it has no
/// non-serializable fields).
#[must_use]
pub fn canonical_members_table_bytes(table: &BTreeMap<Did, MemberEntry>) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(table)
        .expect("canonical DAG-CBOR encoding of a members_table snapshot is infallible")
}

/// The `0x6610` group per-stanza AAD inputs (Inv-20 clause-c; the BLINDED
/// 11-field set per R0.7 §3.10/§4.1).
///
/// `member_dids` is the member-DID list; the assembler canonicalizes (sorts)
/// it before deriving `audience_set_commitment` + `member_count`, so a reorder
/// must NOT change the bytes — that is what makes two engines agree. The raw
/// roster + raw set-id are BLINDED. On the DEFAULT (Sealed-Sender) path the
/// inner-sender-DID lives in `sealed_inner` (recovered post-decrypt) and is
/// NEVER bound into the plaintext AAD (F4-001 / F-LC-9).
#[derive(Clone)]
pub struct GroupAadInputs {
    /// The group per-stanza codepoint (`0x6610`
    /// `MEMBERSHIP_SET_GROUP_MULTI_STANZA` on the DEFAULT path) — BE u16.
    pub codepoint: u16,
    /// Canonical body-CID (the encrypted-payload CID) — a self-describing
    /// CIDv1 (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3`).
    pub body_cid: Vec<u8>,
    /// Member-DID list (canonicalized — sorted — by the assembler; a reorder
    /// is byte-neutral because only the COMMITMENT over the sorted list and
    /// the count are bound). NOT published in the clear (BLINDED).
    ///
    /// **CALLER CONTRACT (R6-tail F-38): this MUST be the DEDUPED members-table
    /// key set** — i.e. the keys of the `BTreeMap<Did, MemberEntry>`
    /// `members_table` (Inv-20 clause-i, one-DID-one-record), which are unique
    /// by construction. The assembler SORTS but does **NOT** dedup: a repeated
    /// DID is hashed twice into `audience_set_commitment` AND counted twice in
    /// `member_count`, so two engines that disagree about duplicates produce
    /// DIFFERENT AAD for the SAME logical membership and cross-engine AEAD-open
    /// fails (the NQ-W4 divergence mode). This precondition is NOT checked at
    /// the assembly seam; it is a caller obligation.
    pub member_dids: Vec<String>,
    /// The group key `K_Set` (keys the `membership_set_id_commitment` keyed MAC).
    pub k_set: [u8; 32],
    /// Per-stanza index — BE u32.
    pub stanza_index: u32,
    /// Total stanza count — BE u32 (truncation/censorship defense; R0.6 D4).
    pub stanza_count: u32,
    /// Member-key generation — BE u32.
    pub member_key_generation: u32,
    /// The raw set identity — BLINDED via keyed MAC into
    /// `membership_set_id_commitment` (never on the wire in the clear).
    pub membership_set_id: Vec<u8>,
    /// Set generation counter — BE u32.
    pub membership_set_generation: u32,
    /// Role-assignment generation — BE u32 (BC-5; `E_ROLE_STALE_AT_VERIFY`).
    pub role_assignments_generation: u32,
    /// **DEFAULT (Sealed-Sender) path:** the inner-sender-DID is sealed INSIDE
    /// this opaque payload, recovered only post-decrypt. NEVER bound into the
    /// plaintext AAD (F4-001 / F-LC-9).
    pub sealed_inner: Vec<u8>,
    /// **NON-default plaintext-sender variant ONLY:** when `Some`, the
    /// sender-DID is bound into the PLAINTEXT AAD (U4). `None` on the DEFAULT
    /// Sealed-Sender path (the shipped default).
    pub plaintext_sender_did: Option<String>,
}

/// R19 secret-hygiene: `Debug` redacts the group key `k_set` so the raw secret
/// never renders into logs / panics (the manual impl replaces the derived
/// `Debug`; all other fields still render for diagnostics). Mirrors the
/// crypto-suite `RecipientSecret` redacted-Debug convention.
impl core::fmt::Debug for GroupAadInputs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GroupAadInputs")
            .field("codepoint", &self.codepoint)
            .field("body_cid", &self.body_cid)
            .field("member_dids", &self.member_dids)
            .field("k_set", &"<redacted>")
            .field("stanza_index", &self.stanza_index)
            .field("stanza_count", &self.stanza_count)
            .field("member_key_generation", &self.member_key_generation)
            .field("membership_set_id", &self.membership_set_id)
            .field("membership_set_generation", &self.membership_set_generation)
            .field(
                "role_assignments_generation",
                &self.role_assignments_generation,
            )
            .field("sealed_inner", &self.sealed_inner)
            .field("plaintext_sender_did", &self.plaintext_sender_did)
            .finish()
    }
}

/// R19 secret-hygiene: wipe the transient `k_set` group-key copy on drop so
/// freed-heap / coredump exposure does not leak it. All field access is by-ref
/// (no partial-move), so the manual `Drop` is hazard-free. No wire /
/// serialization change (`GroupAadInputs` is never (de)serialized — it is an
/// AAD-input holder the assembler reads then drops).
impl Drop for GroupAadInputs {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        self.k_set.zeroize();
    }
}

/// `audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)`
/// over the CANONICAL SORTED recipient-DID list (`lp` = u32-BE length prefix).
/// Replaces the raw roster (BLINDED). The `0x01` domain-separation prefix
/// matches the spec construction. Returns the native 32-byte BLAKE3 output.
#[must_use]
pub fn audience_set_commitment(member_dids: &[String]) -> [u8; 32] {
    let mut sorted = member_dids.to_vec();
    sorted.sort();
    let mut msg = Vec::new();
    msg.push(0x01u8);
    for d in &sorted {
        lp(&mut msg, d.as_bytes());
    }
    blake3::hash(&msg).into()
}

/// `membership_set_id_commitment = blake3::keyed_hash(K_Set,
/// "benten:setid:v1" || membership_set_id)` — the SAME keyed-MAC PRIMITIVE the
/// §3.9 gossip-topic uses (R0.7 §4.1: `HMAC` = `blake3::keyed_hash`; BLAKE3 is
/// 32-wide so the truncation is the identity), but a DISTINCT construction: this
/// §3.10 commitment PREPENDS the `"benten:setid:v1"` domain-separation label,
/// whereas the §3.9 gossip-topic ([`crate::keying::gossip_topic`]) is UNLABELLED
/// and instead APPENDS `BE(generation)`. Same primitive, different preimage —
/// the two never collide (see the `keying::gossip_topic` doc for the
/// authoritative §3.9-labelled-vs-§3.10-unlabelled disambiguation). Replaces the
/// raw set-id (BLINDED).
#[must_use]
pub fn membership_set_id_commitment(k_set: &[u8; 32], membership_set_id: &[u8]) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(SETID_COMMITMENT_LABEL);
    msg.extend_from_slice(membership_set_id);
    blake3::keyed_hash(k_set, &msg).into()
}

/// Assemble the `0x6610` group per-stanza PLAINTEXT-AAD as OPAQUE `Vec<u8>`.
///
/// The return type is a plain byte vector, NOT a crypto-suite type — the
/// m-15 GNC-5 boundary contract: `benten-membership-set` assembles the
/// canonical bytes and hands `&[u8]` to `benten-crypto-suite`; the crypto-suite
/// never sees [`GroupAadInputs`].
///
/// Encoding = the R0.7 §3.10/§4.1 canonical-TLV contract (the BLINDED 11-field
/// set, big-endian, length-injective):
///
/// ```text
/// aad_version (u8) | codepoint (u16 BE) |
/// body_cid (inline self-describing CIDv1) | member_count (u32 BE) |
/// audience_set_commitment (32B) | stanza_index (u32 BE) |
/// stanza_count (u32 BE) | member_key_generation (u32 BE) |
/// membership_set_id_commitment (32B) | membership_set_generation (u32 BE) |
/// role_assignments_generation (u32 BE)
/// [non-default plaintext-sender ONLY] lp(sender_did)
/// ```
///
/// On the DEFAULT (Sealed-Sender) path the sender-DID is NOT bound here — it is
/// sealed inside `sealed_inner`, recovered post-decrypt (F4-001 / F-LC-9).
///
/// # Caller contract — `member_dids` MUST be deduped (R6-tail F-38)
///
/// The assembler CANONICALIZES `t.member_dids` by SORTING it (inside
/// [`audience_set_commitment`]) but does **NOT** dedup it. A duplicate DID is
/// therefore hashed twice into `audience_set_commitment` and counted twice in
/// `member_count`. Callers MUST pass the DEDUPED members-table key set (the
/// keys of the `BTreeMap<Did, MemberEntry>` snapshot — unique by Inv-20
/// clause-i); passing a duplicate-bearing list yields AAD that a peer deriving
/// its roster from the members-table will not reproduce, and the group stanza
/// fails to open. Unchecked here by design (the assembler is byte-frozen); see
/// the [`GroupAadInputs::member_dids`] field doc.
///
/// # Panics
///
/// Panics if the member count exceeds `u32::MAX` (the #46 ceiling makes this
/// unreachable in practice), or if `body_cid` is not exactly
/// [`SELF_DESCRIBING_CID_LEN`] (36) bytes — a malformed CID is a programming
/// error that would shift every later field boundary, so it fails loud here.
#[must_use]
pub fn assemble_group_aad(t: &GroupAadInputs) -> Vec<u8> {
    let mut buf = Vec::new();
    // aad_version prefix (U1/U14 strict-decode / cross-version replay defense).
    buf.push(AAD_VERSION);
    // codepoint — BE u16 (U1; committed in AAD).
    buf.extend_from_slice(&t.codepoint.to_be_bytes());
    // body_cid — INLINE self-describing CIDv1 (self-delimiting multihash; no
    // redundant external lp — uniform with 0x6510/0x6520). WIDTH-ASSERT the
    // canonical 36-byte CIDv1: the field is inline + boundary-load-bearing
    // (the fixed-width integers that follow are positionally addressed off
    // it), so a malformed CID MUST NOT silently shift every later field
    // boundary (U3 length-injectivity). Fail loud at the assembly site.
    assert_eq!(
        t.body_cid.len(),
        SELF_DESCRIBING_CID_LEN,
        "0x6610 AAD body_cid MUST be a canonical {SELF_DESCRIBING_CID_LEN}-byte self-describing CIDv1 (0x01 0x71 0x1e 0x20 || 32-byte BLAKE3)"
    );
    buf.extend_from_slice(&t.body_cid);
    // member_count — BE u32 over the member set. The roster itself is BLINDED
    // into audience_set_commitment.
    let member_count = u32::try_from(t.member_dids.len()).expect("member count fits u32");
    buf.extend_from_slice(&member_count.to_be_bytes());
    // audience_set_commitment — BLAKE3 over the canonically SORTED DID list
    // (BLINDED; replaces the raw roster). The assembler sorts internally, so an
    // UNSORTED input produces identical bytes.
    buf.extend_from_slice(&audience_set_commitment(&t.member_dids));
    // fixed-width BE integers.
    buf.extend_from_slice(&t.stanza_index.to_be_bytes());
    buf.extend_from_slice(&t.stanza_count.to_be_bytes());
    buf.extend_from_slice(&t.member_key_generation.to_be_bytes());
    // membership_set_id_commitment — keyed_hash(K_Set, label || set_id)
    // (BLINDED; replaces the raw membership_set_id).
    buf.extend_from_slice(&membership_set_id_commitment(
        &t.k_set,
        &t.membership_set_id,
    ));
    buf.extend_from_slice(&t.membership_set_generation.to_be_bytes());
    buf.extend_from_slice(&t.role_assignments_generation.to_be_bytes());
    // F4-001 / F-LC-9: the DEFAULT path binds NO plaintext sender. ONLY the
    // EXPLICITLY-non-default plaintext-sender variant appends it (U4).
    if let Some(sender) = &t.plaintext_sender_did {
        lp(&mut buf, sender.as_bytes());
    }
    buf
}

/// Length-prefix helper: writes `len: u32 BE || bytes` (the U3 length-injective
/// framing; the membership band uses u32 per the R0.7 §4.1 per-object width
/// note).
fn lp(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(
        &(u32::try_from(bytes.len()).expect("field length fits u32")).to_be_bytes(),
    );
    buf.extend_from_slice(bytes);
}

#[cfg(test)]
mod domain_registry_mirror {
    /// C-01/C-02 drift defense: `SETID_COMMITMENT_LABEL` is mirrored in the
    /// central [`benten_crypto_suite::domain_registry`] corpus table over which
    /// the prefix-free collision check runs. Pin byte-equality so the mirror
    /// can never silently diverge from this home definition.
    #[test]
    fn setid_commitment_label_matches_central_registry() {
        assert_eq!(
            super::SETID_COMMITMENT_LABEL,
            benten_crypto_suite::domain_registry::SETID_COMMITMENT_LABEL,
            "SETID_COMMITMENT_LABEL drifted from the central domain_registry mirror"
        );
    }
}
