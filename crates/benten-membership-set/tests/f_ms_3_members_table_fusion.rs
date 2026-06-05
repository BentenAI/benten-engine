//! **F-MS-3** — `members_table` fusion: one-DID-one-record.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! family **F-MS-3** (merges A3 + GNI-3).
//!
//! # What this pins (r2-test-landscape §1 Group 9 / R0 §3.5 + §4.2 +
//! Inv-20 clause-i + Inv-22)
//!
//! `members_table: BTreeMap<Did, MemberEntry>` **fuses** members + authorities
//! + role-assignments by construction: one DID → exactly one record (the
//! `BTreeMap` key uniqueness IS the cross-table invariant). Coupling rule:
//! `is_authority ⟹ sig_pubkey.is_some()`. There is **ZERO** `member_type` /
//! nature field on `MemberEntry` (nature is DERIVED — Inv-22). The snapshot is
//! the CURRENT-materialization of the membership event version-chain.
//!
//! # R5 (un-ignored against `benten_membership_set::member`)
//!
//! Drives the real `MembersTable::admit` (validates coupling + upserts) +
//! `MemberEntry` (5-field canonical R0.5 §3.5 shape). The ZERO-nature-field
//! rule is enforced by the exhaustive destructure compile-fence. Would-FAIL:
//! a parallel authorities-table, a stored `member_type`, or an
//! authority-without-pubkey all break a pin.

use benten_core::hlc::BentenHlc;
use benten_id::did::Did;
use benten_membership_set::member::{
    MemberEntry, MemberRef, MembersTable, MembersTableError, SigPubKey,
};
use benten_membership_set::role::RoleId;

fn did(s: &str) -> Did {
    Did::from_string_for_test_fixture(s.to_string())
}

/// The canonical 3-field HLC for a given physical clock (logical=0, node_id=0).
fn hlc(physical_ms: u64) -> BentenHlc {
    BentenHlc::new(physical_ms, 0, 0)
}

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
fn ms3_one_did_one_record() {
    let mut t = MembersTable::new();
    let d = did("did:key:zAlice");
    let first = MemberEntry {
        role: RoleId::Member,
        is_authority: false,
        sig_pubkey: None,
        admitted_at_hlc: hlc(100),
        member_ref: MemberRef::UserDid,
    };
    let second = MemberEntry {
        role: RoleId::Admin, // promoted to Admin
        is_authority: true,
        sig_pubkey: Some(SigPubKey(vec![0xAB; 32])),
        admitted_at_hlc: hlc(200),
        member_ref: MemberRef::UserDid,
    };
    t.admit(d.clone(), first).unwrap();
    assert_eq!(t.len(), 1);
    // Re-admitting the SAME DID overwrites — there is no parallel record.
    t.admit(d.clone(), second.clone()).unwrap();
    assert_eq!(
        t.len(),
        1,
        "one DID → exactly one record; the fused BTreeMap key uniqueness IS the cross-table invariant (Inv-20 clause-i)"
    );
    // The surviving record is the latest write (CURRENT-materialization).
    assert_eq!(t.get(&d), Some(&second));
}

#[test]
fn ms3_authority_requires_pubkey() {
    let mut t = MembersTable::new();
    // Negative: is_authority=true with sig_pubkey=None rejects.
    let bad = MemberEntry {
        role: RoleId::Admin,
        is_authority: true,
        sig_pubkey: None,
        admitted_at_hlc: hlc(1),
        member_ref: MemberRef::UserDid,
    };
    assert_eq!(
        t.admit(did("did:key:zBob"), bad),
        Err(MembersTableError::AuthorityMissingPubkey),
        "is_authority ⟹ sig_pubkey.is_some() (fused authorities coupling)"
    );
    assert_eq!(t.len(), 0, "the rejected entry never lands");
    // Positive: authority WITH pubkey admits.
    let good = MemberEntry {
        role: RoleId::Admin,
        is_authority: true,
        sig_pubkey: Some(SigPubKey(vec![0xCD; 32])),
        admitted_at_hlc: hlc(1),
        member_ref: MemberRef::UserDid,
    };
    assert!(t.admit(did("did:key:zBob"), good).is_ok());
}

#[test]
fn ms3_no_member_type_field_compile_fence() {
    // Compile-fence: this test names EXACTLY the frozen canonical R0.5 §3.5
    // MemberEntry fields. If a future `member_type` / `nature` / `is_ai` field
    // is added to the real struct, the exhaustive destructure below rejects an
    // extra field — enforcing the ZERO-nature-field rule.
    let e = MemberEntry {
        role: RoleId::Member,
        is_authority: false,
        sig_pubkey: None,
        admitted_at_hlc: hlc(7),
        member_ref: MemberRef::UserDid,
    };
    // Exhaustive destructure — adding a nature field breaks this pattern.
    let MemberEntry {
        role,
        is_authority,
        sig_pubkey,
        admitted_at_hlc,
        member_ref,
    } = &e;
    // Observable consequence: the entry carries only keying/governance state,
    // never an operator-nature discriminator.
    assert_eq!(*role, RoleId::Member);
    assert!(!*is_authority);
    assert!(sig_pubkey.is_none());
    assert_eq!(*admitted_at_hlc, hlc(7));
    assert_eq!(*member_ref, MemberRef::UserDid);
}
