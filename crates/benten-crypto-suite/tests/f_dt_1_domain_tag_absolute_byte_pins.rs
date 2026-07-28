//! F-DT-1 — ABSOLUTE byte pins for the frozen domain-separation tag corpus (S-6).
//!
//! # Why this file exists
//!
//! Every domain / KDF-context / AEAD-AAD tag in the corpus was, before this
//! file, pinned only *relatively*: the central
//! [`benten_crypto_suite::domain_registry`] table asserts its own membership
//! against constants that build that same table
//! (`registry_spans_the_full_corpus_wide_scope`), and each home crate asserts
//! `HOME_CONST == registry::MIRROR_CONST`. Both sides of every one of those
//! equalities are symbols, so a coordinated rename of a tag and its mirror
//! leaves the entire suite green.
//!
//! That is not a theoretical gap. Most of these tags feed KEY DERIVATION or
//! AEAD AAD, so their bytes never appear on the wire: the Layer-C CEK context
//! is documented in `benten-drop/src/layer_c.rs` as "a seal-local source, not a
//! round-trip contract" (the open side HPKE-unwraps the CEK and never re-hashes
//! it). Changing such a tag silently re-keys every future derivation — and for
//! the contexts that ARE re-derived at open, silently renders all previously
//! sealed content undecryptable — while every seal/open round-trip test stays
//! self-consistent and green.
//!
//! # What this file pins
//!
//! The literal bytes of all 23 registered tags, in registry order. Because the
//! eleven home-crate drift-asserts already tie each home constant to its
//! registry mirror, nailing the registry side to literals here transitively
//! converts every one of those existing asserts into a real byte pin.
//!
//! This also reaches the four `pub(crate)` tags
//! (`structural_kdf::STRUCTURAL_KDF_ROOT_LABEL` / `STRUCTURAL_KDF_STEP_LABEL`,
//! `swap_matrix::SWAP_MATRIX_AAD_DOMAIN`,
//! `cipher_suite::X25519_CLASSICAL_INFO_V1`) that no integration test can name
//! directly — their VALUES flow into the returned vec, so the positional pin
//! covers them.
//!
//! # If a pin here fails
//!
//! A failure means a frozen domain-separation tag changed. That is a
//! wire/keying-compatibility break, not a test bug. Do NOT edit the expected
//! literal to match the code. Establish why the constant moved.

use benten_crypto_suite::domain_registry::{
    AEAD_CHUNK_CONTEXT, AEAD_RECIPE_CONTEXT, AEAD_WHOLE_CONTEXT, DAK_HKDF_INFO_TAG,
    ENVELOPE_SIG_DOMAIN, EXEC_WORKFLOW_AAD_DOMAIN, GRANT_DOMAIN, KN_DERIVE_CONTEXT,
    KV_DERIVE_CONTEXT, LAMPS_LABEL_MLDSA65_ED25519_SHA512, LAYER_C_CEK_CONTEXT,
    LAYER_C_GROUP_CEK_CONTEXT, MEMBERSHIP_GROUP_CEK_CONTEXT, PROVISIONING_DOMAIN,
    RECIPIENT_SEED_LABEL, REQUEST_DOMAIN, SENDER_AUTH_DOMAIN, SETID_COMMITMENT_LABEL,
    VAULT_AAD_DOMAIN, registered_domain_tags,
};

/// The frozen corpus, in `registered_domain_tags()` order.
///
/// Entries 20-23 are the `pub(crate)` tags that cannot be named from an
/// integration test; they are pinned positionally through the returned vec.
const FROZEN_DOMAIN_TAGS: [&[u8]; 23] = [
    // --- same-key (user-DID Ed25519) signature / AAD family (6) ---
    b"benten/layer-d/device-link-provisioning/v1", // PROVISIONING_DOMAIN
    b"benten/g-core-3f/drop-bundle-envelope/v1",   // ENVELOPE_SIG_DOMAIN
    b"benten/layer-c/sealed-sender-origin-auth/v1", // SENDER_AUTH_DOMAIN
    b"benten-remote-permission-request-v2:",       // REQUEST_DOMAIN
    b"benten-remote-permission-grant-v2:",         // GRANT_DOMAIN
    b"benten-exec-workflow-v1:",                   // EXEC_WORKFLOW_AAD_DOMAIN
    // --- set-id commitment + composite-signature label (2) ---
    b"benten:setid:v1",                // SETID_COMMITMENT_LABEL
    b"COMPSIG-MLDSA65-Ed25519-SHA512", // LAMPS_LABEL_MLDSA65_ED25519_SHA512
    // --- Layer-C content-encryption-key derivation contexts (3) ---
    b"benten-drop:layer-c:cek",          // LAYER_C_CEK_CONTEXT
    b"benten-drop:layer-c:group-cek",    // LAYER_C_GROUP_CEK_CONTEXT
    b"benten-drop:membership-group-cek", // MEMBERSHIP_GROUP_CEK_CONTEXT
    // --- chunked-AEAD AAD info strings (3) ---
    b"benten-aead:whole:",  // AEAD_WHOLE_CONTEXT
    b"benten-aead:chunk:",  // AEAD_CHUNK_CONTEXT
    b"benten-aead:recipe:", // AEAD_RECIPE_CONTEXT
    // --- MembershipSet BLAKE3-KDF contexts (2) ---
    b"benten-membership-set:K(V):v1", // KV_DERIVE_CONTEXT
    b"benten-membership-set:K(N):v1", // KN_DERIVE_CONTEXT
    // --- Vault at-rest (Layer-A) domain tags (2) ---
    b"benten-vault:", // VAULT_AAD_DOMAIN
    b"benten-dak-v1", // DAK_HKDF_INFO_TAG
    // --- deterministic recipient-seed expansion label (1) ---
    b"benten-crypto-suite:recipient-seed", // RECIPIENT_SEED_LABEL
    // --- structural-KDF role-separation HKDF info-tag prefixes (2, pub(crate)) ---
    b"root:codepoint:", // structural_kdf::STRUCTURAL_KDF_ROOT_LABEL
    b"step",            // structural_kdf::STRUCTURAL_KDF_STEP_LABEL
    // --- swap-matrix sign-and-seal AAD-commit prefix (1, pub(crate)) ---
    b"sm-aad:", // swap_matrix::SWAP_MATRIX_AAD_DOMAIN
    // --- classical-combiner keying info string (1, pub(crate); D-95) ---
    b"x25519-classical-v1-benten-0x6400", // cipher_suite::X25519_CLASSICAL_INFO_V1
];

/// The whole frozen corpus, positionally, against literal bytes.
///
/// This is the pin that makes every home-crate `HOME == registry::MIRROR`
/// drift-assert load-bearing.
#[test]
fn registered_domain_tags_match_frozen_literal_bytes() {
    let tags = registered_domain_tags();

    assert_eq!(
        tags.len(),
        FROZEN_DOMAIN_TAGS.len(),
        "the registered domain-tag corpus changed size (frozen at {}); a tag was added or \
         removed — this is a keying/wire-compatibility decision, not a test fix",
        FROZEN_DOMAIN_TAGS.len()
    );

    for (i, (actual, expected)) in tags.iter().zip(FROZEN_DOMAIN_TAGS.iter()).enumerate() {
        assert_eq!(
            actual,
            expected,
            "frozen domain-separation tag at registry index {i} changed from {:?} to {:?}. \
             These bytes feed key derivation / AEAD AAD / signature preimages and mostly \
             never reach the wire, so round-trip tests cannot catch this. Do NOT edit the \
             expected literal to match.",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(actual),
        );
    }
}

/// Per-name literal pins for the 19 publicly-nameable registry constants.
///
/// Redundant with the positional pin above by construction, and deliberately so:
/// this arm names the drifted tag directly in the failure message, and it keeps
/// firing if the corpus vec is ever reordered.
#[test]
fn each_named_registry_tag_matches_its_frozen_literal() {
    assert_eq!(
        PROVISIONING_DOMAIN, b"benten/layer-d/device-link-provisioning/v1",
        "PROVISIONING_DOMAIN is frozen"
    );
    assert_eq!(
        ENVELOPE_SIG_DOMAIN, b"benten/g-core-3f/drop-bundle-envelope/v1",
        "ENVELOPE_SIG_DOMAIN is frozen"
    );
    assert_eq!(
        SENDER_AUTH_DOMAIN, b"benten/layer-c/sealed-sender-origin-auth/v1",
        "SENDER_AUTH_DOMAIN is frozen"
    );
    assert_eq!(
        REQUEST_DOMAIN, b"benten-remote-permission-request-v2:",
        "REQUEST_DOMAIN is frozen"
    );
    assert_eq!(
        GRANT_DOMAIN, b"benten-remote-permission-grant-v2:",
        "GRANT_DOMAIN is frozen"
    );
    assert_eq!(
        EXEC_WORKFLOW_AAD_DOMAIN, b"benten-exec-workflow-v1:",
        "EXEC_WORKFLOW_AAD_DOMAIN is frozen"
    );
    assert_eq!(
        SETID_COMMITMENT_LABEL, b"benten:setid:v1",
        "SETID_COMMITMENT_LABEL is frozen"
    );
    assert_eq!(
        LAMPS_LABEL_MLDSA65_ED25519_SHA512, b"COMPSIG-MLDSA65-Ed25519-SHA512",
        "the LAMPS composite-signature label is frozen (id-MLDSA65-Ed25519-SHA512)"
    );
    assert_eq!(
        LAYER_C_CEK_CONTEXT, b"benten-drop:layer-c:cek",
        "LAYER_C_CEK_CONTEXT is frozen"
    );
    assert_eq!(
        LAYER_C_GROUP_CEK_CONTEXT, b"benten-drop:layer-c:group-cek",
        "LAYER_C_GROUP_CEK_CONTEXT is frozen"
    );
    assert_eq!(
        MEMBERSHIP_GROUP_CEK_CONTEXT, b"benten-drop:membership-group-cek",
        "MEMBERSHIP_GROUP_CEK_CONTEXT is frozen"
    );
    assert_eq!(
        AEAD_WHOLE_CONTEXT, b"benten-aead:whole:",
        "AEAD_WHOLE_CONTEXT is frozen"
    );
    assert_eq!(
        AEAD_CHUNK_CONTEXT, b"benten-aead:chunk:",
        "AEAD_CHUNK_CONTEXT is frozen"
    );
    assert_eq!(
        AEAD_RECIPE_CONTEXT, b"benten-aead:recipe:",
        "AEAD_RECIPE_CONTEXT is frozen"
    );
    assert_eq!(
        KV_DERIVE_CONTEXT, b"benten-membership-set:K(V):v1",
        "KV_DERIVE_CONTEXT is frozen"
    );
    assert_eq!(
        KN_DERIVE_CONTEXT, b"benten-membership-set:K(N):v1",
        "KN_DERIVE_CONTEXT is frozen"
    );
    assert_eq!(
        VAULT_AAD_DOMAIN, b"benten-vault:",
        "VAULT_AAD_DOMAIN is frozen"
    );
    assert_eq!(
        DAK_HKDF_INFO_TAG, b"benten-dak-v1",
        "DAK_HKDF_INFO_TAG is frozen"
    );
    assert_eq!(
        RECIPIENT_SEED_LABEL, b"benten-crypto-suite:recipient-seed",
        "RECIPIENT_SEED_LABEL is frozen"
    );
}

/// Negative control — the pin above is only meaningful if a byte change to a
/// tag would actually be observed. Mutating one byte of a frozen literal MUST
/// make it differ from the live registry entry.
#[test]
fn a_one_byte_tag_mutation_is_observable() {
    let tags = registered_domain_tags();
    let mut mutated = FROZEN_DOMAIN_TAGS[0].to_vec();
    let last = mutated.len() - 1;
    mutated[last] ^= 0x01;
    assert_ne!(
        tags[0],
        mutated.as_slice(),
        "a one-byte mutation of a domain tag MUST be observable — if this fails the \
         comparison in this file is not actually comparing bytes"
    );
}
