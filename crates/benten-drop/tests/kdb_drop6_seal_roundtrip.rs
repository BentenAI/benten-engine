//! GAP-KDB Shape-B — DROP-6: binding-typed seal → open round-trip + AAD
//! threading (positive path, all three codepoints). W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (new seal signature) +
//! `R2-LANDSCAPE` DROP-6.
//!
//! # What this pins
//! The binding-typed seal is a drop-in for the round-trip: sealing to a
//! [`RecipientBinding`] (whose KEM key was recovered from the committed key-set)
//! still delivers the plaintext to the matching real secret, recovers the
//! sender-DID, and threads the audience/body-CID/generation into the AAD (a
//! mismatched audience on open fails the F-2 re-derived origin-auth commitment).
//!
//! # would_fail_on_revert
//! A no-op seal (or one that drops the binding's recovered KEM key) fails the
//! positive round-trip `expect`. Dropping the audience from the AAD binding
//! flips the wrong-audience negative arm from `Err` to `Ok`.
//!
//! # R5 un-ignore
//! Mint the binding-typed seal API; repoint the [`benten_drop::kdb_seal_testing`]
//! stubs; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::group_posture::{
    GroupSealParams, GroupVerifyContext, open_membership_set_group,
};
use benten_drop::layer_c::{LayerCError, open_group_stanza, open_single};
use benten_id::kdb_testing::RecipientBinding;

/// DROP-6 (`0x6510`) — single-recipient round-trip + AAD threading.
#[test]
#[ignore = "RED-PHASE: DROP-6 0x6510 seal→open round-trip + AAD threading — un-ignore at R5"]
fn drop6_single_0x6510_roundtrip_and_aad_threading() {
    let r = seal::real_recipient();
    let binding = RecipientBinding::resolve(&r.did, &r.doc).expect("recipient binding resolves");
    let (sk, sender) = seal::hybrid_did_key_sender();
    let audience = r.did.as_str().as_bytes().to_vec();
    let plaintext = b"drop-6 round-trip payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_to_binding(&binding, &sender, &sk, &body_cid, 3, &plaintext);

    // Positive — the matching real secret + correct audience/generation opens.
    let (recovered, recovered_sender) = open_single(&seal::sec_of(&r.kp), &audience, 3, &env)
        .expect("DROP-6 (0x6510): matching secret + audience/generation MUST open");
    assert_eq!(
        recovered, plaintext,
        "DROP-6 (0x6510): round-trip plaintext"
    );
    assert_eq!(
        recovered_sender, sender,
        "DROP-6 (0x6510): sender recovered"
    );

    // AAD threading — opening with a DIFFERENT audience DID fails closed (the
    // F-2 re-derived origin-auth commitment binds the audience).
    let wrong_audience = b"did:benten:zWrongAudienceForDrop6".to_vec();
    assert!(
        matches!(
            open_single(&seal::sec_of(&r.kp), &wrong_audience, 3, &env),
            Err(LayerCError::SenderOriginAuthFailed)
        ),
        "DROP-6 (0x6510): a mismatched audience DID MUST fail closed — the audience is threaded \
         into the AAD binding. would-FAIL-on-revert: dropping the audience binding opens it."
    );
}

/// DROP-6 (`0x6520`) — Layer-C group round-trip.
#[test]
#[ignore = "RED-PHASE: DROP-6 0x6520 group seal→open round-trip — un-ignore at R5"]
fn drop6_group_0x6520_roundtrip() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let plaintext = b"drop-6 group round-trip payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_group_to_bindings(&bindings, &sender, &sk, &body_cid, 0, &plaintext)
        .expect("group seal");
    let roster = seal::roster_of_bindings(&bindings);
    let (pt_a, _) =
        open_group_stanza(&seal::sec_of(&a.kp), 0, &roster, 0, &env).expect("A opens stanza 0");
    let (pt_b, _) =
        open_group_stanza(&seal::sec_of(&b.kp), 1, &roster, 0, &env).expect("B opens stanza 1");
    assert_eq!(pt_a, plaintext, "DROP-6 (0x6520): A round-trip");
    assert_eq!(pt_b, plaintext, "DROP-6 (0x6520): B round-trip");
}

/// DROP-6 (`0x6610`) — MembershipSet K_Set group round-trip.
#[test]
#[ignore = "RED-PHASE: DROP-6 0x6610 membership-set seal→open round-trip — un-ignore at R5"]
fn drop6_membership_set_0x6610_roundtrip() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let k_set = [0x11u8; 32];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:drop6-roundtrip".to_vec(),
        member_key_generation: 2,
        membership_set_generation: 4,
        role_assignments_generation: 1,
    };
    let plaintext = b"drop-6 membership-set round-trip payload".to_vec();

    let env =
        seal::seal_membership_set_to_bindings(&bindings, &sender, &sk, &k_set, &params, &plaintext)
            .expect("membership-set seal");
    let ctx = GroupVerifyContext {
        member_dids: seal::member_dids_of_bindings(&bindings),
        member_key_generation: 2,
        membership_set_generation: 4,
        role_assignments_generation: 1,
    };
    let (pt_a, _) =
        open_membership_set_group(&seal::sec_of(&a.kp), 0, &ctx, &env).expect("A opens stanza 0");
    let (pt_b, _) =
        open_membership_set_group(&seal::sec_of(&b.kp), 1, &ctx, &env).expect("B opens stanza 1");
    assert_eq!(pt_a, plaintext, "DROP-6 (0x6610): A round-trip");
    assert_eq!(pt_b, plaintext, "DROP-6 (0x6610): B round-trip");
}
