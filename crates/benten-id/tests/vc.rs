//! G14-A2 wave-4a' — Verifiable Credential test pins (un-ignored).
//!
//! Pin sources (per the G14-A2 row in the implementation plan +
//! `crypto-minor-1`):
//!
//! - `vc_issuance_and_verification_round_trip`
//! - `vc_verification_rejects_expired_credential` — `crypto-minor-1`
//! - `vc_verification_rejects_revoked_credential_via_credential_status` — `crypto-minor-1`
//! - `vc_verification_trust_domain_check_against_issuer_allow_list` — `crypto-minor-1`
//!
//! Sibling proptest pin in
//! `crates/benten-id/tests/prop_vc_arbitrary.rs`.

#![allow(clippy::unwrap_used)]

use benten_id::VcError;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_id::vc::{
    Credential, RevocationRegistry, TrustDomain, verify_at, verify_bytes_in_trust_domain,
    verify_in_trust_domain, verify_with_registry,
};

#[test]
fn vc_issuance_and_verification_round_trip() {
    // crypto-minor-1 + plan §3 G14-A2 — issue + verify round-trip.
    let issuer = Keypair::generate();
    let subject = Keypair::generate();
    let issuer_did: Did = issuer.public_key().to_did();
    let subject_did: Did = subject.public_key().to_did();

    let vc: Credential = Credential::builder()
        .issuer(&issuer_did)
        .subject(&subject_did)
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .expires_at(1_000_086_400)
        .sign(&issuer)
        .unwrap();

    // Verify path: signature checks against issuer DID.
    benten_id::vc::verify(&vc, &issuer_did).unwrap();
    assert_eq!(vc.subject(), subject_did.as_str());
    assert_eq!(vc.claim(), ("alumniOf", "ExampleU"));
    assert_eq!(vc.issuer(), issuer_did.as_str());
}

#[test]
fn vc_verification_rejects_expired_credential() {
    // crypto-minor-1 — expired credential rejection at verify_at.
    let issuer = Keypair::generate();
    let subject = Keypair::generate();
    let issuance_secs: u64 = 1_000_000_000;

    let vc = Credential::builder()
        .issuer(&issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(issuance_secs)
        .expires_at(issuance_secs + 60)
        .sign(&issuer)
        .unwrap();

    let err = verify_at(&vc, &issuer.public_key().to_did(), issuance_secs + 120).unwrap_err();
    assert!(matches!(err, VcError::Expired { .. }), "{err:?}");

    // Same VC verifies before exp:
    verify_at(&vc, &issuer.public_key().to_did(), issuance_secs + 30).unwrap();
}

#[test]
fn vc_verification_rejects_revoked_credential_via_credential_status() {
    // crypto-minor-1 — credentialStatus revocation registry consulted.
    let issuer = Keypair::generate();
    let subject = Keypair::generate();

    let vc = Credential::builder()
        .issuer(&issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .credential_status_id("https://example/status/1#42")
        .sign(&issuer)
        .unwrap();

    let registry = RevocationRegistry::new();

    // Initially unrevoked → verify succeeds:
    verify_with_registry(&vc, &issuer.public_key().to_did(), &registry, 1_000_000_001).unwrap();

    // Revoke + retry:
    registry.revoke("https://example/status/1#42");
    let err = verify_with_registry(&vc, &issuer.public_key().to_did(), &registry, 1_000_000_001)
        .unwrap_err();
    assert!(
        matches!(err, VcError::Revoked { ref status_id } if status_id == "https://example/status/1#42"),
        "{err:?}"
    );
}

#[test]
fn vc_verification_trust_domain_check_against_issuer_allow_list() {
    // crypto-minor-1 — trust-domain allow-list gate.
    let trusted_issuer = Keypair::generate();
    let untrusted_issuer = Keypair::generate();
    let subject = Keypair::generate();

    let trust_domain = TrustDomain::new(vec![trusted_issuer.public_key().to_did()]);

    let vc_from_untrusted = Credential::builder()
        .issuer(&untrusted_issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .sign(&untrusted_issuer)
        .unwrap();

    let err = verify_in_trust_domain(&vc_from_untrusted, &trust_domain, 1_000_000_001).unwrap_err();
    assert!(matches!(err, VcError::IssuerNotTrusted { .. }), "{err:?}");

    // Same shape from trusted issuer succeeds:
    let vc_from_trusted = Credential::builder()
        .issuer(&trusted_issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .sign(&trusted_issuer)
        .unwrap();
    verify_in_trust_domain(&vc_from_trusted, &trust_domain, 1_000_000_001).unwrap();
}

/// Pre-freeze audit 2026-08-12 — the composed verify entry points must
/// enforce `expirationDate`, not just signature + allow-list.
///
/// **would-FAIL-on-revert.** Before the required-`now` change,
/// `verify_in_trust_domain` composed the clock-free `verify` and returned
/// `Ok(())` for the expired credential below — an offline reciprocity gate
/// calling it would have admitted a credential that expired a year earlier.
/// Reverting the signature (dropping `now` and calling `verify`) turns the
/// `Expired` assertion here into an `unwrap_err()` panic on `Ok(())`.
///
/// The positive control is load-bearing: without it a guard that rejected
/// *everything* would pass this test for the wrong reason.
#[test]
fn trust_domain_verify_rejects_an_expired_credential_from_a_trusted_issuer() {
    let trusted = Keypair::generate();
    let subject = Keypair::generate();
    let trust_domain = TrustDomain::new(vec![trusted.public_key().to_did()]);

    let issued = 1_000_000_000;
    let vc = Credential::builder()
        .issuer(&trusted.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("reciprocityTier", "partner")
        .issued_at(issued)
        .expires_at(issued + 3600)
        .sign(&trusted)
        .unwrap();

    // Positive control — inside the window, a trusted issuer still verifies.
    verify_in_trust_domain(&vc, &trust_domain, issued + 60)
        .expect("positive control: an unexpired credential from a trusted issuer MUST verify");

    // The substantive arm: expired, trusted, correctly signed → REJECT.
    let err = verify_in_trust_domain(&vc, &trust_domain, issued + 7200).unwrap_err();
    assert!(
        matches!(err, VcError::Expired { .. }),
        "a trusted issuer's EXPIRED credential must be rejected by the trust-domain \
         entry point, not waved through on signature alone; got {err:?}"
    );

    // Same for the raw-bytes boundary, which delegates. Encoded the way the
    // verifier decodes it (`serde_ipld_dagcbor`) rather than through the
    // private canonical-bytes helper, so this exercises the real entry path.
    let bytes = serde_ipld_dagcbor::to_vec(&vc).expect("VC must encode as DAG-CBOR");
    verify_bytes_in_trust_domain(&bytes, &trust_domain, issued + 60)
        .expect("positive control: bytes path must accept inside the window");
    let err = verify_bytes_in_trust_domain(&bytes, &trust_domain, issued + 7200).unwrap_err();
    assert!(
        matches!(err, VcError::Expired { .. }),
        "the raw-bytes trust-domain boundary must enforce expiry too; got {err:?}"
    );
}

/// Companion to the above for the revocation entry point, and the pin for
/// the fact that **expiry and revocation are independent axes**.
///
/// **would-FAIL-on-revert** the same way: `verify_with_registry` previously
/// composed the clock-free `verify`, so an expired-but-never-revoked
/// credential returned `Ok(())` from a function whose whole job is to say
/// whether a credential is still good.
#[test]
fn registry_verify_rejects_expired_even_when_the_registry_says_unrevoked() {
    let issuer = Keypair::generate();
    let subject = Keypair::generate();
    let issued = 1_000_000_000;

    let vc = Credential::builder()
        .issuer(&issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(issued)
        .expires_at(issued + 3600)
        .credential_status_id("https://example/status/1#7")
        .sign(&issuer)
        .unwrap();

    // Registry is EMPTY — nothing is revoked. That must not imply "still valid".
    let registry = RevocationRegistry::new();

    verify_with_registry(&vc, &issuer.public_key().to_did(), &registry, issued + 60)
        .expect("positive control: unexpired + unrevoked MUST verify");

    let err = verify_with_registry(&vc, &issuer.public_key().to_did(), &registry, issued + 7200)
        .unwrap_err();
    assert!(
        matches!(err, VcError::Expired { .. }),
        "an unrevoked credential that has EXPIRED must still be rejected — \
         'not in the revocation list' is not 'valid'; got {err:?}"
    );
}

#[test]
fn vc_verification_rejects_tampered_claims() {
    // Defense-in-depth — tampering claims invalidates signature.
    let issuer = Keypair::generate();
    let subject = Keypair::generate();
    let mut vc = Credential::builder()
        .issuer(&issuer.public_key().to_did())
        .subject(&subject.public_key().to_did())
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .sign(&issuer)
        .unwrap();
    // Mutate the claim value:
    vc.claims.credential_subject.claim_value = "TamperU".to_string();
    let err = benten_id::vc::verify(&vc, &issuer.public_key().to_did()).unwrap_err();
    assert!(matches!(err, VcError::BadSignature), "{err:?}");
}
