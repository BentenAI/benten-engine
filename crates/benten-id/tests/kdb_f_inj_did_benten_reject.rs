//! GAP-KDB Shape-B — `did:benten` injectivity reject matrix (DID-3) + F2
//! DoS cap (DID-7). W0-canary RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §1.1 injectivity (Row-D-13,
//! `HybridTrailingBytes` posture at `did.rs:376-380`) + C7 (F2 DoS cap
//! wired for did:benten; keep `MAX_DID_KEY_STRING_LEN = 4096`).
//! `GAP-KDB-B-R2-LANDSCAPE.md` DID-3 / DID-7.
//!
//! # would_fail_on_revert
//! - DID-3: dropping the exact-consume / trailing-reject / CIDv1-prefix
//!   check makes a malformed did:benten decode as Ok → the `expect_err`
//!   arms flip to Ok and fail.
//! - DID-7: dropping `length_pre_check` before the O(N²) `bs58::decode`
//!   makes an oversized `iss` hang (multi-second) instead of a fast typed
//!   reject → the fast-reject arm times out / flips.
//!
//! # R5 un-ignore
//! Mint `Did::resolve_signing` / `Did::keyset_cid` with exact-consume +
//! trailing-reject + the shared `length_pre_check`; repoint the stubs; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, Instant};

use benten_core::CID_LEN;
use benten_id::did::MAX_DID_KEY_STRING_LEN;
use benten_id::kdb_testing as kdb;

/// A well-formed did:benten payload (`signing_multikey ‖ cid(36)`).
fn honest_payload() -> Vec<u8> {
    let kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&kp.public());
    let kem_mk = kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub("inj/x"),
        &kdb::det_mlkem768_ek("inj/ek"),
    );
    let doc = kdb::KeySetDocument::v1_hybrid(sig_mk.clone(), kem_mk);
    kdb::did_benten_payload(&sig_mk, &doc.cid())
}

// ── DID-3 — exact-consume injectivity reject matrix (design C1) ───────────

#[test]
fn did3_trailing_byte_after_committed_cid_rejects() {
    // A well-formed did:benten body is EXACTLY signing_multikey ‖ CID(36) —
    // no slack. One extra byte MUST fail closed (HybridTrailingBytes-class).
    let mut payload = honest_payload();
    payload.push(0xFF);
    let did = kdb::did_benten_from_payload_for_test(&payload);
    assert!(
        kdb::resolve_signing(&did).is_err(),
        "resolve_signing MUST reject a did:benten with trailing bytes (exact-consume, C1)"
    );
    assert!(
        kdb::committed_keyset_cid(&did).is_err(),
        "keyset_cid MUST reject a did:benten with trailing bytes"
    );
}

#[test]
fn did3_component_truncation_rejects() {
    // Drop the final byte of the committed CID: the body is now too short
    // for the fixed CIDv1(36) tail → fail closed, never a truncated read.
    let full = honest_payload();
    let truncated = &full[..full.len() - 1];
    let did = kdb::did_benten_from_payload_for_test(truncated);
    assert!(
        kdb::committed_keyset_cid(&did).is_err(),
        "keyset_cid MUST reject a did:benten whose committed CID component is truncated"
    );
    // Also truncate inside the signing multikey (drop the whole CID tail +
    // one signing byte) → signing decode is short.
    let sig_short = &full[..full.len() - CID_LEN - 1];
    let did2 = kdb::did_benten_from_payload_for_test(sig_short);
    assert!(
        kdb::resolve_signing(&did2).is_err(),
        "resolve_signing MUST reject a did:benten whose signing multikey is truncated"
    );
}

#[test]
fn did3_committed_component_wrong_cid_prefix_rejects() {
    // The committed 36-byte tail must be a Benten CIDv1
    // (`0x01,0x71,0x1e,0x20 ‖ 32B`). Corrupt the version byte → the tail is
    // not a valid CID → keyset_cid MUST fail closed (never treat arbitrary
    // 36 bytes as a CID).
    let mut payload = honest_payload();
    let cid_off = payload.len() - CID_LEN;
    payload[cid_off] = 0x02; // not CID_V1 (0x01)
    let did = kdb::did_benten_from_payload_for_test(&payload);
    assert!(
        kdb::committed_keyset_cid(&did).is_err(),
        "keyset_cid MUST reject a committed component whose CIDv1 header is malformed"
    );
}

// ── DID-7 — F2 DoS cap under did:benten (design C7) ───────────────────────

#[test]
fn did7_oversized_did_benten_rejects_fast_not_multi_second_hang() {
    // The `iss` is attacker-controlled per UCAN link. A ~5 MB did:benten
    // string MUST be rejected by `length_pre_check` BEFORE the O(N²)
    // `bs58::decode` runs — fast typed reject, not a hang (design C7).
    let did_oversized = benten_id::did::Did::from_string_for_test_fixture(format!(
        "{}{}",
        kdb::DID_BENTEN_PREFIX,
        "1".repeat(5_000_000)
    ));
    let start = Instant::now();
    let result = kdb::resolve_signing(&did_oversized);
    let elapsed = start.elapsed();
    assert!(
        result.is_err(),
        "an oversized did:benten MUST fail closed (F2 length cap)"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "resolve_signing MUST reject via the length pre-check quickly (got {elapsed:?}); \
         a multi-second time means bs58 O(N²) ran on attacker input (C7 regression)"
    );
}

#[test]
fn did7_valid_did_benten_is_under_the_cap_and_resolves() {
    // A real did:benten (payload ≈ 2024 B → ≈ 2764 base58 chars) is well
    // under MAX_DID_KEY_STRING_LEN = 4096: the cap must not reject any
    // legitimate did:benten.
    let (did, _honest, _attacker) = kdb::substituted_recipient_scenario();
    assert!(
        did.as_str().len() < MAX_DID_KEY_STRING_LEN,
        "a valid did:benten ({} chars) MUST be under MAX_DID_KEY_STRING_LEN ({})",
        did.as_str().len(),
        MAX_DID_KEY_STRING_LEN
    );
    assert!(
        kdb::resolve_signing(&did).is_ok(),
        "a valid did:benten under the cap MUST resolve its signing key"
    );
}
