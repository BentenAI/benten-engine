//! GAP-KDB Shape-B — DROP-8: Layer-C sender-origin-auth `resolve_signing`
//! migration for a `did:benten` SENDER (Scope-Min). W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §2 (sender-origin-auth switches
//! `resolve()`/`resolve_hybrid()` → `resolve_signing()`, `layer_c.rs:295-296`)
//! + §7 freeze-item 3 (`resolve_signing` used on the Layer-C did:benten SEND
//! path) + `R2-LANDSCAPE` DROP-8. This is the recipient/sender half of Shape-B
//! that ships at v1-beta WITHOUT the Fork-A UCAN-issuer migration.
//!
//! # What this pins
//! A `did:benten` sender can send a Drop: the recipient's B2 origin-auth
//! resolves the recovered `did:benten` sender's SIGNING key via the method-aware
//! `resolve_signing` (which strips the trailing keyset-CID component before the
//! composite signing decode) and cryptographically verifies the hybrid
//! signature. A forged-origin `did:benten` sender (embedded signing key ≠
//! signer) fails closed.
//!
//! # would_fail_on_revert
//! The OLD sender-resolve (`did:key`-only / no trailing-CID strip) cannot
//! resolve a `did:benten` sender's signing key → the positive round-trip
//! `expect` fails. (A resolve that extracts only the Ed25519 half would be a
//! silent PQ-strip on the origin-auth path — see the Fork-A AUTH-3 flagship.)
//!
//! # R5 un-ignore
//! Switch the Layer-C sender-origin-auth resolve to `resolve_signing`
//! (method/multicodec-aware, composite, trailing-CID strip); drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::RecipientBinding;
use benten_drop::layer_c::{LayerCError, open_single};

/// DROP-8 (`0x6510`) — a `did:benten` sender's Drop round-trips: origin-auth
/// resolves the sender's SIGNING key via `resolve_signing` + hybrid-verifies.
#[test]
fn drop8_did_benten_sender_origin_auth_roundtrips() {
    let r = seal::real_recipient();
    let binding = RecipientBinding::resolve(&r.did, &r.doc).expect("recipient binding resolves");
    let (sender_kp, sender_did) = seal::did_benten_sender();
    // Sanity: the sender IS a did:benten (not a bare did:key).
    assert!(
        sender_did.starts_with(b"did:benten:z"),
        "DROP-8 precondition: the sender DID is a did:benten"
    );
    let audience = r.did.as_str().as_bytes().to_vec();
    let plaintext = b"drop-8 did:benten sender payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_to_binding(&binding, &sender_did, &sender_kp, &body_cid, 0, &plaintext);

    let (recovered, recovered_sender) = open_single(&seal::sec_of(&r.kp), &audience, 0, &env)
        .expect(
            "DROP-8: origin-auth MUST resolve the did:benten sender's SIGNING key via \
             resolve_signing (strip the trailing keyset-CID) + hybrid-verify. \
             would-FAIL-on-revert: the old did:key-only resolve can't decode a did:benten \
             sender's composite signing key.",
        );
    assert_eq!(recovered, plaintext, "DROP-8: round-trip plaintext");
    assert_eq!(
        recovered_sender, sender_did,
        "DROP-8: the did:benten sender DID is recovered from the sealed inner payload"
    );
}

/// DROP-8 (`0x6510`) — a forged-origin `did:benten` sender (embedded signing key
/// ≠ the actual signer) fails closed. This pins that origin-auth verifies
/// against the DID-EMBEDDED signing key, not a wire-provided one.
#[test]
fn drop8_forged_origin_did_benten_sender_fails_closed() {
    let r = seal::real_recipient();
    let binding = RecipientBinding::resolve(&r.did, &r.doc).expect("recipient binding resolves");
    // The DID embeds VICTIM's signing key; the caller signs with ATTACKER's.
    let (attacker_kp, victim_did) = seal::did_benten_sender_key_mismatch();
    let audience = r.did.as_str().as_bytes().to_vec();
    let plaintext = b"drop-8 forged-origin payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_to_binding(
        &binding,
        &victim_did,
        &attacker_kp,
        &body_cid,
        0,
        &plaintext,
    );

    assert!(
        matches!(
            open_single(&seal::sec_of(&r.kp), &audience, 0, &env),
            Err(LayerCError::SenderOriginAuthFailed)
        ),
        "DROP-8: a did:benten sender whose EMBEDDED signing key does not match the signer MUST \
         fail closed — origin-auth resolves the DID-embedded key via resolve_signing and the \
         attacker's signature does not verify against it."
    );
}
