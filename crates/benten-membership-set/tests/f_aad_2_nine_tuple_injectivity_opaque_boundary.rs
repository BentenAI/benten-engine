//! F-AAD-2 (R3-W5) — AAD group per-stanza injectivity + opaque-bytes boundary.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-2** (merges B2 +
//!   T-E2 + WF-C5 + GNI-2 + CE-I1).
//! - R0.6 plan §3.10 / §4.1 (the **`0x6610` group per-stanza AAD = the BLINDED
//!   11-field set**, Inv-20 clause-c) — **MembershipSet group sends honor
//!   Sealed-Sender (F-LC-9):** the inner-sender-DID is bound INSIDE the
//!   sealed/encrypted part per stanza (NOT in plaintext AAD), so the on-wire
//!   plaintext AAD binds the 11-field BLINDED set
//!   `{ aad_version (0x01,u8), codepoint (0x6610,u16 BE),
//!      body_cid (self-describing CIDv1), member_count (u32 BE),
//!      audience_set_commitment (32B), stanza_index (u32 BE),
//!      stanza_count (u32 BE), member_key_generation (u32 BE),
//!      membership_set_id_commitment (32B), membership_set_generation (u32 BE),
//!      role_assignments_generation (u32 BE) }` — where the sealed-inner
//!      sender-DID is the **post-decrypt-verified inner-payload sender-DID, NOT
//!      an on-wire plaintext field** — "load-bearing for inter-member
//!      non-forgeability".
//! - R0.6 plan §4.1 (the AAD encoding contract): "AAD codepoint binding +
//!   `aad_version: u8` prefix + **canonical-TLV** length-injective" (U1/U3/U14).
//! - R0.6 plan §3.10 precision (m-15 GNC-5): the AAD assembly hands
//!   **OPAQUE bytes** to `benten-crypto-suite`; the crypto-suite has **NO
//!   reverse dependency** on membership-set (the AAD is opaque to it).
//! - U1 (codepoint committed in AAD), U3 (canonical-TLV length-injective),
//!   U14.
//!
//! ## R4.5-MIGRATE (R0.6 Sealed-Sender AAD freeze — BLINDED group AAD)
//!
//! The spec of record advanced from R0.5 → **R0.6** (7 Ben-RATIFIED
//! Sealed-Sender AAD freeze decisions, 2026-06-03). The `0x6610` group
//! per-stanza AAD is now the **BLINDED 11-field set** (was the raw 9-tuple
//! with a plaintext roster + raw set-id). The migration applied here:
//!   1. **`audience_set_commitment` (32B) replaces the raw
//!      `sorted_member_dids[]` roster.** `audience_set_commitment =
//!      BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)` over the CANONICAL
//!      SORTED recipient-DID list (`lp` = u32-BE length prefix). The relay
//!      sees only an opaque 32-byte tag; recipients hold the member list and
//!      recompute + verify it. Obeys the project's own §3.9 / Compromise #61
//!      blinding posture (set-identifying material is never published in the
//!      clear).
//!   2. **`membership_set_id_commitment` (32B) replaces the raw
//!      `membership_set_id`.** `membership_set_id_commitment =
//!      HMAC(K_Set, "benten:setid:v1" || membership_set_id)` truncated to 32
//!      bytes — the SAME construction the §3.9 gossip topic uses (the sibling
//!      `f_gossip_transport_placement_and_blinded_topic.rs` stub-shims the
//!      HMAC as `blake3::keyed_hash(K_Set, ·)`; this file reuses the EXACT
//!      same primitive + truncation + BLAKE3 helper).
//!   3. **`stanza_count` (u32 BE) is bound ALONGSIDE `stanza_index`** as a
//!      truncation/censorship defense (without it an active relay can silently
//!      drop trailing stanzas and each surviving stanza still verifies).
//!   4. **`body_cid` is a self-describing CIDv1** (`0x01 0x71 0x1e 0x20 ||
//!      32-byte BLAKE3 digest`), NOT a bare fixed-32 digest (R0.6 BR; restores
//!      U3 length-injectivity; CLAUDE.md baked-in #5 — never hardcode a
//!      hash-width into a frozen wire). The corpus already framed `body_cid`
//!      as a `stub_cid` (self-describing) so this is preserved.
//!
//! **HONEST SCOPE (freeze record).** The group-AAD blinding is
//! identity-HIDING, NOT unlinkability: the commitment recurs for a static
//! group, so a network observer can still link sends to "the same unknown
//! group." Full per-send unlinkability (salt/nonce-rotated commitments) is
//! **U25, CODEPOINT-RESERVE for v1-GM**, additive over this field with no wire
//! break — NOT added now. Recipients hold `K_Set` + the member list ⇒
//! recompute + verify both commitments ⇒ ALL bindings (cross-stanza
//! substitution U17; inter-member non-forgeability) are PRESERVED; the relay
//! sees only opaque 32-byte tags.
//!
//! ## R4-FIX (F4-001 BLOCKER + F4-012 + F4-026 + F4-046 + F4-DS-CITE) — carried
//!
//! - **F4-001 (BLOCKER) — MembershipSet group sends honor Sealed-Sender
//!   (F-LC-9 / BR-1 ruling 1).** The DEFAULT (`0x6600`/`0x6610`) group-send
//!   AAD binds the **sealed-inner-sender-DID INSIDE the sealed per-stanza
//!   payload** (`sealed_inner`), recovered only post-decrypt; the PLAINTEXT
//!   11-field AAD binds ONLY the on-wire fields (NO plaintext sender). The
//!   no-plaintext-sender wire-scan + the paired non-default plaintext-sender
//!   control are carried forward.
//! - **F4-026 (CBOR↔TLV reconciliation):** the assembler is a deterministic
//!   canonical-TLV encoder (`aad_version` byte + BE u16 codepoint +
//!   length-prefixed variable fields + fixed BE integers + the two 32-byte
//!   commitments). (CBOR stays the `members_table`-snapshot encoding —
//!   F-AAD-1 — a DISTINCT inner field; the AAD WRAPPER is TLV.)
//! - **F4-012 (pre-sorted sort arm):** the sort arm hands the assembler an
//!   UNSORTED list and asserts identical bytes (the assembler canonicalizes
//!   the member list — both inside `audience_set_commitment` derivation AND
//!   for `member_count` — internally).
//! - **F4-046 (over-fenced arm5):** the crypto-suite-no-reverse-dep
//!   compile-fence reads a sibling `Cargo.toml` at runtime — un-ignored.
//! - **F4-DS-CITE:** stale `R0.3/R0.5 §` cites bumped to `R0.6 §`.
//!
//! ## pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext end-to-end discipline
//!
//! Each arm drives the PRODUCTION group-AAD assembler
//! (`assemble_group_aad`, stand-in for R5
//! `benten_membership_set::aad::assemble_group_aad`), asserts an OBSERVABLE
//! consequence (distinct bytes per single-field mutation / frozen
//! `aad_version` prefix / canonicalized-sort BEHIND the commitment /
//! sealed-inner sender-DID absent from plaintext AAD / paired non-default
//! sender present / opaque-`Vec<u8>` return type / blinded set-id +
//! audience-set tags do NOT leak the raw roster or raw set-id), and
//! would-FAIL-if-no-op'd (an assembler that dropped a field, used LE, omitted
//! the version prefix, skipped the internal sort, published the raw roster /
//! raw set-id, dropped `stanza_count`, OR leaked the sealed-inner sender-DID
//! into the plaintext AAD fails the corresponding arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`. SELF-CONTAINED stub-shim (no
//! sibling-wave dep) for parallel-safe R3. R5 swaps the shim for the real
//! `benten_membership_set::aad` types + un-ignores.
//!
//! ## Wave-0 DAG edge (M-20)
//!
//! The AAD's `codepoint` field is the V2-era `EncryptedEnvelope` codepoint
//! (`MembershipSetEncryption = 0x6600`, group multi-stanza `0x6610`); every
//! integer field is authored **big-endian** from the first commit; the
//! `aad_version` prefix is the V2-era version byte; the two 32-byte
//! commitments are computed ONCE off-line (M-20: golden frozen vs the stub
//! commitment helpers; R5 confirms vs the real encoder).

#![allow(clippy::unwrap_used)]

// ── SELF-CONTAINED stub-shim (R5 replaces with `benten_membership_set::aad`) ──

/// The frozen AAD version prefix byte (R0.6 §4.1: `aad_version: u8` prefix).
/// V2-era; bumped only on a deliberate AAD wire-format change. Distinct from
/// `ENVELOPE_FORMAT_VERSION_V2` (the envelope wire-format version) — the
/// AAD-version axis is its own byte (R0.6 §4.1).
const AAD_VERSION: u8 = 0x01;

/// The DEFAULT MembershipSet set-keying envelope codepoint (`0x6600`,
/// Sealed-Sender by default — F-LC-9 / BR-1 ruling 1; §4.0 RELOCATED from
/// the M-CONS-FINAL `0x6380` that collided MLS-Application). On this path the
/// inner-sender-DID is sealed INSIDE the per-stanza payload, NEVER in the
/// plaintext AAD.
const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;
/// The DEFAULT group multi-stanza codepoint (`0x6610`, also Sealed-Sender by
/// default).
const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;

/// The §3.9 / setid-commitment domain-separation label (R0.6 §3.10):
/// `membership_set_id_commitment = HMAC(K_Set, "benten:setid:v1" || id)`.
const SETID_COMMITMENT_LABEL: &[u8] = b"benten:setid:v1";

/// The fixture group key `K_Set` (32 bytes). In production this is the
/// multi-stanza-HPKE-Encap'd group key; here a fixed array so the blinded
/// commitments are deterministic for the golden vector.
const K_SET_FIXTURE: [u8; 32] = [0x5e; 32];

// NOTE (extra-reflection-pass): the paired non-default plaintext-sender control
// deliberately does NOT mint a synthetic codepoint. The §4.0 MembershipSet band
// defines no plaintext-sender sibling (only `0x6600`/`0x6610`/`0x6620`-reserve),
// so squatting an unassigned band value (e.g. `0x6601`) would (a) collide with
// the FROZEN-band Inv-18 / NQ-W2 CI scanner's ownership assertion and (b) read
// as a real assignment. The control instead keeps the DEFAULT keying codepoint
// and varies the ONE typed field that actually distinguishes the paths
// (`plaintext_sender_did`) — see `fixture_nondefault_plaintext_sender`.

/// The `0x6610` group per-stanza AAD inputs (Inv-20 clause-c; BLINDED
/// 11-field set per R0.6 §3.10/§4.1). `member_dids` is the member-DID list;
/// the assembler canonicalizes (sorts) it before deriving
/// `audience_set_commitment` + `member_count`, so a reorder must NOT change
/// the bytes — that is what makes two engines agree.
///
/// **R4.5-MIGRATE (R0.6):** the raw roster + raw set-id are BLINDED. The
/// assembler emits `audience_set_commitment` (over the sorted DIDs) and
/// `membership_set_id_commitment` (HMAC over `K_Set`) instead of the raw
/// values, and binds `stanza_count` alongside `stanza_index`.
///
/// **F4-001 / F-LC-9:** the sender-DID is NOT a plaintext field. On the
/// DEFAULT (Sealed-Sender) path the inner-sender-DID lives in `sealed_inner`
/// (recovered post-decrypt) and is NEVER bound into the plaintext AAD.
#[derive(Clone, Debug)]
struct GroupAadInputs {
    /// `MembershipSetEncryption` codepoint family (`0x6600`/`0x6610`) — BE u16.
    codepoint: u16,
    /// Canonical body-CID (the encrypted-payload CID) — a self-describing
    /// CIDv1 (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3`).
    body_cid: Vec<u8>,
    /// Member-DID list (canonicalized — sorted — by the assembler; a reorder
    /// is byte-neutral because only the COMMITMENT over the sorted list and
    /// the count are bound). NOT published in the clear (BLINDED).
    member_dids: Vec<String>,
    /// The group key `K_Set` (keys the `membership_set_id_commitment` HMAC).
    k_set: [u8; 32],
    /// Per-stanza index — BE u32.
    stanza_index: u32,
    /// Total stanza count — BE u32 (truncation/censorship defense; R0.6 D4).
    stanza_count: u32,
    /// Member-key generation — BE u32.
    member_key_generation: u32,
    /// The raw set identity — BLINDED via HMAC into
    /// `membership_set_id_commitment` (never on the wire in the clear).
    membership_set_id: Vec<u8>,
    /// Set generation counter — BE u32.
    membership_set_generation: u32,
    /// Role-assignment generation — BE u32 (BC-5; `E_ROLE_STALE_AT_VERIFY`).
    role_assignments_generation: u32,
    /// **DEFAULT (Sealed-Sender) path:** the inner-sender-DID is sealed
    /// INSIDE this opaque payload, recovered only post-decrypt. NEVER bound
    /// into the plaintext AAD (F4-001 / F-LC-9).
    sealed_inner: Vec<u8>,
    /// **NON-default plaintext-sender variant ONLY:** when `Some`, the
    /// sender-DID is bound into the PLAINTEXT AAD (U4). `None` on the DEFAULT
    /// Sealed-Sender path (the shipped default).
    plaintext_sender_did: Option<String>,
}

impl GroupAadInputs {
    /// The canonical DEFAULT (`0x6600`, Sealed-Sender) fixture. The
    /// sender-DID (`did:key:zSENDER…` — a UNIQUE marker absent from the member
    /// list, so the wire-scan is meaningful) is sealed INSIDE `sealed_inner`,
    /// NEVER in the plaintext AAD.
    fn fixture() -> Self {
        GroupAadInputs {
            codepoint: MEMBERSHIP_SET_ENCRYPTION,
            body_cid: stub_cid(b"body-payload"),
            member_dids: vec!["did:key:zAAA".to_string(), "did:key:zBBB".to_string()],
            k_set: K_SET_FIXTURE,
            stanza_index: 0,
            stanza_count: 1,
            member_key_generation: 1,
            membership_set_id: stub_cid(b"set-id"),
            membership_set_generation: 1,
            role_assignments_generation: 1,
            // Sealed-Sender DEFAULT: the inner-sender-DID is sealed here, NOT
            // in the plaintext AAD. The UNIQUE marker (not a member DID) makes
            // the F4-001 wire-scan distinguish hiding from a broken scan.
            sealed_inner: seal_inner_sender(SENDER_DID_MARKER),
            plaintext_sender_did: None,
        }
    }

    /// The paired non-default plaintext-sender control: the SAME logical send
    /// as `fixture()` — including the SAME (`0x6600`) keying codepoint — but the
    /// sender-DID is bound into the plaintext AAD (U4) instead of sealed inside
    /// `sealed_inner`. NOT a shipped default; exists ONLY to prove the default
    /// path's wire-scan is non-vacuous. Holding the codepoint FIXED makes the
    /// byte-delta between this and the default EXACTLY the appended sender field.
    fn fixture_nondefault_plaintext_sender() -> Self {
        let mut t = GroupAadInputs::fixture();
        t.sealed_inner = Vec::new();
        t.plaintext_sender_did = Some(SENDER_DID_MARKER.to_string());
        t
    }
}

/// A UNIQUE sender-DID marker NOT present in the member-DID list, so the
/// F4-001 wire-scan for it in the plaintext AAD is meaningful (a member-DID
/// would always legitimately appear, even blinded).
const SENDER_DID_MARKER: &str = "did:key:zSENDERuniqueMARKER";

/// Model the sealed inner payload (HPKE inner-payload sender-DID + wrapped
/// CEK in production). The crucial property: this byte blob is the SEALED
/// material — it is opaque, it lives OUTSIDE the plaintext AAD, and the
/// sender-DID inside it is recovered only post-decrypt.
fn seal_inner_sender(sender_did: &str) -> Vec<u8> {
    let mut inner = b"SEALED-INNER".to_vec();
    inner.extend_from_slice(&(sender_did.len() as u32).to_be_bytes());
    inner.extend_from_slice(sender_did.as_bytes());
    inner
}

/// **R4.5-MIGRATE (R0.6 §3.10).** `audience_set_commitment =
/// BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)` over the CANONICAL SORTED
/// recipient-DID list (`lp` = u32-BE length prefix). Replaces the raw roster.
/// The 0x01 domain-separation prefix matches the spec construction. Returns
/// the native 32-byte BLAKE3 output (no truncation needed; BLAKE3 is 32-wide).
fn audience_set_commitment(member_dids: &[String]) -> [u8; 32] {
    let mut sorted = member_dids.to_vec();
    sorted.sort();
    let mut msg = Vec::new();
    msg.push(0x01u8);
    for d in &sorted {
        lp(&mut msg, d.as_bytes());
    }
    blake3::hash(&msg).into()
}

/// **R4.5-MIGRATE (R0.6 §3.10).** `membership_set_id_commitment =
/// HMAC(K_Set, "benten:setid:v1" || membership_set_id)` truncated to 32
/// bytes — the SAME construction §3.9 already uses for the gossip topic. The
/// sibling `f_gossip_transport_placement_and_blinded_topic.rs` stub-shims the
/// HMAC primitive as `blake3::keyed_hash(K_Set, ·)` (truncate-to-32 is already
/// the BLAKE3 output width); this file reuses the EXACT same primitive +
/// truncation + BLAKE3 helper (do NOT invent a different HMAC/hash). R5 routes
/// both through the real `benten-crypto-suite` HMAC over `K_Set`.
fn membership_set_id_commitment(k_set: &[u8; 32], membership_set_id: &[u8]) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(SETID_COMMITMENT_LABEL);
    msg.extend_from_slice(membership_set_id);
    blake3::keyed_hash(k_set, &msg).into()
}

/// PRODUCTION-stand-in: the `0x6610` group per-stanza PLAINTEXT-AAD assembler.
/// Returns OPAQUE `Vec<u8>` — note the return type is a plain byte vector, NOT
/// a crypto-suite type. This is the m-15 GNC-5 boundary contract:
/// `benten-membership-set` assembles the canonical bytes and hands `&[u8]` to
/// `benten-crypto-suite`; the crypto-suite never sees `GroupAadInputs`.
///
/// Encoding = the R0.6 §3.10/§4.1 canonical-TLV contract — the BLINDED
/// 11-field set, big-endian, length-injective:
///   aad_version (u8) | codepoint (u16 BE) |
///   body_cid (lp; self-describing CIDv1) | member_count (u32 BE) |
///   audience_set_commitment (32B) | stanza_index (u32 BE) |
///   stanza_count (u32 BE) | member_key_generation (u32 BE) |
///   membership_set_id_commitment (32B) | membership_set_generation (u32 BE) |
///   role_assignments_generation (u32 BE)
///   [non-default plaintext-sender ONLY] lp(sender_did)
///
/// **F4-001 / F-LC-9:** on the DEFAULT (Sealed-Sender) path the sender-DID is
/// NOT bound here — it is sealed inside `sealed_inner`, recovered post-decrypt.
fn assemble_group_aad(t: &GroupAadInputs) -> Vec<u8> {
    let mut buf = Vec::new();
    // aad_version prefix (U1/U14 strict-decode / cross-version replay defense).
    buf.push(AAD_VERSION);
    // codepoint — BE u16 (U1; committed in AAD).
    buf.extend_from_slice(&t.codepoint.to_be_bytes());
    // body_cid — length-prefixed (self-describing CIDv1; lp preserves the
    // membership-band uniform variable-field framing + length-injectivity).
    lp(&mut buf, &t.body_cid);
    // member_count — BE u32 over the canonical (sorted-deduped-as-presented)
    // member set. The roster itself is BLINDED into audience_set_commitment.
    let member_count = u32::try_from(t.member_dids.len()).expect("member count fits u32");
    buf.extend_from_slice(&member_count.to_be_bytes());
    // audience_set_commitment — BLAKE3 over the canonically SORTED DID list
    // (BLINDED; replaces the raw roster). The assembler sorts internally, so
    // an UNSORTED input produces identical bytes.
    buf.extend_from_slice(&audience_set_commitment(&t.member_dids));
    // fixed-width BE integers.
    buf.extend_from_slice(&t.stanza_index.to_be_bytes());
    buf.extend_from_slice(&t.stanza_count.to_be_bytes());
    buf.extend_from_slice(&t.member_key_generation.to_be_bytes());
    // membership_set_id_commitment — HMAC(K_Set, label || set_id) (BLINDED;
    // replaces the raw membership_set_id).
    buf.extend_from_slice(&membership_set_id_commitment(&t.k_set, &t.membership_set_id));
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
/// framing; membership band uses u32 per the R0.6 §4.1 per-object width note).
fn lp(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn stub_cid(payload: &[u8]) -> Vec<u8> {
    // CIDv1 byte layout: 0x01 || 0x71 (dag-cbor) || 0x1e (blake3) || 0x20
    // (32-byte digest length) || 32-byte blake3 digest. Mirrors
    // benten-core::Cid so R5's real CID slots in byte-identically.
    let digest = blake3::hash(payload);
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(digest.as_bytes());
    cid
}

/// The ABSOLUTE frozen canonical-TLV golden vector for the DEFAULT
/// (`0x6600`, Sealed-Sender) `GroupAadInputs::fixture()` BLINDED 11-field AAD.
///
/// Computed ONCE, off-line, from the canonical-TLV assembler (M-20). **131
/// bytes** (NO plaintext sender field — F4-001 / F-LC-9); leading `0x01`
/// (`aad_version`); bytes 1..3 = `0x6600` (BE codepoint). NEITHER the raw
/// member roster NOR the raw set-id appears — they are BLINDED into the two
/// 32-byte commitments (`audience_set_commitment` +
/// `membership_set_id_commitment`). Any drift in field-order, endianness, the
/// version prefix, the length-prefix framing, the internal sort, the
/// commitment constructions, the `stanza_count` binding, OR a re-introduced
/// plaintext sender / raw roster / raw set-id flips this pin. R5
/// confirms-or-deliberately-updates this frozen literal against the real
/// `benten_membership_set::aad::assemble_group_aad` (M-20).
const EXPECTED_AAD_HEX: &str = "0166000000002401711e20cfa9fea5491b9bf64cdc143778c3ff6e0123d8f7bca130f292b27a9bde54a86000000002f89cac9e8f674417d5c99d74d6a96e7b46065b82bd5e198de9811ae9d34c230d000000000000000100000001b3ae4d07499bd779184c6d28735cf4c2458a5d63904a383e57bbcc940bdebe760000000100000001";

// ── F-AAD-2 arms ────────────────────────────────────────────────────────

/// F-AAD-2 arm 0 — frozen canonical-TLV golden vector + `aad_version` prefix.
///
/// Pins the ABSOLUTE bytes of the canonical DEFAULT BLINDED 11-field fixture
/// (R4.5-MIGRATE + F4-026 + F4-001): the `aad_version` prefix byte, the BE
/// codepoint, the self-describing body-CID, the two 32-byte blinded
/// commitments, `stanza_count`, and NO plaintext sender field. Any field-order
/// / endianness / version-prefix / commitment / count drift — or a
/// re-introduced plaintext sender / raw roster — flips this.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — BLINDED 11-field canonical-TLV golden + aad_version prefix (R0.6 §3.10/§4.1); un-ignore at R5"]
fn f_aad_2_canonical_tlv_golden_vector_and_version_prefix() {
    let bytes = assemble_group_aad(&GroupAadInputs::fixture());
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    // The frozen aad_version prefix byte (strict-decode / cross-version replay
    // defense — U1/U14). R5 reproduces this exact prefix.
    assert_eq!(
        bytes[0], AAD_VERSION,
        "leading byte = the frozen aad_version prefix (R0.6 §4.1)"
    );
    // The BE codepoint immediately follows the version prefix.
    assert_eq!(
        &bytes[1..3],
        &0x6600u16.to_be_bytes(),
        "codepoint is bound BIG-ENDIAN immediately after aad_version (U1; M-19 BE)"
    );
    // Full absolute golden vector.
    assert_eq!(
        hex_encode(&bytes),
        EXPECTED_AAD_HEX,
        "AAD group BLINDED 11-field canonical-TLV bytes drifted from the frozen \
         golden vector — divergent AAD = cross-engine AEAD-open failure (Inv-20 \
         clause-c)"
    );
}

/// F-AAD-2 arm 0b — `aad_version` prefix is byte-bound (cross-version defense).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — aad_version prefix is byte-bound (cross-version replay defense); un-ignore at R5"]
fn f_aad_2_aad_version_is_byte_bound() {
    let base = assemble_group_aad(&GroupAadInputs::fixture());
    let mut bumped = base.clone();
    bumped[0] = AAD_VERSION.wrapping_add(1);
    assert_ne!(
        base, bumped,
        "the aad_version prefix is part of the bound AAD — a version bump changes the bytes (U14)"
    );
}

/// F-AAD-2 arm 1 — single-field mutation distinctness (the bound fields).
///
/// Mutating ANY ONE of the bound BLINDED-11-field plaintext fields changes the
/// assembled bytes (including the fields that FEED the two commitments — a
/// roster change flips `audience_set_commitment`, a set-id or K_Set change
/// flips `membership_set_id_commitment`). This is the membership-side proof
/// that every bound field is byte-bound; the crypto-side "AEAD-open fails"
/// round-trip is co-owned by R3-W0's tf2-family.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — BLINDED-11-field plaintext fields byte-bound (single-field distinctness); un-ignore at R5"]
fn f_aad_2_plaintext_field_single_mutation_distinct() {
    let base = assemble_group_aad(&GroupAadInputs::fixture());

    let mut m = GroupAadInputs::fixture();
    m.codepoint = MEMBERSHIP_SET_GROUP_MULTI_STANZA; // 0x6610
    assert_ne!(base, assemble_group_aad(&m), "codepoint is byte-bound (U1)");

    let mut m = GroupAadInputs::fixture();
    m.body_cid = stub_cid(b"DIFFERENT-payload");
    assert_ne!(base, assemble_group_aad(&m), "body-CID is byte-bound");

    // A roster change flips BOTH member_count AND audience_set_commitment.
    let mut m = GroupAadInputs::fixture();
    m.member_dids.push("did:key:zCCC".to_string());
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "member-set membership is byte-bound (audience_set_commitment + member_count)"
    );

    // A K_Set change flips membership_set_id_commitment (keyed-blinding).
    let mut m = GroupAadInputs::fixture();
    m.k_set[0] ^= 0x01;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "K_Set is byte-bound via membership_set_id_commitment (keyed-blinding)"
    );

    let mut m = GroupAadInputs::fixture();
    m.stanza_index = 7;
    assert_ne!(base, assemble_group_aad(&m), "stanza-index is byte-bound");

    let mut m = GroupAadInputs::fixture();
    m.stanza_count = 9;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "stanza-count is byte-bound (truncation/censorship defense; R0.6 D4)"
    );

    let mut m = GroupAadInputs::fixture();
    m.member_key_generation = 9;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "member-key-generation is byte-bound"
    );

    // A set-id change flips membership_set_id_commitment.
    let mut m = GroupAadInputs::fixture();
    m.membership_set_id = stub_cid(b"OTHER-set");
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_id is byte-bound via membership_set_id_commitment"
    );

    let mut m = GroupAadInputs::fixture();
    m.membership_set_generation = 42;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_generation is byte-bound"
    );

    let mut m = GroupAadInputs::fixture();
    m.role_assignments_generation = 5;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "role_assignments_generation is byte-bound (BC-5; E_ROLE_STALE_AT_VERIFY)"
    );
}

/// F-AAD-2 arm 1b (F4-001 BLOCKER / F-LC-9 / BR-1 ruling 1) — the DEFAULT
/// MembershipSet group send HONORS Sealed-Sender: the inner-sender-DID is
/// sealed INSIDE the per-stanza payload and does NOT appear anywhere in the
/// plaintext AAD bytes.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — DEFAULT group send honors Sealed-Sender, sender-DID NOT in plaintext AAD (F4-001); un-ignore at R5"]
fn f_aad_2_default_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let t = GroupAadInputs::fixture();

    // (a) Typed-shape guard: the DEFAULT path carries NO plaintext_sender_did
    //     (it lives sealed inside `sealed_inner` instead).
    assert!(
        t.plaintext_sender_did.is_none(),
        "F-AAD-2 (F4-001): on the DEFAULT 0x6600/0x6610 group path the inputs \
         MUST NOT carry a plaintext_sender_did — Sealed-Sender binds the \
         inner-sender-DID INSIDE `sealed_inner`."
    );
    assert!(
        !t.sealed_inner.is_empty(),
        "F-AAD-2 (F4-001): the DEFAULT path MUST carry a sealed inner payload \
         (where the post-decrypt-verified inner-sender-DID lives)."
    );

    // (b) WIRE-SCAN: the sender-DID byte sequence MUST NOT appear anywhere in
    //     the PLAINTEXT AAD bytes. The marker is UNIQUE (not a member DID), so
    //     a hit unambiguously means a leak.
    let aad = assemble_group_aad(&t);
    let needle = SENDER_DID_MARKER.as_bytes();
    let leaks = aad.windows(needle.len()).any(|w| w == needle);
    assert!(
        !leaks,
        "F-AAD-2 (F4-001 / F-LC-9 / BR-1 ruling 1): the DEFAULT MembershipSet \
         group send MUST HONOR Sealed-Sender — the inner-sender-DID MUST NOT \
         appear in the PLAINTEXT BLINDED-11-field AAD bytes. It is bound \
         per-stanza INSIDE the sealed payload, recovered only post-decrypt. \
         would-FAIL if the default path bound the sender-DID into the plaintext \
         AAD."
    );
}

/// F-AAD-2 arm 1c (F4-001 paired control) — the non-default plaintext-sender
/// variant (`plaintext_sender_did = Some(..)`, SAME `0x6600` keying codepoint)
/// DOES place the sender-DID in the plaintext AAD (U4).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — paired control: NON-default plaintext-sender DOES carry sender-DID in AAD (F4-001); un-ignore at R5"]
fn f_aad_2_nondefault_plaintext_sender_carries_sender_did_in_aad() {
    let t = GroupAadInputs::fixture_nondefault_plaintext_sender();
    assert_eq!(
        t.plaintext_sender_did.as_deref(),
        Some(SENDER_DID_MARKER),
        "the NON-default variant MUST bind the sender-DID into the plaintext AAD (U4)"
    );

    let aad = assemble_group_aad(&t);
    let needle = SENDER_DID_MARKER.as_bytes();
    let leaks = aad.windows(needle.len()).any(|w| w == needle);
    assert!(
        leaks,
        "F-AAD-2 (F4-001 PAIRED CONTROL): the NON-default plaintext-sender \
         variant MUST place the sender-DID in the plaintext AAD (U4). If this \
         control fails, arm 1b's wire-scan cannot distinguish hiding from a \
         broken scan — the default and non-default paths must differ observably."
    );

    // The two paths MUST differ by EXACTLY the appended length-prefixed sender
    // field — nothing else (codepoint held fixed at the default).
    let default_aad = assemble_group_aad(&GroupAadInputs::fixture());
    let mut expected_nondefault = default_aad.clone();
    expected_nondefault.extend_from_slice(&(needle.len() as u32).to_be_bytes());
    expected_nondefault.extend_from_slice(needle);
    assert_eq!(
        aad, expected_nondefault,
        "the non-default plaintext-sender AAD must equal the default Sealed-Sender \
         AAD with EXACTLY the length-prefixed sender field appended — the sole \
         observable difference is the sender leak (no codepoint confound)"
    );
    assert_ne!(aad, default_aad, "non-default plaintext-sender AAD must differ from default");
}

/// F-AAD-2 arm 1d (R4.5-MIGRATE) — the BLINDED commitments do NOT publish the
/// raw roster or the raw set-id in the clear. The relay sees only opaque
/// 32-byte tags (§3.9 / Compromise #61 blinding posture). would-FAIL if an
/// assembler regressed to publishing the raw member-DID list or raw set-id.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — group AAD BLINDS the roster + set-id (no raw roster/set-id on the wire); un-ignore at R5"]
fn f_aad_2_blinded_commitments_do_not_leak_raw_roster_or_set_id() {
    let t = GroupAadInputs::fixture();
    let aad = assemble_group_aad(&t);

    // The raw member DIDs MUST NOT appear in the plaintext AAD — they are
    // BLINDED into audience_set_commitment.
    for did in &t.member_dids {
        let needle = did.as_bytes();
        let leaks = aad.windows(needle.len()).any(|w| w == needle);
        assert!(
            !leaks,
            "F-AAD-2 (R4.5-MIGRATE / #61): the raw member-DID {did:?} MUST NOT \
             appear in the plaintext group AAD — the roster is BLINDED into \
             audience_set_commitment. would-FAIL if the assembler published the \
             raw roster (the pre-R0.6 shape)."
        );
    }

    // The raw set-id MUST NOT appear — it is BLINDED into
    // membership_set_id_commitment.
    let set_id_needle = &t.membership_set_id;
    let set_id_leaks = aad
        .windows(set_id_needle.len())
        .any(|w| w == set_id_needle.as_slice());
    assert!(
        !set_id_leaks,
        "F-AAD-2 (R4.5-MIGRATE / #61): the raw membership_set_id MUST NOT appear \
         in the plaintext group AAD — it is BLINDED into \
         membership_set_id_commitment (HMAC(K_Set, label || set_id)). would-FAIL \
         if the assembler published the raw set-id (the pre-R0.6 shape)."
    );

    // POSITIVE control: the two 32-byte commitments ARE present (recomputable
    // by recipients holding K_Set + the member list).
    let asc = audience_set_commitment(&t.member_dids);
    let mscid = membership_set_id_commitment(&t.k_set, &t.membership_set_id);
    assert!(
        aad.windows(asc.len()).any(|w| w == asc),
        "F-AAD-2 (R4.5-MIGRATE): the audience_set_commitment MUST be bound in \
         the group AAD (recipients recompute + verify it)."
    );
    assert!(
        aad.windows(mscid.len()).any(|w| w == mscid),
        "F-AAD-2 (R4.5-MIGRATE): the membership_set_id_commitment MUST be bound \
         in the group AAD (recipients recompute + verify it)."
    );
}

/// F-AAD-2 arm 1e (R4.5-MIGRATE) — the commitment constructions reuse the EXACT
/// §3.9 gossip-topic primitives. `audience_set_commitment` is BLAKE3 over
/// `0x01 || lp(did)…` of the SORTED list; `membership_set_id_commitment` is
/// `blake3::keyed_hash(K_Set, label || set_id)` (the gossip-topic HMAC
/// stand-in). would-FAIL if the assembler used a different hash/HMAC or a
/// different domain-separation framing.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — commitments reuse the §3.9 gossip-topic BLAKE3/keyed-hash primitives; un-ignore at R5"]
fn f_aad_2_commitments_reuse_gossip_topic_primitives() {
    let t = GroupAadInputs::fixture();

    // audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1)) over SORTED.
    let mut sorted = t.member_dids.clone();
    sorted.sort();
    let mut asc_msg = Vec::new();
    asc_msg.push(0x01u8);
    for d in &sorted {
        asc_msg.extend_from_slice(&(d.len() as u32).to_be_bytes());
        asc_msg.extend_from_slice(d.as_bytes());
    }
    let asc_expected: [u8; 32] = blake3::hash(&asc_msg).into();
    assert_eq!(
        audience_set_commitment(&t.member_dids),
        asc_expected,
        "audience_set_commitment MUST be BLAKE3(0x01 || lp(did)…) over the SORTED \
         member-DID list (§3.9 construction)"
    );

    // membership_set_id_commitment = blake3::keyed_hash(K_Set, "benten:setid:v1" || set_id).
    let mut id_msg = Vec::new();
    id_msg.extend_from_slice(SETID_COMMITMENT_LABEL);
    id_msg.extend_from_slice(&t.membership_set_id);
    let mscid_expected: [u8; 32] = blake3::keyed_hash(&t.k_set, &id_msg).into();
    assert_eq!(
        membership_set_id_commitment(&t.k_set, &t.membership_set_id),
        mscid_expected,
        "membership_set_id_commitment MUST be keyed_hash(K_Set, \"benten:setid:v1\" || id) \
         — the SAME §3.9 gossip-topic HMAC primitive + truncation (BLAKE3 32-wide)"
    );
}

/// F-AAD-2 arm 2 — member-DID-list canonical sort (the assembler sorts BEHIND
/// the blinded commitment).
///
/// **F4-012 fix:** the input is handed to the assembler UNSORTED (reverse
/// order). The PRODUCTION assembler must canonically sort the member-DID list
/// before deriving `audience_set_commitment`, so the UNSORTED input must
/// produce the SAME bytes as the sorted fixture.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — assembler canonically sorts the member-DID list BEHIND the commitment (unsorted input ⇒ same bytes); un-ignore at R5"]
fn f_aad_2_member_did_list_sort_order_canonical() {
    let base = assemble_group_aad(&GroupAadInputs::fixture());

    // Same DID SET, presented UNSORTED (reverse order). NO in-body sort — the
    // assembler itself must canonicalize inside audience_set_commitment.
    let mut reordered = GroupAadInputs::fixture();
    reordered.member_dids = vec!["did:key:zBBB".to_string(), "did:key:zAAA".to_string()];
    assert_eq!(
        base,
        assemble_group_aad(&reordered),
        "an UNSORTED-but-equal member-DID set MUST assemble to identical bytes — \
         the assembler canonicalizes (sorts) internally before deriving the \
         audience_set_commitment (cross-engine convergence)"
    );
}

/// F-AAD-2 arm 3 — length-injectivity proptest (U3).
///
/// For arbitrary distinct `stanza_index` values, distinct tuples produce
/// distinct bytes and no shorter encoding is a prefix of a longer one.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — group-AAD length-injectivity (U3) proptest; un-ignore at R5"]
fn f_aad_2_length_injectivity_proptest() {
    use proptest::prelude::*;
    proptest!(|(idx_a in any::<u32>(), idx_b in any::<u32>())| {
        prop_assume!(idx_a != idx_b);
        let mut a = GroupAadInputs::fixture();
        a.stanza_index = idx_a;
        let mut b = GroupAadInputs::fixture();
        b.stanza_index = idx_b;
        let ba = assemble_group_aad(&a);
        let bb = assemble_group_aad(&b);
        prop_assert_ne!(&ba, &bb, "distinct stanza-indices ⇒ distinct AAD bytes");
        // Length-injectivity: neither encoding may be a prefix of the other.
        prop_assert!(!is_prefix(&ba, &bb), "no AAD encoding may be a prefix of another (U3)");
        prop_assert!(!is_prefix(&bb, &ba), "no AAD encoding may be a prefix of another (U3)");
    });
}

/// F-AAD-2 arm 4 — opaque-bytes boundary compile-fence (m-15 GNC-5).
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — assembler emits OPAQUE Vec<u8> (no crypto type leak); un-ignore at R5"]
fn f_aad_2_assembler_emits_opaque_bytes() {
    let bytes: Vec<u8> = assemble_group_aad(&GroupAadInputs::fixture());
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    // Compile-fence: the crypto-suite consumes the AAD as OPAQUE `&[u8]`.
    fn consume_opaque_aad(aad: &[u8]) -> usize {
        aad.len()
    }
    assert_eq!(
        consume_opaque_aad(&bytes),
        bytes.len(),
        "the AAD crosses the seam as opaque &[u8] (m-15 GNC-5)"
    );
    assert_eq!(
        bytes[0], AAD_VERSION,
        "canonical-TLV aad_version prefix (R0.6 §4.1) — opaque to crypto-suite"
    );
}

/// F-AAD-2 arm 5 — compile-fence: crypto-suite has NO reverse dep on
/// membership-set (m-15 GNC-5).
///
/// **F4-046:** this arm reads a sibling manifest at runtime (the manifest
/// exists in the workspace at baseline), so it is NOT a RED-phase pin against
/// undelivered code — it is un-ignored now and runs every CI cycle.
#[test]
fn f_aad_2_crypto_suite_no_reverse_dep_compile_fence() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crypto_manifest = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("benten-crypto-suite")
        .join("Cargo.toml");
    let text = std::fs::read_to_string(&crypto_manifest)
        .expect("benten-crypto-suite/Cargo.toml must exist (the only crypto-primitive call site)");
    assert!(
        !text.contains("benten-membership-set"),
        "benten-crypto-suite MUST NOT depend on benten-membership-set — the AAD is OPAQUE bytes \
         across the seam (m-15 GNC-5; no reverse dependency)"
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
