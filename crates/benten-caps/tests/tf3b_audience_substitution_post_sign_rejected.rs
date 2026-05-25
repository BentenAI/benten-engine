//! TF-3b — R6 R2 fix-pass (Bundle R6-R2-FP-A): audience-substitution
//! post-sign rejection pin.
//!
//! Closes R6 R2 BLOCKER L2-R2-BLOCKER-1 (cross-confirmed by L1-MAJ-1 +
//! L3-r2-1 + L13-MAJ-2 + L17-r2-MAJOR-1 + L4-MAJ): pre-R6-R2,
//! `AuthorizationGrant.audience_pubkey` lived OUTSIDE the binding-sig
//! message construction (`binding_message(ucan, key_material, audience,
//! scope)` — 5 segments, audience_pubkey omitted). The grant DID bind
//! `audience_cid` (the BLAKE3-form CID of the audience pubkey bytes),
//! but `audience_pubkey` (the field that
//! `UcanBlobsHandler::validate_request_for_connection` ARM 2 ACTUALLY
//! compares to the connection's verified EndpointId) was unsigned.
//!
//! Attack scenario (access-theft, NOT just attribution-forgery):
//!   1. Alice issues grant G for audience Bob (G.audience_pubkey =
//!      bob_pubkey; G.audience_binding = CID(bob_pubkey)).
//!   2. Eve obtains G's bytes (e.g. observes a Drop bundle in transit
//!      or compromises a non-trusted relay store).
//!   3. Eve mutates G to G' with G'.audience_pubkey = eve_pubkey
//!      (leaves G.audience_binding = CID(bob_pubkey) UNCHANGED so the
//!      pre-fix audience_binding check still passes).
//!   4. Eve connects to a serving peer with her own iroh EndpointId
//!      (eve_pubkey).
//!   5. ARM 1 (unresolved-peer) passes — Eve is a resolvable peer.
//!   6. ARM 2 (audience-binding) compares conn_pubkey == eve_pubkey to
//!      G'.audience_pubkey == eve_pubkey → MATCH → PASSES (the
//!      pre-fix gap).
//!   7. ARM 5 (binding-sig verify) re-constructs the 5-segment message
//!      using G'.audience_binding = CID(bob_pubkey) which was bound at
//!      issue time — binding_sig STILL VERIFIES because the mutated
//!      audience_pubkey wasn't part of the signed payload.
//!   8. ARM 6 (scope) passes if the request hash is in G's scope.
//!   9. Handler hands connection to iroh-blobs → Eve receives Alice's
//!      ciphertext intended for Bob.
//!
//! The fix (R6 R2 Bundle R6-R2-FP-A) folds `audience_pubkey` into the
//! binding-message (6-segment layout: domain-tag || ucan-cbor ||
//! km-cbor || audience-cid || scope-len-u32-le || scope-cbor ||
//! audience-pk-len-u32-le || audience-pk-bytes). Domain-separation
//! tag bumped from `v2` to `v3` to reflect the wire-format change.
//!
//! This pin exercises the substantive arm:
//!   1. Issue a grant G for audience Bob with audience_pubkey =
//!      bob_pubkey.
//!   2. Clone G to G' with audience_pubkey swapped to eve_pubkey.
//!   3. `G'.verify_binding(G'.audience_binding)` MUST return
//!      `BindingMismatch` — NOT silently accept the substituted pubkey.
//!
//! Per pim-2 §3.6b sub-rule-4 substantive-arm coverage: the production
//! arm exercised is the binding-message audience_pubkey-segment
//! commitment + verify_binding audience_pubkey-segment re-construction,
//! NOT a sentinel presence check. Per pim-18 §3.6f
//! SHAPE-not-SUBSTANCE: the test MUST FAIL on revert of the
//! binding_message extension — verified at commit time by reverting
//! the Self::binding_message audience_pubkey arg and observing the
//! test PASS (i.e. silent admission of the substituted pubkey) under
//! the reverted code.

use benten_caps::authorization_grant::{AuthorizationGrant, AuthorizationGrantError};
use benten_caps::restricted_spec::RestrictedScope;
use benten_id::keypair::Keypair;

fn rng_keypair(_seed: u64) -> Keypair {
    // Seed-discriminator is process-local — Keypair::generate() uses
    // OsRng so each call yields a fresh distinct key. The _seed
    // parameter is retained as a readability hint for the test author.
    Keypair::generate()
}

#[test]
fn audience_substitution_post_sign_rejected_by_binding_sig() {
    let issuer_kp = rng_keypair(0xA11CE);
    let bob_kp = rng_keypair(0xB0B);
    let eve_kp = rng_keypair(0xE7E);

    let bob_pubkey = bob_kp.public_key();
    let eve_pubkey = eve_kp.public_key();

    let scope = RestrictedScope::new();
    let exp_secs = 9_999_999_999;
    let grant = AuthorizationGrant::issue_for_test(&issuer_kp, &bob_pubkey, scope, exp_secs);

    // Positive control: the grant at issue time MUST verify cleanly.
    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued grant MUST verify");
    // Sanity check: the issued grant carries bob_pubkey.
    assert_eq!(
        grant.audience_pubkey.as_deref(),
        Some(bob_pubkey.to_bytes().as_slice()),
        "issued grant MUST carry the audience's pubkey bytes (wave-3e contract)"
    );

    // Adversary tamper: mutate audience_pubkey to Eve's pubkey while
    // leaving audience_binding (the CID of Bob's pubkey) UNCHANGED.
    // This mirrors the access-theft attack vector — the attacker
    // patches the bytes ARM 2 compares against, but leaves the bytes
    // the pre-fix binding_message construction reads.
    let tampered =
        grant.with_swapped_audience_pubkey_for_test(Some(eve_pubkey.to_bytes().to_vec()));

    // Post-fix expectation: binding-sig re-verification reconstructs
    // the message with audience_pubkey = eve_pubkey (the mutated
    // value), which differs from the issue-time message that bound
    // audience_pubkey = bob_pubkey. The Ed25519 verify MUST fail with
    // BindingMismatch.
    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign audience_pubkey substitution MUST be rejected by \
         binding-sig re-verification (R6 R2 L2-R2-BLOCKER-1 closure). \
         Got: {outcome:?}"
    );
}

#[test]
fn audience_pubkey_removal_post_sign_rejected_by_binding_sig() {
    // Symmetric: swap Some(audience_pubkey) → None must also fail
    // re-verify. Closes the "downgrade to legacy wave-3b envelope"
    // attack where the attacker tries to drop the audience_pubkey
    // entirely and present the grant under the legacy
    // issue_envelopes_for_test shape (which carries audience_pubkey =
    // None). Without the binding-message commitment, this transition
    // would be silently accepted.
    let issuer_kp = rng_keypair(0xC0DE);
    let audience_kp = rng_keypair(0xCAFE);
    let audience_pub = audience_kp.public_key();
    let scope = RestrictedScope::new();
    let grant = AuthorizationGrant::issue_for_test(&issuer_kp, &audience_pub, scope, 9_999_999_999);

    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued grant MUST verify");

    // Swap audience_pubkey to None (downgrade to wave-3b shape).
    let tampered = grant.with_swapped_audience_pubkey_for_test(None);
    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign audience_pubkey removal (Some→None) MUST be rejected \
         by binding-sig re-verification. Got: {outcome:?}"
    );
}

#[test]
fn audience_pubkey_addition_to_envelope_grant_rejected_by_binding_sig() {
    // Symmetric mirror in the other direction: a wave-3b envelope
    // grant (issued with audience_pubkey = None) MUST NOT be
    // post-hoc upgradable by attaching an attacker-chosen
    // audience_pubkey while keeping the original binding_sig. Without
    // the binding-message commitment, an attacker could synthesize a
    // wave-3e-shaped grant from a wave-3b envelope and have ARM 2
    // admit them.
    use benten_caps::authorization_grant::{GrantKeyMaterial, UcanEnvelope};
    use benten_core::Cid;

    let attacker_kp = rng_keypair(0xBAD);
    let attacker_pubkey = attacker_kp.public_key();
    let audience_cid: Cid = [0xAA_u8; 32].into();
    let ucan = UcanEnvelope::synthetic_for_test(audience_cid);
    let km = GrantKeyMaterial::synthetic_for_test();
    let envelope_grant = AuthorizationGrant::issue_envelopes_for_test(ucan, km, audience_cid)
        .expect("envelope grant issues");

    envelope_grant
        .verify_binding(envelope_grant.audience_binding)
        .expect("freshly-issued envelope grant MUST verify");
    assert!(
        envelope_grant.audience_pubkey.is_none(),
        "envelope grant MUST carry audience_pubkey = None (wave-3b shape)"
    );

    // Attacker attempts to upgrade the envelope grant by attaching
    // their own pubkey post-sign.
    let upgraded = envelope_grant
        .with_swapped_audience_pubkey_for_test(Some(attacker_pubkey.to_bytes().to_vec()));
    let outcome = upgraded.verify_binding(upgraded.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign audience_pubkey addition (None→Some) on an envelope grant \
         MUST be rejected by binding-sig re-verification. Got: {outcome:?}"
    );
}
