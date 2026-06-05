//! **F-W0-1..5 — Wave-0 / `ENVELOPE_FORMAT_VERSION_V2` migration (R5: GREEN).**
//!
//! ADDL R3 wave **W0-crypto-canary** (the SOLE upstream canary). R5 wires the
//! real V2/BE/`EncryptedEnvelope` surface + the real draft-connolly X-Wing
//! combiner (label APPENDED) + the delivered M-19 conformance scanner. The
//! corpus stub-shim is DELETED; every pin drives the production call site.
//!
//! # M-20 golden reconciliation
//!
//! The X-Wing KAT for the fixed synthetic fixture is RECOMPUTED via the real
//! `combine_x_wing` (the corpus base computed it via an HKDF-SHA256 stub; the
//! real construction is `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`
//! with the label APPENDED). The reconciled golden is pinned below + the
//! decision-log records the stub→real delta. The `legacy_hkdf_combine`
//! reference is the real in-tree HKDF-SHA256 combiner shape so the `assert_ne!`
//! "construction changed" arm is meaningful.

use benten_crypto_suite::cipher_suite::{combine_x_wing, x_wing_combiner_preimage};
use benten_crypto_suite::conformance::endianness::{
    info_tag_ascii_flagged, wire_path_le_survivor_count,
};
use benten_crypto_suite::envelope::{
    BindingContext, ENVELOPE_FORMAT_VERSION_V1, ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_MAGIC,
    EncryptedEnvelope, MAX_NONCE_LEN,
};
use sha3::{Digest, Sha3_256};

/// The real draft-connolly `XWingLabel` — 6 bytes `0x5c2e2f2f5e5c` (ASCII
/// `\.//^\`), APPENDED as the combiner pre-image suffix.
const XWING_LABEL: [u8; 6] = [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c];

// Fixed X-Wing combiner fixture (stable inputs so the KAT is deterministic).
const SS_MLKEM: [u8; 32] = [0xA1; 32];
const SS_X25519: [u8; 32] = [0xB2; 32];
const CT_X25519: [u8; 32] = [0xC3; 32];
const PK_X25519: [u8; 32] = [0xD4; 32];

/// M-20-reconciled X-Wing KAT golden for the fixed synthetic fixture —
/// `SHA3-256(SS_MLKEM ‖ SS_X25519 ‖ CT_X25519 ‖ PK_X25519 ‖ XWingLabel)`
/// computed by the REAL `combine_x_wing` (stub returned `[0;32]`; the real
/// construction is the value below). Recomputed via the production encoder
/// per M-20.
const DRAFT_CONNOLLY_X_WING_KAT: [u8; 32] = [
    0xb2, 0x10, 0xb1, 0xb9, 0x26, 0x1f, 0xb3, 0x97, 0xa2, 0xba, 0x4d, 0x51, 0xf8, 0xed, 0xdf, 0x95,
    0x55, 0xed, 0xb3, 0xda, 0xa9, 0xa3, 0x92, 0x0b, 0x8d, 0x77, 0x67, 0xfb, 0x4b, 0x26, 0xf9, 0xaf,
];

/// The legacy in-tree HKDF-SHA256 combiner output for the SAME inputs — the
/// `assert_ne!` "construction changed" reference. Computed via the real
/// upstream `hkdf` crate (the shape the corpus base shipped at
/// `cipher_suite.rs:404`).
fn legacy_hkdf_combine(ss_mlkem: &[u8], ss_x: &[u8], ek_x: &[u8], pub_x: &[u8]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let mut ikm = Vec::new();
    ikm.extend_from_slice(ss_x);
    ikm.extend_from_slice(ss_mlkem);
    ikm.extend_from_slice(ek_x);
    ikm.extend_from_slice(pub_x);
    let salt = {
        let mut h = Sha3_256::new();
        h.update(b"x-wing-v1-benten-0x647a");
        h.finalize().to_vec()
    };
    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"x-wing-v1-benten-0x647a", &mut okm).unwrap();
    okm
}

/// The classical `0x6400` combiner output for a fixed fixture (re-derived
/// consistently with the real-X-Wing rewrite). Computed via the real
/// `classical_combine`.
fn classical_combine_for_fixture() -> [u8; 32] {
    benten_crypto_suite::cipher_suite::classical_combine(&SS_X25519, &CT_X25519, &PK_X25519)
}

/// **F-W0-1** — `0x647A` computes the REAL X-Wing SHA3-256 construction,
/// NOT the in-tree HKDF-SHA256 stand-in.
#[test]
fn x_wing_0x647a_uses_real_sha3_256_construction_not_hkdf() {
    let real = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let legacy = legacy_hkdf_combine(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    assert_ne!(
        real, legacy,
        "real X-Wing SHA3-256 combiner MUST differ from the in-tree HKDF-SHA256 stand-in"
    );
    assert_eq!(
        real, DRAFT_CONNOLLY_X_WING_KAT,
        "0x647A combiner must equal the recomputed draft-connolly X-Wing KAT for the fixed fixture (M-20)"
    );
}

/// **F-W0-1-LABEL (F4-002 + F4-003)** — `XWingLabel = 0x5c2e2f2f5e5c`
/// APPENDED as the combiner pre-image suffix (NOT prepended).
#[test]
fn x_wing_label_is_appended_suffix_not_prepended() {
    assert_eq!(
        XWING_LABEL,
        benten_crypto_suite::cipher_suite::X_WING_LABEL,
        "XWingLabel must be the 6 bytes 0x5c2e2f2f5e5c per draft-connolly-cfrg-xwing-kem-10 §6"
    );

    let pre = x_wing_combiner_preimage(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let n = pre.len();
    assert!(n >= XWING_LABEL.len() + SS_MLKEM.len());

    // APPENDED suffix.
    assert_eq!(
        &pre[n - XWING_LABEL.len()..],
        &XWING_LABEL[..],
        "XWingLabel must be APPENDED as the trailing suffix of the combiner pre-image (NOT prepended)"
    );
    assert_eq!(
        &pre[..SS_MLKEM.len()],
        &SS_MLKEM[..],
        "the pre-image must begin with ss_M (ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel ordering)"
    );
    // NOT prepended.
    assert_ne!(
        &pre[..XWING_LABEL.len()],
        &XWING_LABEL[..],
        "XWingLabel must NOT be prepended (the prepended form is the superseded v01-v02 construction)"
    );

    // Construction-order witness: hashing the appended pre-image reproduces
    // the combiner output.
    let mut h = Sha3_256::new();
    h.update(&pre);
    let preimage_digest: [u8; 32] = h.finalize().into();
    assert_eq!(
        preimage_digest,
        combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519),
        "the combiner output must equal SHA3-256 over the appended-label pre-image"
    );
}

/// **F-W0-1 (cont.)** — the classical `0x6400` combiner stays consistent
/// after the real-X-Wing rewrite.
#[test]
fn classical_0x6400_combiner_consistent_after_rewrite() {
    let hybrid = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    let classical = classical_combine_for_fixture();
    assert_eq!(
        hybrid, DRAFT_CONNOLLY_X_WING_KAT,
        "the hybrid 0x647A combiner MUST be the real draft-connolly X-Wing SHA3-256 construction"
    );
    assert_ne!(
        hybrid, classical,
        "classical 0x6400 and hybrid 0x647A combiners must derive distinct keys"
    );
    assert_ne!(
        classical, [0u8; 32],
        "classical 0x6400 combiner must produce a real derived key"
    );
}

/// **F-W0-2** — X-Wing interop KAT round-trips against the recomputed vector.
#[test]
fn x_wing_interop_kat_byte_for_byte() {
    let derived = combine_x_wing(&SS_MLKEM, &SS_X25519, &CT_X25519, &PK_X25519);
    assert_eq!(
        derived, DRAFT_CONNOLLY_X_WING_KAT,
        "X-Wing combiner output must match the recomputed KAT row byte-for-byte"
    );
    let wrong_ct = combine_x_wing(&SS_MLKEM, &SS_X25519, &[0u8; 32], &PK_X25519);
    assert_ne!(
        wrong_ct, DRAFT_CONNOLLY_X_WING_KAT,
        "X-Wing combiner must bind ct_X (wrong ct_X ⇒ different derived key)"
    );
}

/// **F-W0-3** — BE endianness sweep + the **zero-`to_le_bytes`** scanner.
#[test]
fn zero_to_le_bytes_survives_on_wire_or_aad_paths() {
    assert_eq!(
        wire_path_le_survivor_count(),
        0,
        "NO `to_le_bytes` may survive on any wire/AAD path (M-19 flagship)"
    );
    assert!(
        !info_tag_ascii_flagged(),
        "the X-Wing info-tag ASCII string is not endianness-affected and must NOT be flagged (m-1)"
    );
}

/// **F-W0-3 (cont.)** — the codepoint is serialized BIG-ENDIAN on the wire.
#[test]
fn codepoint_serialized_big_endian_on_wire() {
    let env = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: 0x647A,
        aad_binding: BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        },
        nonce: vec![0u8; 12],
        ciphertext: vec![0xEE; 16],
    };
    let wire = env.to_wire_bytes();
    assert_eq!(wire[0], ENVELOPE_MAGIC, "byte 0 = envelope magic");
    assert_eq!(
        [wire[2], wire[3]],
        0x647Au16.to_be_bytes(),
        "codepoint must be serialized BIG-ENDIAN"
    );
    assert_ne!(
        [wire[2], wire[3]],
        0x647Au16.to_le_bytes(),
        "codepoint must NOT be little-endian"
    );
}

/// **F-W0-4** — `AeadEnvelope` → `EncryptedEnvelope` lift + typed
/// `BindingContext`.
#[test]
fn aead_envelope_lifts_to_encrypted_envelope_with_typed_binding() {
    let vault = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: 0x6100,
        aad_binding: BindingContext::Vault { vault_version: 1 },
        nonce: vec![0u8; 24],
        ciphertext: vec![0x01; 16],
    };
    assert!(matches!(
        vault.aad_binding,
        BindingContext::Vault { vault_version: 1 }
    ));
    let whole = BindingContext::WholeContent {
        plaintext_cid: vec![0xCD; 32],
    };
    assert_ne!(vault.aad_binding, whole);

    let wire = vault.to_wire_bytes();
    assert_eq!(wire[0], ENVELOPE_MAGIC);
    assert_eq!(
        wire[1], ENVELOPE_FORMAT_VERSION_V2,
        "the lifted EncryptedEnvelope MUST serialize byte-1 == V2"
    );
    assert_eq!(
        [wire[2], wire[3]],
        0x6100u16.to_be_bytes(),
        "the lifted EncryptedEnvelope MUST serialize the vault codepoint BIG-ENDIAN"
    );
}

/// **F-W0-5** — exactly ONE `V1→V2` bump + V1 typed-reject.
#[test]
fn single_v1_to_v2_bump_and_v1_typed_rejected() {
    assert_ne!(ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_FORMAT_VERSION_V1);
    let env = EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: 0x647A,
        aad_binding: BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        },
        nonce: vec![0u8; 12],
        ciphertext: vec![0xEE; 16],
    };
    let wire = env.to_wire_bytes();
    assert_eq!(wire[1], ENVELOPE_FORMAT_VERSION_V2);

    let v1_bytes = {
        let mut b = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V1];
        b.extend_from_slice(&0x647Au16.to_be_bytes());
        b.push(0);
        b
    };
    assert!(
        EncryptedEnvelope::from_wire_bytes(&v1_bytes).is_err(),
        "a V1-framed byte stream must be typed-rejected post-V2-freeze"
    );
}

/// **F-W0-BD-1 (F4-009-BD / META #629)** — the V2 bounded-decode decoder
/// REJECTS a hostile declared length-prefix on BOTH bounds BEFORE allocating.
#[test]
fn v2_decode_rejects_hostile_length_prefix_before_allocating() {
    let cases: [(u8, usize, bool, &str); 3] = [
        (255, 0, false, "exceeds both"),
        (20, 0, false, "exceeds-remaining-only-within-MAX"),
        (12, 12, true, "within-both-OK"),
    ];
    assert!((cases[1].0 as usize) <= MAX_NONCE_LEN && (cases[1].0 as usize) > cases[1].1);
    assert!((cases[0].0 as usize) > MAX_NONCE_LEN);

    for (declared, present, expect_ok, label) in cases {
        let frame = {
            let mut b = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V2];
            b.extend_from_slice(&0x647Au16.to_be_bytes());
            b.push(declared);
            b.extend_from_slice(&vec![0u8; present]);
            b
        };
        let decoded = EncryptedEnvelope::decode_nonce_bounded(&frame);
        if expect_ok {
            assert_eq!(
                decoded
                    .unwrap_or_else(|e| panic!("[{label}] must decode, got Err({e})"))
                    .len(),
                declared as usize,
                "[{label}] within-bounds nonce-len decodes to exactly that length"
            );
        } else {
            assert!(
                decoded.is_err(),
                "[{label}] hostile declared length MUST be typed-rejected before allocation (META #629)"
            );
        }
    }
}
