//! GAP-KDB Shape-B — Shape-B codec no-hardcoded-sizes audit (AUDIT-1).
//! W0-canary RED-PHASE. Mirrors the `tf2_no_hardcoded_sizes` precedent.
//!
//! Ref `3bea1294`: `GAP-KDB-B-R2-LANDSCAPE.md` AUDIT-1 (Shape-B codec
//! sizes from named constants) + CLAUDE.md baked-in #5 ("Never hardcode
//! key/sig/ciphertext sizes").
//!
//! # would_fail_on_revert
//! An encoder that assumes an Ed25519-shaped (32 B) or any hardcoded key
//! size makes the concrete ML-DSA-65 (1952 B) / ML-KEM-768 (1184 B)
//! -dimensioned did:benten payload mis-frame → the component-offset
//! asserts (all derived from named size constants, never literals) flip.
//!
//! # R5 un-ignore
//! Mint the did:benten encoder + KeySetDocument codec sourcing every size
//! from `ml_dsa_65_pubkey_len()` / `ML_KEM_768_EK_LEN` / `X25519_PUBLIC_LEN`
//! / `CID_LEN`; repoint the stubs; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::CID_LEN;
use benten_crypto_suite::cipher_suite::{ML_KEM_768_EK_LEN, X25519_PUBLIC_LEN};
use benten_crypto_suite::sizes::ml_dsa_65_pubkey_len;
use benten_id::kdb_testing as kdb;

// The Ed25519-shaped sizes a hardcoded-size regression would assume. The
// ML-DSA-65 / ML-KEM-768 components dwarf them; a codec that assumes these
// cannot frame the real payload.
const ED25519_SHAPED_PUBKEY: usize = 32;

#[test]
#[ignore = "RED-PHASE: AUDIT-1 did:benten codec sources sizes from named constants — un-ignore at R5"]
fn audit1_did_benten_payload_dimensions_from_named_constants_not_literals() {
    let kp = kdb::hybrid_keypair();
    let doc = kdb::KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&kp.public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("audit/x"),
            &kdb::det_mlkem768_ek("audit/ek"),
        ),
    );
    let did = kdb::encode_did_benten(&kp.public(), &doc);
    let payload = kdb::did_benten_payload_of(&did).unwrap();

    // Every offset is derived from a NAMED size constant. The signing
    // multikey is `2 + mldsa(1952) + 2 + ed25519(32)`; the payload is that
    // plus a 36-byte CID. A 32-B / hardcoded assumption fails here.
    let mldsa_len = ml_dsa_65_pubkey_len();
    assert!(
        mldsa_len > ED25519_SHAPED_PUBKEY * 4,
        "ML-DSA-65 pubkey ({mldsa_len} B) must dwarf an Ed25519-shaped size — \
         a codec that hardcodes 32 cannot frame this payload"
    );
    let expected_signing_len = 2 + mldsa_len + 2 + ED25519_SHAPED_PUBKEY;
    let expected_payload_len = expected_signing_len + CID_LEN;
    assert_eq!(
        payload.len(),
        expected_payload_len,
        "did:benten payload length MUST equal the named-constant-derived size \
         (2+ml_dsa_65_pubkey_len()+2+32 signing ‖ CID_LEN), never a hardcoded size (#5)"
    );
}

#[test]
#[ignore = "RED-PHASE: AUDIT-1 KeySetDocument kem multikey dimensions from named constants — un-ignore at R5"]
fn audit1_keyset_kem_multikey_dimensions_from_named_constants() {
    // The kem multikey (X25519-first, C2) is `2 + X25519_PUBLIC_LEN + 2 +
    // ML_KEM_768_EK_LEN`. Sourcing from the named constants means a
    // ciphertext/key-size drift upstream flows through, never a literal.
    let kem_mk = kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub("audit/kx"),
        &kdb::det_mlkem768_ek("audit/kek"),
    );
    let expected = 2 + X25519_PUBLIC_LEN + 2 + ML_KEM_768_EK_LEN;
    assert_eq!(
        kem_mk.len(),
        expected,
        "kem multikey length MUST come from X25519_PUBLIC_LEN + ML_KEM_768_EK_LEN (#5); \
         ML-KEM-768 EK is {ML_KEM_768_EK_LEN} B — a codec that hardcodes an Ed25519-shaped \
         {ED25519_SHAPED_PUBKEY} B assumption cannot frame it"
    );
}
