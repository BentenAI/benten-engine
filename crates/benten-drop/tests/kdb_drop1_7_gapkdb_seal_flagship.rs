//! GAP-KDB Shape-B — THE FLAGSHIP (seal + invariant layers): DROP-1 ★ (seal to
//! a substituted KEM key fails closed, across `0x6510` / `0x6520` / `0x6610`) +
//! DROP-7 ★ (Inv-23 real production-driving firing). W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §0 (the GAP-KDB asymmetry) + §5
//! (seal-API closure — `RecipientBinding`, Inv-23) + `R2-LANDSCAPE` FLAGSHIP-1
//! (three layers that go red→green together: resolver `RK-2`, seal `DROP-1`,
//! invariant `DROP-7`).
//!
//! # THE FLAGSHIP (DROP-1 ★ — GAP-KDB active substitution, SEAL layer)
//! The recipient KEM key is no longer an independently-chosen param. It arrives
//! inside a [`RecipientBinding`] whose sole constructor
//! (`RecipientBinding::resolve`) PROVES the key is committed by the audience DID
//! (a BLAKE3-256 CID 2nd-preimage). A KEM key NOT committed by the DID cannot be
//! bound → the binding-only seal can NEVER target it. This is enforced at ALL
//! THREE encrypt-to-recipient codepoints in this one file (per the `f_lc_gap1`
//! "all three codepoints in one file" precedent).
//!
//! # would_fail_on_revert (per family)
//! - **DROP-1 ★** — Revert to the two-independent-param seal
//!   (`seal_sealed_sender(recipient_pub, audience_did, …)`): a substituted KEM
//!   key that is NOT committed by the audience DID's key-set doc seals
//!   successfully and the attacker holding the substituted secret opens the
//!   envelope. The `expect_err`/round-trip arms below flip.
//! - **DROP-7 ★** — Drop the CID 2nd-preimage check inside `resolve_kem`: the
//!   seal-side `RecipientBinding::resolve(victim_did, attacker_doc)` returns a
//!   binding to K_attacker (active substitution succeeds) and the `is_err`
//!   asserts flip to `Ok` — Inv-23 stops firing.
//!
//! # R5 un-ignore
//! Mint benten-drop's real `RecipientBinding` (sole `resolve` constructor — C4)
//! + the binding-typed seal API (single + group `&[RecipientBinding]` — C9) +
//! Inv-23; repoint the [`benten_drop::kdb_seal_testing`] stub bodies; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::RecipientBinding;
use benten_drop::layer_c::group_posture::{
    GroupSealParams, GroupVerifyContext, open_membership_set_group,
};
use benten_drop::layer_c::{open_group_stanza, open_single};

// ===========================================================================
// DROP-1 ★ — seal to a substituted KEM key fails closed (all three codepoints).
// ===========================================================================

/// DROP-1 ★ (`0x6510`) — single-recipient. The binding to the honestly-
/// committed key-set resolves + seals + round-trips through the honest
/// recipient's REAL secret; the substituted key cannot be bound.
#[test]
fn drop1_flagship_single_0x6510_substituted_kem_key_fails_closed() {
    let s = seal::substitution_scenario();

    // Honest path — a binding to the honestly-committed key-set resolves, and a
    // seal to it round-trips through the honest recipient's REAL secret. (A
    // no-op resolve/seal fails the round-trip.)
    let honest = RecipientBinding::resolve(&s.victim_did, &s.honest_doc)
        .expect("DROP-1: the honestly-committed key-set MUST resolve to a binding");
    let (sk, sender) = seal::hybrid_did_key_sender();
    let audience = s.victim_did.as_str().as_bytes().to_vec();
    let plaintext = b"drop-1 single-recipient confidential payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_to_binding(&honest, &sender, &sk, &body_cid, 0, &plaintext);
    let (recovered, _sender) = open_single(&seal::sec_of(&s.honest_kp), &audience, 0, &env)
        .expect("DROP-1 (0x6510): the honest recipient's REAL secret MUST open the seal");
    assert_eq!(
        recovered, plaintext,
        "DROP-1 (0x6510): honest round-trip plaintext"
    );

    // THE FLAGSHIP: the attacker's substituted KEM key cannot be bound under the
    // victim DID (BLAKE3-256 CID 2nd-preimage), so no seal can target it.
    assert!(
        RecipientBinding::resolve(&s.victim_did, &s.attacker_doc).is_err(),
        "DROP-1 ★ (0x6510): a KEM key NOT committed by the audience DID MUST fail closed; \
         the binding-only seal can therefore never target K_attacker. would-FAIL-on-revert: \
         restoring the two-param (recipient_pub, audience_did) seal lets the attacker seal to \
         K_attacker + open with the substituted secret."
    );
}

/// DROP-1 ★ (`0x6520`) — Layer-C group multi-stanza. Each member opens their
/// own stanza with their real secret; the substituted key cannot enter the
/// `&[RecipientBinding]` roster.
#[test]
fn drop1_flagship_group_0x6520_substituted_kem_key_fails_closed() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("member A binding resolves"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("member B binding resolves"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let plaintext = b"drop-1 group confidential payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_group_to_bindings(&bindings, &sender, &sk, &body_cid, 0, &plaintext)
        .expect("DROP-1 (0x6520): group seal to the binding roster");
    let roster = seal::roster_of_bindings(&bindings);
    let (pt_a, _) = open_group_stanza(&seal::sec_of(&a.kp), 0, &roster, 0, &env)
        .expect("DROP-1 (0x6520): member A's REAL secret opens stanza 0");
    let (pt_b, _) = open_group_stanza(&seal::sec_of(&b.kp), 1, &roster, 0, &env)
        .expect("DROP-1 (0x6520): member B's REAL secret opens stanza 1");
    assert_eq!(pt_a, plaintext, "DROP-1 (0x6520): member A round-trip");
    assert_eq!(pt_b, plaintext, "DROP-1 (0x6520): member B round-trip");

    // THE FLAGSHIP at the group layer: a substituted member key cannot be bound
    // → it can never enter the roster slice → it can never be sealed to.
    let s = seal::substitution_scenario();
    assert!(
        RecipientBinding::resolve(&s.victim_did, &s.attacker_doc).is_err(),
        "DROP-1 ★ (0x6520): a substituted KEM key cannot be bound under a victim DID, so a \
         group roster of `&[RecipientBinding]` can never wrap the CEK to K_attacker. \
         would-FAIL-on-revert: the pre-C9 fabricated-DID roster wrapped to whatever KEM keys \
         it was handed, with zero DID-binding."
    );
}

/// DROP-1 ★ (`0x6610`) — MembershipSet K_Set group. Each member opens their
/// MembershipSet stanza with their real secret; the substituted key cannot
/// enter the `&[RecipientBinding]` roster.
#[test]
fn drop1_flagship_membership_set_0x6610_substituted_kem_key_fails_closed() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("member A binding resolves"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("member B binding resolves"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let k_set = [0x5au8; 32];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:drop1-membership-flagship".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let plaintext = b"drop-1 membership-set confidential payload".to_vec();

    let env =
        seal::seal_membership_set_to_bindings(&bindings, &sender, &sk, &k_set, &params, &plaintext)
            .expect("DROP-1 (0x6610): membership-set seal to the binding roster");
    let ctx = GroupVerifyContext {
        member_dids: seal::member_dids_of_bindings(&bindings),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (pt_a, _) = open_membership_set_group(&seal::sec_of(&a.kp), 0, &ctx, &env)
        .expect("DROP-1 (0x6610): member A opens stanza 0");
    let (pt_b, _) = open_membership_set_group(&seal::sec_of(&b.kp), 1, &ctx, &env)
        .expect("DROP-1 (0x6610): member B opens stanza 1");
    assert_eq!(pt_a, plaintext, "DROP-1 (0x6610): member A round-trip");
    assert_eq!(pt_b, plaintext, "DROP-1 (0x6610): member B round-trip");

    // THE FLAGSHIP at the membership-set layer.
    let s = seal::substitution_scenario();
    assert!(
        RecipientBinding::resolve(&s.victim_did, &s.attacker_doc).is_err(),
        "DROP-1 ★ (0x6610): a substituted KEM key cannot be bound under a victim DID."
    );
}

// ===========================================================================
// DROP-7 ★ — Inv-23 real production-driving firing (invariant layer).
// ===========================================================================

/// DROP-7 ★ — Inv-23 ("a Layer-C seal's KEM key is committed by its audience
/// DID") fires on the PRODUCTION seal-side constructor `RecipientBinding::
/// resolve`. Build a victim DID committing `D_honest(kem = K_honest)`; the
/// attempt to bind `D_attacker(kem = K_attacker)` under it MUST return `Err`
/// (committed-CID != recomputed-CID, BLAKE3-256 2nd-preimage), so no seal to
/// K_attacker is constructible.
#[test]
fn drop7_flagship_inv23_fires_on_seal_side_binding_resolve() {
    let s = seal::substitution_scenario();

    // Precondition — the attacker's key-set has a DISTINCT canonical CID
    // (2nd-preimage resistance: it CANNOT collide with the victim's committed CID).
    assert_ne!(
        s.honest_doc.cid().as_bytes(),
        s.attacker_doc.cid().as_bytes(),
        "precondition: attacker key-set has a distinct CID"
    );

    // Inv-23 FIRES on the real seal-side constructor.
    assert!(
        RecipientBinding::resolve(&s.victim_did, &s.attacker_doc).is_err(),
        "DROP-7 ★: Inv-23 MUST fire on the production seal-side `RecipientBinding::resolve` — \
         a KEM key not committed by the audience DID is unbindable. would-FAIL-on-revert: \
         dropping the CID 2nd-preimage check returns a binding to K_attacker."
    );

    // Control + substance — the honestly-committed key-set DOES bind, and the
    // recovered KEM key is EXACTLY the committed one (a no-op resolve fails this).
    let honest = RecipientBinding::resolve(&s.victim_did, &s.honest_doc)
        .expect("DROP-7 control: the honestly-committed key-set MUST bind (Inv-23 discriminates)");
    assert_eq!(
        honest.kem_pub().to_bytes(),
        seal::pub_of(&s.honest_kp).to_bytes(),
        "DROP-7: the bound KEM key MUST be the honestly-committed key (resolve is not a no-op)"
    );
}

/// DROP-7 ★ (roster shapes) — the Inv-23 firing means a substituted key can
/// never enter EITHER group roster (`0x6520` / `0x6610`): each member slot is a
/// `RecipientBinding`, and the substituted key is unbindable. Anti-downgrade
/// catch-net: there is no seal entry that accepts a raw `(K_attacker,
/// victim_did)` pair (that entry is retired — DROP-3 greps its absence).
#[test]
fn drop7_flagship_inv23_gates_every_group_roster_slot() {
    let s = seal::substitution_scenario();

    // A substituted member cannot be bound → cannot join a `&[RecipientBinding]`
    // roster for either group band. This is what makes the group flagship
    // (DROP-1 0x6520/0x6610) unconstructible.
    assert!(
        RecipientBinding::resolve(&s.victim_did, &s.attacker_doc).is_err(),
        "DROP-7 ★: Inv-23 gates every group roster slot; a substituted member is unbindable."
    );

    // The honest members bind — a roster of real members is fully constructible.
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let roster = [
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    assert_eq!(
        roster.len(),
        2,
        "DROP-7: an all-honest roster is constructible"
    );
}
