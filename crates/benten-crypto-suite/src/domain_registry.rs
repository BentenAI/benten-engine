//! The central domain-separation-tag registry — the single source-of-truth
//! table enumerating EVERY signing / AAD domain-separation tag minted across
//! the Benten corpus, plus the **prefix-free / no-collision** cross-surface
//! invariant (C-01 / C-02).
//!
//! # Why a central table
//!
//! Six Benten surfaces sign with the SAME user-DID Ed25519 key (or bind the
//! SAME key material into an AEAD AAD). Each prefixes a per-surface
//! domain-separation tag into the signed/bound bytes so a signature produced on
//! one surface can NEVER be reinterpreted as a signature on another
//! (cross-context resistance). The safety property is corpus-wide: **no tag may
//! be a byte-prefix of another tag** (prefix-freedom strictly implies
//! distinctness). A lone surface that forgets its prefix — the C-01 origin,
//! `device_link::provisioning_signing_bytes` before this fix — is a
//! cross-context-confusion hole that no per-surface review catches, because the
//! property only exists ACROSS surfaces. This module makes the whole set
//! enumerable in one place and pins the invariant with a test
//! (`all_domain_tags_are_prefix_free`) that fails the build if any future tag
//! collides.
//!
//! # The single source of truth vs. the frozen mirrors
//!
//! [`PROVISIONING_DOMAIN`] is NEW (minted here at C-01) and is defined
//! canonically in THIS module — `device_link::provisioning_signing_bytes`
//! references it directly, so there is exactly one definition and zero drift
//! surface for the new tag.
//!
//! The five pre-existing tags are FROZEN public constants that live in their
//! home crates (`benten-drop`, `benten-engine`, `benten-membership-set`) — all
//! of which depend on this crate, so this crate cannot import them (the
//! dependency edge points the wrong way, and the byte values are permanent
//! anyway). They are mirrored here as the canonical corpus table; each
//! downstream home crate carries a `domain_registry`-equality drift-defense
//! assertion in its own test surface (each can see BOTH its local const and
//! this registry mirror), so the mirror can never silently drift from its home.

/// Provisioning (Layer-D device-link) user-DID-signature domain tag (C-01).
///
/// Prefixed into [`crate`]-external
/// `benten_engine::layer_d::device_link::provisioning_signing_bytes` so the
/// provisioning-offer signature joins the same-key domain-separation family.
/// This is the ONE tag whose canonical definition lives here (it is new at
/// C-01); the device-link sign + verify sites reference it directly.
pub const PROVISIONING_DOMAIN: &[u8] = b"benten/layer-d/device-link-provisioning/v1";

// ---------------------------------------------------------------------------
// Mirrors of the five pre-existing FROZEN domain tags (home-crate is
// source-of-truth; this is the corpus collision table). Each home crate
// drift-asserts equality against these in its own tests.
// ---------------------------------------------------------------------------

/// Mirror of `benten_drop::envelope_sig::ENVELOPE_SIG_DOMAIN` (offline
/// DropBundle envelope signature). Home: `benten-drop` (module-private there).
pub const ENVELOPE_SIG_DOMAIN: &[u8] = b"benten/g-core-3f/drop-bundle-envelope/v1";

/// Mirror of `benten_drop::layer_c::SENDER_AUTH_DOMAIN` (Sealed-Sender
/// ORIGIN-AUTH per-message signature). Home: `benten-drop`.
pub const SENDER_AUTH_DOMAIN: &[u8] = b"benten/layer-c/sealed-sender-origin-auth/v1";

/// Mirror of `benten_engine::layer_d::remote_permission::REQUEST_DOMAIN`
/// (Layer-D remote-permission REQUEST signature). Home: `benten-engine`.
pub const REQUEST_DOMAIN: &[u8] = b"benten-remote-permission-request-v2:";

/// Mirror of `benten_engine::layer_d::remote_permission::GRANT_DOMAIN`
/// (Layer-D remote-permission GRANT signature). Home: `benten-engine`.
pub const GRANT_DOMAIN: &[u8] = b"benten-remote-permission-grant-v2:";

// ---------------------------------------------------------------------------
// Related AAD / label namespaces enumerated for cross-surface completeness.
// These do NOT ride the same-key Ed25519 signature family but DO bind into
// AEAD AAD / composite-signature labels over Benten key material, so they are
// folded into the corpus prefix-free check (the finding's "also reference
// EXEC_WORKFLOW_AAD_DOMAIN / SETID_COMMITMENT_LABEL / LAMPS_LABEL_*").
// ---------------------------------------------------------------------------

/// Mirror of `benten_engine::layer_d::remote_permission::EXEC_WORKFLOW_AAD_DOMAIN`
/// (intra-variant exec-workflow AAD tag). Home: `benten-engine`.
pub const EXEC_WORKFLOW_AAD_DOMAIN: &[u8] = b"benten-exec-workflow-v1:";

/// Mirror of `benten_membership_set::aad::SETID_COMMITMENT_LABEL` (the §3.10
/// setid-commitment domain-separation label). Home: `benten-membership-set`.
pub const SETID_COMMITMENT_LABEL: &[u8] = b"benten:setid:v1";

/// Mirror of the `benten_crypto_suite::sig` LAMPS composite-signature label for
/// the `id-MLDSA65-Ed25519-SHA512` hybrid (RFC LAMPS composite context label).
/// Home: this crate (`sig.rs`, module-private there).
pub const LAMPS_LABEL_MLDSA65_ED25519_SHA512: &[u8] = b"COMPSIG-MLDSA65-Ed25519-SHA512";

/// The complete corpus of domain-separation tags (the single enumerable table).
///
/// The cross-surface prefix-free / no-collision invariant
/// ([`detects_prefix_collision`]) is checked over EXACTLY this set. Every new
/// signing/AAD domain tag minted anywhere in the corpus MUST be added here, so
/// the build-time test forces the prefix-free property corpus-wide.
#[must_use]
pub fn registered_domain_tags() -> Vec<&'static [u8]> {
    vec![
        // The same-key (user-DID Ed25519) signature family (6 surfaces):
        PROVISIONING_DOMAIN,
        ENVELOPE_SIG_DOMAIN,
        SENDER_AUTH_DOMAIN,
        REQUEST_DOMAIN,
        GRANT_DOMAIN,
        EXEC_WORKFLOW_AAD_DOMAIN,
        // Related AAD / composite-signature label namespaces:
        SETID_COMMITMENT_LABEL,
        LAMPS_LABEL_MLDSA65_ED25519_SHA512,
    ]
}

/// Whether `tags` contains any pair where one tag is a byte-prefix of another
/// (which includes exact duplicates — a tag is a prefix of itself's twin).
///
/// Prefix-freedom is the soundness property: if tag `A` is a prefix of tag `B`,
/// a signature whose signed bytes begin with `A` could be a truncation/framing
/// confusion against `B`'s surface. Equality is the degenerate prefix case, so
/// this single check subsumes the simpler distinctness check.
#[must_use]
pub fn detects_prefix_collision(tags: &[&[u8]]) -> bool {
    for (i, a) in tags.iter().enumerate() {
        for (j, b) in tags.iter().enumerate() {
            if i == j {
                continue;
            }
            // `a` is a (non-strict) prefix of `b` — including the equal case.
            if b.starts_with(a) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_domain_tags_are_prefix_free() {
        let tags = registered_domain_tags();
        assert!(
            !detects_prefix_collision(&tags),
            "a domain-separation tag is a byte-prefix of another — cross-context-confusion hole"
        );
    }

    #[test]
    fn all_domain_tags_are_distinct() {
        let tags = registered_domain_tags();
        let mut sorted = tags.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            tags.len(),
            "duplicate domain-separation tag in the registry"
        );
    }

    #[test]
    fn injection_of_a_prefixing_tag_fires_the_scanner() {
        // A tag that is a strict prefix of an existing tag MUST be detected.
        let mut injected = registered_domain_tags();
        injected.push(b"benten/layer-d/device-link-provisioning"); // prefix of PROVISIONING_DOMAIN
        assert!(detects_prefix_collision(&injected));
    }

    #[test]
    fn injection_of_a_duplicate_tag_fires_the_scanner() {
        let mut injected = registered_domain_tags();
        injected.push(PROVISIONING_DOMAIN);
        assert!(detects_prefix_collision(&injected));
    }

    #[test]
    fn provisioning_domain_is_in_the_registry() {
        assert!(registered_domain_tags().contains(&PROVISIONING_DOMAIN));
    }
}
