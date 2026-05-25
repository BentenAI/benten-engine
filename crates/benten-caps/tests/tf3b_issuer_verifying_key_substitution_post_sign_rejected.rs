//! TF-3b — R6 R2 batch-A Item 3 (L1-MAJ-1 closure): issuer_verifying_key
//! self-bind substantive-arm pin.
//!
//! Closes L1-MAJ-1 — the "7-field signed-surface" invariant: pre-batch-A,
//! the binding-message bound 6 segments (domain-tag || ucan-cbor ||
//! km-cbor || audience-cid || scope-len-u32-le || scope-cbor ||
//! audience-pk-len-u32-le || audience-pk-bytes). The
//! `issuer_verifying_key` field (the 32-byte Ed25519 verifying-key the
//! `verify_binding` path uses to verify the binding-sig) was OUTSIDE
//! the signed payload — the classic self-bind invariant gap. A network
//! adversary with the ability to mutate grant bytes in transit could:
//!
//!   1. Receive Alice's grant G (`G.issuer_verifying_key = alice_vk`;
//!      `G.binding_sig = alice_sign(binding_message_v3(alice_vk))`).
//!   2. Mint a fresh keypair (`eve_sk`, `eve_vk`).
//!   3. Build a forged grant G' with `G'.issuer_verifying_key = eve_vk`
//!      and `G'.binding_sig = eve_sign(binding_message_v3(...))` using
//!      the SAME 6-segment message construction.
//!   4. Present G' to a verifier: `verify_binding` parses `eve_vk`,
//!      re-constructs the 6-segment message (which omitted vk), and
//!      verifies Eve's signature with Eve's vk — `BindingMismatch`
//!      ARM does NOT fire because the cryptographic verify succeeds.
//!   5. The verifier admits G' as a "valid grant," now believing
//!      Eve is the issuer of capabilities that should have required
//!      Alice's signing key. Cross-protocol replay + key-substitution
//!      attacks (e.g. mass-issuing forwarded grants under attacker-
//!      controlled vk) succeed silently.
//!
//! The fix (Item 3, BINDING_SIG_DOMAIN v3 → v4 bump) folds
//! `issuer_verifying_key` into the binding-message as the 7th segment
//! (fixed 32 bytes, no length prefix — typed at construction). The
//! binding-message now ANDs the self-bind invariant: `verify_binding`
//! reconstructs the 7-segment message with `self.issuer_verifying_key`
//! and any post-sign vk substitution flips the message bytes, so
//! Ed25519 verify under the swapped vk fails — typed `BindingMismatch`.
//!
//! ## Test shape (§3.6f SUBSTANTIVE-arm)
//!
//! 1. Issue a real grant G with Alice's keypair via
//!    `AuthorizationGrant::issue_for_test(...)`.
//! 2. Positive control: `G.verify_binding(G.audience_binding)` MUST
//!    succeed (sanity that the test fixture works).
//! 3. Adversary tamper: build G' via
//!    `G.with_swapped_issuer_verifying_key_for_test(eve_vk_bytes)`.
//! 4. Adversarial expectation: `G'.verify_binding(G'.audience_binding)`
//!    MUST return `BindingMismatch` (not silently admit Eve's vk).
//!
//! Companion: the wave-3b envelope shape (no scope / no
//! audience_pubkey) has the same self-bind invariant; pin both shapes
//! to lock the 7-segment invariant across the issue-for-test and
//! issue_envelopes_for_test constructor flavours.
//!
//! ## Would-FAIL-on-revert (§3.6f)
//!
//! Revert `binding_message` to drop the `issuer_verifying_key`
//! parameter + the corresponding `msg.extend_from_slice(...)` line →
//! the test exits the assertion with `outcome = Err(...)` ONLY if the
//! Ed25519 sig was over a message that committed to vk; under the
//! revert the message no longer commits to vk so the swapped-vk
//! re-verify with `eve_sig + eve_vk + 6-segment-msg` succeeds + the
//! assert! fires.

use benten_caps::authorization_grant::{AuthorizationGrant, AuthorizationGrantError};
use benten_caps::restricted_spec::RestrictedScope;
use benten_id::keypair::Keypair;

#[test]
fn issuer_verifying_key_substitution_post_sign_rejected_by_binding_sig() {
    let issuer_kp = Keypair::generate();
    let audience_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();

    let audience_pub = audience_kp.public_key();
    let attacker_pub = attacker_kp.public_key();

    let scope = RestrictedScope::new();
    let exp_secs = 9_999_999_999;
    let grant = AuthorizationGrant::issue_for_test(&issuer_kp, &audience_pub, scope, exp_secs);

    // Positive control: the grant verifies at issue time.
    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued grant MUST verify at audience_binding");

    // Sanity: the grant carries the issuer's verifying-key bytes
    // (32 bytes Ed25519).
    assert_eq!(
        grant.issuer_verifying_key.len(),
        32,
        "issuer_verifying_key must be 32 bytes Ed25519"
    );
    assert_ne!(
        grant.issuer_verifying_key.as_slice(),
        attacker_pub.to_bytes().as_slice(),
        "test fixture distinct: issuer vk != attacker vk"
    );

    // Adversary tamper: mutate issuer_verifying_key to attacker's
    // pubkey while leaving binding_sig + audience_binding unchanged.
    // Pre-Item 3 this slipped through (verify_binding parsed
    // attacker_vk and ran Ed25519 against the 6-segment message which
    // didn't commit to vk). Post-Item 3 the 7-segment message commits
    // to vk; the re-construct uses attacker_vk → message bytes differ
    // from the original signed message → Ed25519 verify fails.
    let tampered =
        grant.with_swapped_issuer_verifying_key_for_test(attacker_pub.to_bytes().to_vec());

    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign issuer_verifying_key substitution MUST be rejected by \
         binding-sig re-verification (R6 R2 batch-A Item 3 / L1-MAJ-1 \
         self-bind closure). Got: {outcome:?}"
    );
}

#[test]
fn issuer_verifying_key_self_bind_holds_for_wave_3b_envelope_shape() {
    use benten_caps::authorization_grant::{GrantKeyMaterial, UcanEnvelope};
    use benten_core::Cid;

    // wave-3b envelope shape (no scope, no audience_pubkey) — pins
    // that the 7-segment self-bind invariant holds for the
    // legacy/envelope constructor too.
    let audience = Cid::from_blake3_digest([0u8; 32]);
    let ucan = UcanEnvelope::synthetic_for_test(audience);
    let key_material = GrantKeyMaterial::synthetic_for_test();

    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan, key_material, audience)
        .expect("envelope-shape grant issue MUST succeed for synthetic fixture");

    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued envelope grant MUST verify");

    let attacker_kp = Keypair::generate();
    let tampered = grant
        .with_swapped_issuer_verifying_key_for_test(attacker_kp.public_key().to_bytes().to_vec());

    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "envelope-shape grant: post-sign issuer_verifying_key \
         substitution MUST be rejected. Got: {outcome:?}"
    );
}
