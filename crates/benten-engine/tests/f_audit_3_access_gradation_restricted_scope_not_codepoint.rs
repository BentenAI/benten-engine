//! F-AUDIT-3 (R3-W6 gov-audit) — `AuditAccessGradation` is a UCAN
//! read-scope on `audit:<set_id>:*`, expressed as a `RestrictedScope`
//! arm (m-15 GNC-1) NOT a codepoint. `AdminOnly` = only Admins hold the
//! read cap; `PublicAllMembers` = all members. The 4 reserved variants
//! (MemberOnly / Threshold / TimeLocked / Anonymized) are UCAN-caveat /
//! IVM compositions, NOT codepoints.
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-AUDIT-3; merges
//! K2 + T-G3 + GNI-16):
//!   - R0.5 plan §2.5 BC-8, §3.8 GNC-1, M-17, §4.2; Compromise #58.
//!   - Clone-shape: `crates/benten-ivm/tests/view1_capability_grants.rs`
//!     (cap-grant read-view) + `crates/benten-caps/tests/
//!     tf3b_restricted_spec_contains_decidable.rs` (RestrictedScope
//!     decidable-contains) + grep-defense shape from
//!     `crates/benten-caps/tests/cap_r1_1_audience_binding_grep_defense.rs`.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 `AuditAccessGradation` + the `audit:<set_id>:*` RestrictedScope
//! parse path do not exist at this SHA. Self-contained stub-shim compiles
//! green; bodies `unimplemented!()`. W6 R5 implementer:
//!   1. DELETE `mset_w6_audit_gradation_stub`,
//!   2. INSERT `use benten_membership_set::audit::{AuditAccessGradation,
//!      parse_audit_scope};` + `use benten_caps::RestrictedScope;`,
//!   3. UN-IGNORE,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Arms: (1) AdminOnly + non-Admin → denied; (2) PublicAllMembers +
//! member → admitted; (3) `audit:<set_id>:*` parses through the EXISTING
//! `RestrictedScope` arm WITHOUT adding a 3rd top-level `Scope` arm
//! (struct/grep fence — `Scope` stays EXACTLY 2 arms); (4) the 4 reserved
//! variants resolve to UCAN-caveat/IVM compositions, NOT codepoints
//! (no-audit-gradation-codepoint grep-defense source scan == 0).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::path::Path;

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::audit` surface.
// The `audit:<set_id>:*` scope routes through the EXISTING
// `benten_caps::RestrictedScope` arm (m-15 GNC-1) — `Scope` stays EXACTLY 2
// arms (no 3rd top-level arm).
// =====================================================================
use benten_membership_set::audit::{
    AuditAccessGradation, AuditReadDecision, RequesterRole,
    parse_audit_scope_added_new_top_level_scope_arm, restricted_audit_scope_contains,
};

/// F-AUDIT-3 (a): `AdminOnly` denies a non-Admin member's audit read.
#[test]
fn admin_only_gradation_denies_non_admin_audit_read() {
    let grad = AuditAccessGradation::AdminOnly;
    assert_eq!(
        grad.decide(RequesterRole::Member),
        AuditReadDecision::Deny,
        "F-AUDIT-3: AdminOnly MUST deny a non-Admin member's audit read"
    );
    assert_eq!(
        grad.decide(RequesterRole::Admin),
        AuditReadDecision::Admit,
        "F-AUDIT-3: AdminOnly MUST admit an Admin's audit read (positive control)"
    );
}

/// F-AUDIT-3 (b): `PublicAllMembers` admits any member's audit read but
/// still denies a non-member.
#[test]
fn public_all_members_gradation_admits_member_denies_non_member() {
    let grad = AuditAccessGradation::PublicAllMembers;
    assert_eq!(
        grad.decide(RequesterRole::Member),
        AuditReadDecision::Admit,
        "F-AUDIT-3: PublicAllMembers MUST admit any member's audit read"
    );
    assert_eq!(
        grad.decide(RequesterRole::NonMember),
        AuditReadDecision::Deny,
        "F-AUDIT-3: PublicAllMembers is still set-scoped — a NON-member MUST \
         be denied (the gradation governs members, not the public internet)"
    );
}

/// F-AUDIT-3 (c): `audit:<set_id>:*` parses through the EXISTING
/// `RestrictedScope` arm WITHOUT adding a 3rd top-level `Scope` variant,
/// and is decidably-CONTAINS over concrete requests.
#[test]
fn audit_scope_parses_via_existing_restricted_scope_arm_no_new_top_level_arm() {
    let scope = "audit:set-0x51:*";
    assert!(
        !parse_audit_scope_added_new_top_level_scope_arm(scope),
        "F-AUDIT-3: `audit:<set_id>:*` MUST parse through the EXISTING \
         benten_caps::RestrictedScope arm (m-15 GNC-1) — it MUST NOT add a \
         3rd top-level `Scope` variant (Scope stays EXACTLY 2 arms)"
    );
    assert!(
        restricted_audit_scope_contains(scope, "audit:set-0x51:read:event-0xAB"),
        "F-AUDIT-3: the parsed audit RestrictedScope MUST decidably CONTAIN a \
         concrete in-scope read request (decidable-contains, like \
         tf3b_restricted_spec_contains_decidable)"
    );
    assert!(
        !restricted_audit_scope_contains(scope, "audit:set-0xFF:read:event-0xAB"),
        "F-AUDIT-3: the audit RestrictedScope for set-0x51 MUST NOT contain a \
         request scoped to a DIFFERENT set_id (set-scoped containment)"
    );
}

/// F-AUDIT-3 (d): NONE of the gradation variants — including the 4
/// reserved ones — is backed by a wire codepoint. The reserved variants
/// resolve to UCAN-caveat / IVM compositions (no-audit-gradation-codepoint
/// grep-defense: the gradation is a scope, never a codepoint).
#[test]
fn no_gradation_variant_is_backed_by_a_wire_codepoint() {
    let all = [
        AuditAccessGradation::AdminOnly,
        AuditAccessGradation::PublicAllMembers,
        AuditAccessGradation::MemberOnly,
        AuditAccessGradation::Threshold,
        AuditAccessGradation::TimeLocked,
        AuditAccessGradation::Anonymized,
    ];
    for grad in all {
        assert!(
            grad.backing_codepoint().is_none(),
            "F-AUDIT-3: AuditAccessGradation::{grad:?} MUST NOT be backed by a \
             wire codepoint — gradation is a RestrictedScope arm, never a \
             §4.0 codepoint (F-FREEZE-1 confirms the −4 codepoint shrink)"
        );
    }
    // The 4 reserved variants specifically resolve to caveat/IVM compositions.
    for reserved in [
        AuditAccessGradation::MemberOnly,
        AuditAccessGradation::Threshold,
        AuditAccessGradation::TimeLocked,
        AuditAccessGradation::Anonymized,
    ] {
        assert!(
            reserved.is_reserved_caveat_composition(),
            "F-AUDIT-3: the reserved variant {reserved:?} MUST resolve to a \
             UCAN-caveat / IVM composition (M-17), NOT a codepoint"
        );
    }
}

/// F-AUDIT-3 (d'): GREP-DEFENSE — the W6 membership-set source MUST NOT
/// declare an `AUDIT_GRADATION`-named codepoint constant. Clone of the
/// `cap_r1_1_audience_binding_grep_defense` source-scan shape. This arm is
/// `#[test]` (green now): at baseline the membership-set crate does not yet
/// exist so the scan trivially finds zero — it becomes load-bearing once
/// the crate lands (it would FAIL if a future edit mints a gradation
/// codepoint const). Kept un-ignored so the grep-defense is always live.
#[test]
fn grep_defense_no_audit_gradation_codepoint_constant_in_membership_set_src() {
    let candidate_src_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("benten-membership-set")
        .join("src");

    // At baseline the crate may not exist yet (W4 canary mints it). If
    // absent, the scan is vacuously zero; once present, it is load-bearing.
    if !candidate_src_dir.exists() {
        return;
    }

    let mut offenders: Vec<String> = Vec::new();
    visit_rs_files(&candidate_src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            // A codepoint constant for a gradation would look like a
            // `const AUDIT_*_GRADATION: u16 = 0x...;` or a `Codepoint`-typed
            // gradation binding. Flag any line that pairs an audit-gradation
            // token with a codepoint/u16 hex literal.
            let l = line;
            let mentions_gradation_codepoint = l.contains("AUDIT")
                && l.contains("GRADATION")
                && (l.contains("0x6") || l.contains(": u16") || l.contains("Codepoint"));
            if mentions_gradation_codepoint {
                offenders.push(format!("{}:{}: {}", path.display(), lineno + 1, l.trim()));
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "F-AUDIT-3 grep-defense: AuditAccessGradation MUST be a RestrictedScope \
         arm, NEVER a wire codepoint. Found gradation-codepoint declaration(s):\n{}",
        offenders.join("\n")
    );
}

fn visit_rs_files(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                f(&path, &contents);
            }
        }
    }
}
