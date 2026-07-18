//! GAP-KDB Shape-B — DROP-5: C8 `recipient_key_generation` ⟺ committed-key-set
//! precedence, DROP side. W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` C8 + `R2-LANDSCAPE` DROP-5 + §5
//! **S2** (the design-nail that landed): *at v1-beta the committed key-set is
//! AUTHORITATIVE for the KEM key, and `recipient_key_generation` is
//! MATCHES-OR-FAIL-CLOSED (no rotation)*.
//!
//! # The C8 role separation (S2-resolved)
//! - The **committed key-set** (the binding's DID + key-set doc) identifies
//!   WHICH KEM keypair — authoritatively + deterministically. `recipient_key_
//!   generation` is NOT a key selector; it cannot silently redirect which key
//!   is used.
//! - `recipient_key_generation` is the intra-keypair freshness index bound into
//!   the AAD / origin-auth. With NO rotation at v1-beta, exactly one generation
//!   is valid for the committed key-set: an open whose independently-held
//!   generation DISAGREES with the sealed one MUST fail closed (matches-or-fail-
//!   closed).
//!
//! # would_fail_on_revert
//! - If the KEM key selection leaked a generation index (breaking role
//!   separation), resolving the same binding under a different intended
//!   generation would return a DIFFERENT key — the determinism assert flips.
//! - If `recipient_key_generation` stopped being bound (matches-or-fail-closed
//!   dropped), the disagreeing-generation open flips `Err` → `Ok`.
//!
//! # R5 un-ignore
//! Mint the binding-typed seal (committed key-set = the KEM key) + keep
//! `recipient_key_generation` bound into the AAD/origin-auth; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::{LayerCError, open_single};
use benten_id::kdb_testing::RecipientBinding;

/// DROP-5 — role separation: the committed key-set authoritatively +
/// deterministically fixes the KEM key; the generation does not select it.
#[test]
#[ignore = "RED-PHASE: DROP-5 committed key-set is authoritative for the KEM key (C8 role separation) — un-ignore at R5"]
fn drop5_committed_key_set_is_authoritative_for_the_kem_key() {
    let r = seal::real_recipient();

    // Resolving the same committed key-set twice yields byte-identical bound
    // KEM keys — a pure function of the commitment, NOT a generation index.
    let first = RecipientBinding::resolve(&r.did, &r.doc).expect("binding resolves");
    let second = RecipientBinding::resolve(&r.did, &r.doc).expect("binding resolves again");
    assert_eq!(
        first.kem_pub().to_bytes(),
        second.kem_pub().to_bytes(),
        "DROP-5: the committed key-set alone determines the bound KEM key (deterministic)"
    );
    // …and it is EXACTLY the committed key (not a no-op / not generation-derived).
    assert_eq!(
        first.kem_pub().to_bytes(),
        seal::pub_of(&r.kp).to_bytes(),
        "DROP-5: the committed key-set = which keypair; the generation index is NOT a key selector"
    );
}

/// DROP-5 — matches-or-fail-closed: the sealed `recipient_key_generation` is the
/// intra-keypair freshness index; an open whose independently-held generation
/// DISAGREES fails closed (no rotation → one valid generation).
#[test]
#[ignore = "RED-PHASE: DROP-5 recipient_key_generation matches-or-fail-closed (C8/S2) — un-ignore at R5"]
fn drop5_generation_matches_or_fail_closed() {
    let r = seal::real_recipient();
    let binding = RecipientBinding::resolve(&r.did, &r.doc).expect("binding resolves");
    let (sk, sender) = seal::hybrid_did_key_sender();
    let audience = r.did.as_str().as_bytes().to_vec();
    let plaintext = b"drop-5 generation precedence payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();
    let sealed_generation: u32 = 5;

    let env = seal::seal_to_binding(
        &binding,
        &sender,
        &sk,
        &body_cid,
        sealed_generation,
        &plaintext,
    );

    // Matches — the recipient's held generation equals the sealed one → opens.
    let (recovered, _) = open_single(&seal::sec_of(&r.kp), &audience, sealed_generation, &env)
        .expect("DROP-5: matching generation MUST open");
    assert_eq!(
        recovered, plaintext,
        "DROP-5: matching-generation round-trip"
    );

    // Disagreement — a DIFFERENT held generation fails closed.
    let disagreeing_generation: u32 = 6;
    assert!(
        matches!(
            open_single(
                &seal::sec_of(&r.kp),
                &audience,
                disagreeing_generation,
                &env
            ),
            Err(LayerCError::SenderOriginAuthFailed)
        ),
        "DROP-5 (C8/S2): a recipient_key_generation that DISAGREES with the sealed one MUST fail \
         closed (no rotation at v1-beta → one valid generation). would-FAIL-on-revert: dropping \
         the generation binding opens the disagreeing case."
    );
}
