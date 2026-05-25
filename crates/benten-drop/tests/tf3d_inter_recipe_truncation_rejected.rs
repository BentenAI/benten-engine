//! TF-3d — R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling): inter-Recipe
//! truncation post-encrypt rejection pin.
//!
//! Closes R6 R2 L4-MAJ finding: pre-R6-R2, `DropBundle.content`'s
//! per-Recipe `EncryptedContent` cells were sealed with
//! `aad_whole_content(plaintext_cid)` only — the per-Recipe AEAD
//! committed neither to its position in the list nor to the list
//! length. An attacker (e.g. a malicious relay along the Drop
//! distribution path) could drop one Recipe from `bundle.content`,
//! re-serialize the CBOR bundle, and the remaining N-1 Recipes' AEAD
//! tags would STILL verify individually at `consume_offline` — silent
//! delivery of a truncated bundle to the recipient.
//!
//! Attack scenario:
//!   1. Alice builds a 5-Recipe bundle (recipes 0..=4) for Bob with
//!      shared bundle key K.
//!   2. A malicious relay strips recipes[4] (the last one) before
//!      forwarding to Bob; bundle.content is now length 4.
//!   3. Bob's consume_offline iterates 4 cells; pre-fix, each
//!      decrypt(node, key) succeeds because the per-Recipe AAD
//!      committed only to plaintext_cid (unchanged).
//!   4. Bob believes he received the complete 4-Recipe bundle (no
//!      cryptographic signal of the truncation).
//!
//! The fix (R6 R2 Bundle R6-R2-FP-A L4 sibling) seals each Recipe
//! with extended AAD = `aad_per_recipe(plaintext_cid, recipe_index,
//! total_recipes)` (mirrors the F3 per-chunk `total_chunks` binding
//! that closes the cross-chunk-truncation attack). At decrypt time
//! the consumer reconstructs AAD with `total_recipes =
//! self.content.len()`; if a relay truncated the list, the
//! reconstructed AAD doesn't match the seal-time AAD and every
//! remaining Recipe's per-Recipe AEAD authentication fails.
//!
//! This pin exercises the substantive arm:
//!   1. Build a 5-Recipe bundle B for Bob.
//!   2. Verify positive control: consume_offline(B, bob_kp) returns
//!      5 plaintexts.
//!   3. Construct B' = B with `content` truncated to 4 cells.
//!   4. consume_offline(B', bob_kp) MUST return
//!      `PerNodeAeadAuthenticationFailed` — NOT silently return 4
//!      plaintexts.
//!
//! Per pim-2 §3.6b sub-rule-4 substantive-arm coverage: the
//! production arm exercised is the per-Recipe AAD-binding seal +
//! consume_offline AAD-reconstruction, NOT a sentinel presence check.
//! Per pim-18 §3.6f SHAPE-not-SUBSTANCE: the test MUST FAIL on
//! revert of the per-Recipe AAD extension — verified at commit time
//! by reverting `decrypt_recipe_encrypted_node` to the prior
//! `decrypt` and observing the truncated-bundle consume succeed
//! silently under the reverted code.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_drop::{DropBundle, DropBundleError};
use benten_id::keypair::Keypair;

#[test]
fn inter_recipe_truncation_rejected_by_per_recipe_aad() {
    let issuer_kp = Keypair::generate();
    let recipient_kp = Keypair::generate();

    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);
    assert_eq!(
        bundle.content_count(),
        5,
        "fixture invariant: 5-Recipe bundle MUST have 5 EncryptedContent cells"
    );

    // Positive control — the un-tampered 5-Recipe bundle consumes
    // cleanly under the per-Recipe AAD path.
    let recovered = bundle
        .consume_offline(&recipient_kp)
        .expect("untampered 5-Recipe bundle MUST consume_offline cleanly");
    assert_eq!(
        recovered.len(),
        5,
        "untampered consume_offline MUST yield 5 plaintexts"
    );

    // Adversary tamper: relay strips the last Recipe, leaving a
    // length-4 content vec.
    let mut tampered = bundle.clone();
    tampered.content.pop();
    assert_eq!(
        tampered.content_count(),
        4,
        "tamper invariant: truncated bundle now has 4 cells"
    );

    // Post-fix expectation: consume_offline MUST fail at the
    // per-Recipe AEAD layer (not silently return 4 plaintexts).
    let outcome = tampered.consume_offline(&recipient_kp);
    assert!(
        matches!(
            outcome,
            Err(DropBundleError::PerNodeAeadAuthenticationFailed { .. })
        ),
        "inter-Recipe truncation (5→4) MUST surface as \
         PerNodeAeadAuthenticationFailed at consume_offline — NOT \
         silently return 4 plaintexts. Got: {outcome:?}"
    );
}

#[test]
fn inter_recipe_drop_from_middle_rejected_by_per_recipe_aad() {
    // Mirror: dropping a Recipe from the MIDDLE of the bundle (not
    // just the tail) must also fail. This catches the position-aware
    // half of the AAD binding: even if total_recipes were spoofed to
    // match, recipe_index would shift for all subsequent cells.
    let issuer_kp = Keypair::generate();
    let recipient_kp = Keypair::generate();

    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);

    // Positive control.
    bundle
        .consume_offline(&recipient_kp)
        .expect("untampered bundle MUST consume_offline cleanly");

    // Adversary tamper: drop the middle Recipe (index 2).
    let mut tampered = bundle.clone();
    tampered.content.remove(2);
    assert_eq!(tampered.content_count(), 4);

    let outcome = tampered.consume_offline(&recipient_kp);
    assert!(
        matches!(
            outcome,
            Err(DropBundleError::PerNodeAeadAuthenticationFailed { .. })
        ),
        "inter-Recipe middle-drop (remove index 2) MUST surface as \
         PerNodeAeadAuthenticationFailed at consume_offline. \
         Got: {outcome:?}"
    );
}

#[test]
fn inter_recipe_shuffle_rejected_by_per_recipe_aad() {
    // Mirror: shuffling Recipes (swapping two positions) must also
    // fail. The per-Recipe AAD binds recipe_index, so any reorder
    // flips the AAD bytes for the swapped cells.
    let issuer_kp = Keypair::generate();
    let recipient_kp = Keypair::generate();

    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);

    bundle
        .consume_offline(&recipient_kp)
        .expect("untampered bundle MUST consume_offline cleanly");

    // Adversary tamper: swap Recipe 0 and Recipe 4.
    let mut tampered = bundle.clone();
    tampered.content.swap(0, 4);

    let outcome = tampered.consume_offline(&recipient_kp);
    assert!(
        matches!(
            outcome,
            Err(DropBundleError::PerNodeAeadAuthenticationFailed { .. })
        ),
        "inter-Recipe shuffle (swap 0↔4) MUST surface as \
         PerNodeAeadAuthenticationFailed at consume_offline. \
         Got: {outcome:?}"
    );
}
