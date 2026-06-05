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
//! ## R4-FIX (F4-006 BLOCKER): canonical `MemberEntry` shape reconciliation
//!
//! This file's original `MemberEntry` used a FLATTENED, byte-incompatible
//! shape (`role_ordinal: u8`, `admitted_at_hlc: u64`, `member_ref_tag: u8`)
//! that diverged from F-AAD-1's structured snapshot — two stubs for the SAME
//! frozen type that would serialize to DIFFERENT canonical bytes (a divergent
//! AAD = cross-engine decrypt failure). The fix converges BOTH to the ONE
//! canonical R0.5 §3.5 5-field shape:
//! `MemberEntry { role: RoleId, is_authority: bool, sig_pubkey:
//! Option<SigPubKey>, admitted_at_hlc: Hlc, member_ref: MemberRef }`
//! with the 3-field `Hlc`. The fusion semantics this file pins
//! (one-DID-one-record, the `is_authority ⟹ sig_pubkey` coupling, ZERO
//! nature field) are unchanged — only the field TYPES are reconciled so the
//! two stubs name the same frozen surface.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::member::{MemberEntry, MembersTable}` and
//! un-ignores. Would-FAIL-if-no-op'd: a design with a parallel
//! authorities-table (re-introducing the cross-table invariant), a stored
//! `member_type`, or an authority-without-pubkey all break a pin.

#![allow(dead_code)]

// ── R5: the real crate surface (the in-file stub-shim is deleted) ───────────
//
// The canonical R0.5 §3.5 shapes — `Did` / `Hlc` / `SigPubKey` / `RoleId` /
// `MemberRef` / `MemberEntry` / `MembersTable` — are now the real
// `benten_membership_set::member` surface (byte-compatible with the F-AAD-1
// frozen golden, verified M-20). `MembersTable::admit` is the REAL production
// entry point; the coupling-rule rejection surfaces `MembershipSetError`,
// mirrored 1:1 to a local `MembersTableError` so the pin bodies stay intact.
use benten_membership_set::MembershipSetError;
use benten_membership_set::member::{Did, Hlc, MemberEntry, MemberRef, MembersTable, SigPubKey};
use benten_membership_set::role::RoleId;

/// 1:1 mirror of the fused-record coupling-rule rejection — kept local so the
/// pin bodies compare a named error variant while `admit()` drives the real
/// `MembersTable::admit`.
#[derive(Debug, PartialEq, Eq)]
enum MembersTableError {
    /// `is_authority=true` but `sig_pubkey` absent.
    AuthorityMissingPubkey,
}

impl From<MembershipSetError> for MembersTableError {
    fn from(e: MembershipSetError) -> Self {
        match e {
            MembershipSetError::AuthorityMissingPubkey => MembersTableError::AuthorityMissingPubkey,
            other => panic!("unexpected members-table error in F-MS-3: {other:?}"),
        }
    }
}

/// Drive the REAL `MembersTable::admit`, mapping the production
/// `MembershipSetError` to the local `MembersTableError`.
fn admit(t: &mut MembersTable, did: Did, entry: MemberEntry) -> Result<(), MembersTableError> {
    t.admit(did, entry).map_err(MembersTableError::from)
}

/// Test helper: the canonical 3-field `Hlc` for a given physical clock.
fn hlc(physical_ms: u64) -> Hlc {
    Hlc::at(physical_ms)
}

/// Test helper: a `Did` from a `&str`.
fn did(s: &str) -> Did {
    Did::new(s)
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
    admit(&mut t, d.clone(), first).unwrap();
    assert_eq!(t.len(), 1);
    // Re-admitting the SAME DID overwrites — there is no parallel record.
    admit(&mut t, d.clone(), second.clone()).unwrap();
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
        admit(&mut t, did("did:key:zBob"), bad),
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
    assert!(admit(&mut t, did("did:key:zBob"), good).is_ok());
}

#[test]
fn ms3_no_member_type_field_compile_fence() {
    // Compile-fence: this test names EXACTLY the frozen canonical R0.5 §3.5
    // MemberEntry fields. If a future `member_type` / `nature` / `is_ai` field
    // is added to the real struct, this destructure rejects the extra field —
    // either way the ZERO-nature-field rule is enforced.
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
