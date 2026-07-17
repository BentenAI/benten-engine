//! TF-3e — R6 R2 fix-pass (Bundle R6-R2-FP-A): ARM 2 / ARM 5
//! audience-substitution integration pin.
//!
//! Sibling/integration pin to `tf3b_audience_substitution_post_sign_rejected.rs`
//! (which exercises the bare `verify_binding` API). This pin exercises
//! the FULL `UcanBlobsHandler::validate_request_for_connection`
//! pipeline — specifically the ARM 2 / ARM 5 split that L2-R2-BLOCKER-1
//! named as the production-pathway hazard:
//!
//!   ARM 2 (audience-binding) compared the connection's verified
//!   `EndpointId` to `grant.audience_pubkey` (the field an attacker
//!   could mutate post-sign);
//!
//!   ARM 5 (binding-sig verify) re-constructed the binding-message
//!   using `grant.audience_binding` (CID of the issue-time audience
//!   pubkey — which the attacker could leave UNCHANGED).
//!
//! The split meant a cooperating attacker (Eve obtains Alice's grant
//! for Bob, mutates `audience_pubkey` to her own pubkey, connects with
//! her own iroh EndpointId) passed both ARM 2 (eve_pubkey == eve_pubkey)
//! and ARM 5 (binding_sig over bob's audience_binding still verifies)
//! — silent access-theft.
//!
//! Post R6-R2-FP-A: `audience_pubkey` is folded into the
//! binding-message (`BINDING_SIG_DOMAIN`, now **v4 / 7-segment self-bind**
//! after the subsequent `issuer_verifying_key` self-bind bump — R6-R2-FP-A
//! itself landed the v3/6-segment audience_pubkey fold); the
//! same attacker mutation now causes ARM 5 to surface
//! `BindingSigInvalid` BEFORE ARM 2's pubkey-comparison admits her.
//! This pin exercises the production pipeline end-to-end.
//!
//! Per pim-18 §3.6f SHAPE-not-SUBSTANCE: the production entry point
//! is `validate_request_for_connection` (called from the iroh ALPN
//! handler dispatch), not `verify_binding` directly. Both pins are
//! load-bearing — the bare-API pin catches regressions in the
//! binding_message construction; this pin catches regressions in the
//! arm-ordering / ARM 5 wiring that exposes the bare-API to the
//! production path.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedScope;
use benten_id::keypair::Keypair;
use benten_sync::ucan_blobs_protocol::{UcanBlobsHandler, UcanBlobsHandlerError, UcanBlobsRequest};

#[test]
fn audience_substitution_through_handler_rejected_at_arm5() {
    // Setup: Alice (issuer / serving peer) issues a grant for Bob.
    let alice_kp = Keypair::generate();
    let bob_kp = Keypair::generate();
    let eve_kp = Keypair::generate();
    let bob_pubkey = bob_kp.public_key();
    let eve_pubkey = eve_kp.public_key();

    // Alice serves; her handler clock is past nbf=0 / before
    // exp=u64::MAX so time-based arms ARM 3 / ARM 4 don't fire.
    let handler = UcanBlobsHandler::new_with_clock(&alice_kp, 1_000_000);

    // Grant scope covers a specific ciphertext_hash.
    let attacker_target_hash: benten_core::Cid = [0xAA_u8; 32].into();
    let scope = RestrictedScope::with_hashes(vec![attacker_target_hash]);
    let grant = AuthorizationGrant::issue_for_test(&alice_kp, &bob_pubkey, scope, 9_999_999_999);

    // Positive control: Bob's own request validates cleanly. We use
    // Bob's pubkey both as the connection EndpointId (Spike A2:
    // EndpointId IS the requester's Ed25519 pubkey) and as the
    // grant.audience_pubkey (which the grant was issued bound to at
    // issue time).
    let req_bob = UcanBlobsRequest {
        grant: grant.clone(),
        ciphertext_hash: attacker_target_hash,
    };
    handler
        .validate_request_for_connection(&req_bob, &bob_pubkey)
        .expect("Bob's own request MUST validate cleanly");

    // Attacker (Eve) tampers: she mutates `audience_pubkey` to her
    // own pubkey, then connects with her own EndpointId. Pre-fix,
    // ARM 2 admits her (eve_pubkey == eve_pubkey) and ARM 5
    // re-verifies binding_sig against grant.audience_binding =
    // CID(bob_pubkey) which IS bound — silent access-theft.
    let tampered_grant =
        grant.with_swapped_audience_pubkey_for_test(Some(eve_pubkey.to_bytes().to_vec()));
    let req_eve = UcanBlobsRequest {
        grant: tampered_grant,
        ciphertext_hash: attacker_target_hash,
    };

    // Post-fix: ARM 5 (binding-sig verify) MUST surface
    // `BindingSigInvalid` because the binding-message now binds
    // `audience_pubkey` and the tampered value flips the message
    // bytes vs the issue-time signed message. The arm-ordering puts
    // ARM 5 AFTER ARM 2 but BindingSigInvalid is the failure shape
    // that surfaces because the binding-message re-construction
    // uses self.audience_pubkey (the mutated value) and Ed25519
    // verify against the original signature fails.
    let outcome = handler.validate_request_for_connection(&req_eve, &eve_pubkey);
    assert!(
        matches!(
            outcome,
            Err(UcanBlobsHandlerError::BindingSigInvalid { .. })
        ),
        "Eve's audience-substitution attack MUST surface as \
         BindingSigInvalid at the ARM 5 binding-sig layer — NOT \
         silently admitted by the ARM 2 EndpointId match. \
         R6-R2-FP-A L2-R2-BLOCKER-1 closure. \
         Got: {outcome:?}"
    );
}

#[test]
fn audience_substitution_eve_connecting_as_bob_rejected_at_arm2() {
    // Symmetric: what if Eve doesn't tamper the grant at all but
    // tries to connect with Bob's pubkey (impersonation at the iroh
    // layer)? This is the audience-mismatch arm — ARM 2 must reject
    // before ARM 5. The protection here is at the iroh QUIC layer
    // (Eve cannot forge an Ed25519 signature for Bob's pubkey
    // without Bob's private key) but the typed handler arm STILL
    // fires as the defense-in-depth layer.
    //
    // This is NOT the L2-R2-BLOCKER-1 attack (which is the
    // post-sign-mutation variant) — it's the orthogonal
    // EndpointId-mismatch case. Including it ensures the arm-
    // ordering doesn't regress in a way that admits one variant
    // while rejecting the other.
    let alice_kp = Keypair::generate();
    let bob_kp = Keypair::generate();
    let eve_kp = Keypair::generate();
    let bob_pubkey = bob_kp.public_key();
    let eve_pubkey = eve_kp.public_key();
    let _ = eve_pubkey; // referenced for arc clarity below

    let handler = UcanBlobsHandler::new_with_clock(&alice_kp, 1_000_000);

    let target_hash: benten_core::Cid = [0xBB_u8; 32].into();
    let scope = RestrictedScope::with_hashes(vec![target_hash]);
    let grant = AuthorizationGrant::issue_for_test(&alice_kp, &bob_pubkey, scope, 9_999_999_999);

    // Eve sends Bob's untampered grant but connects with her own
    // EndpointId. ARM 2 MUST reject (eve_pubkey != bob_pubkey).
    let req = UcanBlobsRequest {
        grant,
        ciphertext_hash: target_hash,
    };
    let outcome = handler.validate_request_for_connection(&req, &eve_pubkey);
    assert!(
        matches!(
            outcome,
            Err(UcanBlobsHandlerError::UcanAudienceMismatch { .. })
        ),
        "Eve presenting Bob's untampered grant with her own EndpointId \
         MUST be rejected at ARM 2 UcanAudienceMismatch. \
         Got: {outcome:?}"
    );
}
