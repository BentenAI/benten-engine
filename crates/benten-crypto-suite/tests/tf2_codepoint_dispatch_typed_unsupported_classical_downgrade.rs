//! TF-2 pins (b) classical-only downgrade + (c)/(4) typed-unsupported-
//! algorithm arm (NEVER a silent fallback) + S3 UCAN-Varsig-v1 hybrid
//! header round-trip + the hash-codepoint agility seam.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core Wave R3-A,
//! agent R3-A2, family TF-2 (#1300 CANARY). Pin sources:
//!   - r2-test-landscape TF-2 RED-phase shapes (2) classical-only +
//!     (4) typed-unsupported adversarial; the "UCAN Varsig v1 hybrid
//!     header round-trip" + "hash-codepoint agility seam" covers.
//!   - plan G-CORE-2 def (operative codepoint constants the brief carries:
//!     hash multihash `0x1e` BLAKE3 + `0x1015` SHA-512/256 + `0x16`
//!     SHA3-256 pre-blessed fallbacks) + §1.A.FROZEN item 14
//!     (typed-unsupported-never-silent-fallback P2P-interop invariant).
//!   - CLAUDE.md #5 ("codepoint dispatch has a typed unsupported-algorithm
//!     arm (never a silent fallback)").
//!
//! # RED-PHASE STATUS (pim-12 §3.6e)
//!
//! `benten-crypto-suite` is a STUB at R3-A; the intended G-CORE-2 surface
//! does not exist → compile-but-fail at the `use` line. All
//! `#[ignore]`-staged `RED-PHASE: un-ignore at G-CORE-2`.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// RED-PHASE failure point: intended G-CORE-2 codepoint-dispatch surface.
use benten_crypto_suite::codepoint::{HashCodepoint, SigCodepoint, UnsupportedAlgorithm};
use benten_crypto_suite::hash::HashSeam;
use benten_crypto_suite::sig::{SignatureSuite, SuiteConfig};
use benten_crypto_suite::varsig::{UcanVarsigV1Header, VarsigError};

/// Classical-only Ed25519 is a BUILT non-default downgrade config — it
/// must round-trip (sign→verify) on the production path. Pin (b).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_classical_only_downgrade_round_trips() {
    let suite = SignatureSuite::from_config(SuiteConfig::classical_only());
    assert!(
        !suite.is_hybrid(),
        "classical-only config MUST NOT report hybrid"
    );
    let kp = suite.generate_keypair();
    let msg = b"classical-only downgrade arm round-trip";
    let sig = suite.sign(&kp, msg);
    suite
        .verify(kp.public(), msg, &sig)
        .expect("classical-only Ed25519 downgrade MUST round-trip");
}

/// A hybrid-signed object MUST NOT verify under a classical-only suite
/// that silently ignores the PQ half — the downgrade config is NOT a
/// strip-resistance bypass. would-FAIL if the classical-only verifier
/// silently accepts a hybrid sig by ignoring the PQ component.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_classical_only_does_not_silently_accept_hybrid_by_ignoring_pq() {
    let hybrid = SignatureSuite::v1_default();
    let kp = hybrid.generate_keypair();
    let msg = b"signed under hybrid";
    let sig = hybrid.sign(&kp, msg);

    let classical = SignatureSuite::from_config(SuiteConfig::classical_only());
    // The classical-only suite encounters a hybrid-codepoint signature.
    // It MUST surface a typed codepoint mismatch — NOT silently verify
    // the embedded Ed25519 half and call it good.
    let outcome = classical.verify(kp.public(), msg, &sig);
    assert!(
        outcome.is_err(),
        "a classical-only suite MUST NOT silently accept a hybrid-codepoint \
         signature by ignoring its PQ half (that is a silent downgrade)"
    );
}

/// Typed-unsupported SIGNATURE codepoint: an unknown/reserved sig
/// codepoint MUST hit the typed `UnsupportedAlgorithm` arm — NEVER a
/// silent fallback to Ed25519/the default. Pin (4) / §1.A.FROZEN item 14.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_unknown_sig_codepoint_typed_unsupported_never_silent_fallback() {
    // A reserved-but-unimplemented / never-minted sig codepoint.
    let unknown = SigCodepoint::from_raw_for_test(0xFFFF);
    let dispatch = SignatureSuite::resolve_codepoint(unknown);
    assert!(
        matches!(dispatch, Err(UnsupportedAlgorithm::Signature { codepoint }) if codepoint == 0xFFFF),
        "an unknown sig codepoint MUST return a TYPED UnsupportedAlgorithm \
         error — would-FAIL if it silently falls back to the Ed25519/default \
         suite (the load-bearing fail-closed invariant)"
    );
}

/// Typed-unsupported HASH codepoint: the hash-codepoint agility seam MUST
/// accept the pre-blessed set (BLAKE3 `0x1e`, SHA-512/256 `0x1015`,
/// SHA3-256 `0x16`) and reject anything else with the typed arm — never a
/// silent fallback to BLAKE3.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_hash_codepoint_seam_preblessed_set_and_typed_unsupported() {
    // Pre-blessed agile fallbacks MUST all resolve.
    for cp in [
        HashCodepoint::BLAKE3,       // multihash 0x1e
        HashCodepoint::SHA2_512_256, // multihash 0x1015
        HashCodepoint::SHA3_256,     // multihash 0x16
    ] {
        let h = HashSeam::for_codepoint(cp).expect("pre-blessed hash MUST resolve");
        let digest = h.digest(b"agility");
        assert!(!digest.is_empty(), "pre-blessed hash MUST produce a digest");
    }
    // The default hash codepoint MUST be BLAKE3 0x1e (v1 default within
    // the multiformats framing).
    assert_eq!(
        HashSeam::default_codepoint(),
        HashCodepoint::BLAKE3,
        "v1 default hash MUST be BLAKE3 (multihash 0x1e)"
    );
    // An unknown hash codepoint MUST be typed-unsupported, never a
    // silent BLAKE3 fallback.
    let unknown = HashCodepoint::from_raw_for_test(0x9999);
    let outcome = HashSeam::for_codepoint(unknown);
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::Hash { codepoint }) if codepoint == 0x9999),
        "unknown hash codepoint MUST be typed-unsupported, NEVER a silent \
         BLAKE3 fallback; got {outcome:?}"
    );
}

/// UCAN Varsig v1 hybrid header: the hybrid signature serializes into a
/// UCAN-Varsig-v1 header that carries BOTH halves; deserialize→verify
/// round-trips; a header advertising an unknown sig codepoint surfaces a
/// typed error rather than a silent classical fallback. S3.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_ucan_varsig_v1_hybrid_header_round_trip() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"ucan payload";
    let sig = suite.sign(&kp, msg);

    let header = UcanVarsigV1Header::encode_hybrid(&sig);
    assert!(
        header.advertises_hybrid(),
        "the Varsig v1 header MUST advertise the hybrid codepoint (carries \
         both Ed25519 and ML-DSA-65 halves)"
    );

    let decoded = UcanVarsigV1Header::decode(header.as_bytes())
        .expect("valid hybrid Varsig header MUST decode");
    suite
        .verify(kp.public(), msg, &decoded.signature())
        .expect("Varsig-v1-round-tripped hybrid sig MUST still verify");

    // A Varsig header with an unknown codepoint → typed error, not a
    // silent fallback to classical decoding.
    let bogus = UcanVarsigV1Header::with_raw_codepoint_for_test(0xFFFF);
    let outcome = UcanVarsigV1Header::decode(bogus.as_bytes());
    assert!(
        matches!(outcome, Err(VarsigError::UnsupportedCodepoint { .. })),
        "a Varsig header advertising an unknown codepoint MUST surface a \
         typed error, never a silent classical fallback; got {outcome:?}"
    );
}

/// Additive-codepoint discipline: a reserved-but-unimplemented codepoint
/// (e.g. the NF-1 PQ⊕PQ end-state ML-KEM-768⊕HQC reserved codepoint, or a
/// future signature codepoint) MUST hit the typed-unsupported arm — adding
/// it later must not wire-break, and until built it is never a silent
/// fallback. §1.A.FROZEN item 14 (no-wire-break-on-codepoint-add).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_reserved_unimplemented_codepoint_is_typed_unsupported() {
    let reserved = SigCodepoint::reserved_unimplemented_for_test();
    let dispatch = SignatureSuite::resolve_codepoint(reserved);
    assert!(
        matches!(dispatch, Err(UnsupportedAlgorithm::Signature { .. })),
        "a reserved-but-unimplemented codepoint MUST be typed-unsupported \
         (additive-codepoint discipline: reserved now, buildable later, \
         NEVER a silent fallback in the meantime)"
    );
}
