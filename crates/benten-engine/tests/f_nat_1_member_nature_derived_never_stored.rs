//! F-NAT-1 (R3-W6 gov-audit) — Inv-22: member NATURE is DERIVED, never
//! stored. There is NO `MemberEntry` / Policy / wire nature field;
//! `member_type` is DELETED. `is_ai_operated(did) = (did.method() ==
//! "agent")` (did:agent is an optional allowlist ALIAS, not a stored
//! discriminator); `is_plugin` / `is_autonomous_ai` are derived
//! (Inv-14 / manifest); ownership comes from `root_issuers(agent_did)`;
//! any CACHED nature is an IVM-materialized view — never authoritative.
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-NAT-1; merges
//! K4 + GNI-11/12/13):
//!   - R0.5 plan Inv-22, §1.3, §3.7, §9.1-6.
//!   - Clone-shape: `crates/benten-core/tests/attribution_mirror.rs`
//!     (struct-fence) + `crates/benten-caps/tests/
//!     cap_r1_1_audience_binding_grep_defense.rs` (member_type / MemberKind
//!     grep-defense) + `crates/benten-ivm/tests/view2_event_dispatch.rs`
//!     (IVM-view recompute, nature is a view not a stored field).
//!
//! ## F4-042 — CANONICAL MemberEntry SHAPE
//!
//! The prior R3 stub carried a 3-field `MemberEntry { did, is_authority,
//! sig_pubkey }` — a 3rd divergent shape vs F-AAD-1 / F-MS-3. It is ALIGNED
//! here to the canonical R0.5 §3.5 / §4.2 **5-field** shape
//! `MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc,
//! member_ref }` (matching `f_aad_1_members_table_canonical_cbor_length_
//! injective.rs`). The DID is the `BTreeMap<Did, MemberEntry>` KEY, not a
//! `MemberEntry` field. Crucially, the canonical 5-field shape carries ZERO
//! nature field — so the Inv-22 struct-fence is preserved by-construction.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 nature-derivation surface (`is_ai_operated` / `MemberEntry` /
//! `derive_member_nature` IVM view) does not exist at this SHA.
//! Self-contained stub-shim compiles green; nature-derivation bodies
//! `unimplemented!()`. W6 R5 implementer:
//!   1. DELETE `mset_w6_nature_stub`,
//!   2. INSERT `use benten_membership_set::member::{MemberEntry, MemberRef,
//!      RoleId, Hlc, SigPubKey, is_ai_operated, derive_member_nature};`,
//!   3. UN-IGNORE,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Arms: (1) `MemberEntry` has ZERO nature field (struct-fence — a
//! nature-setter is absent; the canonical 5-field shape carries none); (2)
//! `is_ai_operated` derives PURELY from the DID method-parse (`did:agent:`
//! → true; `did:key:` → false); (3) the derived nature is IVM-recomputed
//! from the event chain, never read from a stored authoritative field
//! (recompute yields the same answer with no stored field present); (4)
//! GREP-DEFENSE — `member_type` / `MemberKind` / nature-field source scan
//! == 0.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::path::Path;

// Real DID surface — used to anchor the method-parse arm to the live
// `benten_id::did::Did` type at baseline (the W6 `is_ai_operated` derives
// from exactly this method string).
use benten_id::did::Did;

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::member` surface —
// the canonical 5-field `MemberEntry` (ZERO nature field; Inv-22) + the
// DERIVED `MemberNature` IVM view (`is_ai_operated` / `derive_member_nature`).
// =====================================================================
use benten_membership_set::member::{
    Hlc, MemberEntry, MemberNature, MemberRef, RoleId, SigPubKey, derive_member_nature,
    is_ai_operated,
};

/// F-NAT-1 (a): `MemberEntry` carries ZERO nature field (struct-fence).
/// The nature-storage path does not exist — `has_any_nature_field()` is
/// `false`. The canonical 5-field shape `{role, is_authority, sig_pubkey,
/// admitted_at_hlc, member_ref}` demonstrably has no nature field.
#[test]
fn member_entry_has_zero_nature_field() {
    // Construct a canonical 5-field entry — proving the shape compiles with
    // NO nature field present (a nature field would be a 6th field here).
    let entry = MemberEntry {
        role: RoleId::Member,
        is_authority: false,
        sig_pubkey: None,
        admitted_at_hlc: Hlc {
            physical_ms: 1_900_000_000_000,
            logical: 0,
            node_id: 1,
        },
        member_ref: MemberRef::UserDid,
    };
    // Read a field so the 5-field literal is actually USED (not a no-effect
    // underscore binding) — the literal compiling with exactly these 5 fields
    // is the structural-shape pin (a 6th `nature`/`member_type` field would
    // fail compilation here).
    assert_eq!(
        entry.role,
        RoleId::Member,
        "F-NAT-1: the canonical 5-field MemberEntry literal compiles + reads back."
    );
    assert!(
        !MemberEntry::has_any_nature_field(),
        "F-NAT-1 (Inv-22): MemberEntry MUST carry ZERO nature field — \
         `member_type` is DELETED; nature is DERIVED, never stored. \
         would-FAIL if W6 reintroduces a stored nature discriminator"
    );
}

/// F-NAT-1 (b): `is_ai_operated` derives PURELY from the DID method-parse —
/// `did:agent:` → true; `did:key:` → false. Drives the derivation against
/// the REAL `benten_id::did::Did` method string so the parse is anchored to
/// the live DID type.
#[test]
fn is_ai_operated_derives_from_did_method_parse_only() {
    // The real DID type — the method string the W6 derivation consumes.
    let agent_did = Did::from_string_for_test_fixture("did:agent:example-agent-001".to_string());
    let key_did = Did::from_string_for_test_fixture("did:key:z6MkExample".to_string());

    let agent_method = did_method_of(agent_did.as_str());
    let key_method = did_method_of(key_did.as_str());
    assert_eq!(
        agent_method, "agent",
        "fixture sanity: did:agent: method-parse"
    );
    assert_eq!(key_method, "key", "fixture sanity: did:key: method-parse");

    assert!(
        is_ai_operated(&agent_method),
        "F-NAT-1: `is_ai_operated` MUST be TRUE for a did:agent: DID — it \
         derives from `did.method() == \"agent\"` (an allowlist alias, NOT a \
         stored field)"
    );
    assert!(
        !is_ai_operated(&key_method),
        "F-NAT-1: `is_ai_operated` MUST be FALSE for a did:key: DID — nature \
         is method-derived, not stored; would-FAIL if W6 keys nature off a \
         stored discriminator instead of the method-parse"
    );
}

/// F-NAT-1 (c): the derived nature is an IVM-materialized view — recomputed
/// from the (did-method, manifest) inputs, never read from a stored
/// authoritative field. Two recomputations from identical inputs agree
/// (deterministic derivation), and `is_plugin` / `is_autonomous_ai` follow
/// from the manifest/method, not a stored flag.
#[test]
fn member_nature_is_ivm_recomputed_never_authoritative_stored() {
    let nature_a: MemberNature =
        derive_member_nature("agent", /* has_install_manifest */ true);
    let nature_b: MemberNature = derive_member_nature("agent", true);
    assert_eq!(
        nature_a, nature_b,
        "F-NAT-1: nature derivation MUST be deterministic — recomputing the \
         IVM view from identical inputs yields the identical nature (a view, \
         not a mutable stored field)"
    );
    assert!(
        nature_a.is_ai_operated,
        "F-NAT-1: a did:agent: member's derived nature MUST report \
         is_ai_operated"
    );
    assert!(
        nature_a.is_plugin,
        "F-NAT-1: with an install manifest present, `is_plugin` MUST be \
         DERIVED true (Inv-14 / manifest), not read from a stored flag"
    );

    // A did:key: member with no manifest derives a fully-natural nature.
    let human: MemberNature = derive_member_nature("key", false);
    assert!(
        !human.is_ai_operated && !human.is_plugin && !human.is_autonomous_ai,
        "F-NAT-1: a did:key: member with no manifest MUST derive all-false \
         nature flags (nothing is stored; the absence of method=agent + \
         manifest IS the derivation)"
    );
}

/// F-NAT-1 (d): GREP-DEFENSE — the W6 membership-set source MUST NOT
/// declare a `member_type` / `MemberKind` nature discriminator. Clone of
/// the `cap_r1_1_audience_binding_grep_defense` source scan. `#[test]`
/// (green now): vacuously zero until the crate lands, then load-bearing.
#[test]
fn grep_defense_no_member_type_or_member_kind_nature_field_in_membership_set_src() {
    let candidate_src_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("benten-membership-set")
        .join("src");

    if !candidate_src_dir.exists() {
        return;
    }

    let mut offenders: Vec<String> = Vec::new();
    visit_rs_files(&candidate_src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            let l = line.trim_start();
            // Flag a struct field or enum named for a stored member-nature
            // discriminator. We match the field/type tokens that the
            // Inv-22 deletion forbids: `member_type` field, `MemberKind`
            // nature enum, `member_nature:` stored field.
            let is_member_type_field = l.starts_with("member_type") || l.contains("member_type:");
            let is_member_kind_type = l.contains("enum MemberKind") || l.contains("MemberKind {");
            let is_stored_nature_field = l.starts_with("member_nature:")
                || (l.contains("nature:") && l.contains("MemberNature"));
            if is_member_type_field || is_member_kind_type || is_stored_nature_field {
                offenders.push(format!("{}:{}: {}", path.display(), lineno + 1, l.trim()));
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "F-NAT-1 grep-defense (Inv-22): member nature MUST be DERIVED, never \
         stored — no `member_type` field, no `MemberKind` nature enum, no \
         stored `member_nature` field. Found offending declaration(s):\n{}",
        offenders.join("\n")
    );
}

/// Parse the method segment of a DID string (`did:<method>:<id>`).
fn did_method_of(did: &str) -> String {
    did.split(':').nth(1).unwrap_or("").to_string()
}

fn visit_rs_files(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
