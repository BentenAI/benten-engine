//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pins for the
//! encryption-envelope-per-codepoint table (Rows D-9 / D-79).
//!
//! Row D-9 names "encryption-envelope per codepoint" as a MISSING byte
//! pin: one canonical hex per frozen encryption codepoint. The Layer-C
//! envelope codepoint band lives in `benten_drop::layer_c`:
//!
//! - `0x6500` `LAYER_C_DROP` (plaintext-sender, non-default)
//! - `0x6510` `DROP_TO_RECIPIENT_SEALED_SENDER` (v1-beta DEFAULT)
//! - `0x6520` `LAYER_C_DROP_MULTI_RECIPIENT` (group multi-stanza)
//! - `0x6610` MembershipSet group multi-stanza
//! - `0x647a` `HYBRID_X25519_MLKEM768` (the cipher codepoint — the
//!   ChaCha20-Poly1305-under-X-Wing AEAD envelope)
//!
//! # What surface pins each codepoint
//!
//! The codepoint is bound BIG-ENDIAN into the sender-auth binding message
//! `M_auth` (`build_m_auth`), so pinning `build_m_auth` over a fixed
//! `SenderAuthBinding` with each `envelope_codepoint` gives ONE canonical
//! hex per band — the same function seal + verify both run, so a drift in
//! the codepoint-binding position/endianness fails the pin. The 0x6510
//! DEFAULT band ALSO gets its on-wire AAD serializer pinned
//! (`sealed_aad::serialize_sealed_sender_aad`). The 0x647a cipher
//! codepoint is pinned at `AeadEnvelope::to_wire_bytes` (BE codepoint,
//! M-19).
//!
//! **D-43:** the `0x6500` plaintext-sender band now carries its on-wire row
//! too — via the PRODUCTION encoder `BindingContext::plaintext_aad_bytes`
//! (not a test-only mirror). Previously this table pinned `0x6500` only at
//! `M_auth`, so the table's "one hex per codepoint" claim was one row short
//! on the on-wire axis.
//!
//! **GCS-24:** the NON-DEFAULT plaintext-sender AAD trailer carries a
//! *band-specific* `sender_did` length-prefix WIDTH (`0x6500` u16-BE,
//! `0x6520` u16-BE, `0x6610` u32-BE — Row D-42 / C-07). Those trailers are
//! one-way AAD: both seal and open RECOMPUTE them from their own structs and
//! nothing ever parses them back, so a width flip is bilaterally consistent
//! and every round-trip stays green. The `by_band_*` arms below therefore pin
//! the width STRUCTURALLY (the byte-delta between the sender-bearing AAD and
//! its sender-less prefix is exactly `width + len`) as well as absolutely.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! Every surface here is a pure deterministic serializer over FIXED
//! inputs (fixed body-CID, fixed sender-DID, fixed commitments, fixed
//! nonce). Hex captured via throwaway compute + pasted below.
//!
//! would-FAIL-on-drift: a codepoint value change, a BE→LE endianness
//! flip, or a field-order change all change these bytes.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_crypto_suite::aead::AeadEnvelope;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_drop::layer_c::group_posture::{GroupAadInputs, assemble_group_aad_local};
use benten_drop::layer_c::{
    AAD_VERSION, BindingContext, HpkeRecipientStanza, LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT,
    SENDER_AUTH_SIG_CODEPOINT, SenderAuthBinding, build_m_auth, sealed_aad,
};

/// The MembershipSet group multi-stanza envelope codepoint (0x6610).
const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;
/// The Sealed-Sender DEFAULT codepoint (0x6510).
const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Fixed canonical 36-byte CIDv1 (self-describing) for the body_cid.
fn fixed_body_cid_bytes() -> Vec<u8> {
    Cid::from_blake3_digest([0xCC; 32]).as_bytes().to_vec()
}

/// A fixed `SenderAuthBinding` with the given envelope codepoint. All
/// other fields fixed so the only variation across the table rows is the
/// `envelope_codepoint` (isolating the codepoint-binding freeze).
fn fixed_m_auth_hex(envelope_codepoint: u16) -> String {
    let body_cid = fixed_body_cid_bytes();
    let sender_did = b"did:key:zFixedSenderForBytePin".to_vec();
    let audience_commitment = vec![0xBB; 32];
    let generations: Vec<u32> = vec![7];
    let binding = SenderAuthBinding {
        sig_codepoint: SENDER_AUTH_SIG_CODEPOINT,
        envelope_codepoint,
        sender_did: &sender_did,
        body_cid: &body_cid,
        audience_commitment: &audience_commitment,
        generations: &generations,
        stanza_count: 3,
        body_aad_digest: [0xEE; 32],
    };
    to_hex(&build_m_auth(&binding))
}

/// Encryption-envelope per codepoint — `M_auth` codepoint-binding golden
/// hex for each Layer-C band. One canonical hex per envelope codepoint.
#[test]
fn encryption_envelope_m_auth_per_codepoint_golden_hex() {
    // The rows differ ONLY in the 2-byte envelope_codepoint (BE) bound
    // after `SENDER_AUTH_DOMAIN || sig_codepoint(0x0001)`.
    // 0x6500 — plaintext-sender drop (non-default).
    assert_eq!(
        fixed_m_auth_hex(LAYER_C_DROP),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165000000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6500 M_auth codepoint-binding drifted"
    );
    // 0x6510 — Sealed-Sender DEFAULT.
    assert_eq!(
        fixed_m_auth_hex(DROP_TO_RECIPIENT_SEALED_SENDER),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165100000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6510 M_auth codepoint-binding drifted"
    );
    // 0x6520 — group multi-recipient.
    assert_eq!(
        fixed_m_auth_hex(LAYER_C_DROP_MULTI_RECIPIENT),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165200000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6520 M_auth codepoint-binding drifted"
    );
    // 0x6610 — MembershipSet group multi-stanza.
    assert_eq!(
        fixed_m_auth_hex(MEMBERSHIP_SET_GROUP_MULTI_STANZA),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000166100000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6610 M_auth codepoint-binding drifted"
    );
}

/// The DEFAULT 0x6510 on-wire envelope AAD serializer — ABSOLUTE golden
/// hex (`aad_version u8 | codepoint u16-BE | aud_len u32-BE | audience |
/// body_cid(36) | recipient_key_gen u32-BE`).
#[test]
fn sealed_sender_0x6510_on_wire_aad_golden_hex() {
    let aad = sealed_aad::SealedSenderAad {
        aad_version: sealed_aad::AAD_VERSION,
        codepoint: sealed_aad::DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: b"did:key:zFixedAudienceForPin".to_vec(),
        body_cid: fixed_body_cid_bytes(),
        recipient_key_generation: 9,
    };
    let got = to_hex(&sealed_aad::serialize_sealed_sender_aad(&aad));
    assert_eq!(
        got,
        "0165100000001c6469643a6b65793a7a466978656441756469656e6365466f7250696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000009",
        "0x6510 on-wire Sealed-Sender AAD framing drifted"
    );
}

/// The 0x647a cipher codepoint — `AeadEnvelope::to_wire_bytes` framing
/// (the ChaCha20-Poly1305-under-X-Wing envelope). Pins the BE codepoint
/// (M-19) at the encryption-envelope layer.
#[test]
fn cipher_codepoint_0x647a_envelope_golden_hex() {
    let env = AeadEnvelope {
        format_version: 0x01,
        cipher_codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        nonce: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ],
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };
    let got = to_hex(&env.to_wire_bytes());
    assert_eq!(
        got, "ae01647a0c0102030405060708090a0b0cdeadbeef",
        "0x647a cipher-codepoint envelope framing drifted"
    );
}

// ---------------------------------------------------------------------------
// D-43 — the 0x6500 plaintext-sender ON-WIRE row (production encoder).
// ---------------------------------------------------------------------------

/// D-43 — the `0x6500` plaintext-sender on-wire AAD, ABSOLUTE golden hex.
///
/// PROVENANCE: captured from the PRODUCTION encoder at R6 round #1 (M-20 —
/// goldens are never hand-authored) by running:
///
///   CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///     cargo nextest run -p benten-drop --features benten-drop/testing \
///     --test canonical_bytes_v1_envelope_codepoints \
///     plaintext_sender_0x6500_on_wire_aad_golden_hex --no-capture
///
/// Format is lowercase hex, no `0x`, no separators (this file's `to_hex`). The
/// command is kept so a future maintainer can RE-DERIVE the value when a wire
/// change is deliberate and ratified. **If this test fails and you did not
/// intend a wire change, `plaintext_aad_bytes` regressed — fix the encoder, not
/// this literal.**
const PLAINTEXT_SENDER_0X6500_ON_WIRE_AAD_HEX: &str = "0165000000001c6469643a6b65793a7a466978656441756469656e6365466f7250696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000009001e6469643a6b65793a7a466978656453656e646572466f724279746550696e";

/// The fixed `0x6500` plaintext-sender binding. Deliberately reuses this file's
/// `fixed_body_cid_bytes()` and the same audience/sender strings as the sibling rows,
/// so the only thing this row adds over the `0x6510` row is the codepoint + the
/// trailing plaintext-sender region.
fn fixed_plaintext_sender_binding_0x6500() -> BindingContext {
    BindingContext::DropPlaintextSender {
        aad_version: AAD_VERSION,
        codepoint: LAYER_C_DROP, // 0x6500
        audience_did: b"did:key:zFixedAudienceForPin".to_vec(),
        body_cid: fixed_body_cid_bytes(),
        recipient_key_generation: 9,
        sender_did: b"did:key:zFixedSenderForBytePin".to_vec(),
    }
}

/// D-43 — the `0x6500` (plaintext-sender, non-default) band's ON-WIRE envelope AAD
/// framing, pinned to an ABSOLUTE golden through the PRODUCTION encoder
/// `BindingContext::plaintext_aad_bytes`.
///
/// Layout: `aad_version u8 | codepoint u16-BE | aud_len u32-BE | audience |
/// body_cid(36) | recipient_key_gen u32-BE | sender_len u16-BE | sender_did`.
///
/// would-FAIL-on-mutation (one line, in `BindingContext::plaintext_aad_bytes`):
///   `out.extend_from_slice(&codepoint.to_be_bytes());`
///     ->  `out.extend_from_slice(&codepoint.to_le_bytes());`
/// on the `DropPlaintextSender` arm — the codepoint bytes flip 65 00 -> 00 65 and this
/// golden flips. Also fails on a dropped/reordered field or a length-prefix width
/// change anywhere in the layout.
///
/// NOTE on the sibling row: `sealed_sender_0x6510_on_wire_aad_golden_hex` above pins
/// `sealed_aad::serialize_sealed_sender_aad`, which is `#[cfg(any(test, feature =
/// "testing"))]` — a TEST-ONLY mirror of the production encoder. This D-43 row uses the
/// production path directly; the mirror-equality arm below couples the two.
#[test]
fn plaintext_sender_0x6500_on_wire_aad_golden_hex() {
    let got = to_hex(&fixed_plaintext_sender_binding_0x6500().plaintext_aad_bytes());
    assert_eq!(
        got, PLAINTEXT_SENDER_0X6500_ON_WIRE_AAD_HEX,
        "D-43: the 0x6500 plaintext-sender ON-WIRE AAD framing drifted. Fix the \
         encoder; re-derive this literal ONLY if the wire change is deliberate \
         and ratified.\nactual> {got}"
    );
}

/// D-43 (coupling) — the PRODUCTION `0x6510` encoder and the `testing`-gated
/// `sealed_aad::serialize_sealed_sender_aad` mirror MUST emit identical bytes over
/// identical inputs.
///
/// This matters because `sealed_sender_0x6510_on_wire_aad_golden_hex` pins the MIRROR,
/// not production. Nothing in the tree asserted the two agree, so the mirror could be
/// updated in a golden-refresh pass while production stayed put (or vice versa) — the
/// `signed_wire_change_sweep_all_embedding_goldens` failure shape.
///
/// would-FAIL-on-mutation (one line, in `sealed_aad::serialize_sealed_sender_aad`):
///   move `out.extend_from_slice(&aad.recipient_key_generation.to_be_bytes());` ABOVE
///   `out.extend_from_slice(&aad.body_cid);`
/// — the mirror reorders, production does not, and this equality breaks. (Honest
/// scope note: that same mutation ALSO flips the sibling golden, so this arm is
/// defense-in-depth for the golden-REFRESH flow rather than a uniquely-caught class.
/// Its unique value is that a coordinated "edit mirror + refresh its golden" pass —
/// which is exactly what a wire change looks like in this repo — can no longer leave
/// production behind silently.)
#[test]
fn sealed_sender_0x6510_production_encoder_matches_testing_mirror() {
    let audience = b"did:key:zFixedAudienceForPin".to_vec();
    let production = BindingContext::DropSealedSender {
        aad_version: AAD_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: audience.clone(),
        body_cid: fixed_body_cid_bytes(),
        recipient_key_generation: 9,
    }
    .plaintext_aad_bytes();

    let mirror = sealed_aad::serialize_sealed_sender_aad(&sealed_aad::SealedSenderAad {
        aad_version: sealed_aad::AAD_VERSION,
        codepoint: sealed_aad::DROP_TO_RECIPIENT_SEALED_SENDER,
        audience_did: audience,
        body_cid: fixed_body_cid_bytes(),
        recipient_key_generation: 9,
    });

    assert_eq!(
        to_hex(&production),
        to_hex(&mirror),
        "D-43: the PRODUCTION 0x6510 encoder (BindingContext::plaintext_aad_bytes) and \
         the `testing`-gated mirror (sealed_aad::serialize_sealed_sender_aad) MUST emit \
         identical bytes. The on-wire golden at \
         `sealed_sender_0x6510_on_wire_aad_golden_hex` pins the MIRROR — without this \
         coupling a wire change could refresh the mirror + its golden and leave \
         production behind."
    );
    // The two AAD_VERSION / codepoint re-exports must also be the same values, or the
    // equality above would be comparing two different bands.
    assert_eq!(AAD_VERSION, sealed_aad::AAD_VERSION);
    assert_eq!(
        DROP_TO_RECIPIENT_SEALED_SENDER,
        sealed_aad::DROP_TO_RECIPIENT_SEALED_SENDER
    );
}

// ---------------------------------------------------------------------------
// GCS-24 — the BY-BAND `sender_did` length-prefix WIDTH freeze (Row D-42 / C-07).
//
// VERIFIED SCOPE (read this before extending): the `sender_len` prefixes on the
// non-default plaintext-sender AAD trailers have NO DECODER anywhere in the tree.
// `plaintext_aad_bytes` / `assemble_group_aad_local` produce AAD that BOTH seal and
// open recompute from their own structs; nothing parses a `sender_len` back. (The
// `sd_len = u32::from_be_bytes(...)` reads in `open_group_stanza` /
// `open_membership_set_group` parse the INNER SEALED payload's sender-DID prefix, a
// different field on a different surface.)
// So a width flip on these trailers is bilaterally consistent — seal/open round-trips
// stay green — exactly the plugin-install-record preimage shape. Only an absolute
// golden or a structural width relation can catch it.
//
// PRE-EXISTING COVERAGE: `0x6500`'s u16 trailer IS pinned, at
// `f_lc_hpke_encrypt_to_recipient_sealed_sender.rs::f_lc_09_plaintext_sender_len_is_u16_be_not_u32_frozen_golden`
// (absolute golden + explicit offset read + an `assert_ne!` against the u32 reading).
// `0x6520` (u16) and `0x6610` (u32) had NO width pin of any kind. The two arms below
// close those, and are written as byte-DELTA relations so they enforce the width even
// before the absolute goldens are captured.
// ---------------------------------------------------------------------------

/// The non-default plaintext-sender DID used by both GCS-24 fixtures. 28 bytes, so a
/// u16 prefix reads `001c` and a u32 prefix reads `0000001c` — the delta assertions
/// below distinguish them by LENGTH, not by value, so they cannot be satisfied by a
/// coincidence.
const GCS24_SENDER_DID: &[u8] = b"did:key:zGcs24PlaintextSndr1";

/// A fixed `0x6520` group stanza; `plaintext_sender` selects the DEFAULT
/// (Sealed-Sender, `None`) vs the NON-DEFAULT plaintext-sender variant. Everything else
/// is held identical so the two AADs differ ONLY by the trailer.
fn gcs24_stanza_0x6520(plaintext_sender: Option<Vec<u8>>) -> HpkeRecipientStanza {
    HpkeRecipientStanza {
        codepoint: LAYER_C_DROP_MULTI_RECIPIENT, // 0x6520
        body_cid: fixed_body_cid_bytes(),
        recipient_dids: vec![
            b"did:key:zGcs24RecipientA".to_vec(),
            b"did:key:zGcs24RecipientB".to_vec(),
        ],
        stanza_index: 0,
        stanza_count: 2,
        recipient_key_generation: 4,
        sealed_inner: Vec::new(),
        plaintext_sender_did: plaintext_sender,
        wrapped_cek: Vec::new(),
    }
}

/// A fixed `0x6610` group AAD input set; `plaintext_sender` selects DEFAULT vs the
/// NON-DEFAULT plaintext-sender variant, everything else held identical.
fn gcs24_group_aad_0x6610(plaintext_sender: Option<String>) -> GroupAadInputs {
    GroupAadInputs {
        codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA, // 0x6610 (local literal)
        body_cid: fixed_body_cid_bytes(),
        member_dids: vec![
            "did:key:zGcs24MemberA".to_owned(),
            "did:key:zGcs24MemberB".to_owned(),
        ],
        k_set: [0x4B; 32],
        stanza_index: 0,
        stanza_count: 2,
        member_key_generation: 4,
        membership_set_id: b"gcs24-set-id".to_vec(),
        membership_set_generation: 5,
        role_assignments_generation: 6,
        plaintext_sender_did: plaintext_sender,
    }
}

/// GCS-24 — `0x6520` group plaintext-sender AAD trailer: the ABSOLUTE golden.
///
/// PROVENANCE: captured from the production encoder at R6 round #1 (M-20) by
/// running:
///
///   CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=6 \
///     cargo nextest run -p benten-drop --features benten-drop/testing \
///     --test canonical_bytes_v1_envelope_codepoints \
///     by_band_0x6520_plaintext_sender_trailer_is_u16_be --no-capture
///
/// The captured value ends `001c` ‖ the 28-byte sender DID — the `001c` is the
/// **u16**-BE length that distinguishes this band from `0x6610` below. Re-derive
/// ONLY for a deliberate, ratified wire change; otherwise a failure here means
/// the encoder regressed.
const GCS24_0X6520_PLAINTEXT_SENDER_AAD_HEX: &str = "01652001711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc0002b84ef94da8e47bdc423d920ed4864d5394980eb125ab84c56ab714d05bda557e000000000000000200000004001c6469643a6b65793a7a4763733234506c61696e74657874536e647231";

/// GCS-24 — `0x6610` group plaintext-sender AAD trailer: the ABSOLUTE golden.
///
/// PROVENANCE: captured at R6 round #1 with the same command as above, test
/// name `by_band_0x6610_plaintext_sender_trailer_is_u32_be`. The captured value
/// ends `0000001c` ‖ the 28-byte sender DID — a **u32**-BE length, deliberately
/// asymmetric with `0x6520`'s u16 (Row D-42 / C-07). Re-derive ONLY for a
/// deliberate, ratified wire change.
const GCS24_0X6610_PLAINTEXT_SENDER_AAD_HEX: &str = "01661001711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc0000000232af51fb7f353b308c0fe9b31aee6fd1114e43120647ba83eebcdbb83d3a96bd0000000000000002000000041bd91ed5e01ce83e0daa28dfaabd07879a326b956ac0b73306e333582d25e61300000005000000060000001c6469643a6b65793a7a4763733234506c61696e74657874536e647231";

/// GCS-24 — the `0x6520` group per-stanza plaintext-sender trailer is `sender_len
/// u16-BE | sender_did`, pinned STRUCTURALLY (byte-delta) and ABSOLUTELY (golden).
///
/// The structural half is the load-bearing one: the sender-bearing AAD MUST be the
/// sender-less AAD plus EXACTLY `2 + sender.len()` bytes. That is a width statement no
/// round-trip can satisfy accidentally, and it holds before the golden is captured.
///
/// would-FAIL-on-mutation (one line, in `HpkeRecipientStanza::plaintext_aad_bytes`):
///   `let len = u16::try_from(sender.len()).expect("sender DID len fits u16");`
///     ->  `let len = u32::try_from(sender.len()).expect("sender DID len fits u32");`
/// The delta becomes `4 + len` and this arm fails. Nothing in the tree caught that
/// before: the trailer is one-way AAD (seal and open both RECOMPUTE it), so a width
/// flip is bilaterally consistent and every group seal/open round-trip stays green.
///
/// The width asymmetry is DELIBERATE and wire-locked — `0x6520` u16 vs `0x6610` u32,
/// per `layer_c.rs` §"`sender_did` length-prefix width — BY BAND" and
/// `docs/V1-WIRE-FORMAT-INVENTORY.md §26`. A future refactor MUST NOT unify them.
#[test]
fn by_band_0x6520_plaintext_sender_trailer_is_u16_be() {
    let sender = GCS24_SENDER_DID.to_vec();
    let default_aad = gcs24_stanza_0x6520(None).plaintext_aad_bytes();
    let ps_aad = gcs24_stanza_0x6520(Some(sender.clone())).plaintext_aad_bytes();

    assert!(
        ps_aad.starts_with(&default_aad),
        "GCS-24: the 0x6520 plaintext-sender AAD MUST be the DEFAULT (Sealed-Sender) \
         AAD plus a trailing sender region — the sender is APPENDED (U4), never \
         interleaved"
    );
    assert_eq!(
        ps_aad.len() - default_aad.len(),
        2 + sender.len(),
        "GCS-24: the 0x6520 trailer MUST be `sender_len u16-BE (2 bytes) | sender_did` \
         — a u32 prefix would make this delta {} (Row D-42 / C-07: the 0x6520 width is \
         u16, DELIBERATELY different from 0x6610's u32; do not unify them)",
        4 + sender.len()
    );

    let off = default_aad.len();
    assert_eq!(
        usize::from(u16::from_be_bytes([ps_aad[off], ps_aad[off + 1]])),
        sender.len(),
        "GCS-24: the 2-byte trailer prefix MUST be the u16-BE sender-DID length"
    );
    assert_eq!(
        &ps_aad[off + 2..],
        sender.as_slice(),
        "GCS-24: the sender-DID MUST follow the 2-byte u16-BE prefix immediately"
    );

    let got = to_hex(&ps_aad);
    assert_eq!(
        got, GCS24_0X6520_PLAINTEXT_SENDER_AAD_HEX,
        "GCS-24: the 0x6520 plaintext-sender AAD drifted from its FROZEN golden. \
         Fix the encoder; re-derive this literal ONLY if the wire change is \
         deliberate and ratified.\nactual> {got}"
    );
}

/// GCS-24 — the `0x6610` MembershipSet group plaintext-sender trailer is `sender_len
/// u32-BE | sender_did` (matching that band's u32-BE per-DID roster framing), pinned
/// STRUCTURALLY and ABSOLUTELY.
///
/// would-FAIL-on-mutation (one line, in
/// `group_posture::assemble_group_aad_local`):
///   `let len = u32::try_from(sender.len()).expect("sender DID len fits u32");`
///     ->  `let len = u16::try_from(sender.len()).expect("sender DID len fits u16");`
/// The delta becomes `2 + len` and this arm fails. As with `0x6520`, the trailer is
/// one-way AAD with no decoder, so the mutation is otherwise invisible — and the
/// existing `f_02_group_aad_11field_and_f_01_truncation` golden pins only the DEFAULT
/// path (`plaintext_sender_did: None`), which never reaches this branch.
#[test]
fn by_band_0x6610_plaintext_sender_trailer_is_u32_be() {
    let sender = String::from_utf8(GCS24_SENDER_DID.to_vec())
        .expect("GCS24_SENDER_DID is ASCII by construction");
    let default_aad = assemble_group_aad_local(&gcs24_group_aad_0x6610(None));
    let ps_aad = assemble_group_aad_local(&gcs24_group_aad_0x6610(Some(sender.clone())));

    assert!(
        ps_aad.starts_with(&default_aad),
        "GCS-24: the 0x6610 plaintext-sender AAD MUST be the DEFAULT (Sealed-Sender) \
         11-field AAD plus a trailing sender region (F4-001/F-LC-9: the DEFAULT path \
         binds NO plaintext sender)"
    );
    assert_eq!(
        ps_aad.len() - default_aad.len(),
        4 + sender.len(),
        "GCS-24: the 0x6610 trailer MUST be `sender_len u32-BE (4 bytes) | sender_did` \
         — a u16 prefix would make this delta {} (Row D-42 / C-07: 0x6610 uses u32 to \
         match its u32-BE per-DID roster framing, DELIBERATELY unlike 0x6520's u16)",
        2 + sender.len()
    );

    let off = default_aad.len();
    assert_eq!(
        usize::try_from(u32::from_be_bytes([
            ps_aad[off],
            ps_aad[off + 1],
            ps_aad[off + 2],
            ps_aad[off + 3],
        ]))
        .unwrap(),
        sender.len(),
        "GCS-24: the 4-byte trailer prefix MUST be the u32-BE sender-DID length"
    );
    assert_eq!(
        &ps_aad[off + 4..],
        sender.as_bytes(),
        "GCS-24: the sender-DID MUST follow the 4-byte u32-BE prefix immediately"
    );

    let got = to_hex(&ps_aad);
    assert_eq!(
        got, GCS24_0X6610_PLAINTEXT_SENDER_AAD_HEX,
        "GCS-24: the 0x6610 plaintext-sender AAD drifted from its FROZEN golden. \
         Fix the encoder; re-derive this literal ONLY if the wire change is \
         deliberate and ratified.\nactual> {got}"
    );
}

/// GCS-24 (cross-band) — the two group bands' trailer widths are DIFFERENT, and that
/// asymmetry is the freeze. Pins the "do not unify" instruction as an executable fact
/// so a refactor that harmonises them fails here with the reason attached.
///
/// would-FAIL-on-mutation: making EITHER width match the other (either single-line
/// change named in the two arms above) collapses the two deltas to equality.
#[test]
fn by_band_group_sender_len_widths_are_deliberately_asymmetric() {
    let sender = GCS24_SENDER_DID.to_vec();
    let sender_str = String::from_utf8(GCS24_SENDER_DID.to_vec())
        .expect("GCS24_SENDER_DID is ASCII by construction");

    let delta_6520 = gcs24_stanza_0x6520(Some(sender.clone()))
        .plaintext_aad_bytes()
        .len()
        - gcs24_stanza_0x6520(None).plaintext_aad_bytes().len();
    let delta_6610 = assemble_group_aad_local(&gcs24_group_aad_0x6610(Some(sender_str))).len()
        - assemble_group_aad_local(&gcs24_group_aad_0x6610(None)).len();

    assert_eq!(
        delta_6520,
        2 + sender.len(),
        "0x6520 trailer width is u16-BE"
    );
    assert_eq!(
        delta_6610,
        4 + sender.len(),
        "0x6610 trailer width is u32-BE"
    );
    assert_ne!(
        delta_6520, delta_6610,
        "GCS-24 (Row D-42 / C-07): the 0x6520 (u16-BE) and 0x6610 (u32-BE) \
         plaintext-sender length-prefix widths are DELIBERATELY different and \
         wire-locked. A refactor that unifies them is a wire break on one of the two \
         bands — and because these trailers are one-way AAD with no decoder, nothing \
         else in the tree would notice. See `layer_c.rs` \"sender_did length-prefix \
         width — BY BAND\" and `docs/V1-WIRE-FORMAT-INVENTORY.md §26`."
    );
}
