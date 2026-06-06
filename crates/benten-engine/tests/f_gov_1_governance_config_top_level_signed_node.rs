//! F-GOV-1 (R3-W6 gov-audit) — `GovernanceConfig{tier: Flat | Moderated |
//! Polycentric}` is a TOP-LEVEL signed Node (InstallRecord precedent
//! `PLUGIN-MANIFEST.md:68`), NEVER a field inside the sealed
//! `MembershipSetPolicy`. Tier promotion = add a Node + grants + roles —
//! NO re-key, NO new identity, NO Kind change. Garden / Grove are
//! signed-Node CONTENT, NOT sub-codepoints.
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-GOV-1; merges
//! K3 + GNI-18):
//!   - R0.5 plan §2.6, §3.6.B, §4.2, §9.1-6.
//!   - InstallRecord top-level-signed-Node precedent
//!     (`docs/PLUGIN-MANIFEST.md`); grep-defense shape from
//!     `crates/benten-caps/tests/cap_r1_1_audience_binding_grep_defense.rs`.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 `GovernanceConfig` Node + tier-promotion path do not exist at
//! this SHA. Self-contained stub-shim compiles green; bodies
//! `unimplemented!()`. W6 R5 implementer:
//!   1. DELETE `mset_w6_governance_stub`,
//!   2. INSERT `use benten_membership_set::governance::{GovernanceConfig,
//!      GovernanceTier, promote_tier};`,
//!   3. UN-IGNORE,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Arms: (1) GovernanceConfig is a top-level SIGNED Node (signature over
//! canonical bytes verifies; tamper rejects); (2) tier promotion
//! Atrium→Garden leaves Kind unchanged + K_Set unchanged + adds a new Node
//! (NO re-key / NO new identity / NO Kind change); (3) GovernanceConfig is
//! NOT a `MembershipSetPolicy` field (struct-fence); (4) Garden/Grove are
//! signed-Node content, NOT sub-codepoints (grep-defense).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::path::Path;

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::governance` surface.
// =====================================================================
use benten_membership_set::governance::{
    GovernanceConfig, GovernanceTier, MembershipSetPolicy, PromotionEffect, promote_tier,
};

/// F-GOV-1 (a): the GovernanceConfig is a top-level SIGNED Node — its
/// signature over the canonical bytes verifies, and tampering the signed
/// tier field breaks the signature (tamper-evidence).
#[test]
fn governance_config_is_a_top_level_signed_node() {
    let cfg = GovernanceConfig::new_signed(GovernanceTier::Moderated, "Garden");
    assert!(
        cfg.signature_verifies(),
        "F-GOV-1: GovernanceConfig MUST be a top-level SIGNED Node — its \
         signature over the canonical bytes MUST verify (InstallRecord \
         precedent, PLUGIN-MANIFEST.md:68)"
    );
    assert!(
        !cfg.signature_verifies_after_tier_tamper(),
        "F-GOV-1: tampering the signed `tier` field MUST break the signature \
         (tamper-evidence — the config can't drift post-sign)"
    );
}

/// F-GOV-1 (b): tier promotion (Atrium→Garden, i.e. Flat→Moderated) adds a
/// new governance Node but leaves the Kind unchanged, the K_Set unrotated,
/// and mints NO new identity.
///
/// would-FAIL if W6 re-keys or changes the Kind on a tier promotion.
#[test]
fn tier_promotion_adds_node_without_rekey_or_kind_change() {
    let effect: PromotionEffect = promote_tier(GovernanceTier::Flat, GovernanceTier::Moderated);

    assert!(
        effect.new_governance_node_added,
        "F-GOV-1: a tier promotion MUST add a new governance Node (+ grants + \
         roles)"
    );
    assert!(
        !effect.kind_changed,
        "F-GOV-1: a tier promotion MUST NOT change the MembershipSet Kind — \
         Garden is governance CONTENT, not a new Kind"
    );
    assert!(
        !effect.k_set_rotated,
        "F-GOV-1: a tier promotion MUST NOT re-key (K_Set unchanged) — \
         promotion is additive, not a security-boundary change"
    );
    assert!(
        !effect.new_identity_minted,
        "F-GOV-1: a tier promotion MUST NOT mint a new identity — the set \
         keeps its identity across governance changes"
    );
}

/// F-GOV-1 (c): GovernanceConfig is NEVER a field inside the sealed
/// `MembershipSetPolicy` (struct-fence). It lives as a top-level signed
/// Node, decoupled from the sealed policy.
#[test]
fn governance_config_is_not_a_sealed_policy_field() {
    assert!(
        !MembershipSetPolicy::has_embedded_governance_config_field(),
        "F-GOV-1: GovernanceConfig MUST NOT be embedded as a field inside the \
         sealed MembershipSetPolicy — it is a TOP-LEVEL signed Node \
         (decoupling lets governance change without touching the sealed \
         key-management policy)"
    );
}

/// F-GOV-1 (d): GREP-DEFENSE — Garden / Grove MUST NOT be backed by a wire
/// sub-codepoint; they are signed-Node content. Source scan of the W6
/// membership-set crate finds no `GARDEN`/`GROVE` codepoint constant.
/// `#[test]` (green now): vacuously zero until the crate lands, then
/// load-bearing (would FAIL if a future edit mints a Garden/Grove
/// sub-codepoint).
#[test]
fn grep_defense_no_garden_or_grove_subcodepoint_in_membership_set_src() {
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
            let l = line;
            let upper = l.to_uppercase();
            let mentions_garden_grove_codepoint = (upper.contains("GARDEN")
                || upper.contains("GROVE"))
                && (l.contains("0x6") || l.contains(": u16") || l.contains("Codepoint"));
            if mentions_garden_grove_codepoint {
                offenders.push(format!("{}:{}: {}", path.display(), lineno + 1, l.trim()));
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "F-GOV-1 grep-defense: Garden / Grove MUST be signed-Node CONTENT, \
         NEVER a wire sub-codepoint. Found offending declaration(s):\n{}",
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
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
