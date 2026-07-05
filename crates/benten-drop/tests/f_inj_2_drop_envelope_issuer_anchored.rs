//! Phase-4-Meta-Core F-INJ-2 — DropBundle envelope-sig issuer-anchoring pin.
//!
//! The envelope signature (`envelope_sig`) verifies over the bundle HEADER
//! under `bundle.issuer_verifying_key` — an ATTACKER-CONTROLLABLE field
//! anchored to nothing on its own. On the PRE-FIX code the strip attack
//! succeeds:
//!   1. re-author the header (mutate any header field, e.g. `spec_cid`),
//!   2. mint a FRESH keypair,
//!   3. re-sign `build_envelope_message(&forged)` with the fresh key,
//!   4. overwrite `issuer_verifying_key` with the fresh public key.
//! `verify_envelope_signature()` then passes (a signature-by-nobody), and —
//! because the ORIGINAL `auth_grant` is still bound to the legitimate
//! recipient — `consume_offline()` would proceed to decrypt.
//!
//! The fix adds a verify-time anchor in `consume_offline` (Layer 2b, AFTER
//! the grant binding verifies): the bundle's `issuer_verifying_key` MUST
//! equal the authoritative issuer of the trusted `AuthorizationGrant`
//! (`auth_grant.issuer_verifying_key`, cryptographically self-bound via the
//! grant's 7-segment binding-message). On mismatch: typed
//! `DropBundleError::EnvelopeIssuerMismatch`. No wire byte / CBOR field /
//! golden vector changes — verify-time only.
//!
//! ## Would-FAIL-on-revert
//!
//! Revert the F-INJ-2 Layer-2b anchor in `consume_offline` and the strip
//! attack's `consume_offline()` returns `Ok(_)` (accepts the forged bundle),
//! so the `expect_err` below panics — the test fails on revert.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_drop::{DropBundle, DropBundleError};
use benten_id::keypair::Keypair;

// The strip-attack primitives (`build_envelope_message` + `sign_envelope`)
// live in the `envelope_sig` module — the exact surface the attacker uses to
// forge a self-consistent-but-hollow envelope signature.
use benten_drop::envelope_sig::{build_envelope_message, sign_envelope};

/// The reproduced strip attack: re-author the header + fresh-key re-sign +
/// overwrite `issuer_verifying_key`. `consume_offline()` MUST reject with the
/// typed `EnvelopeIssuerMismatch` — the envelope-sig is now anchored to the
/// trusted grant issuer, not to whatever key the attacker pins into the
/// header.
#[test]
fn f_inj_2_strip_attack_rejected_by_issuer_anchor() {
    let issuer_kp = Keypair::generate();
    let recipient_kp = Keypair::generate();

    // Honest bundle: issuer signs the envelope AND issues the grant, so
    // `bundle.issuer_verifying_key == auth_grant.issuer_verifying_key`.
    let honest = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);

    // Baseline: the honest bundle consumes cleanly (the anchor holds).
    honest
        .consume_offline(&recipient_kp)
        .expect("baseline: honest bundle consumes (issuer anchor holds)");

    // ---- The strip attack ------------------------------------------------
    let attacker_kp = Keypair::generate();
    let mut forged = honest.clone();

    // (1) Re-author the header. Mutate a header field the envelope-sig binds
    //     (spec_cid) so the ORIGINAL envelope-sig no longer verifies — the
    //     attacker must re-sign, which is the whole point.
    forged.spec_cid = benten_core::Cid::from_blake3_digest([0xEE; 32]);

    // (2)+(3) Fresh keypair re-signs the forged envelope message.
    let forged_msg = build_envelope_message(&forged);
    forged.envelope_sig = sign_envelope(&attacker_kp, &forged_msg);

    // (4) Overwrite the issuer_verifying_key with the attacker's key so
    //     Layer-1 verify_envelope_signature passes (signature-by-nobody).
    forged.issuer_verifying_key = attacker_kp.public_key().to_bytes().to_vec();

    // Layer-1 (envelope-sig alone) is fooled — it verifies against the
    // attacker-pinned key. This is exactly the hollow-signature property the
    // anchor exists to close; assert it so the test documents WHY Layer-2b is
    // load-bearing.
    forged
        .verify_envelope_signature()
        .expect("strip attack: envelope-sig alone verifies against the attacker-pinned key");

    // The recipient's trust path MUST reject: the forged bundle's
    // issuer_verifying_key (attacker) != auth_grant.issuer_verifying_key
    // (the honest issuer, still self-bound in the untouched grant).
    let err = forged
        .consume_offline(&recipient_kp)
        .expect_err("F-INJ-2: strip attack MUST be rejected by the envelope-issuer anchor");
    assert!(
        matches!(err, DropBundleError::EnvelopeIssuerMismatch { .. }),
        "F-INJ-2: strip attack must surface typed EnvelopeIssuerMismatch; got {err:?}",
    );
}

/// Minimal-mutation variant: leave the header + envelope-sig honest but swap
/// ONLY `issuer_verifying_key` to an unrelated key. This pins the anchor
/// directly (independent of the re-sign mechanics): a bundle whose
/// envelope-issuer key does not match the trusted grant's issuer is rejected
/// even if nothing else is touched. (Here Layer-1 fails first because the
/// honest envelope-sig no longer matches the swapped key — so we assert the
/// consume rejects; the load-bearing property is that consume does NOT
/// accept a bundle with a mismatched envelope-issuer.)
#[test]
fn f_inj_2_mismatched_envelope_issuer_never_accepted() {
    let issuer_kp = Keypair::generate();
    let recipient_kp = Keypair::generate();
    let stranger_kp = Keypair::generate();

    let honest = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);

    // Sanity: honest issuer key matches the grant issuer.
    assert_eq!(
        honest.issuer_verifying_key, honest.auth_grant.issuer_verifying_key,
        "fixture sanity: honest bundle binds the envelope issuer to the grant issuer",
    );

    let mut swapped = honest.clone();
    swapped.issuer_verifying_key = stranger_kp.public_key().to_bytes().to_vec();

    // The anchor's precondition holds: the swapped envelope-issuer differs
    // from the grant issuer.
    assert_ne!(
        swapped.issuer_verifying_key, swapped.auth_grant.issuer_verifying_key,
        "the swapped bundle's envelope issuer differs from the grant issuer",
    );

    // consume_offline MUST NOT accept it. (It rejects — whether at Layer-1,
    // because the honest sig no longer matches the swapped key, or at
    // Layer-2b — the load-bearing property is non-acceptance.)
    swapped
        .consume_offline(&recipient_kp)
        .expect_err("F-INJ-2: a bundle with a mismatched envelope-issuer MUST NOT consume");
}
