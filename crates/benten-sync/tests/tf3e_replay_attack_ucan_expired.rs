//! TF-3e pins — G-CORE-3e UCAN replay-attack defense (expired tokens).
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3e (A-1):
//!     "Replay attack: Bob re-presents an expired UCAN (`exp` past) →
//!     typed `UcanExpired`. Couples §4.37 InstallRecord replay-defense
//!     pattern."
//!   - §5 adversarial: "Replay attack (UCAN expiry + install-record).
//!     Owner: G-CORE-3e (UCAN expiry) + G-CORE-8 §4.37 (InstallRecord
//!     replay-store). Test: re-present an expired UCAN → typed
//!     `UcanExpired`; re-present a previously-applied InstallRecord →
//!     typed `InstallRecordAlreadyApplied`; record-and-check is atomic
//!     around admission."
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R6
//!     "Revocation reach": UCAN revocation cuts future serves
//!     (already-derived keys remain decryptable; mitigation = tight
//!     nbf/exp + key rotation).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3e (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_id::keypair::Keypair;
// RED-PHASE failure points.
use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedSpec;
use benten_sync::ucan_blobs_protocol::{UcanBlobsHandler, UcanBlobsHandlerError, UcanBlobsRequest};

// ---------------------------------------------------------------------------
// PIN 1 — A-1: Expired UCAN (exp in the past) → typed UcanExpired.
// ---------------------------------------------------------------------------
// Bob presents a previously-valid UCAN whose `exp` field is now in the
// past. The handler validates expiry against an injected clock and
// returns typed `UcanExpired`. Would-FAIL-IF-NO-OP'd: if the handler
// skips expiry check, an attacker who steals a never-expired UCAN can
// re-present it indefinitely.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3e"]
fn tf3e_expired_ucan_replay_yields_typed_ucan_expired() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();

    // Inject a clock far in the future relative to the UCAN's exp.
    let now_secs: u64 = 1_900_000_000;
    let exp_secs: u64 = 1_000_000_000; // past

    let handler = UcanBlobsHandler::new_with_clock(&kp_alice, now_secs);

    let spec = RestrictedSpec::with_hashes(vec![[11u8; 32].into()]);
    let expired_grant =
        AuthorizationGrant::issue_for_test(&kp_alice, kp_bob.public_key(), spec, exp_secs);

    let req = UcanBlobsRequest {
        grant: expired_grant,
        ciphertext_hash: [11u8; 32].into(),
    };
    let result = handler.validate_request_for_connection(&req, &kp_bob.public_key());
    assert!(
        matches!(result, Err(UcanBlobsHandlerError::UcanExpired { .. })),
        "Expired UCAN (exp in the past relative to injected clock) MUST \
         yield typed UcanExpired. NEVER silently serve bytes. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — UCAN with `nbf` in the future also rejected.
// ---------------------------------------------------------------------------
// Defense-in-depth: not-before-time check. A UCAN whose `nbf` is in the
// future (relative to the injected clock) MUST also be rejected.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3e"]
fn tf3e_ucan_nbf_in_future_typed_not_yet_valid() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();

    let now_secs: u64 = 1_000_000_000;
    let nbf_secs: u64 = 2_000_000_000; // future

    let handler = UcanBlobsHandler::new_with_clock(&kp_alice, now_secs);
    let spec = RestrictedSpec::with_hashes(vec![[13u8; 32].into()]);
    let early_grant = AuthorizationGrant::issue_with_nbf_for_test(
        &kp_alice,
        kp_bob.public_key(),
        spec,
        nbf_secs,
        u64::MAX,
    );

    let req = UcanBlobsRequest {
        grant: early_grant,
        ciphertext_hash: [13u8; 32].into(),
    };
    let result = handler.validate_request_for_connection(&req, &kp_bob.public_key());
    assert!(
        matches!(
            result,
            Err(UcanBlobsHandlerError::UcanNotYetValid { .. })
                | Err(UcanBlobsHandlerError::UcanExpired { .. })
        ),
        "UCAN with `nbf` in the future MUST be rejected (typed \
         UcanNotYetValid or UcanExpired). NEVER silently serve bytes. \
         got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — Revocation cuts future serves (R6 reach).
// ---------------------------------------------------------------------------
// Couples R6 revocation reach. After a UCAN is explicitly revoked by
// the issuer, the handler refuses subsequent requests presenting the
// revoked grant. Already-decrypted plaintext is NOT revoked
// (cryptographic limit; documented at SECURITY-POSTURE.md per R6).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3e"]
fn tf3e_revoked_grant_yields_typed_revoked() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);

    let spec = RestrictedSpec::with_hashes(vec![[15u8; 32].into()]);
    let grant = AuthorizationGrant::issue_for_test(&kp_alice, kp_bob.public_key(), spec, u64::MAX);
    let grant_cid = grant.grant_cid_for_test();

    // Alice revokes the grant. The handler observes the revocation
    // via the revocation store (couples Phase-3 revocation infrastructure).
    handler.record_revocation_for_test(&grant_cid);

    let req = UcanBlobsRequest {
        grant,
        ciphertext_hash: [15u8; 32].into(),
    };
    let result = handler.validate_request_for_connection(&req, &kp_bob.public_key());
    assert!(
        matches!(result, Err(UcanBlobsHandlerError::GrantRevoked { .. })),
        "Post-revocation grant request MUST yield typed GrantRevoked. \
         R6 reach: future serves cut. (Already-derived plaintext at \
         Bob's side remains decryptable — that's the documented \
         cryptographic limit.) got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
