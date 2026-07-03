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
//! Ben ratified **option (b)**: benten-drop OWNS the `0x6610` 11-field group-AAD
//! byte-assembly itself ([`benten_drop::layer_c::group_posture::assemble_group_aad_local`])
//! — there is NO production dependency on `benten-membership-set` (which carries a
//! native-only `benten-sync` edge that would invert drop's layering + contaminate its
//! wasm/freeze graph). A TEST-ONLY cross-check (the dev-dependency below) asserts the
//! local assembler reproduces the canonical
//! [`benten_membership_set::aad::assemble_group_aad`] bytes BYTE-FOR-BYTE, so the two
//! engines stay locked — single source of truth, zero drift. These arms PIN that:
//!
//! - **M-20 golden:** the live seal's bound per-stanza AAD reproduces the LOCAL
//!   `assemble_group_aad_local` bytes BYTE-FOR-BYTE AND equals the frozen absolute hex.
//!   If the live seal ever drifts back to the 6-field shape (or any field-order /
//!   endianness / width / blinding drift), the golden flips and cross-engine AEAD-open
//!   breaks.
//! - **ZERO-DRIFT cross-check (dev-dep, TEST-ONLY):** the local `assemble_group_aad_local`
//!   bytes equal the canonical `benten_membership_set::aad::assemble_group_aad` bytes
//!   byte-for-byte over the SAME inputs — so option (b)'s local copy can never silently
//!   diverge from the band-owner's canonical encoder.
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

use benten_crypto_suite::cipher_suite::{
    CipherSuite, CipherSuiteCodepoint, RecipientPublic, RecipientSecret,
};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_drop::layer_c::group_posture::{
    GroupAadInputs, GroupError, GroupSealParams, GroupVerifyContext,
    MEMBERSHIP_SET_GROUP_MULTI_STANZA, assemble_group_aad_local, open_membership_set_group,
    seal_membership_set_group,
};
use benten_drop::layer_c::{
    EncryptedEnvelope, LayerCError, group_roster_for_test, open_group_stanza, seal_group_multi,
};
use benten_id::did::Did;
// F-02 option-(b) TEST-ONLY cross-check (dev-dependency ONLY): the canonical
// band-owner encoder, aliased to keep it visibly distinct from benten-drop's
// LOCAL `assemble_group_aad_local`. Asserted byte-equal — zero drift.
use benten_membership_set::aad::{
    GroupAadInputs as CanonicalGroupAadInputs, assemble_group_aad as assemble_group_aad_canonical,
};

// ── Shared fixtures ─────────────────────────────────────────────────────────

/// Canonical fixture recipient SECRET seeds (R9 GAP-1). Each seed is the
/// recipient's PRIVATE seed for the deterministic-from-SECRET-seed KAT keypair;
/// the derived REAL public key drives the `did:key:z…` roster + the blinded
/// `audience_set_commitment` deterministically. (The old design paired
/// `sk = pk + 0x80` placeholder fingerprints — deleted with the GAP-1 fix.)
const FIXTURE_SEEDS: [u8; 3] = [0x10, 0x11, 0x12];

/// The deterministic REAL hybrid keypair for a fixture seed (secret seed → both
/// key halves via BLAKE3 expansion; `.public()`/`.secret()` genuinely match).
fn fixture_kp(seed: u8) -> benten_crypto_suite::cipher_suite::RecipientKeypair {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a wire-locked")
        .generate_recipient_keypair_deterministic(&[seed; 32])
}
/// The fixture recipient PUBLIC keys (the seal's `recipient_pubs` input).
fn fixture_pks() -> Vec<RecipientPublic> {
    FIXTURE_SEEDS
        .iter()
        .map(|&s| {
            RecipientPublic::from_bytes(
                CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
                &fixture_kp(s).public().to_bytes(),
            )
            .expect("re-parse of recipient public must succeed")
        })
        .collect()
}
/// The fixture recipient SECRET keys (the open path's `recipient_sec` input).
fn fixture_sks() -> Vec<RecipientSecret> {
    FIXTURE_SEEDS
        .iter()
        .map(|&s| {
            RecipientSecret::from_bytes(
                CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
                &fixture_kp(s).secret().to_bytes(),
            )
            .expect("re-parse of recipient secret must succeed")
        })
        .collect()
}
const FIXTURE_K_SET: [u8; 32] = [0x33u8; 32];
const FIXTURE_SET_ID: &[u8] = b"benten:set:test-membership-group";
const FIXTURE_PLAINTEXT: &[u8] = b"group payload";

/// B2 ORIGIN-AUTH helper: a real sender (LAMPS-hybrid keypair + matching
/// hybrid `did:key` bytes). The seal signs `M_auth` with the keypair; the open
/// resolves the did:key back + verifies. (The per-stanza AAD golden is
/// UNAFFECTED — the sender-DID is sealed inside `sealed_inner`, never in the
/// AAD — so the F-02 AAD goldens stay byte-identical.)
fn hybrid_sender() -> (SigKeypair, Vec<u8>) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did_str = Did::from_hybrid_public_key(&kp.public()).to_string();
    (kp, did_str.into_bytes())
}

/// The independently-held `GroupVerifyContext` over the fixture roster (all
/// generations = 1, matching `fixture_params`).
fn fixture_verify_ctx() -> GroupVerifyContext {
    let member_dids = group_roster_for_test(&fixture_pks())
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    GroupVerifyContext {
        member_dids,
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    }
}

fn fixture_params() -> GroupSealParams {
    GroupSealParams {
        membership_set_id: FIXTURE_SET_ID.to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    }
}

/// Re-derive the canonical `did:key:z…` roster the seal builds from the
/// recipient PUBLIC keys (mirrors `group_posture::group_roster` — BLAKE3 over
/// the REAL public key bytes; R9 GAP-1). Routed through the production
/// `group_roster_for_test` so it can never drift from the seal-side derivation.
fn fixture_roster() -> Vec<String> {
    group_roster_for_test(&fixture_pks())
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect()
}

/// The self-describing CIDv1 over the fixture body digest (`0x01 0x71 0x1e 0x20 ‖ blake3`).
fn fixture_body_cid() -> Vec<u8> {
    let digest = blake3::hash(FIXTURE_PLAINTEXT);
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(digest.as_bytes());
    cid
}

/// The LOCAL `GroupAadInputs` for stanza `idx` of the fixture send — the EXACT
/// inputs the live seal routes through the benten-drop-OWNED
/// [`assemble_group_aad_local`] (F-02 option-(b)).
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
        plaintext_sender_did: None,
    }
}

/// The CANONICAL (band-owner) `GroupAadInputs` for the SAME fixture stanza — the
/// dev-dep-only cross-check input. Field-for-field identical to the LOCAL inputs;
/// the cross-check asserts both encoders produce identical bytes (zero drift).
fn fixture_canonical_aad_inputs(idx: u32, stanza_count: u32) -> CanonicalGroupAadInputs {
    CanonicalGroupAadInputs {
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
/// `seal_membership_set_group` binds (= `assemble_group_aad_local(fixture_aad_inputs(0, 3))`).
/// **127 bytes** (NO plaintext sender field — F-LC-9; matches the membership-set
/// `f_aad_2` golden length). Layout (R0.7 §3.10/§4.1):
/// `aad_version u8 | codepoint(0x6610) u16 BE | body_cid (36B) | member_count u32 BE |
/// audience_set_commitment 32B | stanza_index u32 BE | stanza_count u32 BE |
/// member_key_generation u32 BE | membership_set_id_commitment 32B |
/// membership_set_generation u32 BE | role_assignments_generation u32 BE`.
/// Any drift in field-order / endianness / width / blinding / the version prefix /
/// the codepoint — or a regression back to the 6-field shape — flips this pin
/// (= cross-engine AEAD-open failure; Inv-20 clause-c).
///
/// R9 GAP-1: only the 32-byte `audience_set_commitment` component changed —
/// the recipient-key representation went placeholder(`[u8; 32]`) → real hybrid
/// pubkey, so the roster DIDs (`BLAKE3(label ‖ pk.to_bytes())`) — and thus the
/// blinded commitment over them — recompute. The AAD SHAPE + field-set +
/// field-order + widths + endianness + blinding + version-prefix + codepoint
/// are UNCHANGED (every other byte of this golden is byte-identical to the
/// pre-fix value; the `member_count`, `body_cid`, generations, set-id
/// commitment, index/count all match). The cross-engine byte-equality arm
/// (`f_02_local_assembler_matches_canonical_membership_set_byte_for_byte`) and
/// the seal→open round-trip arm (`f_02_seal_open_round_trip_under_11_field_aad`)
/// still pass — the real interop contract is intact. This is a fixture
/// regeneration forced by the GAP-1 fix, NOT a wire-freeze mutation (no data
/// ever shipped under the placeholder keying).
const F_02_LIVE_SEAL_STANZA0_AAD_HEX: &str = "01661001711e20632048ee454f9854f70d9ea7e52f27518b11f0d610140a1bcedd9b9b34e38c97000000039c283693545dc213ff8f97cbc3a8cc7af4e12c769697b495b174016166e0d2b80000000000000003000000013d7ae18fc21b0ad50fa86ad1a620ed2d343a191abd7e08525ad1d9a8849d71f50000000100000001";

/// F-02 arm 1 — the live seal binds the LOCAL 11-field AAD BYTE-FOR-BYTE.
///
/// Two independent assertions: (a) the live seal's stanza-0 AAD reproduces the
/// benten-drop-LOCAL `assemble_group_aad_local` bytes (the production source of
/// truth under F-02 option-(b)), and (b) it equals the frozen absolute golden hex
/// (M-20). would-FAIL if the live seal regressed to the 6-field shape or drifted
/// in field-order/width/blinding. (The zero-drift cross-check that the LOCAL bytes
/// equal the canonical band-owner bytes is a SEPARATE dev-dep-only arm below.)
#[test]
fn f_02_live_0x6610_seal_binds_canonical_11_field_aad_golden() {
    let (sender_kp, sender) = hybrid_sender();
    let env = seal_membership_set_group(
        &fixture_pks(),
        &sender,
        &sender_kp,
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );
    let live_aad = env.stanza_aad_for_test(0);

    // (a) single-source-of-truth: live seal == benten-drop-LOCAL assembler.
    let local = assemble_group_aad_local(&fixture_aad_inputs(0, 3));
    assert_eq!(
        to_hex(&live_aad),
        to_hex(&local),
        "F-02: the live 0x6610 seal MUST route its per-stanza AAD through the \
         benten-drop-LOCAL assemble_group_aad_local (the BLINDED 11-field set) — \
         single source of truth (F-02 option-(b)). A divergent encoder = \
         cross-engine AEAD-open failure."
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
    let base = assemble_group_aad_local(&fixture_aad_inputs(0, 3));

    // member_key_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.member_key_generation = 2;
    assert_ne!(
        base,
        assemble_group_aad_local(&m),
        "member_key_generation is byte-bound"
    );

    // membership_set_id (blinded into membership_set_id_commitment)
    let mut m = fixture_aad_inputs(0, 3);
    m.membership_set_id = b"benten:set:DIFFERENT".to_vec();
    assert_ne!(
        base,
        assemble_group_aad_local(&m),
        "membership_set_id is byte-bound (blinded)"
    );

    // membership_set_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.membership_set_generation = 7;
    assert_ne!(
        base,
        assemble_group_aad_local(&m),
        "membership_set_generation is byte-bound"
    );

    // role_assignments_generation
    let mut m = fixture_aad_inputs(0, 3);
    m.role_assignments_generation = 9;
    assert_ne!(
        base,
        assemble_group_aad_local(&m),
        "role_assignments_generation is byte-bound"
    );

    // member roster (flips member_count + audience_set_commitment)
    let mut m = fixture_aad_inputs(0, 3);
    m.member_dids.push("did:key:zEXTRA".to_string());
    assert_ne!(
        base,
        assemble_group_aad_local(&m),
        "the member roster is byte-bound (member_count + audience_set_commitment)"
    );
}

/// F-02 arm 1b (ZERO-DRIFT cross-check, dev-dep TEST-ONLY) — benten-drop's LOCAL
/// `assemble_group_aad_local` reproduces the canonical band-owner
/// `benten_membership_set::aad::assemble_group_aad` bytes BYTE-FOR-BYTE over the
/// SAME inputs.
///
/// This is the load-bearing safety net for F-02 option-(b): benten-drop owns a
/// LOCAL copy of the 11-field encoder (so its production tree stays sync-free),
/// and this arm guarantees the local copy can NEVER silently diverge from the
/// canonical encoder. If a future edit to either assembler changes field-order /
/// width / endianness / blinding / the version-prefix / codepoint on one side
/// only, this byte-equality flips — surfacing the drift at test time. The
/// cross-check runs over several stanza positions + a multi-recipient roster so a
/// per-stanza index/count divergence is also caught.
#[test]
fn f_02_local_assembler_matches_canonical_membership_set_byte_for_byte() {
    for (idx, count) in [(0u32, 3u32), (1, 3), (2, 3), (0, 1)] {
        let local = assemble_group_aad_local(&fixture_aad_inputs(idx, count));
        let canonical = assemble_group_aad_canonical(&fixture_canonical_aad_inputs(idx, count));
        assert_eq!(
            to_hex(&local),
            to_hex(&canonical),
            "F-02 (option-(b) zero-drift): benten-drop's LOCAL assemble_group_aad_local \
             MUST byte-match the canonical benten_membership_set::aad::assemble_group_aad \
             (stanza_index={idx}, stanza_count={count}). A divergence = the two engines \
             would compute different AADs for the same group send = cross-engine \
             AEAD-open failure."
        );
    }
}

/// F-04 (R12) — the FROZEN leading wire byte `AAD_VERSION` is byte-mirrored
/// across the two engines. The `f_02_local_assembler_matches_..._byte_for_byte`
/// arm proves the two ASSEMBLERS agree over the same inputs, but both sides read
/// their OWN `AAD_VERSION` const — a simultaneous value drift in BOTH crates'
/// `AAD_VERSION` would keep the assemblers byte-equal to each other while
/// silently changing the on-the-wire format. This pin closes that gap: the two
/// crates' `AAD_VERSION` constants MUST be byte-equal to each other AND equal the
/// frozen `0x01` (R0.7 §4.1; the dedicated AAD-prefix byte, DISTINCT from the
/// envelope serialization-format byte). Same shape as the `domain_registry_mirror`
/// cross-crate byte-equality pins.
#[test]
fn f_04_aad_version_byte_mirrors_across_engines() {
    assert_eq!(
        benten_drop::layer_c::AAD_VERSION,
        benten_membership_set::aad::AAD_VERSION,
        "F-04: benten-drop's layer_c::AAD_VERSION drifted from the canonical \
         benten_membership_set::aad::AAD_VERSION — a simultaneous both-sides drift \
         would keep the AAD assemblers byte-equal while changing the wire format."
    );
    assert_eq!(
        benten_drop::layer_c::AAD_VERSION,
        0x01,
        "F-04: AAD_VERSION is the FROZEN dedicated AAD-prefix byte 0x01 (R0.7 §4.1)."
    );
}

/// F-02 arm 3 — a real seal→open round-trip succeeds (the open path reconstructs
/// the SAME 11-field AAD). A seal-vs-open AAD mismatch would make EVERY group
/// decrypt fail; this arm proves the 11-field change is consistent across seal+open.
#[test]
fn f_02_seal_open_round_trip_under_11_field_aad() {
    let (sender_kp, sender) = hybrid_sender();
    let sks = fixture_sks();
    let env = seal_membership_set_group(
        &fixture_pks(),
        &sender,
        &sender_kp,
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );
    let (pt, recovered_sender) = open_membership_set_group(&sks[1], 1, &fixture_verify_ctx(), &env)
        .expect("F-02: the 0x6610 group stanza MUST open + origin-verify under the 11-field AAD");
    assert_eq!(
        pt, FIXTURE_PLAINTEXT,
        "F-02: round-trip plaintext preserved"
    );
    assert_eq!(
        recovered_sender, sender,
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
    let (sender_kp, sender) = hybrid_sender();
    let ctx = fixture_verify_ctx();
    let sks = fixture_sks();
    let env = seal_membership_set_group(
        &fixture_pks(),
        &sender,
        &sender_kp,
        &FIXTURE_K_SET,
        &fixture_params(),
        FIXTURE_PLAINTEXT,
    );

    // Sanity: untruncated, the survivor at index 1 opens fine (so the FAIL below
    // is the count check firing, not an unrelated decrypt failure — this is the
    // would-FAIL-on-revert witness: revert the check and the truncated open
    // would behave just like this Ok).
    assert!(
        open_membership_set_group(&sks[1], 1, &ctx, &env).is_ok(),
        "pre-condition: the index-1 survivor opens fine on the FULL envelope"
    );

    let truncated = env.with_last_stanza_dropped_for_test();
    assert_eq!(truncated.stanza_len_for_test(), 2, "one stanza was dropped");
    assert_eq!(
        truncated.bound_stanza_count_for_test(),
        3,
        "the bound stanza_count is UNCHANGED (the relay only dropped wire stanzas)"
    );

    let outcome = open_membership_set_group(&sks[1], 1, &ctx, &truncated);
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
    let (sender_kp, sender) = hybrid_sender();
    let pks = fixture_pks();
    let sks = fixture_sks();
    let roster = group_roster_for_test(&pks);
    let env = seal_group_multi(
        &pks,
        &sender,
        &sender_kp,
        &body_cid,
        /* recipient_key_generation = */ 1,
        FIXTURE_PLAINTEXT,
    )
    .expect("group seal within recipient limit");

    // Pre-condition: the index-1 survivor opens fine on the FULL envelope.
    assert!(
        open_group_stanza(&sks[1], 1, &roster, 1, &env).is_ok(),
        "pre-condition: the index-1 survivor opens fine on the FULL 0x6520 envelope"
    );

    let truncated = drop_last_0x6520_stanza(&env);
    let outcome = open_group_stanza(&sks[1], 1, &roster, 1, &truncated);
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
