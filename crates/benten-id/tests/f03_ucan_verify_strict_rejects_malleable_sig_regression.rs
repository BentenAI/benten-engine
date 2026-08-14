//! F-03 regression (fix b) — the UCAN authority chain-walk verify uses
//! Ed25519 `verify_strict`.
//!
//! The chain-walk (`validate_chain_at` → `authority_verify::
//! verify_authority_signature` → `benten_crypto_suite::sig::
//! SignatureSuite::verify`) `verify_strict`s the classical half at both the
//! classical-only and the LAMPS-hybrid arms. `verify_strict` is strictly
//! STRONGER than the lenient `verify`: on top of scalar-`S` canonicity it also
//! rejects a non-canonically-encoded `R` point and a small-order public key —
//! closing signature malleability comprehensively.
//!
//! ## Honest scope note (ground-truthed 2026-07-20)
//!
//! In THIS workspace's `ed25519-dalek`, the lenient `verify` ALREADY rejects a
//! non-canonical scalar `S` (`S >= L`) — verified directly: for an `S + L`
//! malleation, `verify` and `verify_strict` BOTH reject, while the original
//! signature verifies. So the specific "malleate `S` → distinct `ucan_cid` →
//! dodge revocation" step from the F-03 finding is already blocked at the base
//! `verify` here; `verify_strict` is the belt-and-suspenders hardening for the
//! `R`-encoding / small-order-key axes (un-constructible for a real,
//! honestly-generated issuer key, hence not behaviorally reproducible in a
//! stable unit test). The LOAD-BEARING revocation-bypass closure for the
//! realistic vector — a re-randomized ML-DSA-65 *hybrid* signature over the
//! same claims yielding a distinct envelope-CID — is the payload-CID revocation
//! re-key (F-03 fix a; `benten-caps` test
//! `f03_revocation_payload_cid_sig_independent_regression.rs`, which DOES
//! fail-on-revert).
//!
//! This file therefore guards fix (b) two ways:
//! 1. a SOURCE pin that the crypto-suite authority Ed25519 verify sites use
//!    `verify_strict` (would-FAIL-on-revert of the `verify_strict`→`verify`
//!    change — the clean regression guard for the specific hardening); and
//! 2. a behavioral PROPERTY pin that a non-canonical-`S` malleated signature is
//!    rejected on the UCAN authority path (guards the malleability-rejection
//!    property against a future switch to a genuinely-lenient verify).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::UcanError;
use benten_id::keypair::Keypair;
use benten_id::ucan::{Ucan, validate_chain_at};

/// The Ed25519 group order `L` in little-endian (32 bytes). `S' = S + L`
/// is the classic non-canonical-`S` malleation.
const ED25519_L_LE: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

/// Return a byte-distinct, non-canonical 64-byte Ed25519 signature over the
/// SAME message: `R` unchanged, `S' = S + L`.
fn malleate_ed25519_sig(sig: &[u8]) -> Vec<u8> {
    assert_eq!(sig.len(), 64, "classical Ed25519 signature is 64 bytes");
    let mut out = sig.to_vec();
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = u16::from(out[32 + i]) + u16::from(ED25519_L_LE[i]) + carry;
        out[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    assert_eq!(carry, 0, "S + L must fit in 32 bytes");
    out
}

/// SOURCE fails-on-revert pin: the crypto-suite authority Ed25519 verify sites
/// use `verify_strict`. Reverting either arm to the lenient `.verify(` makes
/// this fire.
#[test]
fn crypto_suite_authority_ed25519_verify_is_strict_f03() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../benten-crypto-suite/src/sig.rs"
    );
    let src = std::fs::read_to_string(path).expect("crypto-suite sig.rs readable");
    let squished: String = src.chars().filter(|c| !c.is_whitespace()).collect();

    // Both authority Ed25519 verify sites: the classical-only arm (over `msg`)
    // and the LAMPS-hybrid arm (over `m_prime`) MUST be strict.
    assert!(
        squished.contains(".verify_strict(msg,&classical_sig)"),
        "F-03 REGRESSION (fix b): the classical-only authority Ed25519 verify \
         MUST use `verify_strict` (reverted to lenient `.verify`)"
    );
    assert!(
        squished.contains(".verify_strict(&m_prime,&classical_sig)"),
        "F-03 REGRESSION (fix b): the LAMPS-hybrid authority Ed25519 verify \
         MUST use `verify_strict` (reverted to lenient `.verify`)"
    );
    // Neither authority Ed25519 half may use the lenient form.
    assert!(
        !squished.contains(".classical.verify(msg,&classical_sig)")
            && !squished.contains(".classical.verify(&m_prime,&classical_sig)"),
        "F-03 REGRESSION (fix b): an authority Ed25519 verify uses the lenient \
         `.verify` — it MUST be `verify_strict`"
    );
}

/// PROPERTY pin: a non-canonical-`S` malleated signature is REJECTED on the
/// UCAN authority chain-walk. (In this dalek version the base `verify` already
/// enforces `S`-canonicity — see the module note; this guards the property
/// against a future switch to a genuinely-lenient verify.)
#[test]
fn ucan_authority_verify_rejects_noncanonical_s_malleated_signature_f03() {
    let now = 1_000_000_000u64;
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let token = Ucan::builder()
        .issuer_did(&issuer.public_key().to_did())
        .audience_did(&audience.public_key().to_did())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&issuer);

    // Sanity anchor: the honest, canonical signature verifies.
    validate_chain_at(std::slice::from_ref(&token), now)
        .expect("the honest canonical signature must verify");

    let mut malleated = token.clone();
    malleated.signature = malleate_ed25519_sig(&token.signature);
    assert_ne!(malleated.signature, token.signature);

    let err = validate_chain_at(std::slice::from_ref(&malleated), now).expect_err(
        "a malleated (non-canonical S) UCAN signature MUST be rejected on the \
         authority path",
    );
    assert!(
        matches!(err, UcanError::BadSignature { .. }),
        "expected UcanError::BadSignature; got {err:?}",
    );
}
