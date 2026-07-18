//! GAP-KDB Shape-B — DROP-9 (TIER-A): group `audience_set_commitment`
//! real-DID golden + sort-order-independence (C9 wire bytes). W2 benten-drop.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 / C9 (the commitment roster is
//! the REAL recipient DIDs) + `R2-LANDSCAPE` DROP-9 (TIER-A freeze-perfect
//! golden). `layer_c::audience_set_commitment` sorts + length-prefixes the
//! roster internally (`BLAKE3(0x01 ‖ lp_u32(did) …)`).
//!
//! # M-20 golden discipline
//! The golden below is captured via a THROWAWAY run of the REAL
//! `layer_c::audience_set_commitment` over three DETERMINISTIC `did:benten`
//! member DIDs — never hand-authored. A hand-authored golden matching a buggy
//! encoder freezes the bug (TIER-A reviewer law).
//!
//! # Live freeze-guard + would_fail_on_revert
//! The golden arm is LIVE (guards the frozen C9 commitment wire over real
//! `did:benten` identities): any drift in the commitment construction (domain
//! prefix, `u32-BE` length framing, sort order, digest) flips the golden. The
//! sort-order-independence arm pins that permuting the roster yields the SAME
//! commitment (the internal sort). The ignored arm ties the C9 seal to this
//! exact commitment.
//!
//! # R5 un-ignore (seal-binds-commitment arm only)
//! Mint the binding-typed group seal; drop `#[ignore]` on the seal arm. The
//! golden + sort-independence arms are live now.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::open_group_stanza;
use benten_id::did::Did;
use benten_id::kdb_testing::{self as kdb, RecipientBinding};

/// The frozen `audience_set_commitment` over the three deterministic
/// `did:benten` member DIDs `det_member_did("drop9/member/{a,b,c}")`.
/// Captured via M-20 throwaway from the real `layer_c::audience_set_commitment`.
const DROP9_COMMIT_GOLDEN: &str =
    "b2cf884a13e7a0e4df9413238df1e405ab559800ab6bc35ba0d221d4a86819f3";

/// A fully-deterministic `did:benten` member DID (stable across runs: BLAKE3-XOF
/// component fills + BLAKE3 CID + bs58 assembly). The KEM key is opaque
/// deterministic bytes (the commitment binds the DID string, not the KEM key).
fn det_member_did(seed: &str) -> Did {
    let sig_mk = kdb::det_signing_multikey(seed);
    let doc = kdb::KeySetDocument::v1_hybrid(
        sig_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub(&format!("{seed}/kx")),
            &kdb::det_mlkem768_ek(&format!("{seed}/kek")),
        ),
    );
    let payload = kdb::did_benten_payload(&sig_mk, &doc.cid());
    kdb::did_benten_from_payload_for_test(&payload)
}

fn det_members() -> Vec<Vec<u8>> {
    ["drop9/member/a", "drop9/member/b", "drop9/member/c"]
        .iter()
        .map(|s| det_member_did(s).as_str().as_bytes().to_vec())
        .collect()
}

/// DROP-9 (TIER-A, LIVE) — the group commitment over the real `did:benten`
/// member DIDs equals the frozen golden (frozen C9 wire bytes).
#[test]
fn drop9_group_commitment_real_did_golden() {
    let members = det_members();
    let commit = seal::audience_set_commitment(&members);
    assert_eq!(
        kdb::golden_hex(&commit),
        DROP9_COMMIT_GOLDEN,
        "DROP-9 (TIER-A): the audience_set_commitment over the real did:benten member DIDs MUST \
         equal the frozen golden. would-FAIL-on-revert: any drift in the domain prefix / u32-BE \
         length framing / sort order / digest flips it — the C9 commitment wire is frozen."
    );
}

/// DROP-9 (LIVE) — sort-order-independence: permuting the roster yields the
/// SAME commitment (the commitment sorts the DID list internally, so wire order
/// cannot change identity-set membership).
#[test]
fn drop9_group_commitment_sort_order_independent() {
    let members = det_members();
    let mut reversed = members.clone();
    reversed.reverse();
    assert_ne!(
        members, reversed,
        "precondition: the reversed roster is a different order"
    );
    assert_eq!(
        seal::audience_set_commitment(&members),
        seal::audience_set_commitment(&reversed),
        "DROP-9: audience_set_commitment MUST be sort-order-independent (canonical sorted roster)"
    );
}

/// DROP-9 (ignored → R5) — the C9 binding-typed group seal binds EXACTLY this
/// commitment over the real binding DIDs: an honest member recomputes it from
/// the real-DID roster (in ANY order) and opens.
#[test]
#[ignore = "RED-PHASE: DROP-9 C9 group seal binds the real-DID commitment (any order) — un-ignore at R5"]
fn drop9_c9_seal_binds_real_did_commitment_order_independent() {
    let a = seal::real_recipient();
    let b = seal::real_recipient();
    let bindings = vec![
        RecipientBinding::resolve(&a.did, &a.doc).expect("A binds"),
        RecipientBinding::resolve(&b.did, &b.doc).expect("B binds"),
    ];
    let (sk, sender) = seal::hybrid_did_key_sender();
    let plaintext = b"drop-9 c9 commitment payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_group_to_bindings(&bindings, &sender, &sk, &body_cid, 0, &plaintext)
        .expect("group seal");

    // The recipient recomputes the commitment from its independently-held
    // real-DID roster (F-2). Order-independence: the SORTED commitment matches
    // regardless of the roster order the recipient holds.
    let mut roster = seal::roster_of_bindings(&bindings);
    roster.reverse();
    let (pt_a, _) = open_group_stanza(&seal::sec_of(&a.kp), 0, &roster, 0, &env).expect(
        "DROP-9: the C9 seal binds the SORTED real-DID commitment; a reversed held roster \
         still opens (sort-order-independent)",
    );
    assert_eq!(
        pt_a, plaintext,
        "DROP-9: C9 seal round-trip via real-DID commitment"
    );
}
