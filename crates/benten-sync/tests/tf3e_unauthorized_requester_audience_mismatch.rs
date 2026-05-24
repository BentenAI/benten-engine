//! TF-3e pins — G-CORE-3e unauthorized requester audience mismatch.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3e (F-1):
//!     "Unauthorized requester (UCAN audience ≠ requester EndpointId):
//!     handler returns typed `UcanAudienceMismatch` → connection closed."
//!   - §2 G-CORE-3e (A-2): "Audience-substitution attack: Bob presents
//!     Carol's UCAN (audience binding); handler verifies audience matches
//!     connection's verified EndpointId → fails."
//!   - §5 adversarial pattern row: "Unresolvable peer-DID at recheck +
//!     sync-hydrate denial. Owner: G-CORE-8 (security-r1-2). Test:
//!     present sentinel `<unresolved-peer>` peer-DID at §4.36 recheck
//!     path AND §4.25 sync-hydrate path; both return `UnresolvedDeny`,
//!     NEVER `Admitted`."
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R3 — the
//!     audience-binding property of `AuthorizationGrant` (the
//!     binding_sig binds UCAN audience to key material; mismatch =
//!     refused).
//!
//! ============================================================================
//! LANDED at G-CORE-3e (pim-12 / §3.6e closure) (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::unnested_or_patterns)]

use benten_id::keypair::Keypair;
// RED-PHASE failure points.
use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedScope;
use benten_sync::ucan_blobs_protocol::{UcanBlobsHandler, UcanBlobsHandlerError, UcanBlobsRequest};

// ---------------------------------------------------------------------------
// PIN 1 — F-1: UCAN audience ≠ requester EndpointId → UcanAudienceMismatch.
// ---------------------------------------------------------------------------
// Alice grants Bob a UCAN (audience = Bob's pubkey). At handler runtime,
// the requester's verified EndpointId (from the iroh QUIC layer) is
// compared against the UCAN's audience field. A mismatch — e.g. the
// connection is verified as Carol's EndpointId but the UCAN audience
// is Bob's — MUST return typed `UcanAudienceMismatch` and close the
// connection.
//
// Would-FAIL-IF-NO-OP'd: if the handler skips audience comparison,
// any peer with a stolen-but-still-valid UCAN can request bytes
// (defeats the audience-binding property of the AuthorizationGrant).
#[test]

fn tf3e_unauthorized_requester_audience_mismatch_typed() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let kp_carol = Keypair::generate(); // attacker / wrong audience

    let handler = UcanBlobsHandler::new(&kp_alice);

    // Alice grants Bob (audience = Bob's pubkey) a scope.
    let spec = RestrictedScope::with_hashes(vec![[5u8; 32].into()]);
    let grant_to_bob =
        AuthorizationGrant::issue_for_test(&kp_alice, kp_bob.public_key(), spec, u64::MAX);

    // Carol presents Bob's UCAN over Carol's connection. The handler
    // sees the connection's verified EndpointId = Carol's pubkey but
    // the UCAN's audience = Bob's pubkey → mismatch.
    let req = UcanBlobsRequest {
        grant: grant_to_bob,
        ciphertext_hash: [5u8; 32].into(),
    };
    let connection_verified_endpoint_id = kp_carol.public_key();

    let result = handler.validate_request_for_connection(&req, &connection_verified_endpoint_id);
    assert!(
        matches!(
            result,
            Err(UcanBlobsHandlerError::UcanAudienceMismatch { .. })
        ),
        "UCAN audience (Bob) ≠ requester EndpointId (Carol) MUST yield \
         typed UcanAudienceMismatch. NEVER silently serve bytes. \
         got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — A-2: Bob presents Carol's UCAN; handler refuses.
// ---------------------------------------------------------------------------
// Bidirectional check: not just "wrong requester for grant" but also
// "wrong grant for requester". Bob (verified EndpointId = Bob's pubkey)
// presents Carol's UCAN (audience = Carol). The handler MUST refuse.
#[test]

fn tf3e_audience_substitution_attack_bob_presents_carols_ucan() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let kp_carol = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);

    // Alice grants Carol a scope.
    let spec = RestrictedScope::with_hashes(vec![[7u8; 32].into()]);
    let grant_to_carol =
        AuthorizationGrant::issue_for_test(&kp_alice, kp_carol.public_key(), spec, u64::MAX);

    // Bob connects (EndpointId = Bob's pubkey) and presents Carol's UCAN.
    let req = UcanBlobsRequest {
        grant: grant_to_carol,
        ciphertext_hash: [7u8; 32].into(),
    };
    let result = handler.validate_request_for_connection(&req, &kp_bob.public_key());
    assert!(
        matches!(
            result,
            Err(UcanBlobsHandlerError::UcanAudienceMismatch { .. })
        ),
        "Bob presenting Carol's UCAN (audience-substitution attack) MUST \
         yield typed UcanAudienceMismatch. NEVER silently serve bytes. \
         got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — A-3: Unresolvable peer-DID at handler entry → UnresolvedDeny.
// ---------------------------------------------------------------------------
// Couples §4.25 sync-hydrate denial + the §5 "unresolvable peer-DID at
// recheck" adversarial pattern. The handler MUST never proceed with
// the sentinel `<unresolved-peer>` peer-DID; it returns `UnresolvedDeny`,
// NEVER `Admitted`.
#[test]

fn tf3e_unresolvable_peer_did_yields_typed_unresolved_deny() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);

    // Construct a request whose grant references a peer-DID that
    // cannot be resolved (the sentinel pattern).
    let spec = RestrictedScope::with_hashes(vec![[9u8; 32].into()]);
    let grant_with_unresolved_peer = AuthorizationGrant::issue_with_unresolved_peer_for_test(
        &kp_alice,
        kp_bob.public_key(),
        spec,
        u64::MAX,
    );

    let req = UcanBlobsRequest {
        grant: grant_with_unresolved_peer,
        ciphertext_hash: [9u8; 32].into(),
    };
    let result = handler.validate_request_for_connection(&req, &kp_bob.public_key());
    assert!(
        matches!(result, Err(UcanBlobsHandlerError::UnresolvedDeny { .. })),
        "Unresolvable peer-DID at handler MUST return typed \
         UnresolvedDeny, NEVER Admitted (security-r1-2 enum-invariant). \
         got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
