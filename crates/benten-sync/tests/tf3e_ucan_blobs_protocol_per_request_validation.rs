//! TF-3e pins — G-CORE-3e UCAN-gated iroh-blobs custom-ALPN handler
//! per-request validation.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3e (P-1):
//!     "Custom-ALPN handler: requester opens connection → presents
//!     `AuthorizationGrant` → handler validates UCAN against
//!     `RestrictedScope` (via G-CORE-3b chain validator) → handler
//!     dispatches to `iroh_blobs::provider::handle_connection` for the
//!     ciphertext_hash. Spike A2 + Spike H+1.2 validated the reuse
//!     pattern over real iroh QUIC; this is the substantive proof."
//!   - §2 G-CORE-3e (P-3): "End-to-end share: Alice writes a Recipe
//!     subgraph → grants Bob a SubgraphSpec UCAN → Bob requests → Bob's
//!     engine decrypts via two-CID mapping + key derived from carried
//!     canonical-path (D-4M-R4 — BFS-order carried in grant; recipient
//!     does NOT re-walk)."
//!   - §2 G-CORE-3e (F-2): "Request for ciphertext NOT in the granted
//!     SubgraphSpec scope: typed `NotInScope` → no bytes served."
//!   - `00-implementation-plan.md` §3 G-CORE-3e wave def: "The custom-ALPN
//!     handler validates the requester's UCAN per request, then hands
//!     the same `Connection` to iroh-blobs's `provider::handle_connection`
//!     (which serves bytes by hash — the ciphertext_hash from the two-CID
//!     mapping)."
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 (Option
//!     B two-CID + per-chunk-AEAD; Flavor B per-request UCAN check;
//!     Spike A2 + H+1.2 validated).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3e (pim-12 / §3.6e).
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
// RED-PHASE failure points — G-CORE-3e surface.
use benten_sync::two_cid_store::TwoCidStore;
use benten_sync::ucan_blobs_protocol::{
    UcanBlobsHandler, UcanBlobsHandlerError, UcanBlobsRequest, UcanBlobsResponse,
};
// RED-PHASE failure point — G-CORE-3b surface that 3e consumes.
use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedScope;

// ---------------------------------------------------------------------------
// PIN 1 — Per-request UCAN validation on the custom-ALPN handler.
// ---------------------------------------------------------------------------
// Production-arm P-1. The handler MUST validate the requester's UCAN
// per request, then hand to iroh-blobs. The validation step is
// observable: a request with a malformed UCAN MUST be rejected BEFORE
// any bytes flow.
//
// Would-FAIL-IF-NO-OP'd: a stub that hands every connection to
// iroh-blobs without validating the UCAN exposes ciphertext to
// unauthenticated requesters.
#[test]

fn tf3e_per_request_ucan_validation_rejects_malformed_grant() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);

    // Construct a malformed grant (e.g. a UCAN signed by a non-matching
    // key, or a binding_sig that doesn't verify).
    let bad_grant = AuthorizationGrant::malformed_for_test(&kp_bob);
    let req = UcanBlobsRequest {
        grant: bad_grant,
        ciphertext_hash: [0u8; 32].into(),
    };

    let result = handler.validate_request(&req);
    assert!(
        matches!(
            result,
            Err(UcanBlobsHandlerError::GrantValidation { .. })
                | Err(UcanBlobsHandlerError::BindingSigInvalid { .. })
        ),
        "Malformed AuthorizationGrant MUST be rejected at per-request \
         validation, BEFORE any bytes are served via iroh-blobs. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — Request out-of-scope ciphertext returns typed NotInScope; no
// bytes served.
// ---------------------------------------------------------------------------
// Production-arm F-2. The handler validates the UCAN-bound RestrictedScope
// against the requested ciphertext_hash. If the hash is NOT in the
// granted scope, the handler returns `NotInScope` and serves zero bytes.
//
// Would-FAIL-IF-NO-OP'd: a stub that hands every authenticated
// connection to iroh-blobs (without checking scope) would serve any
// content to any authenticated peer.
#[test]

fn tf3e_request_out_of_scope_ciphertext_typed_not_in_scope() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);

    // Grant Bob a scope covering {hash_a, hash_b}.
    let spec = RestrictedScope::with_hashes(vec![[1u8; 32].into(), [2u8; 32].into()]);
    let grant = AuthorizationGrant::issue_for_test(
        &kp_alice,
        kp_bob.public_key(),
        spec,
        /* expiry */ u64::MAX,
    );

    // Bob requests hash_c (not in scope).
    let req = UcanBlobsRequest {
        grant,
        ciphertext_hash: [3u8; 32].into(),
    };

    let result = handler.validate_request(&req);
    assert!(
        matches!(result, Err(UcanBlobsHandlerError::NotInScope { .. })),
        "Request for ciphertext NOT in granted RestrictedScope scope MUST \
         return typed NotInScope; the handler MUST NOT dispatch to \
         iroh-blobs for the out-of-scope hash. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — Dispatch to iroh-blobs occurs ONLY after positive UCAN
// validation.
// ---------------------------------------------------------------------------
// The handler's substantive behavior: it dispatches to
// `iroh_blobs::provider::handle_connection` only on positive UCAN
// validation. A test seam exposes the dispatch counter; the counter
// MUST increment only after positive validation paths.
#[test]

fn tf3e_dispatch_to_iroh_blobs_only_after_positive_validation() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let handler = UcanBlobsHandler::new(&kp_alice);
    let initial_dispatch_count = handler.dispatch_count_for_test();

    // Failure path: malformed grant.
    let bad_grant = AuthorizationGrant::malformed_for_test(&kp_bob);
    let _ = handler.validate_request(&UcanBlobsRequest {
        grant: bad_grant,
        ciphertext_hash: [0u8; 32].into(),
    });
    assert_eq!(
        handler.dispatch_count_for_test(),
        initial_dispatch_count,
        "Failed UCAN validation MUST NOT increment iroh-blobs dispatch \
         count — bytes never flow on a failed-validation path."
    );

    // Positive path: well-formed grant in-scope.
    let hash = [42u8; 32].into();
    let spec = RestrictedScope::with_hashes(vec![hash]);
    let grant = AuthorizationGrant::issue_for_test(
        &kp_alice,
        kp_bob.public_key(),
        spec,
        /* expiry */ u64::MAX,
    );
    let ok = handler.serve_request_for_test(UcanBlobsRequest {
        grant,
        ciphertext_hash: hash,
    });
    assert!(ok.is_ok(), "positive validation path must succeed");
    assert_eq!(
        handler.dispatch_count_for_test(),
        initial_dispatch_count + 1,
        "Positive UCAN validation MUST increment iroh-blobs dispatch \
         count by exactly 1 (the dispatch happens after validation)."
    );
}
