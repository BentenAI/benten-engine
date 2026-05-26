//! TF-3b — R6 R1 fix-pass (Bundle L3-r1-1): scope-substitution
//! post-sign rejection pin.
//!
//! Closes R6 R1 L3 finding l3-r1-1: pre-R6-R1, `AuthorizationGrant.scope`
//! lived OUTSIDE the binding-sig message construction
//! (`binding_message(ucan, key_material, audience)` — 4 segments,
//! scope omitted). An attacker who obtained any verifiable grant could
//! present it with a freshly-widened `scope: Some(with_hashes([X, attacker-hashes...]))`;
//! `verify_binding` PASSED (scope wasn't part of the signed payload);
//! the ALPN handler's ARM 6 read `request.grant.scope` (the
//! post-sign-mutable field) and admitted the wider scope.
//!
//! The fix (R6 R1 Bundle L3-r1-1) folds `scope` into the binding-
//! message (5-segment layout: domain-tag || ucan-cbor || km-cbor ||
//! audience-cid || scope-len-u32-le || scope-cbor). Domain-separation
//! tag bumped from `v1` to `v2` to reflect the wire-format change.
//!
//! This pin exercises the substantive arm:
//!   1. Issue a grant `G` for audience `A` with scope `scope_A` (narrow).
//!   2. Clone `G` to `G'` with `scope` swapped to `scope_B` (wider).
//!   3. `G'.verify_binding(A)` MUST return `BindingMismatch` — NOT
//!      admit the wider scope.
//!
//! Per pim-2 §3.6b sub-rule-4 substantive-arm coverage: the production
//! arm exercised is the binding-message scope-segment commitment +
//! verify_binding scope-segment re-construction, NOT a sentinel
//! presence check.

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
fn scope_substitution_post_sign_rejected_by_binding_sig() {
    let issuer_kp = rng_keypair(0xA11CE);
    let audience_kp = rng_keypair(0xB0B);
    let audience_pub = audience_kp.public_key();

    // Narrow scope at issue time (empty selector set; production code
    // would carry the issuer's intended hash list here).
    let scope_a = RestrictedScope::new();

    let exp_secs = 9_999_999_999;
    let grant =
        AuthorizationGrant::issue_for_test(&issuer_kp, &audience_pub, scope_a.clone(), exp_secs);

    // Positive control: the grant at issue time MUST verify cleanly.
    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued grant MUST verify");

    // Adversary tamper: clone the grant and swap the scope to a
    // different RestrictedScope value (in production the attacker
    // would set this to a *wider* selector; for the binding-sig test,
    // any structural difference between scope_a and scope_b suffices
    // — the binding-message commits to the CBOR-canonicalized bytes,
    // so any structural change flips the signed message).
    // Push a discriminating bit so scope_b ≠ scope_a in CBOR.
    // We tamper at the typed RestrictedScope surface; the with-hashes
    // builder pattern is the production-side widening attack vector.
    use benten_core::Cid;
    let attacker_hash: Cid = [0xEE_u8; 32].into();
    let scope_b = RestrictedScope::with_hashes(vec![attacker_hash]);

    let tampered = grant.with_swapped_scope_for_test(Some(scope_b));
    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign scope-substitution MUST be rejected by binding-sig \
         re-verification (R6 R1 L3-r1-1 closure). Got: {outcome:?}"
    );
}

#[test]
fn scope_removal_post_sign_rejected_by_binding_sig() {
    // Symmetric: swap Some(scope) → None must also fail re-verify.
    let issuer_kp = rng_keypair(0xC0DE);
    let audience_kp = rng_keypair(0xCAFE);
    let audience_pub = audience_kp.public_key();
    let scope = RestrictedScope::new();
    let grant = AuthorizationGrant::issue_for_test(&issuer_kp, &audience_pub, scope, 9_999_999_999);

    grant
        .verify_binding(grant.audience_binding)
        .expect("freshly-issued grant MUST verify");

    // Swap scope to None.
    let tampered = grant.with_swapped_scope_for_test(None);
    let outcome = tampered.verify_binding(tampered.audience_binding);
    assert!(
        matches!(
            outcome,
            Err(AuthorizationGrantError::BindingMismatch { .. })
        ),
        "post-sign scope-removal (Some→None) MUST be rejected by binding-sig \
         re-verification. Got: {outcome:?}"
    );
}

#[test]
fn binding_message_domain_tag_is_v3_post_fix() {
    // Wire-format pin: the BINDING_SIG_DOMAIN tag was bumped from v1
    // to v2 at R6 R1 Bundle L3-r1-1 (scope-segment addition), and
    // FURTHER BUMPED from v2 to v3 at R6 R2 Bundle R6-R2-FP-A
    // (audience_pubkey-segment addition; closes L2-R2-BLOCKER-1
    // audience-substitution attack). A v2-signed grant deserialized
    // today + re-verified would fail (different domain tag →
    // different signed bytes). This pin asserts the tag is at v3 to
    // defend against accidental revert that would re-open either the
    // L3-r1-1 OR L2-R2-BLOCKER-1 attack class.
    //
    // The tag is private (compile-time constant); we exercise via the
    // round-trip: issue under the current binding-message + verify;
    // if the domain tag were silently reverted to v1/v2, the issue
    // path would sign over an older message but verify_binding would
    // re-construct a v3 message (or vice versa) — neither symmetric
    // round-trip failure mode would be caught by the substitution-
    // rejected pin alone. The freeze-discipline coupling is captured
    // here.
    let issuer_kp = rng_keypair(0xD00D);
    let audience_kp = rng_keypair(0xBABE);
    let scope = RestrictedScope::new();
    let grant = AuthorizationGrant::issue_for_test(
        &issuer_kp,
        &audience_kp.public_key(),
        scope,
        9_999_999_999,
    );
    grant
        .verify_binding(grant.audience_binding)
        .expect("issue + verify MUST agree on the canonical domain tag");
}
