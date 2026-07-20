//! The central domain-separation-tag registry — the single source-of-truth
//! table enumerating every **signing / KDF / AEAD-AAD** cross-surface tag
//! minted across the Benten corpus, plus the **prefix-free / no-collision**
//! cross-surface invariant (C-01 / C-02). (Scope carve-out: **public
//! content-hash namespaces** — BLAKE3 CIDv1 content addressing, the §3.9
//! gossip-topic derivation — are NOT registered tags; see the carve-out note
//! below.)
//!
//! # Why a central table
//!
//! Many Benten surfaces derive keys, sign binding messages, or AAD-commit over
//! the SAME key material. Each prefixes a per-surface domain/context tag into
//! the derived/signed/bound bytes so bytes produced on one surface can NEVER be
//! reinterpreted (re-keyed / re-parsed) as another surface's input
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
//! # The corpus-wide registered scope (the widened set)
//!
//! `SECURITY-PROOFS.md` §4.1 + `THREAT-MODEL.md` §5 commit this registry to
//! span every **signing / KDF / AEAD-AAD** cross-surface tag, not just the
//! same-key signature family (the tightened scope — public content-hash
//! namespaces are carved out, see below). The registered surfaces, by family:
//!
//! - **Same-key (user-DID) signature / AAD domains** — [`PROVISIONING_DOMAIN`],
//!   [`ENVELOPE_SIG_DOMAIN`], [`SENDER_AUTH_DOMAIN`], [`REQUEST_DOMAIN`],
//!   [`GRANT_DOMAIN`], [`EXEC_WORKFLOW_AAD_DOMAIN`].
//! - **Set-id commitment + composite-signature label** —
//!   [`SETID_COMMITMENT_LABEL`], [`LAMPS_LABEL_MLDSA65_ED25519_SHA512`].
//! - **Layer-C content-encryption-key (CEK) derivation contexts** —
//!   [`LAYER_C_CEK_CONTEXT`] (`0x6500`/`0x6510`), [`LAYER_C_GROUP_CEK_CONTEXT`]
//!   (`0x6520`), [`MEMBERSHIP_GROUP_CEK_CONTEXT`] (`0x6610`).
//! - **Chunked-AEAD AAD info strings** — [`AEAD_WHOLE_CONTEXT`],
//!   [`AEAD_CHUNK_CONTEXT`], [`AEAD_RECIPE_CONTEXT`].
//! - **MembershipSet KDF contexts** — [`KV_DERIVE_CONTEXT`] (`K(V)`),
//!   [`KN_DERIVE_CONTEXT`] (`K(N)`).
//! - **Vault at-rest contexts** — [`VAULT_AAD_DOMAIN`] (the Layer-A vault
//!   AEAD AAD label), [`DAK_HKDF_INFO_TAG`] (the DAK HKDF info-tag).
//! - **Deterministic recipient-seed expansion** — [`RECIPIENT_SEED_LABEL`]
//!   (the Layer-C deterministic-keypair BLAKE3 expansion label).
//!
//! # Scope carve-out — public content-hash namespaces are NOT registered tags
//!
//! The registry scope is signing / KDF / AEAD-AAD domain separators. **Public
//! content-hash namespaces are deliberately OUTSIDE the registered set** — they
//! are not domain-separation tags in the cross-context-confusion sense (they
//! address public content, they do not key/sign/AAD-bind secret material), so
//! there is nothing to prefix-free-check against the tag corpus. Two instances:
//!
//! - **BLAKE3 CIDv1 content addressing** — the multiformats content-hash
//!   framing (`0x01 0x71 0x1e 0x20 || BLAKE3`) is an un-labelled hash over
//!   public canonical bytes; it is not a keyed/signed domain separator.
//! - **The §3.9 gossip-topic derivation** — a
//!   `blake3::keyed_hash(K_Set, membership_set_id || BE(generation))` with NO
//!   domain-separation label (R0.7 §3.9 authoritative, golden byte-confirmed):
//!   its keyed preimage SHAPE, not a label string, is the separator, so there
//!   is no tag to register (mirrors the content-hash carve-out — a preimage /
//!   framing acts as the separator, not a registered label).
//!
//! # NAMED UN-ENROLLED tag — `X25519_CLASSICAL_INFO_V1` (R17 F-09)
//!
//! `cipher_suite::X25519_CLASSICAL_INFO_V1`
//! (`b"x25519-classical-v1-benten-0x6400"`) is a keying domain-separation
//! info string folded into the `0x6400` classical combiner preimage. UNLIKE
//! the two carve-outs above, it DOES key material — but it is a single
//! self-contained combiner surface (not a cross-surface separator), so it is
//! left OUT of the registered corpus at v1-beta rather than being enrolled or
//! given a permanent exemption. This is a NAMED hardening item (enroll it in
//! [`registered_domain_tags`] with a `domain_registry`-equality drift-assert
//! at its home, OR promote this paragraph to a permanent documented exemption)
//! carried at `docs/V1-FROZEN-INTERFACE-DEFERRED.md`; NOT resolved this round.
//!
//! # The single source of truth vs. the home-crate mirrors
//!
//! [`PROVISIONING_DOMAIN`] is NEW (minted here at C-01) and is defined
//! canonically in THIS module — `device_link::provisioning_signing_bytes`
//! references it directly, so there is exactly one definition and zero drift
//! surface for the new tag.
//!
//! Every OTHER registered tag is a FROZEN constant whose canonical home is the
//! crate that uses it (`benten-drop`, `benten-engine`, `benten-membership-set`,
//! and `benten-crypto-suite`'s own `aead` / `sig` modules) — all home crates
//! depend on (or are) this crate, so the byte values are mirrored here as the
//! canonical corpus collision table. Each home carries a `domain_registry`-
//! equality drift-defense assertion in its own test surface (it can see BOTH
//! its local const and this registry mirror), so the mirror can never silently
//! drift from its home.

/// Provisioning (Layer-D device-link) user-DID-signature domain tag (C-01).
///
/// Prefixed into [`crate`]-external
/// `benten_engine::layer_d::device_link::provisioning_signing_bytes` so the
/// provisioning-offer signature joins the same-key domain-separation family.
/// This is the ONE tag whose canonical definition lives here (it is new at
/// C-01); the device-link sign + verify sites reference it directly.
pub const PROVISIONING_DOMAIN: &[u8] = b"benten/layer-d/device-link-provisioning/v1";

// ---------------------------------------------------------------------------
// Mirrors of the four pre-existing FROZEN same-key signature-family domain
// tags (home-crate is source-of-truth; this is the corpus collision table).
// PROVISIONING_DOMAIN above is the FIFTH same-key tag but is NEW-at-C-01 (its
// canonical home is THIS module, not a mirror). Each home crate drift-asserts
// equality against these in its own tests.
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

// ---------------------------------------------------------------------------
// Layer-C content-encryption-key (CEK) BLAKE3 derivation contexts. Home:
// `benten-drop` (`layer_c.rs`). Each home const carries a `domain_registry`-
// equality drift assertion in its own test surface.
// ---------------------------------------------------------------------------

/// Mirror of `benten_drop::layer_c::LAYER_C_CEK_CONTEXT` (the single-recipient
/// `0x6500`/`0x6510` per-send CEK derivation context). Home: `benten-drop`.
pub const LAYER_C_CEK_CONTEXT: &[u8] = b"benten-drop:layer-c:cek";

/// Mirror of `benten_drop::layer_c::LAYER_C_GROUP_CEK_CONTEXT` (the `0x6520`
/// group bulk-CEK derivation context). Home: `benten-drop`.
pub const LAYER_C_GROUP_CEK_CONTEXT: &[u8] = b"benten-drop:layer-c:group-cek";

/// Mirror of `benten_drop::layer_c::group_posture::MEMBERSHIP_GROUP_CEK_CONTEXT`
/// (the `0x6610` MembershipSet group bulk-CEK derivation context). Home:
/// `benten-drop`.
pub const MEMBERSHIP_GROUP_CEK_CONTEXT: &[u8] = b"benten-drop:membership-group-cek";

// ---------------------------------------------------------------------------
// Chunked-AEAD AAD info strings. Home: this crate (`aead.rs`). The intra-crate
// `aead::tests::aead_contexts_match_central_registry` drift-asserts equality.
// ---------------------------------------------------------------------------

/// Mirror of `crate::aead::AEAD_WHOLE_CONTEXT` (whole-content AEAD AAD info
/// string). Home: `crate::aead`.
pub const AEAD_WHOLE_CONTEXT: &[u8] = b"benten-aead:whole:";

/// Mirror of `crate::aead::AEAD_CHUNK_CONTEXT` (per-chunk AEAD AAD info
/// string). Home: `crate::aead`.
pub const AEAD_CHUNK_CONTEXT: &[u8] = b"benten-aead:chunk:";

/// Mirror of `crate::aead::AEAD_RECIPE_CONTEXT` (per-Recipe AEAD AAD info
/// string). Home: `crate::aead`.
pub const AEAD_RECIPE_CONTEXT: &[u8] = b"benten-aead:recipe:";

// ---------------------------------------------------------------------------
// MembershipSet BLAKE3-KDF contexts. Home: `benten-membership-set`
// (`keying.rs`, as `&str` consts; the registry stores the label bytes). The
// home `keying::domain_registry_mirror` test drift-asserts equality.
// ---------------------------------------------------------------------------

/// Mirror of `benten_membership_set::keying::KV_DERIVE_CONTEXT` (the `K(V)`
/// membership version-node key KDF context). Home: `benten-membership-set`.
pub const KV_DERIVE_CONTEXT: &[u8] = b"benten-membership-set:K(V):v1";

/// Mirror of `benten_membership_set::keying::KN_DERIVE_CONTEXT` (the `K(N)`
/// per-Node content-key KDF context). Home: `benten-membership-set`.
pub const KN_DERIVE_CONTEXT: &[u8] = b"benten-membership-set:K(N):v1";

// ---------------------------------------------------------------------------
// Vault at-rest (Layer-A) domain tags. Home: `crate::vault`. The intra-crate
// `vault::tests::vault_domain_tags_match_central_registry` drift-asserts
// equality against these mirrors.
// ---------------------------------------------------------------------------

/// Mirror of `crate::vault::VAULT_AAD_DOMAIN` (the Layer-A vault AEAD AAD
/// domain-separation label, prefixed ahead of the vault codepoint). Home:
/// `crate::vault`.
pub const VAULT_AAD_DOMAIN: &[u8] = b"benten-vault:";

/// Mirror of `crate::vault::DAK_HKDF_INFO_TAG` (the Device-Auth-Key HKDF
/// info-tag, R0.5 §3.1). Home: `crate::vault`.
pub const DAK_HKDF_INFO_TAG: &[u8] = b"benten-dak-v1";

// ---------------------------------------------------------------------------
// Deterministic recipient-seed expansion label. Home: `crate::cipher_suite`.
// The intra-crate `cipher_suite::tests::cipher_suite_domain_tags_match_central_registry`
// drift-asserts equality against this mirror.
// ---------------------------------------------------------------------------

/// Mirror of `crate::cipher_suite::RECIPIENT_SEED_LABEL` (the Layer-C
/// deterministic recipient-keypair BLAKE3 expansion domain-separation label).
/// Home: `crate::cipher_suite`.
pub const RECIPIENT_SEED_LABEL: &[u8] = b"benten-crypto-suite:recipient-seed";

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
        // Set-id commitment + composite-signature label namespaces:
        SETID_COMMITMENT_LABEL,
        LAMPS_LABEL_MLDSA65_ED25519_SHA512,
        // Layer-C content-encryption-key (CEK) derivation contexts:
        LAYER_C_CEK_CONTEXT,
        LAYER_C_GROUP_CEK_CONTEXT,
        MEMBERSHIP_GROUP_CEK_CONTEXT,
        // Chunked-AEAD AAD info strings:
        AEAD_WHOLE_CONTEXT,
        AEAD_CHUNK_CONTEXT,
        AEAD_RECIPE_CONTEXT,
        // MembershipSet BLAKE3-KDF contexts:
        KV_DERIVE_CONTEXT,
        KN_DERIVE_CONTEXT,
        // Vault at-rest (Layer-A) domain tags:
        VAULT_AAD_DOMAIN,
        DAK_HKDF_INFO_TAG,
        // Deterministic recipient-seed expansion label:
        RECIPIENT_SEED_LABEL,
    ]
}

/// Whether `tags` contains any pair where one tag is a byte-prefix of another
/// (which includes exact duplicates — a tag is a prefix of itself's twin).
///
/// Prefix-freedom is the soundness property: if tag `A` is a prefix of tag `B`,
/// a signature whose signed bytes begin with `A` could be a truncation/framing
/// confusion against `B`'s surface. Equality is the degenerate prefix case, so
/// this single check subsumes the simpler distinctness check.
#[cfg(any(test, feature = "testing"))]
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

    /// The widened corpus enumerates EXACTLY the 19 cross-surface tags the
    /// SECURITY-PROOFS §4.1 / THREAT-MODEL §5 scope names. Locking the count
    /// makes the prefix-free invariant forward-fire on ANY tag change: adding a
    /// tag without updating this count fails the build (forcing a deliberate
    /// re-confirmation that the new tag clears the prefix-free check), and the
    /// per-family membership assertion below catches an accidental drop of any
    /// named surface.
    #[test]
    fn registry_spans_the_full_corpus_wide_scope() {
        let tags = registered_domain_tags();
        assert_eq!(
            tags.len(),
            19,
            "registered_domain_tags() count changed — re-confirm the new/removed tag is \
             prefix-free and update SECURITY-PROOFS §4.1 / THREAT-MODEL §5 scope"
        );
        // Every named cross-surface surface MUST be present (drop-detection).
        for expected in [
            // same-key signature / AAD family:
            PROVISIONING_DOMAIN,
            ENVELOPE_SIG_DOMAIN,
            SENDER_AUTH_DOMAIN,
            REQUEST_DOMAIN,
            GRANT_DOMAIN,
            EXEC_WORKFLOW_AAD_DOMAIN,
            // set-id commitment + composite-signature label:
            SETID_COMMITMENT_LABEL,
            LAMPS_LABEL_MLDSA65_ED25519_SHA512,
            // Layer-C CEK derivation contexts:
            LAYER_C_CEK_CONTEXT,
            LAYER_C_GROUP_CEK_CONTEXT,
            MEMBERSHIP_GROUP_CEK_CONTEXT,
            // chunked-AEAD AAD info strings:
            AEAD_WHOLE_CONTEXT,
            AEAD_CHUNK_CONTEXT,
            AEAD_RECIPE_CONTEXT,
            // MembershipSet KDF contexts:
            KV_DERIVE_CONTEXT,
            KN_DERIVE_CONTEXT,
            // Vault at-rest (Layer-A) domain tags:
            VAULT_AAD_DOMAIN,
            DAK_HKDF_INFO_TAG,
            // Deterministic recipient-seed expansion label:
            RECIPIENT_SEED_LABEL,
        ] {
            assert!(
                tags.contains(&expected),
                "a registered cross-surface domain tag is missing from registered_domain_tags()"
            );
        }
    }

    /// Injecting a `"<existing-tag>-suffix"` style mint fires the scanner — the
    /// concrete SECURITY-PROOFS §4.1 example over a widened-set tag.
    #[test]
    fn injection_of_a_suffix_extended_cek_tag_fires_the_scanner() {
        let mut injected = registered_domain_tags();
        injected.push(b"benten-drop:layer-c:cek-v2"); // prefixed by LAYER_C_CEK_CONTEXT
        assert!(detects_prefix_collision(&injected));
    }
}
