//! F2 pre-auth DoS closure pins: DID-resolution length cap.
//!
//! `bs58::decode` is O(N²) in the body length, and the live UCAN chain
//! validators (`validate_chain_for_capability` → `validate_chain_inner`)
//! resolve the attacker-controlled `iss` DID of every chain link BEFORE
//! verifying its signature. Without the length gate a multi-MB junk
//! `iss` × up to `MAX_UCAN_PROOF_DEPTH` links is quadratic-CPU
//! exhaustion reachable pre-signature-check.
//!
//! These pins exercise the REAL length pre-check arm: they would FAIL
//! (hang for many seconds, or return the wrong error) if the
//! `length_pre_check` gate at the top of `Did::resolve` /
//! `Did::resolve_hybrid` were reverted.

#![allow(clippy::unwrap_used)]

use std::time::{Duration, Instant};

use benten_id::DidError;
use benten_id::did::{Did, MAX_DID_KEY_STRING_LEN};
use benten_id::keypair::Keypair;
use benten_id::ucan::{Capability, Ucan, validate_chain_for_capability};

/// A well-formed `did:key` prefix followed by an oversized base58-ish
/// body — far longer than any structurally-valid DID, so it must be
/// rejected by the length gate before base58 decode runs.
fn oversized_did_string(len: usize) -> String {
    // '1' is a valid base58btc alphabet char, so the string is
    // structurally plausible input for the decoder (the point is that
    // the LENGTH gate fires before decode, not the alphabet).
    let mut s = String::with_capacity(len + 16);
    s.push_str("did:key:z");
    s.extend(std::iter::repeat_n('1', len));
    s
}

#[test]
fn resolve_rejects_oversized_did_fast_with_body_too_long() {
    // ~10 MB junk body: without the gate this feeds bs58's O(N²) decode
    // (multi-second). With the gate it must return BodyTooLong in well
    // under a second.
    let did = Did::from_string_for_test_fixture(oversized_did_string(10 * 1024 * 1024));
    let start = Instant::now();
    let result = did.resolve();
    let elapsed = start.elapsed();

    match result {
        Err(DidError::BodyTooLong { got, max }) => {
            assert_eq!(max, MAX_DID_KEY_STRING_LEN, "max must equal the cap const");
            assert!(
                got > max,
                "reported got ({got}) must exceed the cap ({max})"
            );
        }
        other => panic!("expected BodyTooLong, got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_millis(500),
        "oversized DID resolve must reject fast (pre-decode), took {elapsed:?}"
    );
}

#[test]
fn resolve_hybrid_rejects_oversized_did_fast_with_body_too_long() {
    let did = Did::from_string_for_test_fixture(oversized_did_string(10 * 1024 * 1024));
    let start = Instant::now();
    let result = did.resolve_hybrid();
    let elapsed = start.elapsed();

    // Note: the hybrid PublicKey intentionally does not implement Debug
    // (secret-hygiene), so match the error arm explicitly rather than
    // `{result:?}`.
    match result {
        Err(DidError::BodyTooLong { .. }) => {}
        Err(other) => panic!("oversized hybrid DID must reject with BodyTooLong, got {other:?}"),
        Ok(_) => panic!("oversized hybrid DID must reject with BodyTooLong, got Ok"),
    }
    assert!(
        elapsed < Duration::from_millis(500),
        "oversized hybrid DID resolve must reject fast (pre-decode), took {elapsed:?}"
    );
}

#[test]
fn resolve_accepts_did_at_the_cap_boundary_unaffected() {
    // A DID string exactly at the cap length must NOT be rejected by the
    // length gate (it fails later, on base58/multicodec grounds, but
    // NOT with BodyTooLong) — proves the gate is a filter on oversized
    // input, not a blanket rewrite that changes valid-shape outcomes.
    let did = Did::from_string_for_test_fixture(oversized_did_string(
        MAX_DID_KEY_STRING_LEN - "did:key:z".len(),
    ));
    let result = did.resolve();
    assert!(
        !matches!(result, Err(DidError::BodyTooLong { .. })),
        "a DID at the cap boundary must not trip BodyTooLong, got {result:?}"
    );
}

#[test]
fn real_did_key_resolves_unchanged_under_the_cap() {
    // Regression guard: a genuine Ed25519 did:key must still resolve to
    // its exact pubkey — the cap must not perturb any legitimate DID.
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    assert!(
        did.as_str().len() < MAX_DID_KEY_STRING_LEN,
        "a real did:key is far under the cap"
    );
    let resolved = did.resolve().expect("real did:key must resolve");
    assert_eq!(
        resolved.to_bytes(),
        kp.public_key().to_bytes(),
        "resolve must recover the exact pubkey bytes"
    );
}

#[test]
fn ucan_chain_with_oversized_iss_rejects_fast_not_multi_second_hang() {
    // The load-bearing end-to-end pin: the production chain validator
    // resolves the attacker-controlled leaf `iss` BEFORE signature
    // verification. Build a real signed leaf, then overwrite its `iss`
    // with a ~10 MB junk string. The walker must reject quickly
    // (BadSignature — resolve errors map to BadSignature at the chain
    // walk) rather than hang in bs58's O(N²) decode.
    let kp = Keypair::generate();
    let aud = Keypair::generate();
    let aud_did = aud.public_key().to_did();

    let mut leaf = Ucan::builder()
        .issuer(kp.public_key().to_did().as_str())
        .audience(aud_did.as_str())
        .capability("/zone/posts", "read")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&kp);

    // Attacker inflates the issuer DID string. This is the exact field
    // the chain walker feeds to Did::resolve() before checking the sig.
    leaf.claims.iss = oversized_did_string(10 * 1024 * 1024);

    let required = Capability::new("/zone/posts", "read");
    let chain = [leaf];

    let start = Instant::now();
    let result = validate_chain_for_capability(&chain, &aud_did, &required, 1);
    let elapsed = start.elapsed();

    assert!(
        result.is_err(),
        "chain with oversized iss must be rejected, got {result:?}"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "oversized-iss chain validation must reject fast (pre-decode length gate), \
         took {elapsed:?}"
    );
}

#[test]
fn resolve_rejects_trailing_bytes_after_ed25519_key_f04() {
    // F-04: the classical `did:key` codec must EXACT-consume. A body of
    // `0xed01 ‖ pk ‖ <trailing>` must be REJECTED — NOT silently
    // truncated to the same key as the canonical form (that made the
    // codec non-injective and diverged `resolve` from the exact-consuming
    // `resolve_signing`). Would-FAIL-on-revert of the `!= 2+32` gate.
    let kp = Keypair::generate();
    let canonical = kp.public_key().to_did();

    // Control: the canonical did:key resolves fine.
    canonical.resolve().expect("canonical did:key must resolve");

    // Rebuild the body with ONE trailing junk byte appended.
    let body = canonical
        .as_str()
        .strip_prefix("did:key:z")
        .expect("did:key:z prefix");
    let mut decoded = bs58::decode(body).into_vec().expect("valid base58 body");
    assert_eq!(
        decoded.len(),
        2 + 32,
        "canonical ed25519 did:key body is 0xed01 ‖ 32-byte pk"
    );
    decoded.push(0xFF);
    let tampered = Did::from_string_for_test_fixture(format!(
        "did:key:z{}",
        bs58::encode(&decoded).into_string()
    ));

    match tampered.resolve() {
        Err(DidError::BodyTooShort { got, min }) => {
            assert_eq!(min, 2 + 32, "expected-length const");
            assert_eq!(
                got,
                2 + 32 + 1,
                "the trailing byte must be COUNTED, not ignored"
            );
        }
        other => panic!(
            "F-04 REGRESSION: trailing bytes after the ed25519 key must be \
             rejected (exact-consume), got {other:?}"
        ),
    }
}
