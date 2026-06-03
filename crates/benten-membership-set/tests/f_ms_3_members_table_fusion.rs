//! **F-MS-3** — `members_table` fusion: one-DID-one-record.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, family **F-MS-3** (merges A3 + GNI-3).
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
//! Red-phase intent: a second record for an existing DID is structurally
//! impossible (`BTreeMap` overwrite, not a parallel table); an
//! `is_authority=true, sig_pubkey=None` entry rejects; a compile-fence proves
//! `MemberEntry` has no `member_type` field.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::member::{MemberEntry, MembersTable}` and
//! un-ignores. Would-FAIL-if-no-op'd: a design with a parallel
//! authorities-table (re-introducing the cross-table invariant), a stored
//! `member_type`, or an authority-without-pubkey all break a pin.

#![allow(dead_code)]

use std::collections::BTreeMap;

// ── self-contained in-file stub-shim ────────────────────────────────────────

/// Stand-in for `benten_id::did::Did` (a content-addressed DID string at R5).
type Did = String;

/// Stand-in for a signing public key (`Option<SigPubKey>` presence is the
/// authority discriminant).
type SigPubKey = Vec<u8>;

/// Stand-in for `benten_membership_set::member::MemberEntry`.
///
/// **ZERO nature field by construction** — `is_ai` / `is_plugin` / member_type
/// are DERIVED (Inv-22), never stored. The fields below are EXACTLY the frozen
/// member surface (R0 §4.2). A `member_type: SomeEnum` field here would be the
/// red-phase FAILURE.
#[derive(Clone, PartialEq, Eq, Debug)]
struct MemberEntry {
    role_ordinal: u8,              // RoleId ordinal (governance axis; F-MS-4)
    is_authority: bool,            // fused authorities-table membership
    sig_pubkey: Option<SigPubKey>, // present IFF is_authority (coupling rule)
    admitted_at_hlc: u64,          // member-property clock (NOT Inv-21 created_at_hlc)
    member_ref_tag: u8,            // KEYING axis tag (NOT nature)
                                   // NO `member_type` / `nature` / `is_ai` field — nature is DERIVED (Inv-22).
}

#[derive(Debug, PartialEq, Eq)]
enum MembersTableError {
    /// `is_authority=true` but `sig_pubkey` absent.
    AuthorityMissingPubkey,
}

impl MemberEntry {
    /// Production-shaped validation of the fused-record coupling rule. At R5
    /// this is enforced inside `MembersTable::admit`.
    fn validate(&self) -> Result<(), MembersTableError> {
        if self.is_authority && self.sig_pubkey.is_none() {
            return Err(MembersTableError::AuthorityMissingPubkey);
        }
        Ok(())
    }
}

/// Stand-in for the fused `members_table`. The `BTreeMap<Did, MemberEntry>`
/// key uniqueness IS the one-DID-one-record invariant — there is no parallel
/// authorities/role-assignments table to drift against.
#[derive(Default)]
struct MembersTable {
    table: BTreeMap<Did, MemberEntry>,
}

impl MembersTable {
    /// Production-shaped admit: validates the coupling rule, then upserts.
    /// Re-admitting an existing DID OVERWRITES (one record) — it never creates
    /// a second record.
    fn admit(&mut self, did: Did, entry: MemberEntry) -> Result<(), MembersTableError> {
        entry.validate()?;
        self.table.insert(did, entry);
        Ok(())
    }

    fn len(&self) -> usize {
        self.table.len()
    }

    fn get(&self, did: &Did) -> Option<&MemberEntry> {
        self.table.get(did)
    }
}

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-3 — a second record for an existing DID is structurally impossible (BTreeMap fuses to one-DID-one-record); un-ignore at R5"]
fn ms3_one_did_one_record() {
    let mut t = MembersTable::default();
    let did = "did:key:zAlice".to_string();
    let first = MemberEntry {
        role_ordinal: 2,
        is_authority: false,
        sig_pubkey: None,
        admitted_at_hlc: 100,
        member_ref_tag: 0,
    };
    let second = MemberEntry {
        role_ordinal: 4, // promoted to Admin
        is_authority: true,
        sig_pubkey: Some(vec![0xAB; 32]),
        admitted_at_hlc: 200,
        member_ref_tag: 0,
    };
    t.admit(did.clone(), first).unwrap();
    assert_eq!(t.len(), 1);
    // Re-admitting the SAME DID overwrites — there is no parallel record.
    t.admit(did.clone(), second.clone()).unwrap();
    assert_eq!(
        t.len(),
        1,
        "one DID → exactly one record; the fused BTreeMap key uniqueness IS the cross-table invariant (Inv-20 clause-i)"
    );
    // The surviving record is the latest write (CURRENT-materialization).
    assert_eq!(t.get(&did), Some(&second));
}

#[test]
#[ignore = "RED-PHASE: F-MS-3 — is_authority ⟹ sig_pubkey.is_some() coupling rule; authority-without-pubkey rejects; un-ignore at R5"]
fn ms3_authority_requires_pubkey() {
    let mut t = MembersTable::default();
    // Negative: is_authority=true with sig_pubkey=None rejects.
    let bad = MemberEntry {
        role_ordinal: 4,
        is_authority: true,
        sig_pubkey: None,
        admitted_at_hlc: 1,
        member_ref_tag: 0,
    };
    assert_eq!(
        t.admit("did:key:zBob".to_string(), bad),
        Err(MembersTableError::AuthorityMissingPubkey),
        "is_authority ⟹ sig_pubkey.is_some() (fused authorities coupling)"
    );
    assert_eq!(t.len(), 0, "the rejected entry never lands");
    // Positive: authority WITH pubkey admits.
    let good = MemberEntry {
        role_ordinal: 4,
        is_authority: true,
        sig_pubkey: Some(vec![0xCD; 32]),
        admitted_at_hlc: 1,
        member_ref_tag: 0,
    };
    assert!(t.admit("did:key:zBob".to_string(), good).is_ok());
}

#[test]
#[ignore = "RED-PHASE: F-MS-3 — MemberEntry has ZERO nature/member_type field (Inv-22 derived); compile-fence; un-ignore at R5"]
fn ms3_no_member_type_field_compile_fence() {
    // Compile-fence: this test names EXACTLY the frozen MemberEntry fields. If
    // a future `member_type` / `nature` / `is_ai` field is added to the real
    // struct, the R5 un-ignore (which constructs the real MemberEntry) will
    // either fail to compile (missing field) or this destructure will reject
    // an extra field — either way the ZERO-nature-field rule is enforced.
    let e = MemberEntry {
        role_ordinal: 2,
        is_authority: false,
        sig_pubkey: None,
        admitted_at_hlc: 7,
        member_ref_tag: 0,
    };
    // Exhaustive destructure — adding a nature field breaks this pattern.
    let MemberEntry {
        role_ordinal,
        is_authority,
        sig_pubkey,
        admitted_at_hlc,
        member_ref_tag,
    } = &e;
    // Observable consequence: the entry carries only keying/governance state,
    // never an operator-nature discriminator.
    assert_eq!(*role_ordinal, 2);
    assert!(!*is_authority);
    assert!(sig_pubkey.is_none());
    assert_eq!(*admitted_at_hlc, 7);
    assert_eq!(*member_ref_tag, 0);
}
