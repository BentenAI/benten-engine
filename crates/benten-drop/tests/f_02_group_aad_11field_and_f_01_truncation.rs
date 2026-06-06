//! F-02 + F-01 — the `0x6610`/`0x6520` group encrypt-to-recipient path
//! (Layer-C / MembershipSet) Ben-ratified R6 corrections.
//!
//! # F-02 — the live `0x6610` group seal binds the canonical BLINDED 11-field AAD
//!
//! R6 found TWO divergent encoders: the LIVE `group_posture::seal_membership_set_group`
//! seal bound only a **6-field** per-stanza AAD (`aad_version | codepoint | body_cid |
//! audience_set_commitment | stanza_index | stanza_count`), while the golden
//! (`benten-membership-set`'s `f_aad_2`) + `docs/SECURITY-PROOFS.md` §3.3 specify the
//! **BLINDED 11-field set** (adding `member_count`, `member_key_generation`,
//! `membership_set_id_commitment`, `membership_set_generation`,
//! `role_assignments_generation`, and the canonical field ORDER). Ben ratified: the
//! 11-field set is canonical.
//!
//! The fix routes the live seal's per-stanza AAD through the canonical
//! [`benten_membership_set::aad::assemble_group_aad`] — single source of truth, zero
//! drift. These arms PIN that:
//!
//! - **M-20 golden:** the live seal's bound per-stanza AAD reproduces the canonical
//!   `assemble_group_aad` bytes BYTE-FOR-BYTE (the absolute hex is frozen). If the live
//!   seal ever drifts back to the 6-field shape (or any field-order / endianness /
//!   width / blinding drift), the golden flips and cross-engine AEAD-open breaks.
//! - **Round-trip:** a real seal→open succeeds (open reconstructs the SAME 11-field AAD;
//!   a seal-vs-open AAD mismatch would make EVERY group decrypt fail).
//! - **The 5 new fields are LOAD-BEARING:** mutating any one of them flips the AAD (so
//!   the seal genuinely binds the full 11-field set, not a 6-field subset relabelled).
//!
//! # F-01 — group truncation/censorship defense (the delivered-vs-bound stanza-count check)
//!
//! `docs/SECURITY-PROOFS.md` §3.3/§4.1 + `THREAT-MODEL.md` claim a delivered-stanzas
//! vs bound-`stanza_count` check is DELIVERED on the group open paths, but the code
//! lacked it: each surviving stanza's AAD binds `stanza_count`, yet nothing compared the
//! DELIVERED count to the bound count, so a relay could silently drop trailing stanzas to
//! censor a co-recipient and the survivors would still open fine. The fix adds the check
//! on BOTH group open paths (`0x6610` `open_membership_set_group` + `0x6520`
//! `open_group_stanza`); it fails closed (typed error) on a mismatch.
//!
//! The negative arms construct a valid group seal, DROP a stanza (the exact relay
//! truncation), and assert the open FAILS CLOSED — substantive, not a tautology.
//! **would-FAIL-on-revert:** reverting the count check makes the truncated open
//! PASS THROUGH (the survivor's own stanza still authenticates), which is precisely
//! the censorship the check exists to stop.

#![allow(clippy::unwrap_used)]

use benten_drop::layer_c::group_posture::{
    GroupError, GroupSealParams, MEMBERSHIP_SET_GROUP_MULTI_STANZA, open_membership_set_group,
    seal_membership_set_group,
};
use benten_drop::layer_c::{
    EncryptedEnvelope, LayerCError, RecipientPubKey, open_group_stanza, seal_group_multi,
};
use benten_membership_set::aad::{GroupAadInputs, assemble_group_aad};

// ── Shared fixtures ─────────────────────────────────────────────────────────

/// Canonical fixture recipient pubkey fingerprints (deterministic → the derived
/// `did:key:z…` roster + the blinded `audience_set_commitment` are deterministic).
const FIXTURE_PKS: [RecipientPubKey; 3] = [[0x10u8; 32], [0x11u8; 32], [0x12u8; 32]];
/// Paired secret fingerprints (the open path expands `sk - 0x80` back to the pk).
const FIXTURE_SKS: [[u8; 32]; 3] = [[0x90u8; 32], [0x91u8; 32], [0x92u8; 32]];
const FIXTURE_K_SET: [u8; 32] = [0x33u8; 32];
const FIXTURE_SET_ID: &[u8] = b"benten:set:test-membership-group";
const FIXTURE_SENDER: &[u8] = b"did:key:zGroupSenderUNIQUEMARKER";
const FIXTURE_PLAINTEXT: &[u8] = b"group payload";

fn fixture_params() -> GroupSealParams {
    GroupSealParams {
        membership_set_id: FIXTURE_SET_ID.to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    }
}

/// Re-derive the canonical `did:key:z…` roster the seal builds from the pubkey
/// fingerprints (mirrors `group_posture::group_roster` — BLAKE3 over the pubkey).
fn fixture_roster() -> Vec<String> {
    FIXTURE_PKS
        .iter()
        .map(|pk| {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-drop:layer-c:recipient-did");
            h.update(pk);
            let d = h.finalize();
            let mut did = b"did:key:z".to_vec();
            did.extend_from_slice(d.as_bytes());
            // The roster bytes are non-UTF8 (raw BLAKE3 after the ASCII prefix);
            // the seal does the same lossy round-trip, so this matches byte-faithfully.
            String::from_utf8_lossy(&did).into_owned()
        })
        .collect()
}

/// The self-describing CIDv1 over the fixture body digest (`0x01 0x71 0x1e 0x20 ‖ blake3`).
fn fixture_body_cid() -> Vec<u8> {
    let digest = blake3::hash(FIXTURE_PLAINTEXT);
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(digest.as_bytes());
    cid
}

/// The canonical `GroupAadInputs` for stanza `idx` of the fixture send — the
/// EXACT inputs the live seal routes through `assemble_group_aad`.
fn fixture_aad_inputs(idx: u32, stanza_count: u32) -> GroupAadInputs {
    GroupAadInputs {
        codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        body_cid: fixture_body_cid(),
        member_dids: fixture_roster(),
        k_set: FIXTURE_K_SET,
        stanza_index: idx,
        stanza_count,
        member_key_generation: 1,
        membership_set_id: FIXTURE_SET_ID.to_vec(),
        membership_set_generation: 1,
        role_assignments_generation: 1,
        sealed_inner: Vec::new(),
        plaintext_sender_did: None,
    }
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

// ── F-02 ────────────────────────────────────────────────────────────────────

/// The ABSOLUTE frozen golden vector for the live `0x6610` seal's per-stanza AAD
/// (stanza_index = 0, stanza_count = 3) over the canonical fixture — the BLINDED
/// 11-field set. Computed off-line (M-20) from the SAME bytes the live
/// `seal_membership_set_group` binds (= `assemble_group_aad(fixture_aad_inputs(0, 3))`).
/// **127 bytes** (NO plaintext sender field — F-LC-9; matches the membership-set
/// `f_aad_2` golden length). Layout (R0.7 §3.10/§4.1):
/// `aad_version u8 | codepoint(0x6610) u16 BE | body_cid (36B) | member_count u32 BE |
/// audience_set_commitment 32B | stanza_index u32 BE | stanza_count u32 BE |
/// member_key_generation u32 BE | membership_set_id_commitment 32B |
/// membership_set_generation u32 BE | role_assignments_generation u32 BE`.
/// Any drift in field-order / endianness / width / blinding / the version prefix /
/// the codepoint — or a regression back to the 6-field shape — flips this pin
/// (= cross-engine AEAD-open failure; Inv-20 clause-c).
const F_02_LIVE_SEAL_STANZA0_AAD_HEX: &str = "01661001711e20632048ee454f9854f70d9ea7e52f27518b11f0d610140a1bcedd9b9b34e38c970000000371ff0a9870f21ac454ec95a9f4a9c9600bd2be1d190a55016d7338bd63f7475d0000000000000003000000013d7ae18fc21b0ad50fa86ad1a620ed2d343a191abd7e08525ad1d9a8849d71f50000000100000001";

/// F-02 arm 1 — the live seal binds the canonical 11-field AAD BYTE-FOR-BYTE.
///
/// Two independent assertions: (a) the live seal's stanza-0 AAD reproduces the
/// canonical `assemble_group_aad` bytes (single source of truth), and (b) it equals
/// the frozen absolute golden hex (M-20). would-FAIL if the live seal regressed to
/// the 6-field shape or drifted in field-order/width/blinding.
#[test]
fn f_02_live_0x6610_seal_binds_canonical_11_field_aad_golden() {
    let env = seal_membership_set_group(
        &FIXTURE_PKS,
        &FIXTURE_SENDER.to_vec(),
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );
    let live_aad = env.stanza_aad_for_test(0);

    // (a) single-source-of-truth: live seal == canonical assemble_group_aad.
    let canonical = assemble_group_aad(&fixture_aad_inputs(0, 3));
    assert_eq!(
        to_hex(&live_aad),
        to_hex(&canonical),
        "F-02: the live 0x6610 seal MUST route its per-stanza AAD through the \
         canonical assemble_group_aad (the BLINDED 11-field set) — single source \
         of truth. A divergent encoder = cross-engine AEAD-open failure."
    );

    // (b) M-20 absolute golden — the 11-field set is 127 bytes (NO plaintext sender):
    // 1 (aad_version) + 2 (codepoint) + 36 (body_cid) + 4 (member_count) +
    // 32 (audience_set_commitment) + 4 (stanza_index) + 4 (stanza_count) +
    // 4 (member_key_generation) + 32 (membership_set_id_commitment) +
    // 4 (membership_set_generation) + 4 (role_assignments_generation) = 127.
    // (The old 6-field shape was 79 bytes; a length regression means a field
    // was dropped.) This matches the membership-set `f_aad_2` golden length.
    assert_eq!(
        live_aad.len(),
        127,
        "F-02: the BLINDED 11-field AAD is 127 bytes; the old 6-field shape was 79."
    );
    assert_eq!(
        &live_aad[0..3],
        &[0x01, 0x66, 0x10],
        "F-02: aad_version(0x01) ‖ codepoint(0x6610 BE) lead the canonical AAD."
    );
    assert_eq!(
        to_hex(&live_aad),
        F_02_LIVE_SEAL_STANZA0_AAD_HEX,
        "F-02: the live 0x6610 seal per-stanza AAD drifted from the frozen \
         BLINDED-11-field golden vector (M-20)."
    );
}

/// F-02 arm 2 — the 5 NEW (vs the old 6-field) bound fields are LOAD-BEARING.
///
/// Mutating any of `member_count`-input (roster), `member_key_generation`,
/// `membership_set_id`, `membership_set_generation`, `role_assignments_generation`
/// flips the assembled AAD — proving the live seal binds the FULL 11-field set, not
/// a 6-field subset with new fields ignored. (The roster + set-id are blinded, so the
/// mutation flips one of the two 32-byte commitments.)
#[test]
fn f_02_new_11_field_members_are_load_bearing() {
    let base = assemble_group_aad(&fixture_aad_inputs(0, 3));

    // member_key_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.member_key_generation = 2;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "member_key_generation is byte-bound"
    );

    // membership_set_id (blinded into membership_set_id_commitment)
    let mut m = fixture_aad_inputs(0, 3);
    m.membership_set_id = b"benten:set:DIFFERENT".to_vec();
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_id is byte-bound (blinded)"
    );

    // membership_set_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.membership_set_generation = 7;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "membership_set_generation is byte-bound"
    );

    // role_assignments_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.role_assignments_generation = 9;
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "role_assignments_generation is byte-bound"
    );

    // member roster (flips member_count + audience_set_commitment)
    let mut m = fixture_aad_inputs(0, 3);
    m.member_dids.push("did:key:zEXTRA".to_string());
    assert_ne!(
        base,
        assemble_group_aad(&m),
        "the member roster is byte-bound (member_count + audience_set_commitment)"
    );
}

/// F-02 arm 3 — a real seal→open round-trip succeeds (the open path reconstructs
/// the SAME 11-field AAD). A seal-vs-open AAD mismatch would make EVERY group
/// decrypt fail; this arm proves the 11-field change is consistent across seal+open.
#[test]
fn f_02_seal_open_round_trip_under_11_field_aad() {
    let env = seal_membership_set_group(
        &FIXTURE_PKS,
        &FIXTURE_SENDER.to_vec(),
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );
    let (pt, recovered_sender) = open_membership_set_group(&FIXTURE_SKS[1], 1, &env)
        .expect("F-02: the 0x6610 group stanza MUST open under the 11-field AAD");
    assert_eq!(
        pt, FIXTURE_PLAINTEXT,
        "F-02: round-trip plaintext preserved"
    );
    assert_eq!(
        recovered_sender,
        FIXTURE_SENDER.to_vec(),
        "F-02: the inner-sender-DID is recovered post-decrypt (Sealed-Sender honored)"
    );
}

// ── F-01 (0x6610 path) ───────────────────────────────────────────────────────

/// F-01 arm 1 (`0x6610`) — dropping a stanza makes the open FAIL CLOSED.
///
/// Construct a valid 3-recipient group seal, then DROP the last stanza WITHOUT
/// touching the bound `stanza_count` — exactly an active relay censoring a
/// co-recipient. The open MUST fail closed with [`GroupError::StanzaCountMismatch`].
///
/// **would-FAIL-on-revert:** the survivor at index 1 STILL authenticates its own
/// stanza (its AAD binds index 1 of 3, unchanged), so without the
/// `delivered != bound` check the open returns Ok(plaintext) — the censorship
/// succeeds silently. The arm below proves the survivor would otherwise open by
/// FIRST opening the full (untruncated) envelope successfully.
#[test]
fn f_01_0x6610_dropped_stanza_fails_closed() {
    let env = seal_membership_set_group(
        &FIXTURE_PKS,
        &FIXTURE_SENDER.to_vec(),
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );

    // Sanity: untruncated, the survivor at index 1 opens fine (so the FAIL below
    // is the count check firing, not an unrelated decrypt failure — this is the
    // would-FAIL-on-revert witness: revert the check and the truncated open
    // would behave just like this Ok).
    assert!(
        open_membership_set_group(&FIXTURE_SKS[1], 1, &env).is_ok(),
        "pre-condition: the index-1 survivor opens fine on the FULL envelope"
    );

    let truncated = env.with_last_stanza_dropped_for_test();
    assert_eq!(truncated.stanza_len_for_test(), 2, "one stanza was dropped");
    assert_eq!(
        truncated.bound_stanza_count_for_test(),
        3,
        "the bound stanza_count is UNCHANGED (the relay only dropped wire stanzas)"
    );

    let outcome = open_membership_set_group(&FIXTURE_SKS[1], 1, &truncated);
    assert_eq!(
        outcome,
        Err(GroupError::StanzaCountMismatch {
            delivered: 2,
            bound: 3,
        }),
        "F-01 (0x6610): a truncated group envelope MUST fail closed — \
         delivered (2) != bound stanza_count (3). would-FAIL on revert: the \
         survivor authenticates its own stanza, so without the count check the \
         censored open returns Ok. Got: {outcome:?}"
    );
}

// ── F-01 (0x6520 path) ───────────────────────────────────────────────────────

/// Extract a truncated `0x6520` `HpkeMultiBase` envelope (drop the last stanza,
/// leave the per-stanza `stanza_count` UNCHANGED — the relay-truncation model).
fn drop_last_0x6520_stanza(env: &EncryptedEnvelope) -> EncryptedEnvelope {
    let EncryptedEnvelope::HpkeMultiBase {
        format_version,
        cek_aead_ciphertext,
        cek_aead_nonce,
        stanzas,
    } = env
    else {
        panic!("expected HpkeMultiBase");
    };
    let mut stanzas = stanzas.clone();
    stanzas.pop();
    EncryptedEnvelope::HpkeMultiBase {
        format_version: *format_version,
        cek_aead_ciphertext: cek_aead_ciphertext.clone(),
        cek_aead_nonce: *cek_aead_nonce,
        stanzas,
    }
}

/// F-01 arm 2 (`0x6520`) — dropping a stanza makes the Layer-C group open FAIL
/// CLOSED with [`LayerCError::StanzaCountMismatch`].
///
/// Same structure as the `0x6610` arm: a valid 3-recipient `0x6520` group seal,
/// last stanza dropped, the index-1 survivor's open must fail closed.
/// would-FAIL-on-revert is witnessed by the FULL-envelope open succeeding first.
#[test]
fn f_01_0x6520_dropped_stanza_fails_closed() {
    let body_cid = *blake3::hash(FIXTURE_PLAINTEXT).as_bytes();
    let env = seal_group_multi(
        &FIXTURE_PKS,
        &FIXTURE_SENDER.to_vec(),
        &body_cid,
        /* recipient_key_generation = */ 1,
        FIXTURE_PLAINTEXT,
    );

    // Pre-condition: the index-1 survivor opens fine on the FULL envelope.
    assert!(
        open_group_stanza(&FIXTURE_SKS[1], 1, &env).is_ok(),
        "pre-condition: the index-1 survivor opens fine on the FULL 0x6520 envelope"
    );

    let truncated = drop_last_0x6520_stanza(&env);
    let outcome = open_group_stanza(&FIXTURE_SKS[1], 1, &truncated);
    assert_eq!(
        outcome,
        Err(LayerCError::StanzaCountMismatch {
            delivered: 2,
            bound: 3,
        }),
        "F-01 (0x6520): a truncated Layer-C group envelope MUST fail closed — \
         delivered (2) != bound stanza_count (3). would-FAIL on revert: the \
         survivor authenticates its own stanza, so without the count check the \
         censored open returns Ok. Got: {outcome:?}"
    );
}
