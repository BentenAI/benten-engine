//! GAP-KDB Shape-B — DROP-4: C9 roster-replacement — the group
//! `audience_set_commitment` roster AND the wrap-targets are derived from ONE
//! `&[RecipientBinding]` slice, retiring the fabricated-DID placeholder
//! (`layer_c.rs:1093`, `group_roster` hashing KEM keys into `did:key:z…`). W2
//! benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (group paths take
//! `&[RecipientBinding]`) + C9 ("REPLACE the pubkey-derived placeholder roster
//! … Derive both `audience_set_commitment` and wrap-targets from one
//! `&[RecipientBinding]` slice") + `R2-LANDSCAPE` DROP-4.
//!
//! # The defect C9 closes
//! Today `group_roster(recipient_pubs)` FABRICATES audience DIDs by hashing the
//! KEM keys (`did:key:z ‖ blake3(kem_pub)`) → the blinded roster commitment
//! binds KEM-key-hashes, NOT recipient identities (zero identity binding). C9
//! derives the roster from the REAL recipient DIDs carried by the bindings.
//!
//! # would_fail_on_revert (discriminating)
//! With the C9 fix, an honest member recomputes the B2 `audience_set_commitment`
//! over the REAL member DIDs and opens; the OLD KEM-key-hashed placeholder
//! roster does NOT open (the commitment the seal bound is over real DIDs). Under
//! the fabricated-DID revert, BOTH arms flip: the real-DID roster fails and the
//! KEM-hash roster opens.
//!
//! # R5 un-ignore
//! Migrate the group seal to derive the commitment + wrap-targets from one
//! `&[RecipientBinding]` slice (the real DIDs); repoint the stubs; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::group_posture::{
    GroupError, GroupSealParams, GroupVerifyContext, open_membership_set_group,
};
use benten_drop::layer_c::{LayerCError, group_roster_for_test, open_group_stanza};
use benten_id::kdb_testing::RecipientBinding;

/// DROP-4 (`0x6520`) — the group commitment is over the REAL binding DIDs.
/// Opening with the real-DID roster succeeds; opening with the OLD KEM-key-
/// hashed placeholder roster fails closed (the seal bound identities, not
/// key-hashes).
#[test]
#[ignore = "RED-PHASE: DROP-4 0x6520 commitment over real binding DIDs (C9) — un-ignore at R5"]
fn drop4_group_0x6520_commitment_over_real_binding_dids() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let plaintext = b"drop-4 c9 roster replacement payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_group_to_bindings(&bindings, &sender, &sk, &body_cid, 0, &plaintext)
        .expect("group seal to bindings");

    // The REAL binding DIDs are the commitment roster — opening with them works.
    let real_roster = seal::roster_of_bindings(&bindings);
    let (pt_a, _) = open_group_stanza(&seal::sec_of(&a.kp), 0, &real_roster, 0, &env)
        .expect("DROP-4: opening with the REAL binding-DID roster MUST succeed (C9)");
    assert_eq!(
        pt_a, plaintext,
        "DROP-4 (0x6520): real-DID roster round-trip"
    );

    // The OLD fabricated-DID roster (KEM-key hashes) is NOT what the seal bound
    // → the F-2 re-derived commitment mismatches → fail closed.
    let fabricated_roster = group_roster_for_test(&[seal::pub_of(&a.kp), seal::pub_of(&b.kp)]);
    assert_ne!(
        real_roster, fabricated_roster,
        "precondition: the real DIDs differ from the KEM-key-hashed placeholder DIDs"
    );
    assert!(
        matches!(
            open_group_stanza(&seal::sec_of(&a.kp), 0, &fabricated_roster, 0, &env),
            Err(LayerCError::SenderOriginAuthFailed)
        ),
        "DROP-4 (0x6520): the KEM-key-hashed placeholder roster MUST NOT open a C9-sealed \
         envelope — the commitment binds REAL identities. would-FAIL-on-revert: the \
         fabricated-DID `group_roster` seal flips both arms (real fails, KEM-hash opens)."
    );
}

/// DROP-4 (`0x6610`) — the MembershipSet commitment is over the REAL binding
/// member DIDs; the OLD KEM-key-hashed member roster fails closed.
#[test]
#[ignore = "RED-PHASE: DROP-4 0x6610 commitment over real binding member DIDs (C9) — un-ignore at R5"]
fn drop4_membership_set_0x6610_commitment_over_real_binding_dids() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let k_set = [0x33u8; 32];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:drop4-c9".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let plaintext = b"drop-4 membership c9 payload".to_vec();

    let env =
        seal::seal_membership_set_to_bindings(&bindings, &sender, &sk, &k_set, &params, &plaintext)
            .expect("membership-set seal to bindings");

    // Real binding member-DID ctx opens.
    let real_ctx = GroupVerifyContext {
        member_dids: seal::member_dids_of_bindings(&bindings),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let (pt_a, _) = open_membership_set_group(&seal::sec_of(&a.kp), 0, &real_ctx, &env)
        .expect("DROP-4: opening with the REAL binding member-DID ctx MUST succeed (C9)");
    assert_eq!(
        pt_a, plaintext,
        "DROP-4 (0x6610): real member-DID round-trip"
    );

    // The OLD fabricated member-DID ctx (KEM-key hashes) fails closed.
    let fabricated_member_dids: Vec<String> =
        group_roster_for_test(&[seal::pub_of(&a.kp), seal::pub_of(&b.kp)])
            .iter()
            .map(|d| String::from_utf8_lossy(d).into_owned())
            .collect();
    assert_ne!(
        real_ctx.member_dids, fabricated_member_dids,
        "precondition: real member DIDs differ from the KEM-key-hashed placeholders"
    );
    let fabricated_ctx = GroupVerifyContext {
        member_dids: fabricated_member_dids,
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    assert!(
        matches!(
            open_membership_set_group(&seal::sec_of(&a.kp), 0, &fabricated_ctx, &env),
            Err(GroupError::SenderOriginAuthFailed)
        ),
        "DROP-4 (0x6610): the KEM-key-hashed placeholder member roster MUST NOT open a \
         C9-sealed envelope. would-FAIL-on-revert: the fabricated-DID roster flips both arms."
    );
}
