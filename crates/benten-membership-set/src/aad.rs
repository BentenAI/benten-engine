//! AAD assembly — the `members_table` canonical-CBOR snapshot (F-AAD-1) + the
//! `0x6610` group multi-stanza per-stanza BLINDED 11-field AAD (F-AAD-2).
//!
//! # The opaque-bytes boundary (m-15 GNC-5)
//!
//! Every assembler here returns an OPAQUE `Vec<u8>` and hands `&[u8]` to
//! `benten-crypto-suite`. The crypto-suite has **NO reverse dependency** on
//! `benten-membership-set` (the AAD is opaque to it) — the F-CRATE-2
//! compile-fence + `dependency_edges` assert the direction.
//!
//! # `members_table` canonical-CBOR (F-AAD-1 / NQ-W4)
//!
//! [`canonical_members_table_bytes`] serializes the fused snapshot to canonical
//! DAG-CBOR (sorted map keys, shortest-form integer encoding, no indefinite
//! lengths). It is **length-injective** (U3) — a truncation/extension produces
//! a distinct byte string that is not a prefix-collision — and
//! **insertion-order-independent** (the `BTreeMap` fixes iteration order). If
//! two engines serialize the SAME logical membership to DIFFERENT bytes the AAD
//! differs and cross-engine AEAD-open fails (the NQ-W4 failure mode).
//!
//! # `0x6610` group AAD (F-AAD-2 / Inv-20 clause-c)
//!
//! [`assemble_group_aad`] emits the BLINDED 11-field set per R0.7 §3.10/§4.1 as
//! a deterministic canonical-TLV byte string (big-endian everywhere on the
//! wire). The raw roster + raw set-id are BLINDED into two 32-byte commitments
//! (`audience_set_commitment` + `membership_set_id_commitment`) — the relay
//! sees only opaque tags; recipients hold `K_Set` + the member list and
//! recompute + verify both. MembershipSet group sends HONOR Sealed-Sender
//! (F-LC-9): the inner-sender-DID is bound INSIDE the sealed per-stanza payload
//! (recovered only post-decrypt), NEVER in the plaintext AAD.

use benten_core::Cid;

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use benten_id::did::Did;

use crate::kind::MEMBERSHIP_SET_GROUP_MULTI_STANZA;
use crate::member::MemberEntry;

/// The frozen AAD version prefix byte (R0.7 §4.1: `aad_version: u8` prefix).
/// Bumped only on a deliberate AAD wire-format change (its own axis, distinct
/// from any envelope version byte).
pub const AAD_VERSION: u8 = 0x01;

/// The §3.9 / setid-commitment domain-separation label (R0.6 §3.10):
/// `membership_set_id_commitment = keyed_hash(K_Set, "benten:setid:v1" || id)`.
pub const SETID_COMMITMENT_LABEL: &[u8] = b"benten:setid:v1";

/// Serialize the fused `members_table` to canonical DAG-CBOR (the F-AAD-1
/// length-injective wire contract NQ-W4 freezes).
///
/// The canonical `serde_ipld_dagcbor` encoder produces map keys sorted by
/// canonical byte order, shortest-form integer encoding, and no indefinite
/// lengths. A `BTreeMap` additionally fixes iteration order at the type level,
/// so the bytes are independent of insertion order — two engines that admit
/// members in different orders still materialize byte-identical AAD.
///
/// # Panics
///
/// Panics only if the (infallible-in-practice) canonical encoder errors — the
/// member snapshot is always serializable.
#[must_use]
pub fn canonical_members_table_bytes(table: &BTreeMap<Did, MemberEntry>) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(table)
        .expect("canonical DAG-CBOR encoding of a members_table snapshot is infallible")
}

// ── 0x6610 group multi-stanza per-stanza AAD (BLINDED 11-field set) ───────────

/// The inputs to the `0x6610` group per-stanza AAD (Inv-20 clause-c; the
/// BLINDED 11-field set per R0.7 §3.10/§4.1).
///
/// `member_dids` is the recipient-DID list; [`assemble_group_aad`]
/// canonicalizes (sorts) it before deriving `audience_set_commitment` +
/// `member_count`, so a reorder is byte-neutral (the cross-engine convergence
/// property). The raw roster + raw set-id are BLINDED into the two commitments.
///
/// On the DEFAULT (Sealed-Sender) path the inner-sender-DID lives inside
/// `sealed_inner` (recovered post-decrypt) and is NEVER bound into the
/// plaintext AAD (F4-001 / F-LC-9).
#[derive(Clone, Debug)]
pub struct GroupAadInputs {
    /// The group per-stanza codepoint (`0x6610`
    /// [`MEMBERSHIP_SET_GROUP_MULTI_STANZA`] on the DEFAULT path) — BE u16.
    pub codepoint: u16,
    /// Canonical body-CID (the encrypted-payload CID) — a self-describing
    /// CIDv1 (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3`), bound inline.
    pub body_cid: Vec<u8>,
    /// Member-DID list (canonicalized — sorted — by the assembler; BLINDED).
    pub member_dids: Vec<String>,
    /// The group key `K_Set` (keys the `membership_set_id_commitment` MAC).
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
    /// DEFAULT (Sealed-Sender) path: the inner-sender-DID is sealed INSIDE this
    /// opaque payload, recovered only post-decrypt. NEVER bound into the
    /// plaintext AAD (F4-001 / F-LC-9).
    pub sealed_inner: Vec<u8>,
    /// NON-default plaintext-sender variant ONLY: when `Some`, the sender-DID
    /// is bound into the PLAINTEXT AAD (U4). `None` on the DEFAULT path.
    pub plaintext_sender_did: Option<String>,
}

/// `audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)` over
/// the CANONICAL SORTED recipient-DID list (`lp` = u32-BE length prefix).
///
/// Replaces the raw roster (R4.5-MIGRATE / R0.6 §3.10). The relay sees only the
/// opaque 32-byte tag; recipients hold the member list and recompute + verify.
/// The `0x01` domain-separation prefix matches the §3.9 gossip-topic
/// construction; BLAKE3 is 32-wide so no truncation is needed.
#[must_use]
pub fn audience_set_commitment(member_dids: &[String]) -> [u8; 32] {
    let mut sorted: Vec<&String> = member_dids.iter().collect();
    sorted.sort();
    let mut msg = Vec::new();
    msg.push(0x01u8);
    for d in &sorted {
        lp(&mut msg, d.as_bytes());
    }
    blake3::hash(&msg).into()
}

/// `membership_set_id_commitment = keyed_hash(K_Set, "benten:setid:v1" || id)`.
///
/// The SAME construction §3.9 already uses for the gossip topic, where R0.7
/// §4.1 clarifies `HMAC` = `blake3::keyed_hash` (native BLAKE3 keyed MAC; no
/// hmac/sha2 dep). BLAKE3 is 32-wide so truncate-to-32 is the native output.
#[must_use]
pub fn membership_set_id_commitment(k_set: &[u8; 32], membership_set_id: &[u8]) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(SETID_COMMITMENT_LABEL);
    msg.extend_from_slice(membership_set_id);
    blake3::keyed_hash(k_set, &msg).into()
}

/// Assemble the `0x6610` group per-stanza PLAINTEXT AAD — the BLINDED 11-field
/// set, big-endian, length-injective. Returns OPAQUE `Vec<u8>` (m-15 GNC-5).
///
/// Encoding (R0.7 §3.10/§4.1 canonical-TLV contract):
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
/// # Panics
///
/// Panics if `member_dids.len()` exceeds `u32::MAX` (never on any real roster
/// — the per-Kind ceilings cap it at 32).
#[must_use]
pub fn assemble_group_aad(t: &GroupAadInputs) -> Vec<u8> {
    let mut buf = Vec::new();
    // aad_version prefix (U1/U14 strict-decode / cross-version replay defense).
    buf.push(AAD_VERSION);
    // codepoint — BE u16 (U1; committed in AAD; M-19 big-endian).
    buf.extend_from_slice(&t.codepoint.to_be_bytes());
    // body_cid — INLINE self-describing CIDv1 (self-delimiting multihash; no
    // redundant external lp — uniform with 0x6510/0x6520).
    buf.extend_from_slice(&t.body_cid);
    // member_count — BE u32. The roster itself is BLINDED into
    // audience_set_commitment.
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
/// framing; membership band uses u32 per the R0.7 §4.1 per-object width note).
fn lp(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(bytes);
}

/// Build a self-describing CIDv1 byte layout (`0x01 0x71 0x1e 0x20 || 32-byte
/// BLAKE3 digest`) over `payload` — the dag-cbor (`0x71`) + BLAKE3 (`0x1e`)
/// framing, byte-identical to [`benten_core::Cid`]'s wire form.
#[must_use]
pub fn self_describing_cid_bytes(payload: &[u8]) -> Vec<u8> {
    let digest = blake3::hash(payload);
    let mut cid = Vec::with_capacity(4 + 32);
    cid.extend_from_slice(&[0x01u8, 0x71, 0x1e, 0x20]);
    cid.extend_from_slice(digest.as_bytes());
    cid
}

/// The self-describing CIDv1 bytes of a real [`benten_core::Cid`] (the body-CID
/// the group AAD binds). Parity with [`self_describing_cid_bytes`] for the real
/// content-addressed path.
#[must_use]
pub fn body_cid_bytes(cid: &Cid) -> Vec<u8> {
    cid.as_bytes().to_vec()
}
