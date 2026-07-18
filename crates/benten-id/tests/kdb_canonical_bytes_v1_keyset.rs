//! GAP-KDB Shape-B — `KeySetDocument` v1 canonical-bytes golden pins
//! (families KSD-1, KSD-2, KSD-6, KSD-7). W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §1.2 (KeySetDocument schema) +
//! corrections C1/C2. `GAP-KDB-B-R2-LANDSCAPE.md` KSD-1/2/6/7.
//!
//! # Frozen schema under pin (design §1.2)
//! Canonical DAG-CBOR map, keys in canonical order `{v, kem, sig, kem_cp,
//! sig_cp}` (length-first then bytewise), definite-length byte strings,
//! `v=1`, `sig_cp=0x0001`, `kem_cp=0x647a`. `kem` multikey X25519-first
//! (C2): `0xec,0x01 ‖ x25519(32) ‖ 0x8c,0x24 ‖ mlkem768_ek(1184)`.
//!
//! # Golden capture (M-20)
//! `KSD_TINY_CANONICAL_HEX` + `KSD_TINY_CID_HEX` were captured via a
//! throwaway over `KeySetDocument::to_canonical_bytes` / `.cid` (the frozen
//! serializer) with TINY deterministic opaque `sig`(8)/`kem`(6) payloads —
//! the whole canonical skeleton (map header, key order, key encodings,
//! byte-string headers, cp values) is inspectable end-to-end. At R5 the
//! production encoder MUST reproduce these bytes byte-for-byte.
//!
//! # would_fail_on_revert
//! A production encoder that reorders keys, uses indefinite lengths, emits
//! a different field set, or a different cp value produces bytes `!=` the
//! frozen golden → the `assert_eq` flips. A codec that ignores the field
//! values (no-op) fails equally.
//!
//! # R5 un-ignore
//! Replace the `kdb_testing::KeySetDocument` stub with a `pub use
//! benten_id::keyset::KeySetDocument;` re-export; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::kdb_testing as kdb;

/// The full canonical DAG-CBOR bytes of a tiny KeySetDocument
/// `{v:1, sig:det("KSD-golden/sig",8), kem:det("KSD-golden/kem",6),
/// sig_cp:0x0001, kem_cp:0x647a}`. Captured via M-20 throwaway from the
/// frozen serializer.
const KSD_TINY_CANONICAL_HEX: &str =
    "a5617601636b656d46507ad5f33c4d637369674849eff4745249930c666b656d5f637019647a667369675f637001";

/// `self_describing_cid(BLAKE3-256(canonical))` of that tiny document.
const KSD_TINY_CID_HEX: &str =
    "01711e20a758d149a0e03d20fe15a057dcbfd3f48a9a2acf5587a662ca74299ab9e5acbe";

fn tiny_doc() -> kdb::KeySetDocument {
    kdb::KeySetDocument::v1_hybrid(
        kdb::det_bytes("KSD-golden/sig", 8),
        kdb::det_bytes("KSD-golden/kem", 6),
    )
}

// ── KSD-1 — golden canonical-bytes + golden CID ───────────────────────────

#[test]
#[ignore = "RED-PHASE: KSD-1 KeySetDocument golden canonical bytes — un-ignore at R5"]
fn ksd1_canonical_bytes_match_frozen_golden() {
    let doc = tiny_doc();
    assert_eq!(
        kdb::golden_hex(&doc.to_canonical_bytes()),
        KSD_TINY_CANONICAL_HEX,
        "KeySetDocument canonical DAG-CBOR MUST match the frozen v1-beta golden \
         (map {{v,kem,sig,kem_cp,sig_cp}}, canonical key order, definite lengths)"
    );
}

#[test]
#[ignore = "RED-PHASE: KSD-1 KeySetDocument golden CID — un-ignore at R5"]
fn ksd1_cid_matches_frozen_golden_and_is_blake3_over_canonical() {
    let doc = tiny_doc();
    assert_eq!(
        kdb::golden_hex(doc.cid().as_bytes()),
        KSD_TINY_CID_HEX,
        "KeySetDocument CID MUST match the frozen golden (self-describing CIDv1 \
         over BLAKE3-256 of the canonical bytes)"
    );
    // The CID is exactly the self-describing hash of the canonical bytes
    // (design §1.2) — the commitment binding resolve_kem checks.
    let expect_digest = blake3_of(&kdb::from_golden_hex(KSD_TINY_CANONICAL_HEX));
    assert_eq!(
        &doc.cid().as_bytes()[4..],
        &expect_digest[..],
        "CID digest MUST be BLAKE3-256 over the canonical bytes (the commitment preimage)"
    );
    assert_eq!(
        &doc.cid().as_bytes()[..4],
        &kdb::CID_V1_DAGCBOR_BLAKE3_HEADER,
        "CID header MUST be the self-describing 0x01,0x71,0x1e,0x20 prefix"
    );
}

// ── KSD-2 — canonical key-order + definite-length injectivity ──────────────

#[test]
#[ignore = "RED-PHASE: KSD-2 canonical key-order + definite-length injectivity — un-ignore at R5"]
fn ksd2_canonical_key_order_and_definite_lengths() {
    let bytes = tiny_doc().to_canonical_bytes();
    // 5-pair definite-length map.
    assert_eq!(
        bytes[0], 0xa5,
        "MUST be a definite-length 5-pair map (0xa5)"
    );
    // First key is "v" (text-1) — canonical order puts the len-1 key first.
    assert_eq!(
        &bytes[1..3],
        &[0x61, b'v'],
        "canonical order: `v` key first"
    );
    // Second key is "kem" (text-3) — `kem` < `sig` bytewise at equal length.
    assert_eq!(
        &bytes[4..8],
        &[0x63, b'k', b'e', b'm'],
        "canonical order: `kem` before `sig`"
    );
    // No indefinite-length markers anywhere (0xbf map / 0x5f bytes / 0x7f text).
    assert!(
        !bytes
            .iter()
            .any(|&b| b == 0x5f || b == 0xbf || b == 0x7f || b == 0xff),
        "canonical DAG-CBOR MUST NOT use any indefinite-length encoding"
    );
}

#[test]
#[ignore = "RED-PHASE: KSD-2 field-injectivity (distinct fields ⇒ distinct bytes) — un-ignore at R5"]
fn ksd2_distinct_fields_give_distinct_canonical_bytes() {
    let a = kdb::KeySetDocument::v1_hybrid(kdb::det_bytes("a/sig", 8), kdb::det_bytes("a/kem", 6));
    let b = kdb::KeySetDocument::v1_hybrid(kdb::det_bytes("b/sig", 8), kdb::det_bytes("a/kem", 6));
    assert_ne!(
        a.to_canonical_bytes(),
        b.to_canonical_bytes(),
        "documents differing in the `sig` field MUST have distinct canonical bytes (injective)"
    );
    assert_ne!(a.cid().as_bytes(), b.cid().as_bytes(), "…and distinct CIDs");
}

// ── KSD-6 — frozen field-value pins (sig_cp=0x0001, kem_cp=0x647a) ─────────

#[test]
#[ignore = "RED-PHASE: KSD-6 frozen sig_cp/kem_cp field values — un-ignore at R5"]
fn ksd6_frozen_codepoint_field_values() {
    let doc = tiny_doc();
    assert_eq!(
        doc.sig_cp(),
        0x0001,
        "sig_cp MUST be LAMPS id-MLDSA65-Ed25519-SHA512 (0x0001)"
    );
    assert_eq!(
        doc.kem_cp(),
        0x647a,
        "kem_cp MUST be HYBRID_X25519_MLKEM768 (0x647a)"
    );
    assert_eq!(doc.version(), 1, "v MUST be 1 (v1-beta freeze)");
    // The kem_cp value 0x647a is present in the canonical bytes as the
    // minimal CBOR uint `0x19 0x64 0x7a` (not a wrong/hardcoded value).
    let bytes = doc.to_canonical_bytes();
    assert!(
        windows_contains(&bytes, &[0x19, 0x64, 0x7a]),
        "kem_cp MUST encode as the minimal CBOR uint 0x19 0x64 0x7a (0x647a) in the canonical bytes"
    );
}

// ── KSD-7 — kem multikey X25519-first golden (C2, no reorder) ──────────────

#[test]
#[ignore = "RED-PHASE: KSD-7 kem multikey X25519-first layout (C2) — un-ignore at R5"]
fn ksd7_kem_multikey_is_x25519_first_then_mlkem768() {
    use benten_crypto_suite::cipher_suite::{ML_KEM_768_EK_LEN, X25519_PUBLIC_LEN};
    let x = kdb::det_x25519_pub("ksd7/x");
    let ek = kdb::det_mlkem768_ek("ksd7/ek");
    let doc = kdb::KeySetDocument::v1_hybrid(
        kdb::det_signing_multikey("ksd7"),
        kdb::kem_multikey_hybrid(&x, &ek),
    );
    let kem = doc.kem();

    // X25519-FIRST (C2 — matches the already-frozen RecipientPublic::to_bytes
    // order; deletes the §5 REORDER foot-gun).
    assert_eq!(
        &kem[0..2],
        &kdb::X25519_PUB_MULTICODEC,
        "kem multikey MUST start with x25519-pub 0xec01 (C2 X25519-first)"
    );
    assert_eq!(
        &kem[2..2 + X25519_PUBLIC_LEN],
        &x[..],
        "…then the 32-byte X25519 public key"
    );
    let mlkem_off = 2 + X25519_PUBLIC_LEN;
    assert_eq!(
        &kem[mlkem_off..mlkem_off + 2],
        &kdb::MLKEM768_PUB_MULTICODEC,
        "…then mlkem-768-pub 0x120c (varint 0x8c24)"
    );
    assert_eq!(
        &kem[mlkem_off + 2..],
        &ek[..],
        "…then the ML-KEM-768 encapsulation key"
    );
    assert_eq!(
        kem.len(),
        2 + X25519_PUBLIC_LEN + 2 + ML_KEM_768_EK_LEN,
        "kem multikey length MUST be the two registered components, no slack"
    );
}

// ── helpers ───────────────────────────────────────────────────────────────

fn blake3_of(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

fn windows_contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
