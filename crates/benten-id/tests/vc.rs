//! R3-B RED-PHASE pins for `benten-id` Verifiable Credential surface
//! (G14-A2 wave-4a').
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G14-A2 must-pass column):
//!
//! - `tests/vc_issuance_and_verification_round_trip` — plan §3 G14-A2
//! - `tests/vc_verification_rejects_expired_credential` — `crypto-minor-1`
//! - `tests/vc_verification_rejects_revoked_credential_via_credential_status` — `crypto-minor-1`
//! - `tests/vc_verification_trust_domain_check_against_issuer_allow_list` — `crypto-minor-1`
//!
//! Sibling proptest pin in `crates/benten-id/tests/prop_vc_arbitrary.rs`
//! (`prop_vc_verification_arbitrary_malformed_input_no_panic`).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent (PR #85): every test stays `#[ignore]`'d
//! until G14-A2 wave-4a' implementer un-ignores AND replaces the
//! `unimplemented!()` body with a real assertion against the live
//! `benten_id::vc` API. The cited `benten_id::vc::*` types do not exist
//! at R3-B landing time — the crate is the empty stub landed at R3-A.
//!
//! Per `feedback_end_to_end_test_pin_for_closed_claims` (§3.6b pim-2),
//! sentinel-presence (`vc.is_some()`) does not satisfy the close-claim;
//! the un-ignored test must drive the production VC issuance + verify
//! entry point and assert observable behavioral consequence.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 wave-4a' fills benten-id::vc (issuance + verify round-trip)"]
fn vc_issuance_and_verification_round_trip() {
    // plan §3 G14-A2 pin. G14-A2 implementer wires this against the
    // real API:
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let subject = benten_id::keypair::Keypair::generate();
    //   let vc = benten_id::vc::Credential::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .subject(subject.public_key().to_did())
    //       .claim("alumniOf", "ExampleU")
    //       .issued_at_now()
    //       .exp_in_secs(86400)
    //       .sign(&issuer)
    //       .unwrap();
    //   let verified = benten_id::vc::verify(&vc, &issuer.public_key().to_did()).unwrap();
    //   assert_eq!(verified.subject(), subject.public_key().to_did());
    //   assert_eq!(verified.claim("alumniOf"), Some("ExampleU"));
    //
    // OBSERVABLE consequence: the issuance → verify path round-trips
    // a structured claim end-to-end; tampered VCs fail verify
    // (covered in subsequent tests).
    unimplemented!("G14-A2 wires Credential::builder() → sign() → verify() round-trip");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-1 — expired credential rejection"]
fn vc_verification_rejects_expired_credential() {
    // crypto-minor-1 pin. The VC `expirationDate` field MUST be
    // enforced at verification time. A credential presented after its
    // exp window rejects with a typed error.
    //
    // Implementer wires:
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let issuance_secs = 1_000_000_000;
    //   let vc = benten_id::vc::Credential::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .issued_at(issuance_secs)
    //       .exp_in_secs(60)        // expires at issuance_secs + 60
    //       .sign(&issuer)
    //       .unwrap();
    //   let err = benten_id::vc::verify_at(
    //       &vc,
    //       &issuer.public_key().to_did(),
    //       issuance_secs + 120,
    //   ).unwrap_err();
    //   assert!(matches!(err, benten_id::vc::VerifyError::Expired { .. }));
    //
    // OBSERVABLE consequence: post-expiration credential rejects with
    // a typed Expired error variant carrying the exp timestamp for
    // diagnostic reporting.
    unimplemented!("G14-A2 wires VC exp-window rejection with typed Expired variant");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-1 — credentialStatus revocation"]
fn vc_verification_rejects_revoked_credential_via_credential_status() {
    // crypto-minor-1 pin. The W3C VC `credentialStatus` field names a
    // revocation registry; verification MUST consult the registry and
    // reject if the credential has been revoked.
    //
    // Implementer wires:
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let revocation_registry = benten_id::vc::RevocationRegistry::new();
    //   let vc = benten_id::vc::Credential::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .credential_status_id("https://example/status/1#42")
    //       .sign(&issuer)
    //       .unwrap();
    //   // Initially unrevoked → verify succeeds:
    //   benten_id::vc::verify_with_registry(&vc, &issuer.public_key().to_did(), &revocation_registry).unwrap();
    //   // Revoke + retry:
    //   revocation_registry.revoke("https://example/status/1#42");
    //   let err = benten_id::vc::verify_with_registry(
    //       &vc, &issuer.public_key().to_did(), &revocation_registry).unwrap_err();
    //   assert!(matches!(err, benten_id::vc::VerifyError::Revoked { .. }));
    //
    // OBSERVABLE consequence: the same VC verifies pre-revoke and
    // rejects post-revoke; the revocation registry consultation is
    // load-bearing in the verify flow.
    unimplemented!("G14-A2 wires credentialStatus revocation registry consultation at verify");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-1 — issuer trust-domain allow-list"]
fn vc_verification_trust_domain_check_against_issuer_allow_list() {
    // crypto-minor-1 pin. Even a syntactically valid VC signed by a
    // legitimate Ed25519 keypair MUST reject if the issuer DID is not
    // in the verifier's trust-domain allow-list. Defends against the
    // "anyone with an Ed25519 keypair can mint VCs" attack class.
    //
    // Implementer wires:
    //
    //   let trusted_issuer = benten_id::keypair::Keypair::generate();
    //   let untrusted_issuer = benten_id::keypair::Keypair::generate();
    //   let trust_domain = benten_id::vc::TrustDomain::new(
    //       vec![trusted_issuer.public_key().to_did()]
    //   );
    //
    //   let vc_from_untrusted = benten_id::vc::Credential::builder()
    //       .issuer(untrusted_issuer.public_key().to_did())
    //       .sign(&untrusted_issuer)
    //       .unwrap();
    //
    //   let err = benten_id::vc::verify_in_trust_domain(
    //       &vc_from_untrusted, &trust_domain).unwrap_err();
    //   assert!(matches!(err, benten_id::vc::VerifyError::IssuerNotTrusted { .. }));
    //
    //   // Same VC from trusted issuer succeeds:
    //   let vc_from_trusted = benten_id::vc::Credential::builder()
    //       .issuer(trusted_issuer.public_key().to_did())
    //       .sign(&trusted_issuer)
    //       .unwrap();
    //   benten_id::vc::verify_in_trust_domain(&vc_from_trusted, &trust_domain).unwrap();
    //
    // OBSERVABLE consequence: signature-valid VCs from untrusted
    // issuers reject with typed IssuerNotTrusted variant; same shape
    // from trusted issuer verifies. Trust-domain check is an
    // independent gate on top of signature verification.
    unimplemented!("G14-A2 wires issuer trust-domain allow-list gate at VC verify");
}
