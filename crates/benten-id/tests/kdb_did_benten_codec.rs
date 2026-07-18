//! GAP-KDB Shape-B — `did:benten` codec positives (families DID-1, DID-2,
//! DID-5, DID-6). W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §1.1 (DID string shape) +
//! `GAP-KDB-B-R2-LANDSCAPE.md` §1 benten-id did:benten codec.
//!
//! # Frozen layout under pin (design §1.1, corrections C1)
//! `did:benten:z` + base58btc(
//!   `[0x91,0x24 ‖ mldsaPK(1952)]` ‖ `[0xed,0x01 ‖ ed25519PK(32)]`
//!   ‖ `[0x01,0x71,0x1e,0x20 ‖ blake3(keysetDoc)(32)]`  (CIDv1, 36 B)
//! )
//! — signing multikey (ML-DSA-first) then the committed key-set CID, with
//! **NO framing byte** between them (C1 dropped `BENTEN_KEYSET_COMMIT`).
//!
//! # would_fail_on_revert
//! - DID-1: a no-op / mis-ordered / framing-byte-inserting encoder makes
//!   the decoded payload `!=` `signing_multikey ‖ cid` → the `assert_eq`
//!   flips fail.
//! - DID-2: encode↔decode not an inverse (or `resolve_signing` /
//!   `keyset_cid` not recovering the exact key/CID) flips the round-trip
//!   assert.
//! - DID-5: perturbing the did:key encoding under GAP-KDB flips the
//!   (baseline, non-ignored) byte-identity regression.
//! - DID-6: an encoder that ignores the committed key-set CID emits the
//!   same string for two different key-sets → `assert_ne` flips.
//!
//! # R5 un-ignore
//! Mint `Did::from_benten_keyset` / `Did::resolve_signing` /
//! `Did::keyset_cid`; repoint the `kdb_testing` free-fn stubs at them; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::CID_LEN;
use benten_crypto_suite::sizes::ml_dsa_65_pubkey_len;
use benten_id::did::{ED25519_MULTICODEC, MLDSA65_PUB_MULTICODEC};
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair;

// ── DID-1 — golden byte-layout (design §1.1, C1) ──────────────────────────

#[test]
fn did1_golden_byte_layout_signing_multikey_then_cid_no_framing_byte() {
    let (did, honest_doc, _attacker) = kdb::substituted_recipient_scenario();
    // The victim DID was built (via the frozen §1.1 layout) to commit
    // cid(honest_doc). At R5, `encode_did_benten` (→ Did::from_benten_keyset)
    // must reproduce this exact string from (signing key, honest_doc).
    let kp = kdb::hybrid_keypair();
    let re_encoded = kdb::encode_did_benten(&kp.public(), &honest_doc);

    // Method prefix (a distinct method — design §1.1).
    assert!(
        re_encoded.as_str().starts_with(kdb::DID_BENTEN_PREFIX),
        "did:benten MUST use the `did:benten:z` method prefix; got {}",
        re_encoded.as_str()
    );

    let payload = kdb::did_benten_payload_of(&re_encoded)
        .expect("did:benten method-specific-id must base58btc-decode");

    let sig_mk = kdb::signing_multikey_of(&kp.public());
    let expected = kdb::did_benten_payload(&sig_mk, &honest_doc.cid());
    assert_eq!(
        payload, expected,
        "did:benten payload MUST be exactly `signing_multikey ‖ keysetDocCID(36)` \
         (design §1.1) — ML-DSA-first signing multikey then the committed CID"
    );

    // Exact component framing (belt-and-suspenders on the golden).
    assert_eq!(
        &payload[0..2],
        &MLDSA65_PUB_MULTICODEC,
        "ML-DSA-65 multicodec 0x1211"
    );
    let ed_off = 2 + ml_dsa_65_pubkey_len();
    assert_eq!(
        &payload[ed_off..ed_off + 2],
        &ED25519_MULTICODEC,
        "Ed25519 multicodec 0xed01 follows the ML-DSA component"
    );

    // C1 — NO framing byte between the signing multikey and the CID: the
    // byte immediately after the signing multikey is the CIDv1 version
    // marker 0x01, not a `BENTEN_KEYSET_COMMIT` varint.
    let cid_off = sig_mk.len();
    assert_eq!(
        payload.len(),
        cid_off + CID_LEN,
        "payload is exactly signing_multikey ‖ 36-byte CID — no framing byte, no trailing"
    );
    assert_eq!(
        &payload[cid_off..],
        honest_doc.cid().as_bytes(),
        "the trailing 36 bytes are the committed key-set CID (design C1: no framing byte)"
    );
    assert_eq!(
        payload[cid_off],
        benten_core::CID_V1,
        "first byte of the committed component is the CIDv1 version 0x01 (C1: no framing byte)"
    );
}

// ── DID-2 — encode↔decode round-trip byte-identity ────────────────────────

#[test]
fn did2_encode_decode_round_trip_recovers_signing_key_and_committed_cid() {
    let kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&kp.public());
    let kem_mk = kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub("did2/x"),
        &kdb::det_mlkem768_ek("did2/ek"),
    );
    let doc = kdb::KeySetDocument::v1_hybrid(sig_mk.clone(), kem_mk);

    let did = kdb::encode_did_benten(&kp.public(), &doc);

    // Decode → re-encode is a string-level inverse.
    let payload = kdb::did_benten_payload_of(&did).unwrap();
    let rebuilt = kdb::did_benten_from_payload_for_test(&payload);
    assert_eq!(
        did.as_str(),
        rebuilt.as_str(),
        "did:benten encode/decode MUST be a byte-exact inverse"
    );

    // resolve_signing recovers the exact composite signing key (zero-I/O).
    let resolved =
        kdb::resolve_signing(&did).expect("resolve_signing must recover the signing key");
    assert_eq!(
        resolved.to_lamps_composite_bytes().unwrap(),
        kp.public().to_lamps_composite_bytes().unwrap(),
        "resolve_signing MUST recover the exact embedded composite signing key"
    );

    // keyset_cid recovers the committed CID == cid(doc).
    let committed =
        kdb::committed_keyset_cid(&did).expect("keyset_cid must recover the committed CID");
    assert_eq!(
        committed.as_bytes(),
        doc.cid().as_bytes(),
        "keyset_cid MUST recover the committed key-set-doc CID"
    );
}

// ── DID-5 — did:key degenerate zero-migration (classical + hybrid) ────────
//
// Baseline (non-ignored) regression guard: the did:key STRING encoding is
// untouched by GAP-KDB (design §6 — did:key bytes unchanged → zero
// migration for the classical/authority world). This arm is LIVE — it
// fails immediately if a GAP-KDB change perturbs did:key.

#[test]
fn did5_didkey_classical_encoding_unchanged_regression() {
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    assert!(
        did.as_str().starts_with("did:key:z"),
        "classical did:key encoding MUST remain did:key:z… (zero migration)"
    );
    // NOT a did:benten — the two methods are lexically un-confusable.
    assert!(
        !did.as_str().starts_with(kdb::DID_BENTEN_PREFIX),
        "a classical did:key MUST NOT be mistaken for a did:benten"
    );
    let recovered = did
        .resolve()
        .expect("classical did:key still resolves unchanged");
    assert_eq!(
        recovered.to_bytes(),
        kp.public_key().to_bytes(),
        "classical did:key round-trip byte-identity is preserved (design §6)"
    );
}

#[test]
fn did5_resolve_signing_accepts_bare_didkey_degenerate() {
    // A bare Ed25519 did:key is the signing-only degenerate identity
    // (design §6). The method-aware resolve_signing MUST recover its key
    // exactly as the legacy resolve() does — no migration.
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    let via_signing = kdb::resolve_signing(&did).expect("resolve_signing accepts a bare did:key");
    // The bare did:key is signing-only classical (pq=None); the exact
    // 32-byte identity round-trip is pinned by the live
    // `did5_didkey_classical_encoding_unchanged_regression` via resolve().
    assert!(
        !via_signing.is_hybrid(),
        "resolve_signing on a bare did:key MUST recover a classical-only signing key (zero migration)"
    );
    assert!(
        did.resolve().is_ok(),
        "the bare did:key also resolves via the legacy path unchanged"
    );
}

// ── DID-6 — string injectivity, CID-sensitivity ──────────────────────────

#[test]
fn did6_changing_committed_keyset_changes_the_did_string() {
    // Same signing key, DIFFERENT key-set (different KEM key ⇒ different
    // canonical CID). The encoder commits the CID, so the DID strings MUST
    // differ — an encoder that ignores the doc CID would emit one string.
    let kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&kp.public());

    let doc_a = kdb::KeySetDocument::v1_hybrid(
        sig_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("did6/a/x"),
            &kdb::det_mlkem768_ek("did6/a/ek"),
        ),
    );
    let doc_b = kdb::KeySetDocument::v1_hybrid(
        sig_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("did6/b/x"),
            &kdb::det_mlkem768_ek("did6/b/ek"),
        ),
    );
    assert_ne!(
        doc_a.cid().as_bytes(),
        doc_b.cid().as_bytes(),
        "precondition: the two key-sets have distinct canonical CIDs"
    );

    let did_a = kdb::encode_did_benten(&kp.public(), &doc_a);
    let did_b = kdb::encode_did_benten(&kp.public(), &doc_b);
    assert_ne!(
        did_a.as_str(),
        did_b.as_str(),
        "a did:benten committing a different key-set MUST be a different DID string \
         (design §1.1 — the DID is the content-address of the key-set; §3.5s / Inv-15)"
    );
}
