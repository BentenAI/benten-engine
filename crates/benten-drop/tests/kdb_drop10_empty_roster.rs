//! GAP-KDB Shape-B — DROP-10: group-seal empty `&[RecipientBinding]`
//! degenerate-cardinality reject. W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (group paths take
//! `&[RecipientBinding]`) + `R2-LANDSCAPE` DROP-10. The band already caps the
//! UPPER cardinality (`validate_group_roster_len` ≤ `MAX_LAYER_C_GROUP_
//! RECIPIENTS`); C9's single-slice roster adds the missing LOWER bound.
//!
//! # What this pins
//! An empty binding roster is a degenerate send (a group envelope with zero
//! recipients — no one can open it, and the `audience_set_commitment` over an
//! empty roster is a fixed constant that leaks nothing but binds nothing). Both
//! group seals MUST typed-reject an empty `&[RecipientBinding]`, never emit a
//! zero-stanza envelope.
//!
//! # would_fail_on_revert
//! Without the non-empty check, `seal_group_to_bindings(&[])` /
//! `seal_membership_set_to_bindings(&[])` return `Ok(<0-stanza envelope>)` and
//! the `is_err` asserts flip to `Ok`.
//!
//! # R5 un-ignore
//! Add the non-empty roster check to the binding-typed group seals; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::group_posture::GroupSealParams;
use benten_id::kdb_testing::RecipientBinding;

/// DROP-10 (`0x6520`) — an empty group roster is a typed reject.
#[test]
#[ignore = "RED-PHASE: DROP-10 0x6520 empty binding roster typed-reject — un-ignore at R5"]
fn drop10_group_0x6520_empty_roster_rejects() {
    let (sk, sender) = seal::hybrid_did_key_sender();
    let empty: Vec<RecipientBinding> = Vec::new();
    let plaintext = b"drop-10 empty group payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let result = seal::seal_group_to_bindings(&empty, &sender, &sk, &body_cid, 0, &plaintext);
    assert!(
        result.is_err(),
        "DROP-10 (0x6520): an empty &[RecipientBinding] MUST typed-reject, never emit a \
         zero-stanza envelope. would-FAIL-on-revert: the missing non-empty check returns \
         Ok(<0-stanza envelope>)."
    );
}

/// DROP-10 (`0x6610`) — an empty MembershipSet roster is a typed reject.
#[test]
#[ignore = "RED-PHASE: DROP-10 0x6610 empty binding roster typed-reject — un-ignore at R5"]
fn drop10_membership_set_0x6610_empty_roster_rejects() {
    let (sk, sender) = seal::hybrid_did_key_sender();
    let empty: Vec<RecipientBinding> = Vec::new();
    let k_set = [0x77u8; 32];
    let params = GroupSealParams {
        membership_set_id: b"benten:set:drop10-empty".to_vec(),
        member_key_generation: 1,
        membership_set_generation: 1,
        role_assignments_generation: 1,
    };
    let plaintext = b"drop-10 empty membership payload".to_vec();

    let result =
        seal::seal_membership_set_to_bindings(&empty, &sender, &sk, &k_set, &params, &plaintext);
    assert!(
        result.is_err(),
        "DROP-10 (0x6610): an empty &[RecipientBinding] MUST typed-reject, never emit a \
         zero-stanza envelope."
    );
}
