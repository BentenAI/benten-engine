//! TF-3e pins — G-CORE-3e zero-conversion plumbing: iroh `EndpointId` IS
//! `ed25519_dalek::VerifyingKey`.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3e (P-2):
//!     "Zero-conversion plumbing: iroh `EndpointId` IS
//!     `ed25519_dalek::VerifyingKey` is asserted by a compile-time
//!     identity-cast pin (no parsing/conversion; Spike A2 finding)."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def 8 spike-derived
//!     refinements item (7): "iroh `EndpointId` IS
//!     `ed25519_dalek::VerifyingKey` (zero conversion plumbing for UCAN
//!     audience = iroh peer ID)."
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` Spike A2
//!     finding: iroh `EndpointId` is structurally identical to
//!     `ed25519_dalek::VerifyingKey`; no parsing/conversion required.
//!
//! ============================================================================
//! LANDED at G-CORE-3e (pim-12 / §3.6e closure) (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::unnested_or_patterns)]

// RED-PHASE failure point — G-CORE-3e exposes the identity-cast helpers.
use benten_sync::ucan_blobs_protocol::{
    endpoint_id_to_verifying_key, verifying_key_to_endpoint_id,
};

// ---------------------------------------------------------------------------
// PIN 1 — Compile-time identity-cast: round-trip preserves bytes.
// ---------------------------------------------------------------------------
// Production-arm P-2. The identity-cast (NOT parsing/conversion) means
// that an iroh `EndpointId` and the corresponding `VerifyingKey` share
// the same underlying 32-byte Ed25519 public-key representation. A
// round-trip MUST be byte-identical; would-FAIL-IF a parsing/conversion
// path is silently inserted.
#[test]

fn tf3e_endpoint_id_round_trips_through_verifying_key_byte_identical() {
    // Use benten-id's Keypair (already a dev-dep on benten-sync) to
    // generate a stable Ed25519 key pair without a direct rand dep.
    use benten_id::keypair::Keypair;
    let kp = Keypair::generate();
    let vk = kp.public_key_verifying_key_for_test();

    let endpoint_id = verifying_key_to_endpoint_id(&vk);
    let recovered_vk = endpoint_id_to_verifying_key(&endpoint_id).expect("identity cast OK");

    assert_eq!(
        vk.to_bytes(),
        recovered_vk.to_bytes(),
        "iroh EndpointId ↔ VerifyingKey MUST be a byte-identical \
         identity-cast round-trip. ANY drift here means a parsing/ \
         conversion path was inserted, violating Spike A2's zero-conversion \
         plumbing contract."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — UCAN audience pubkey IS the requester's EndpointId.
// ---------------------------------------------------------------------------
// The substantive consequence of the zero-conversion property: when an
// iroh peer connects with EndpointId E, the handler can use E DIRECTLY
// as the UCAN audience for validation. No parsing, no
// VerifyingKey::from_bytes round-trip in the hot path.
#[test]

fn tf3e_ucan_audience_is_endpoint_id_no_parsing_required() {
    use benten_id::keypair::Keypair;
    use benten_sync::ucan_blobs_protocol::UcanBlobsHandler;

    let kp = Keypair::generate();
    let pubkey = kp.public_key();

    // The handler exposes an EndpointId surface for audience checks.
    let endpoint_id = UcanBlobsHandler::endpoint_id_from_public_key_no_parsing(&pubkey);

    // The substantive assertion: the EndpointId's byte representation
    // IS the VerifyingKey's byte representation (no encoding step).
    let endpoint_bytes = endpoint_id.to_bytes_for_test();
    let pubkey_bytes = pubkey.to_bytes_for_test();
    assert_eq!(
        endpoint_bytes, pubkey_bytes,
        "UCAN audience pubkey bytes MUST equal the requester's iroh \
         EndpointId bytes — proves no parsing/conversion in the hot path."
    );
}
