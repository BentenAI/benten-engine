//! F-GOV-1 (R3-W6 gov-audit) — `GovernanceConfig{tier: Flat | Moderated |
//! Polycentric}` is a TOP-LEVEL signed Node (InstallRecord precedent
//! `PLUGIN-MANIFEST.md:68`), NEVER a field inside the sealed
//! `MembershipSetPolicy`. Tier promotion = add a Node + grants + roles —
//! NO re-key, NO new identity, NO Kind change. Garden / Grove are
//! signed-Node CONTENT, NOT sub-codepoints.
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-GOV-1; merges
//! K3 + GNI-18):
//!   - R0.3 plan §2.6, §3.6.B, §4.2, §9.1-6.
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
// RED-PHASE stub-shim — DELETE at W6 implementation; replace with:
//     use benten_membership_set::governance::{
//         GovernanceConfig, GovernanceTier, MembershipSetPolicy,
//         PromotionEffect, promote_tier,
//     };
// =====================================================================
mod mset_w6_governance_stub {
    //! Local stub matching the intended W6 governance surface. Bodies
    //! `unimplemented!()`.

    /// The governance tier. Garden / Grove are CONTENT of a top-level
    /// signed Node — NOT sub-codepoints, NOT sealed-policy fields.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GovernanceTier {
        Flat,
        Moderated,
        Polycentric,
    }

    /// The top-level signed governance config Node (InstallRecord
    /// precedent). Carries the tier + a signature over its canonical
    /// bytes. NEVER a field inside `MembershipSetPolicy`.
    #[derive(Clone, Debug)]
    pub struct GovernanceConfig {
        pub tier: GovernanceTier,
        /// Detached signature over the canonical Node bytes.
        pub signature: Vec<u8>,
        /// Garden / Grove labelling lives as Node CONTENT here.
        pub content_label: String,
    }

    impl GovernanceConfig {
        pub fn new_signed(_tier: GovernanceTier, _content_label: &str) -> Self {
            unimplemented!(
                "W6 stub — R5 replaces this module with \
                 `use benten_membership_set::governance::*;`"
            )
        }
        /// Verify the detached signature over the canonical Node bytes
        /// (a top-level signed Node, not a sealed-policy field).
        pub fn signature_verifies(&self) -> bool {
            unimplemented!("W6 stub — top-level signed-Node signature verify")
        }
        /// Verify after tampering the tier byte (must fail — tamper-evidence).
        pub fn signature_verifies_after_tier_tamper(&self) -> bool {
            unimplemented!("W6 stub — tamper a signed field → signature must fail")
        }
    }

    /// The MembershipSet sealed policy. W6 real shape MUST NOT carry a
    /// `governance_config` field — this stub exists only so the
    /// struct-fence arm can name it. The boolean reports whether the
    /// sealed policy has an embedded GovernanceConfig field (real impl:
    /// `false`).
    pub struct MembershipSetPolicy;
    impl MembershipSetPolicy {
        pub fn has_embedded_governance_config_field() -> bool {
            unimplemented!(
                "W6 stub — GovernanceConfig is a TOP-LEVEL signed Node, NOT a \
                 MembershipSetPolicy field"
            )
        }
    }

    /// The observable effect of a tier promotion. Real impl: a new Node is
    /// added; the Kind + K_Set are UNCHANGED; no new identity is minted.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PromotionEffect {
        pub kind_changed: bool,
        pub k_set_rotated: bool,
        pub new_identity_minted: bool,
        pub new_governance_node_added: bool,
    }

    /// W6 stub: promote the governance tier (e.g. Atrium → Garden via
    /// Flat → Moderated). Real impl adds a Node + grants + roles WITHOUT
    /// re-keying, minting a new identity, or changing the Kind.
    pub fn promote_tier(
        _from: GovernanceTier,
        _to: GovernanceTier,
    ) -> PromotionEffect {
        unimplemented!("W6 stub — tier promotion = add Node + grants + roles; no re-key")
    }
}

use mset_w6_governance_stub::{
    GovernanceConfig, GovernanceTier, MembershipSetPolicy, PromotionEffect, promote_tier,
};

/// F-GOV-1 (a): the GovernanceConfig is a top-level SIGNED Node — its
/// signature over the canonical bytes verifies, and tampering the signed
/// tier field breaks the signature (tamper-evidence).
#[test]
#[ignore = "RED-PHASE: F-GOV-1 — GovernanceConfig is a top-level signed Node; un-ignore at W6 R5 (delete mset_w6_governance_stub; insert real `use`)"]
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
#[ignore = "RED-PHASE: F-GOV-1 — tier promotion adds Node, NO re-key / NO Kind change / NO new identity; un-ignore at W6 R5"]
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
#[ignore = "RED-PHASE: F-GOV-1 — GovernanceConfig is NOT a MembershipSetPolicy field; un-ignore at W6 R5"]
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
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                f(&path, &contents);
            }
        }
    }
}
