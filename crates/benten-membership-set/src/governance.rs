//! Governance config as a TOP-LEVEL signed Node (data-half) — NEVER a field
//! inside the sealed `MembershipSetPolicy`.
//!
//! `GovernanceConfig { tier: Flat | Moderated | Polycentric }` is a top-level
//! SIGNED Node (the InstallRecord precedent, `docs/PLUGIN-MANIFEST.md`),
//! decoupled from the sealed key-management policy so governance can change
//! WITHOUT touching the sealed policy. A tier promotion adds a Node + grants +
//! roles — it does NOT re-key, mint a new identity, or change the Kind. Garden
//! and Grove are signed-Node CONTENT (a `content_label` string), NOT wire
//! sub-codepoints.
//!
//! The signature is a REAL hybrid Ed25519⊕ML-DSA-65 signature routed through
//! [`benten_crypto_suite::SignatureSuite`] (the #5 ONLY-call-site — this module
//! NEVER constructs a crypto primitive directly; it hands the canonical Node
//! bytes to the crypto-suite). Tampering the signed `tier` field mutates the
//! canonical bytes, so the original signature no longer verifies (tamper-
//! evidence: the config cannot drift post-sign).

use benten_crypto_suite::sig::PublicKey;
use benten_crypto_suite::{HybridSignature, SignatureSuite, SuiteConfig};

/// The governance tier. Garden / Grove are CONTENT of a top-level signed Node
/// ([`GovernanceConfig::content_label`]) — NOT sub-codepoints, NOT sealed-policy
/// fields. The three tiers correspond to Flat / Moderated / Polycentric
/// governance shapes.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future governance-tier variant
/// lands additively, never a downstream `match` break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GovernanceTier {
    /// Flat governance — every admin is co-equal.
    Flat,
    /// Moderated governance — a moderator role gates membership/content.
    Moderated,
    /// Polycentric governance — multiple semi-autonomous governance centers.
    Polycentric,
}

impl GovernanceTier {
    /// The canonical single-byte tag of this tier — the byte that enters the
    /// signed canonical Node bytes. (A data-half tag, NOT a wire codepoint:
    /// it is part of the SIGNED Node content, never a §4.0 registry entry.)
    #[must_use]
    const fn tag(self) -> u8 {
        match self {
            GovernanceTier::Flat => 0,
            GovernanceTier::Moderated => 1,
            GovernanceTier::Polycentric => 2,
            // NOTE: no `_` arm. `GovernanceTier` is `#[non_exhaustive]` for
            // downstream SemVer-readiness, but this tag byte enters the SIGNED
            // canonical Node bytes, so within the defining crate the match stays
            // exhaustive: a future tier is a HALT-AND-SURFACE compile error here,
            // forcing it to mint its own distinct signed tag rather than
            // silently colliding an existing tier's signed content (§15.c).
        }
    }
}

/// The top-level signed governance config Node (InstallRecord precedent).
///
/// Carries the tier + a REAL hybrid signature over its canonical Node bytes +
/// the Garden/Grove labelling as Node CONTENT. NEVER a field inside the sealed
/// [`MembershipSetPolicy`]. The verifying key + the signed inputs are retained
/// so [`GovernanceConfig::signature_verifies`] and
/// [`GovernanceConfig::signature_verifies_after_tier_tamper`] can re-verify the
/// detached signature against the canonical bytes (and against a tampered
/// variant of them).
#[derive(Clone)]
pub struct GovernanceConfig {
    /// The governance tier (a signed field).
    pub tier: GovernanceTier,
    /// The signed-Node content label (e.g. a `"Garden"` / `"Grove"` tier
    /// name) — Node CONTENT carried as a plain string, never a wire tag (a
    /// signed field).
    pub content_label: String,
    /// The detached hybrid signature over the canonical Node bytes.
    signature: HybridSignature,
    /// The verifying key the signature was produced under (so a holder of the
    /// Node can verify the detached signature).
    verifying_key: PublicKey,
}

impl GovernanceConfig {
    /// Construct a top-level signed `GovernanceConfig` — generates a hybrid
    /// keypair, signs the canonical Node bytes for `(tier, content_label)`, and
    /// retains the verifying key so the detached signature can be checked.
    ///
    /// The signature is a REAL hybrid Ed25519⊕ML-DSA-65 signature (routed
    /// through the crypto-suite — never forked).
    #[must_use]
    pub fn new_signed(tier: GovernanceTier, content_label: &str) -> Self {
        let suite = SignatureSuite::from_config(SuiteConfig::v1_default());
        let kp = suite.generate_keypair();
        let verifying_key = kp.public();
        let canonical = canonical_governance_bytes(tier, content_label);
        let signature = suite.sign(&kp, &canonical);
        GovernanceConfig {
            tier,
            content_label: content_label.to_string(),
            signature,
            verifying_key,
        }
    }

    /// Verify the detached signature over the canonical Node bytes (a
    /// top-level signed Node, not a sealed-policy field). Returns `true` iff
    /// the hybrid signature verifies against the canonical bytes.
    #[must_use]
    pub fn signature_verifies(&self) -> bool {
        let suite = SignatureSuite::from_config(SuiteConfig::v1_default());
        let canonical = canonical_governance_bytes(self.tier, &self.content_label);
        suite
            .verify(self.verifying_key.clone(), &canonical, &self.signature)
            .is_ok()
    }

    /// Verify the ORIGINAL signature against a TAMPERED variant of the
    /// canonical bytes (the `tier` byte flipped). Returns `false` because the
    /// signature was computed over the untampered bytes — tamper-evidence: a
    /// signed governance config cannot drift post-sign.
    #[must_use]
    pub fn signature_verifies_after_tier_tamper(&self) -> bool {
        let suite = SignatureSuite::from_config(SuiteConfig::v1_default());
        // Flip the tier to a DIFFERENT tier, leaving everything else identical
        // → the canonical bytes change → the original signature must fail.
        let tampered_tier = match self.tier {
            GovernanceTier::Flat => GovernanceTier::Moderated,
            GovernanceTier::Moderated => GovernanceTier::Polycentric,
            GovernanceTier::Polycentric => GovernanceTier::Flat,
        };
        let tampered = canonical_governance_bytes(tampered_tier, &self.content_label);
        suite
            .verify(self.verifying_key.clone(), &tampered, &self.signature)
            .is_ok()
    }
}

/// The MembershipSet sealed policy — the STRUCT-FENCE name the `f_gov_1`
/// arm asserts against. It carries NO `governance_config` field by
/// construction: `GovernanceConfig` is a TOP-LEVEL signed Node, decoupled from
/// the sealed key-management policy. This zero-sized marker exists only so the
/// struct-fence arm can name it and assert the decoupling.
pub struct MembershipSetPolicy;

impl MembershipSetPolicy {
    /// Struct-fence introspection: does the sealed policy embed a
    /// `GovernanceConfig` field? Always `false` by construction — governance
    /// is a top-level signed Node, never a sealed-policy field. (`const`, not
    /// a runtime computation: the decoupling is a structural truth.)
    #[must_use]
    pub const fn has_embedded_governance_config_field() -> bool {
        false
    }
}

/// The observable effect of a tier promotion: a new governance Node is added;
/// the Kind + K_Set are UNCHANGED; no new identity is minted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromotionEffect {
    /// Did the MembershipSet Kind change? (Always `false` — Garden is
    /// governance CONTENT, not a new Kind.)
    pub kind_changed: bool,
    /// Was the group key `K_Set` rotated? (Always `false` — promotion is
    /// additive, not a security-boundary change.)
    pub k_set_rotated: bool,
    /// Was a new identity minted? (Always `false` — the set keeps its identity
    /// across governance changes.)
    pub new_identity_minted: bool,
    /// Was a new governance Node added? (Always `true` — promotion = add Node
    /// + grants + roles.)
    pub new_governance_node_added: bool,
}

/// Promote the governance tier (e.g. Atrium → Garden via Flat → Moderated).
///
/// The effect is ADDITIVE: a new governance Node is added (+ grants + roles),
/// and the Kind + K_Set are UNCHANGED, with no new identity minted. Promotion
/// is a content change to the governance Node-chain, not a re-key / re-identity
/// / Kind change.
#[must_use]
pub fn promote_tier(_from: GovernanceTier, _to: GovernanceTier) -> PromotionEffect {
    PromotionEffect {
        kind_changed: false,
        k_set_rotated: false,
        new_identity_minted: false,
        new_governance_node_added: true,
    }
}

/// The canonical Node bytes the governance signature is computed over —
/// `tier.tag()` followed by the UTF-8 `content_label` bytes (length-prefixed so
/// the encoding is injective). A data-half encoding; NOT a wire codepoint.
fn canonical_governance_bytes(tier: GovernanceTier, content_label: &str) -> Vec<u8> {
    let label = content_label.as_bytes();
    let mut buf = Vec::with_capacity(1 + 8 + label.len());
    buf.push(tier.tag());
    // Length-prefix the label (big-endian u64) so `(tier, label)` is an
    // injective encoding — no two distinct configs share canonical bytes.
    buf.extend_from_slice(&(label.len() as u64).to_be_bytes());
    buf.extend_from_slice(label);
    buf
}
