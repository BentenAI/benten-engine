//! F-LC-GAP1 — the encrypt-to-recipient surface keys off REAL hybrid recipient
//! key material, NOT a public-derived placeholder (R9 GAP-1 closure pin).
//!
//! # The defect this pins closed (GAP-1)
//!
//! The pre-fix Layer-C seal/open path modeled recipient keys as `[u8; 32]`
//! placeholder fingerprints and derived the recipient "secret" from the PUBLIC
//! fingerprint (`sk[i] = pk[i].wrapping_sub(0x80)` per byte), then expanded
//! that public fingerprint into a "keypair" via
//! `generate_recipient_keypair_deterministic`. Net: the secret carried ZERO
//! entropy independent of the public key — ANY party holding the recipient's
//! PUBLIC key could reconstruct the "secret" and decrypt. That is a total break
//! of encrypt-to-recipient confidentiality on the frozen v1-beta surface.
//!
//! The fix retypes seal to take the recipient's REAL public key
//! (`&RecipientPublic`) and open to take the recipient's REAL secret
//! (`&RecipientSecret`), wrapping/unwrapping directly via the vetted X-Wing KEM.
//! The secret now carries genuine entropy: it is the ML-KEM-768 decapsulation
//! key + X25519 static secret, unrecoverable from the public key.
//!
//! # would-FAIL-on-revert
//!
//! On the PRE-FIX code, the `negative_forged_from_public` arms below would have
//! OPENED: an attacker holding only `kp_A.public()` forged the "secret" as
//! `pk + 0x80` and the placeholder open expanded that public seed back to the
//! SAME keypair the seal wrapped to — so decryption SUCCEEDED. This test asserts
//! the forged secret FAILS closed, so reverting the fix (restoring the
//! `wrapping_sub(0x80)` derivation) flips every negative arm from `Err` to `Ok`
//! — the exact confidentiality break the fix removes. The `negative_wrong_key`
//! arms independently pin that a DIFFERENT real recipient's secret cannot open.
//!
//! Covers all THREE encrypt-to-recipient codepoints:
//!   - single-recipient Sealed-Sender `0x6510` (`seal_sealed_sender` / `open_single`)
//!   - Layer-C group multi-stanza `0x6520` (`seal_group_multi` / `open_group_stanza`)
//!   - MembershipSet K_Set group `0x6610`
//!     (`group_posture::seal_membership_set_group` / `open_membership_set_group`)

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::cipher_suite::{
    CipherSuite, CipherSuiteCodepoint, RecipientKeypair, RecipientPublic, RecipientSecret,
};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_drop::layer_c::group_posture::{
    GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
    seal_membership_set_group,
};
use benten_drop::layer_c::{
    EncryptedEnvelope, LayerCError, group_roster_for_test, open_group_stanza, open_single,
    seal_group_multi, seal_sealed_sender,
};
use benten_id::did::Did;

/// A REAL hybrid recipient keypair with genuine, independent secret entropy —
/// the production keying path (`generate_recipient_keypair` seeds both halves
/// from the crate OS RNG). Two calls yield DISTINCT keypairs.
fn real_kp() -> RecipientKeypair {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a wire-locked")
        .generate_recipient_keypair()
}

/// Re-parse the public half through the frozen `to_bytes`/`from_bytes` surface
/// so the test drives the same serialization the engine-wiring slice will use.
fn pub_of(kp: &RecipientKeypair) -> RecipientPublic {
    RecipientPublic::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &kp.public().to_bytes(),
    )
    .expect("re-parse of recipient public must succeed")
}

/// Re-parse the secret half through the frozen `to_bytes`/`from_bytes` surface.
fn sec_of(kp: &RecipientKeypair) -> RecipientSecret {
    RecipientSecret::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &kp.secret().to_bytes(),
    )
    .expect("re-parse of recipient secret must succeed")
}

/// The GAP-1 forgery attempt: a party holding ONLY the recipient's PUBLIC key
/// tries to manufacture a matching secret. There is no valid `RecipientSecret`
/// derivable from a `RecipientPublic` (the public byte layout is
/// `x25519_pub(32) ‖ mlkem768_ek(1184)`; the secret layout is
/// `x25519_sec(32) ‖ mlkem768_dk(2400)` — different length AND different key
/// material). Attempting to parse the PUBLIC bytes as a `RecipientSecret`
/// fail-closed rejects on the length invariant; that rejection IS the closure.
/// (On the pre-fix code the attacker did NOT need a valid secret at all — the
/// open path itself reconstructed the key from the public fingerprint.)
fn forged_secret_from_public_bytes(pub_bytes: &[u8]) -> Result<RecipientSecret, ()> {
    RecipientSecret::from_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, pub_bytes)
        .map_err(|_| ())
}

/// A real sender (LAMPS-hybrid keypair + matching hybrid `did:key` bytes) so the
/// B2 origin-auth signs + verifies for real.
fn hybrid_sender() -> (SigKeypair, Vec<u8>) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did_str = Did::from_hybrid_public_key(&kp.public()).to_string();
    (kp, did_str.into_bytes())
}

// ===========================================================================
// 0x6510 — single-recipient Sealed-Sender.
// ===========================================================================

/// GAP-1 (`0x6510`) — seal to `kp_A.public()`; ONLY `kp_A.secret()` opens.
///
/// Positive: the matching real secret round-trips. Negative-wrong-key: a
/// DIFFERENT recipient's real secret fails closed. Negative-forged-from-public:
/// a party holding only `kp_A.public()` cannot manufacture an opening secret.
#[test]
fn f_lc_gap1_single_0x6510_real_recipient_keying() {
    let kp_a = real_kp();
    let kp_b = real_kp();
    let audience = b"did:key:zRecipientAudienceGAP1".to_vec();
    let (sender_kp, sender) = hybrid_sender();
    let plaintext = b"gap-1 single-recipient confidential payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal_sealed_sender(
        &pub_of(&kp_a),
        &audience,
        &sender,
        &sender_kp,
        &body_cid,
        0,
        &plaintext,
    );

    // Positive — the intended recipient's REAL secret round-trips.
    let (recovered, recovered_sender) = open_single(&sec_of(&kp_a), &audience, 0, &env)
        .expect("GAP-1: kp_A's REAL secret MUST open the envelope sealed to kp_A's public");
    assert_eq!(recovered, plaintext, "GAP-1 (0x6510): round-trip plaintext");
    assert_eq!(recovered_sender, sender, "GAP-1 (0x6510): sender recovered");

    // Negative-wrong-key — a DIFFERENT real recipient cannot open.
    assert!(
        matches!(
            open_single(&sec_of(&kp_b), &audience, 0, &env),
            Err(LayerCError::AeadAuthenticationFailed)
        ),
        "GAP-1 (0x6510): a NON-matching real recipient secret MUST fail closed \
         (the X-Wing shared secret differs → AEAD unwrap fails)."
    );

    // Negative-forged-from-public (the GAP-1 attack) — a party holding ONLY
    // kp_A's public bytes cannot mint an opening secret. On the PRE-FIX code the
    // open path reconstructed the key from the public fingerprint, so this
    // OPENED; the fix removes that derivation, so no public-only forgery works.
    let pub_bytes = kp_a.public().to_bytes();
    match forged_secret_from_public_bytes(&pub_bytes) {
        Err(()) => { /* fail-closed at parse — the public bytes are not a valid secret */ }
        Ok(forged) => {
            assert!(
                matches!(
                    open_single(&forged, &audience, 0, &env),
                    Err(LayerCError::AeadAuthenticationFailed)
                ),
                "GAP-1 (0x6510): a secret forged from kp_A's PUBLIC bytes MUST \
                 NOT open the envelope. would-FAIL-on-revert: the pre-fix \
                 `sk = pk + 0x80` path OPENED this."
            );
        }
    }
}

// ===========================================================================
// 0x6520 — Layer-C group multi-stanza.
// ===========================================================================

/// GAP-1 (`0x6520`) — each recipient opens their stanza ONLY with their own
/// real secret; a wrong/forged-from-public secret fails closed.
#[test]
fn f_lc_gap1_group_0x6520_real_recipient_keying() {
    let kp_a = real_kp();
    let kp_b = real_kp();
    let kp_c = real_kp(); // a non-member
    let pks = [pub_of(&kp_a), pub_of(&kp_b)];
    let roster = group_roster_for_test(&pks);
    let (sender_kp, sender) = hybrid_sender();
    let plaintext = b"gap-1 group confidential payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal_group_multi(&pks, &sender, &sender_kp, &body_cid, 0, &plaintext);

    // Positive — each member opens their OWN stanza with their real secret.
    let (pt_a, _) = open_group_stanza(&sec_of(&kp_a), 0, &roster, 0, &env)
        .expect("GAP-1: member A's REAL secret MUST open stanza 0");
    let (pt_b, _) = open_group_stanza(&sec_of(&kp_b), 1, &roster, 0, &env)
        .expect("GAP-1: member B's REAL secret MUST open stanza 1");
    assert_eq!(pt_a, plaintext, "GAP-1 (0x6520): member A round-trip");
    assert_eq!(pt_b, plaintext, "GAP-1 (0x6520): member B round-trip");

    // Negative-wrong-key — a non-member's real secret cannot open A's stanza.
    assert!(
        matches!(
            open_group_stanza(&sec_of(&kp_c), 0, &roster, 0, &env),
            Err(LayerCError::AeadAuthenticationFailed)
        ),
        "GAP-1 (0x6520): a non-member real secret MUST fail closed on stanza 0."
    );

    // Negative-forged-from-public — kp_A's public bytes cannot mint a secret
    // that opens A's stanza (the GAP-1 attack; pre-fix it OPENED).
    match forged_secret_from_public_bytes(&kp_a.public().to_bytes()) {
        Err(()) => { /* fail-closed at parse */ }
        Ok(forged) => {
            assert!(
                matches!(
                    open_group_stanza(&forged, 0, &roster, 0, &env),
                    Err(LayerCError::AeadAuthenticationFailed)
                ),
                "GAP-1 (0x6520): a secret forged from a member's PUBLIC bytes MUST \
                 NOT open their stanza. would-FAIL-on-revert (`sk = pk + 0x80`)."
            );
        }
    }
}

// ===========================================================================
// 0x6610 — MembershipSet K_Set group.
// ===========================================================================

fn verify_ctx(pks: &[RecipientPublic]) -> GroupVerifyContext {
    let member_dids = group_roster_for_test(pks)
        .iter()
        .map(|d| String::from_utf8_lossy(d).into_owned())
        .collect();
    GroupVerifyContext {
        member_dids,
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    }
}

/// GAP-1 (`0x6610`) — each member opens their MembershipSet stanza ONLY with
/// their own real secret; a wrong/forged-from-public secret fails closed.
#[test]
fn f_lc_gap1_membership_set_group_0x6610_real_recipient_keying() {
    let kp_a = real_kp();
    let kp_b = real_kp();
    let kp_c = real_kp(); // a non-member
    let pks = [pub_of(&kp_a), pub_of(&kp_b)];
    let ctx = verify_ctx(&pks);
    let (sender_kp, sender) = hybrid_sender();
    let k_set = [0x5au8; 32];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:gap1-membership-group".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let plaintext = b"gap-1 membership-set confidential payload".to_vec();

    let env = seal_membership_set_group(&pks, &sender, &sender_kp, &k_set, &params, &plaintext);

    // Positive — each member opens their OWN stanza with their real secret.
    let (pt_a, _) = open_membership_set_group(&sec_of(&kp_a), 0, &ctx, &env)
        .expect("GAP-1: member A's REAL secret MUST open 0x6610 stanza 0");
    let (pt_b, _) = open_membership_set_group(&sec_of(&kp_b), 1, &ctx, &env)
        .expect("GAP-1: member B's REAL secret MUST open 0x6610 stanza 1");
    assert_eq!(pt_a, plaintext, "GAP-1 (0x6610): member A round-trip");
    assert_eq!(pt_b, plaintext, "GAP-1 (0x6610): member B round-trip");

    // Negative-wrong-key — a non-member's real secret cannot open A's stanza.
    assert!(
        matches!(
            open_membership_set_group(&sec_of(&kp_c), 0, &ctx, &env),
            Err(GroupError::AeadAuthenticationFailed)
        ),
        "GAP-1 (0x6610): a non-member real secret MUST fail closed on stanza 0."
    );

    // Negative-forged-from-public — kp_A's public bytes cannot mint a secret
    // that opens A's stanza (the GAP-1 attack; pre-fix it OPENED).
    match forged_secret_from_public_bytes(&kp_a.public().to_bytes()) {
        Err(()) => { /* fail-closed at parse */ }
        Ok(forged) => {
            assert!(
                matches!(
                    open_membership_set_group(&forged, 0, &ctx, &env),
                    Err(GroupError::AeadAuthenticationFailed)
                ),
                "GAP-1 (0x6610): a secret forged from a member's PUBLIC bytes MUST \
                 NOT open their stanza. would-FAIL-on-revert (`sk = pk + 0x80`)."
            );
        }
    }
}
