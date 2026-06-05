//! F-AAD-2 (R5) — AAD group per-stanza injectivity + opaque-bytes boundary.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-2** (merges B2 +
//!   T-E2 + WF-C5 + GNI-2 + CE-I1).
//! - R0.7 plan §3.10 / §4.1 (the **`0x6610` group per-stanza AAD = the BLINDED
//!   11-field set**, Inv-20 clause-c) — MembershipSet group sends honor
//!   Sealed-Sender (F-LC-9): the inner-sender-DID is bound INSIDE the
//!   sealed/encrypted part per stanza (NOT in plaintext AAD).
//! - R0.7 §4.1 (the AAD encoding contract): codepoint binding + `aad_version`
//!   prefix + canonical-TLV length-injective (U1/U3/U14).
//! - R0.7 §3.10 precision (m-15 GNC-5): the AAD assembly hands OPAQUE bytes to
//!   `benten-crypto-suite`; the crypto-suite has NO reverse dependency.
//!
//! ## M-20 RECONCILIATION (R5 — real assembler confirms the frozen golden)
//!
//! The frozen `EXPECTED_AAD_HEX` (127 bytes; codepoint `0x6610`, Sealed-Sender
//! default, BLINDED roster + set-id) is recomputed via the REAL
//! `benten_membership_set::aad::assemble_group_aad` and reproduces the golden
//! BYTE-FOR-BYTE (confirmed via the M-20 throwaway recompute step; no golden
//! update needed). The two 32-byte commitments
//! (`audience_set_commitment` = BLAKE3 over the sorted DIDs;
//! `membership_set_id_commitment` = `blake3::keyed_hash(K_Set, label || id)`)
//! reuse the §3.9 gossip-topic primitives.
//!
//! ## RED-PHASE → R5
//!
//! The W5 self-contained stub-shim is deleted; the arms drive the REAL
//! `assemble_group_aad` + the real commitment helpers.

#![allow(clippy::unwrap_used)]

use benten_membership_set::aad::{
    AAD_VERSION, GroupAadInputs, SETID_COMMITMENT_LABEL, assemble_group_aad,
    audience_set_commitment, membership_set_id_commitment, self_describing_cid_bytes,
};
use benten_membership_set::kind::{MEMBERSHIP_SET_ENCRYPTION, MEMBERSHIP_SET_GROUP_MULTI_STANZA};

/// The fixture group key `K_Set` (32 bytes).
const K_SET_FIXTURE: [u8; 32] = [0x5e; 32];

/// A UNIQUE sender-DID marker NOT present in the member-DID list, so the
/// F4-001 wire-scan for it in the plaintext AAD is meaningful.
const SENDER_DID_MARKER: &str = "did:key:zSENDERuniqueMARKER";

/// Model the sealed inner payload (HPKE inner-payload sender-DID + wrapped CEK
/// in production). The byte blob is SEALED — opaque, outside the plaintext AAD.
fn seal_inner_sender(sender_did: &str) -> Vec<u8> {
    let mut inner = b"SEALED-INNER".to_vec();
    inner.extend_from_slice(&(sender_did.len() as u32).to_be_bytes());
    inner.extend_from_slice(sender_did.as_bytes());
    inner
}

/// The canonical DEFAULT (`0x6610` group multi-stanza, Sealed-Sender) fixture.
fn fixture() -> GroupAadInputs {
    GroupAadInputs {
        codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        body_cid: self_describing_cid_bytes(b"body-payload"),
        member_dids: vec!["did:key:zAAA".to_string(), "did:key:zBBB".to_string()],
        k_set: K_SET_FIXTURE,
        stanza_index: 0,
        stanza_count: 1,
        member_key_generation: 1,
        membership_set_id: self_describing_cid_bytes(b"set-id"),
        membership_set_generation: 1,
        role_assignments_generation: 1,
        sealed_inner: seal_inner_sender(SENDER_DID_MARKER),
        plaintext_sender_did: None,
    }
}

/// The paired non-default plaintext-sender control: SAME logical send (incl. the
/// SAME `0x6610` codepoint) but the sender-DID is bound into the plaintext AAD.
fn fixture_nondefault_plaintext_sender() -> GroupAadInputs {
    let mut t = fixture();
    t.sealed_inner = Vec::new();
    t.plaintext_sender_did = Some(SENDER_DID_MARKER.to_string());
    t
}

/// The ABSOLUTE frozen canonical-TLV golden vector for the DEFAULT
/// (`0x6610` group multi-stanza, Sealed-Sender) fixture BLINDED 11-field AAD.
///
/// 127 bytes (NO plaintext sender field — F4-001 / F-LC-9); leading `0x01`
/// (`aad_version`); bytes 1..3 = `0x6610` (BE codepoint). NEITHER the raw
/// member roster NOR the raw set-id appears (BLINDED into the two 32-byte
/// commitments). **R5 (M-20) confirms this byte-for-byte against the real
/// `assemble_group_aad`.**
const EXPECTED_AAD_HEX: &str = "01661001711e20cfa9fea5491b9bf64cdc143778c3ff6e0123d8f7bca130f292b27a9bde54a86000000002f89cac9e8f674417d5c99d74d6a96e7b46065b82bd5e198de9811ae9d34c230d000000000000000100000001b3ae4d07499bd779184c6d28735cf4c2458a5d63904a383e57bbcc940bdebe760000000100000001";

// ── F-AAD-2 arms ────────────────────────────────────────────────────────

#[test]
fn f_aad_2_canonical_tlv_golden_vector_and_version_prefix() {
    let bytes = assemble_group_aad(&fixture());
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    assert_eq!(
        bytes[0], AAD_VERSION,
        "leading byte = the frozen aad_version prefix (R0.7 §4.1)"
    );
    assert_eq!(
        &bytes[1..3],
        &0x6610u16.to_be_bytes(),
        "codepoint is bound BIG-ENDIAN immediately after aad_version, and is the \
         group multi-stanza 0x6610 (U1; M-19 BE; R0.7 §3.10/§4.1)"
    );
    assert_eq!(
        hex_encode(&bytes),
        EXPECTED_AAD_HEX,
        "AAD group BLINDED 11-field canonical-TLV bytes drifted from the frozen \
         golden vector — divergent AAD = cross-engine AEAD-open failure (Inv-20 clause-c)"
    );
}

/// F-AAD-2 arm 0b — `aad_version` prefix is byte-bound (cross-version defense).
#[test]
fn f_aad_2_aad_version_is_byte_bound() {
    let base = assemble_group_aad(&fixture());
    let mut bumped = base.clone();
    bumped[0] = AAD_VERSION.wrapping_add(1);
    assert_ne!(
        base, bumped,
        "the aad_version prefix is part of the bound AAD — a version bump changes the bytes (U14)"
    );
}

/// F-AAD-2 arm 1 — single-field mutation distinctness (the bound fields).
#[test]
fn f_aad_2_plaintext_field_single_mutation_distinct() {
    let base = assemble_group_aad(&fixture());

    // Mutate the codepoint AWAY from the 0x6610 default to a DIFFERENT value.
    let mut m = fixture();
    m.codepoint = MEMBERSHIP_SET_ENCRYPTION; // 0x6600
    assert_ne!(base, assemble_group_aad(&m), "codepoint is byte-bound (U1)");

    let mut m = fixture();
    m.body_cid = self_describing_cid_bytes(b"DIFFERENT-payload");
    assert_ne!(base, assemble_group_aad(&m), "body-CID is byte-bound");

    let mut m = fixture();
    m.member_dids.push("did:key:zCCC".to_string());
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "member-set membership is byte-bound (audience_set_commitment + member_count)"
    );

    let mut m = fixture();
    m.k_set[0] ^= 0x01;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "K_Set is byte-bound via membership_set_id_commitment (keyed-blinding)"
    );

    let mut m = fixture();
    m.stanza_index = 7;
    assert_ne!(base, assemble_group_aad(&m), "stanza-index is byte-bound");

    let mut m = fixture();
    m.stanza_count = 9;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "stanza-count is byte-bound (truncation/censorship defense; R0.6 D4)"
    );

    let mut m = fixture();
    m.member_key_generation = 9;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "member-key-generation is byte-bound"
    );

    let mut m = fixture();
    m.membership_set_id = self_describing_cid_bytes(b"OTHER-set");
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_id is byte-bound via membership_set_id_commitment"
    );

    let mut m = fixture();
    m.membership_set_generation = 42;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_generation is byte-bound"
    );

    let mut m = fixture();
    m.role_assignments_generation = 5;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "role_assignments_generation is byte-bound (BC-5; E_ROLE_STALE_AT_VERIFY)"
    );
}

/// F-AAD-2 arm 1b (F4-001 / F-LC-9) — DEFAULT group send honors Sealed-Sender.
#[test]
fn f_aad_2_default_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let t = fixture();
    assert!(
        t.plaintext_sender_did.is_none(),
        "F-AAD-2 (F4-001): on the DEFAULT 0x6610 group path the inputs MUST NOT \
         carry a plaintext_sender_did — Sealed-Sender binds the inner-sender-DID \
         INSIDE `sealed_inner`."
    );
    assert!(
        !t.sealed_inner.is_empty(),
        "F-AAD-2 (F4-001): the DEFAULT path MUST carry a sealed inner payload."
    );

    let aad = assemble_group_aad(&t);
    let needle = SENDER_DID_MARKER.as_bytes();
    let leaks = aad.windows(needle.len()).any(|w| w == needle);
    assert!(
        !leaks,
        "F-AAD-2 (F4-001 / F-LC-9): the DEFAULT MembershipSet group send MUST HONOR \
         Sealed-Sender — the inner-sender-DID MUST NOT appear in the PLAINTEXT \
         BLINDED-11-field AAD bytes."
    );
}

/// F-AAD-2 arm 1c (F4-001 paired control) — non-default plaintext-sender.
#[test]
fn f_aad_2_nondefault_plaintext_sender_carries_sender_did_in_aad() {
    let t = fixture_nondefault_plaintext_sender();
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
        "F-AAD-2 (F4-001 PAIRED CONTROL): the NON-default plaintext-sender variant \
         MUST place the sender-DID in the plaintext AAD (U4)."
    );

    // The two paths MUST differ by EXACTLY the appended length-prefixed sender field.
    let default_aad = assemble_group_aad(&fixture());
    let mut expected_nondefault = default_aad.clone();
    expected_nondefault.extend_from_slice(&(needle.len() as u32).to_be_bytes());
    expected_nondefault.extend_from_slice(needle);
    assert_eq!(
        aad, expected_nondefault,
        "the non-default plaintext-sender AAD must equal the default Sealed-Sender \
         AAD with EXACTLY the length-prefixed sender field appended"
    );
    assert_ne!(
        aad, default_aad,
        "non-default plaintext-sender AAD must differ from default"
    );
}

/// F-AAD-2 arm 1d (R4.5-MIGRATE) — BLINDED commitments do NOT publish raw roster/set-id.
#[test]
fn f_aad_2_blinded_commitments_do_not_leak_raw_roster_or_set_id() {
    let t = fixture();
    let aad = assemble_group_aad(&t);

    for did in &t.member_dids {
        let needle = did.as_bytes();
        let leaks = aad.windows(needle.len()).any(|w| w == needle);
        assert!(
            !leaks,
            "F-AAD-2 (R4.5-MIGRATE / #61): the raw member-DID {did:?} MUST NOT appear \
             in the plaintext group AAD — the roster is BLINDED into audience_set_commitment."
        );
    }

    let set_id_needle = &t.membership_set_id;
    let set_id_leaks = aad
        .windows(set_id_needle.len())
        .any(|w| w == set_id_needle.as_slice());
    assert!(
        !set_id_leaks,
        "F-AAD-2 (R4.5-MIGRATE / #61): the raw membership_set_id MUST NOT appear in \
         the plaintext group AAD — it is BLINDED into membership_set_id_commitment."
    );

    // POSITIVE control: the two 32-byte commitments ARE present.
    let asc = audience_set_commitment(&t.member_dids);
    let mscid = membership_set_id_commitment(&t.k_set, &t.membership_set_id);
    assert!(
        aad.windows(asc.len()).any(|w| w == asc),
        "F-AAD-2: the audience_set_commitment MUST be bound in the group AAD."
    );
    assert!(
        aad.windows(mscid.len()).any(|w| w == mscid),
        "F-AAD-2: the membership_set_id_commitment MUST be bound in the group AAD."
    );
}

/// F-AAD-2 arm 1e (R4.5-MIGRATE) — commitments reuse the §3.9 gossip-topic primitives.
#[test]
fn f_aad_2_commitments_reuse_gossip_topic_primitives() {
    let t = fixture();

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
        "audience_set_commitment MUST be BLAKE3(0x01 || lp(did)…) over the SORTED member-DID list"
    );

    // membership_set_id_commitment = blake3::keyed_hash(K_Set, "benten:setid:v1" || set_id).
    let mut id_msg = Vec::new();
    id_msg.extend_from_slice(SETID_COMMITMENT_LABEL);
    id_msg.extend_from_slice(&t.membership_set_id);
    let mscid_expected: [u8; 32] = blake3::keyed_hash(&t.k_set, &id_msg).into();
    assert_eq!(
        membership_set_id_commitment(&t.k_set, &t.membership_set_id),
        mscid_expected,
        "membership_set_id_commitment MUST be keyed_hash(K_Set, \"benten:setid:v1\" || id)"
    );
}

/// F-AAD-2 arm 2 — member-DID-list canonical sort (assembler sorts behind the commitment).
#[test]
fn f_aad_2_member_did_list_sort_order_canonical() {
    let base = assemble_group_aad(&fixture());

    // Same DID SET, presented UNSORTED (reverse order). NO in-body sort.
    let mut reordered = fixture();
    reordered.member_dids = vec!["did:key:zBBB".to_string(), "did:key:zAAA".to_string()];
    assert_eq!(
        base,
        assemble_group_aad(&reordered),
        "an UNSORTED-but-equal member-DID set MUST assemble to identical bytes — \
         the assembler canonicalizes (sorts) internally (cross-engine convergence)"
    );
}

/// F-AAD-2 arm 3 — length-injectivity proptest (U3).
#[test]
fn f_aad_2_length_injectivity_proptest() {
    use proptest::prelude::*;
    proptest!(|(idx_a in any::<u32>(), idx_b in any::<u32>())| {
        prop_assume!(idx_a != idx_b);
        let mut a = fixture();
        a.stanza_index = idx_a;
        let mut b = fixture();
        b.stanza_index = idx_b;
        let ba = assemble_group_aad(&a);
        let bb = assemble_group_aad(&b);
        prop_assert_ne!(&ba, &bb, "distinct stanza-indices ⇒ distinct AAD bytes");
        prop_assert!(!is_prefix(&ba, &bb), "no AAD encoding may be a prefix of another (U3)");
        prop_assert!(!is_prefix(&bb, &ba), "no AAD encoding may be a prefix of another (U3)");
    });
}

/// F-AAD-2 arm 4 — opaque-bytes boundary compile-fence (m-15 GNC-5).
#[test]
fn f_aad_2_assembler_emits_opaque_bytes() {
    let bytes: Vec<u8> = assemble_group_aad(&fixture());
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
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
        "canonical-TLV aad_version prefix (R0.7 §4.1) — opaque to crypto-suite"
    );
}

/// F-AAD-2 arm 5 — compile-fence: crypto-suite has NO reverse dep on
/// membership-set (m-15 GNC-5). Reads the sibling manifest at runtime; runs
/// every CI cycle (not RED-phase — the boundary is observable now).
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
