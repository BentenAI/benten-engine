//! S-3 closure — frozen const VALUE pins for `benten-crypto-suite`.
//!
//! # Why this file exists
//!
//! The required `cargo-public-api` baseline (`docs/public-api/benten-crypto-suite.txt`)
//! records const TYPES only — never values. A pure value edit
//! (`ENVELOPE_MAGIC: u8 = 0xae` -> `= 0xbe`) produces an EMPTY baseline diff and
//! ships silently. Worse, the pre-existing in-tree assertions are tautological:
//! `aead.rs` asserts `bytes[0] == ENVELOPE_MAGIC` where `bytes[0]` was written
//! FROM `ENVELOPE_MAGIC`, and `domain_registry`'s
//! `all_domain_tags_are_prefix_free` builds its vector FROM the constants and
//! checks only a value-independent structural property.
//!
//! # Discipline
//!
//! A pin records what IS. If a pin here fails, the change is a v1 wire-format /
//! key-derivation break. Do NOT edit the expected value to match the code —
//! either revert the constant or take the break through an explicit freeze
//! decision.
//!
//! # Scope boundary
//!
//! The domain-separation / KDF-context tag corpus is NOT pinned here. It is
//! owned in full — including the three `pub(crate)` tags an integration test
//! cannot name — by `tests/f_dt_1_domain_tag_absolute_byte_pins.rs`.
//! `DAK_HKDF_INFO_TAG` is likewise already pinned at
//! `tests/f_va_2_argon2id_dak_derivation.rs`. The
//! `SigCodepoint`/`HashCodepoint`/`CipherSuiteCodepoint` associated constants
//! are already pinned at `tests/canonical_bytes_v1_codepoints_and_aad.rs` and
//! are not repeated; the free-standing `registry::*` integers below are a
//! DIFFERENT, previously-unpinned set.

use benten_crypto_suite::aead::{
    ENVELOPE_FORMAT_VERSION_V1 as AEAD_ENVELOPE_FORMAT_VERSION_V1,
    ENVELOPE_MAGIC as AEAD_ENVELOPE_MAGIC, IROH_BLOCK_SIZE, WHOLE_CONTENT_AEAD_THRESHOLD,
};
use benten_crypto_suite::cipher_suite::{
    ML_KEM_768_CT_LEN, ML_KEM_768_DK_LEN, ML_KEM_768_EK_LEN, ML_KEM_768_SS_LEN, X_WING_LABEL,
    X25519_PUBLIC_LEN, X25519_SECRET_LEN,
};
use benten_crypto_suite::envelope::{
    ENVELOPE_FORMAT_VERSION_V1, ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_MAGIC, MAX_NONCE_LEN,
};
use benten_crypto_suite::registry as cpr;
use benten_crypto_suite::swap_matrix::AUDIT_LANDED_PURE_PQ_FLAG;
use benten_crypto_suite::vault::{
    OWASP_DEFAULT, SYMMETRIC_AEAD_12B_CODEPOINT, VAULT_ARGON2_MAX_M_COST, VAULT_ARGON2_MAX_P_COST,
    VAULT_ARGON2_MAX_T_COST, VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, VAULT_XNONCE_LEN,
};

// ---------------------------------------------------------------------------
// Group 1 — envelope header bytes (src/envelope.rs).
//
// WHAT BREAKS IF THESE CHANGE: byte 0 and byte 1 of every encryption envelope.
// A peer on the old value typed-rejects everything from a peer on the new one.
// ---------------------------------------------------------------------------

#[test]
fn envelope_header_bytes_are_frozen() {
    assert_eq!(
        ENVELOPE_MAGIC, 0xae,
        "ENVELOPE_MAGIC is wire byte 0 of every envelope"
    );
    assert_eq!(
        ENVELOPE_FORMAT_VERSION_V1, 0x01,
        "ENVELOPE_FORMAT_VERSION_V1 is a frozen wire version byte"
    );
    assert_eq!(
        ENVELOPE_FORMAT_VERSION_V2, 0x02,
        "ENVELOPE_FORMAT_VERSION_V2 is a frozen wire version byte"
    );
    assert_eq!(
        MAX_NONCE_LEN, 24,
        "MAX_NONCE_LEN bounds the nonce field parsed from untrusted envelope bytes"
    );
}

/// GCS-16 — `ENVELOPE_MAGIC` and `ENVELOPE_FORMAT_VERSION_V1` are each defined
/// TWICE (`src/aead.rs:64,68` and `src/envelope.rs:47,50`). Both `aead::` copies
/// are in the frozen public-api baseline, and `aead::AeadEnvelope::to_wire_bytes`
/// writes wire byte 0 from the `aead::` copy while `src/vault.rs:447` writes its
/// frame magic from the `envelope::` copy — two homes, one wire byte.
///
/// The pin directly above covers only the `envelope::` home. `aead.rs:537`'s
/// `assert_eq!(bytes[0], ENVELOPE_MAGIC)` is a tautology its own doc-comment
/// admits: `bytes[0]` was written FROM that same constant. So nothing on the
/// crate compares the two homes to each other. The bytes agree today; this pins
/// that they STAY agreeing.
///
/// MUTATION THAT MUST MAKE THIS FAIL: change `ENVELOPE_MAGIC` in `src/aead.rs`
/// alone (e.g. `0xae` -> `0xaf`), leaving `src/envelope.rs` untouched. Every
/// AEAD seal/open round-trip stays green (both ends read the same `aead::`
/// copy), and `envelope_header_bytes_are_frozen` stays green (it reads the
/// `envelope::` copy) — the two homes have silently diverged onto different
/// wire bytes, and only this test sees it.
#[test]
fn dual_homed_envelope_header_constants_agree_across_both_homes() {
    assert_eq!(
        AEAD_ENVELOPE_MAGIC, ENVELOPE_MAGIC,
        "aead::ENVELOPE_MAGIC and envelope::ENVELOPE_MAGIC are the SAME wire byte 0 \
         written by two different encoders (aead::AeadEnvelope::to_wire_bytes vs \
         vault::serialize_vault); they MUST NOT diverge (GCS-16)"
    );
    assert_eq!(
        AEAD_ENVELOPE_FORMAT_VERSION_V1, ENVELOPE_FORMAT_VERSION_V1,
        "aead::ENVELOPE_FORMAT_VERSION_V1 and envelope::ENVELOPE_FORMAT_VERSION_V1 are \
         the SAME wire byte 1 discriminator defined in two homes (GCS-16)"
    );
    // Absolute arm: a coordinated edit to BOTH homes would keep the equality
    // above green, so nail each home to its frozen literal too.
    assert_eq!(
        AEAD_ENVELOPE_MAGIC, 0xae,
        "the aead:: home of ENVELOPE_MAGIC is frozen at 0xae"
    );
    assert_eq!(
        AEAD_ENVELOPE_FORMAT_VERSION_V1, 0x01,
        "the aead:: home of ENVELOPE_FORMAT_VERSION_V1 is frozen at 0x01"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — AEAD chunking geometry (src/aead.rs).
//
// WHAT BREAKS IF THESE CHANGE: chunk boundaries move, so the per-chunk AAD
// (which commits to chunk_index and total_chunks) no longer matches; every
// previously-chunked ciphertext fails to open.
// ---------------------------------------------------------------------------

#[test]
fn aead_chunking_geometry_is_frozen() {
    assert_eq!(
        IROH_BLOCK_SIZE,
        16 * 1024,
        "IROH_BLOCK_SIZE is the per-chunk AEAD chunk size — moving it re-cuts every chunk boundary"
    );
    assert_eq!(IROH_BLOCK_SIZE, 16_384, "IROH_BLOCK_SIZE literal value");
    assert_eq!(
        WHOLE_CONTENT_AEAD_THRESHOLD,
        64 * 1024,
        "WHOLE_CONTENT_AEAD_THRESHOLD selects whole-vs-chunked mode — moving it changes which \
         AAD domain a given payload is sealed under"
    );
    assert_eq!(
        WHOLE_CONTENT_AEAD_THRESHOLD, 65_536,
        "WHOLE_CONTENT_AEAD_THRESHOLD literal value"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — primitive sizes + X-Wing combiner label (src/cipher_suite.rs,
// re-exported from src/mlkem.rs).
//
// WHAT BREAKS IF THESE CHANGE: X_WING_LABEL is APPENDED into the X-Wing
// combiner preimage per draft-connolly-cfrg-xwing-kem-10 5.3. A byte change
// silently forks every hybrid KEM shared secret. The ML-KEM / X25519 lengths
// are FIPS-203 / RFC-7748 fixed; a change means the code no longer implements
// the named algorithm.
// ---------------------------------------------------------------------------

#[test]
fn kem_primitive_sizes_and_xwing_label_are_frozen() {
    assert_eq!(
        X_WING_LABEL,
        [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c],
        "X_WING_LABEL is the 6-byte ASCII combiner label, APPENDED per \
         draft-connolly-cfrg-xwing-kem-10 5.3 — a change forks every hybrid shared secret"
    );
    assert_eq!(X_WING_LABEL, *br"\.//^\", "X_WING_LABEL ASCII cross-check");

    assert_eq!(X25519_PUBLIC_LEN, 32, "X25519 public key length (RFC 7748)");
    assert_eq!(X25519_SECRET_LEN, 32, "X25519 secret key length (RFC 7748)");

    assert_eq!(
        ML_KEM_768_EK_LEN, 1184,
        "ML-KEM-768 encapsulation key length (FIPS 203)"
    );
    assert_eq!(
        ML_KEM_768_CT_LEN, 1088,
        "ML-KEM-768 ciphertext length (FIPS 203)"
    );
    assert_eq!(
        ML_KEM_768_DK_LEN, 2400,
        "ML-KEM-768 decapsulation key length (FIPS 203)"
    );
    assert_eq!(
        ML_KEM_768_SS_LEN, 32,
        "ML-KEM-768 shared secret length (FIPS 203)"
    );
}

// ---------------------------------------------------------------------------
// Group 4 — Layer-A vault parameters (src/vault.rs).
//
// WHAT BREAKS IF THESE CHANGE: OWASP_DEFAULT feeds Argon2id; a param change
// yields a different DAK, so every existing vault fails to unlock. The
// VAULT_ARGON2_MAX_* ceilings are the Compromise #28 / META #629 DoS guard on
// ATTACKER-SUPPLIED header params — widening them re-opens a memory-exhaustion
// DoS, and because the guard's own tests build fixtures from the constants,
// widening is currently invisible to CI.
// ---------------------------------------------------------------------------

#[test]
fn vault_argon2_params_and_ceilings_are_frozen() {
    assert_eq!(
        OWASP_DEFAULT.m_cost, 19_456,
        "Argon2id memory cost (KiB) — changes the DAK"
    );
    assert_eq!(
        OWASP_DEFAULT.t_cost, 2,
        "Argon2id time cost — changes the DAK"
    );
    assert_eq!(
        OWASP_DEFAULT.p_cost, 1,
        "Argon2id parallelism — changes the DAK"
    );

    assert_eq!(
        VAULT_ARGON2_MAX_M_COST, 65_536,
        "DoS ceiling on attacker-supplied Argon2id m_cost (KiB) — widening re-opens memory exhaustion"
    );
    assert_eq!(
        VAULT_ARGON2_MAX_T_COST, 10,
        "DoS ceiling on attacker-supplied Argon2id t_cost — widening re-opens CPU pinning"
    );
    assert_eq!(
        VAULT_ARGON2_MAX_P_COST, 4,
        "DoS ceiling on attacker-supplied Argon2id p_cost"
    );

    assert_eq!(
        VAULT_XNONCE_LEN, 24,
        "vault XChaCha20 nonce length — a wire field width"
    );
    assert_eq!(
        VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, 0x6100,
        "vault envelope codepoint — wire-locked"
    );
    assert_eq!(
        SYMMETRIC_AEAD_12B_CODEPOINT, 0x6101,
        "12-byte-nonce symmetric AEAD codepoint — wire-locked"
    );
}

// ---------------------------------------------------------------------------
// Group 5 — free-standing codepoint registry integers (src/registry.rs).
//
// WHAT BREAKS IF THESE CHANGE: these are the on-the-wire envelope codepoints
// and the reserved band bases. Reusing or moving one is the #1341 0x647b
// incident class. Note these are DISTINCT from the enum associated constants
// already pinned in canonical_bytes_v1_codepoints_and_aad.rs — nothing
// currently ties the two together, so this group also cross-checks the pair.
// ---------------------------------------------------------------------------

#[test]
fn codepoint_registry_integers_are_frozen() {
    assert_eq!(cpr::VAULT_ENVELOPE, 0x6100);
    assert_eq!(cpr::SYMMETRIC_AEAD_12B, 0x6101);

    assert_eq!(cpr::CIPHER_CLASSICAL_X25519, 0x6400);
    assert_eq!(cpr::CIPHER_HYBRID_X25519_MLKEM768, 0x647a);
    assert_eq!(cpr::CIPHER_HYBRID_MLKEM768_HQC, 0x647b);
    assert_eq!(cpr::CIPHER_PURE_PQ_MLKEM768_ONLY, 0x647c);

    assert_eq!(cpr::DEVICE_LINK_BAND_BASE, 0x6310);
    assert_eq!(cpr::REMOTE_PERMISSION_BAND_BASE, 0x6320);
    assert_eq!(cpr::MLS_APPLICATION_BASE, 0x6380);
    assert_eq!(cpr::MLS_WELCOME_BASE, 0x6390);
    assert_eq!(cpr::CGKA_COMMIT_BASE, 0x63A0);
    assert_eq!(cpr::BIRD_OF_PREY_BASE, 0x63B0);
    assert_eq!(cpr::DRAFT_PRABEL_BASE, 0x63C0);

    assert_eq!(cpr::LAYER_C_DROP, 0x6500);
    assert_eq!(cpr::DROP_TO_RECIPIENT_SEALED_SENDER, 0x6510);
    assert_eq!(cpr::LAYER_C_DROP_MULTI_RECIPIENT, 0x6520);

    assert_eq!(cpr::MEMBERSHIP_SET_ENCRYPTION, 0x6600);
    assert_eq!(cpr::MEMBERSHIP_SET_GROUP_MULTI_STANZA, 0x6610);
    assert_eq!(cpr::MEMBERSHIP_SET_SUBSET_REF, 0x6620);

    assert_eq!(cpr::LIFECYCLE_BAND_BASE, 0x6700);
    assert_eq!(cpr::EXPERIMENTAL_BASE, 0xFE00);
    assert_eq!(cpr::EXTENDED_CODEPOINT_ESCAPE, 0xFFFF);

    assert_eq!(
        *cpr::BENTEN_ENVELOPE_RANGE.start(),
        0x6100,
        "Benten envelope codepoint band start — wire-locked"
    );
    assert_eq!(
        *cpr::BENTEN_ENVELOPE_RANGE.end(),
        0x6FFF,
        "Benten envelope codepoint band end — wire-locked"
    );
}

/// Cross-check: the free-standing registry integers and the enum associated
/// constants are two independent declarations of the same wire values. Nothing
/// in the tree previously tied them together, so they could silently diverge.
#[test]
fn registry_integers_agree_with_codepoint_enum() {
    use benten_crypto_suite::codepoint::CipherSuiteCodepoint as C;

    assert_eq!(cpr::CIPHER_CLASSICAL_X25519, C::CLASSICAL_X25519.raw());
    assert_eq!(
        cpr::CIPHER_HYBRID_X25519_MLKEM768,
        C::HYBRID_X25519_MLKEM768.raw()
    );
    assert_eq!(
        cpr::CIPHER_HYBRID_MLKEM768_HQC,
        C::HYBRID_MLKEM768_HQC.raw()
    );
    assert_eq!(
        cpr::CIPHER_PURE_PQ_MLKEM768_ONLY,
        C::PURE_PQ_MLKEM768_ONLY.raw()
    );
}

// ---------------------------------------------------------------------------
// Group 6 — swap-matrix safety gate (src/swap_matrix.rs).
//
// WHAT BREAKS IF THIS CHANGES: flipping this to `true` promotes the pure-PQ
// arm to a usable sole trust path BEFORE the independent audit lands, violating
// the C-GM-AUDIT / NF-2 gate. It is a one-token edit with no baseline diff.
// ---------------------------------------------------------------------------

#[test]
// `clippy::assertions_on_constants` fires here because the asserted expression
// IS a constant — which is precisely the point. This pin exists to make a
// one-token edit of that constant fail CI. A `const { assert!(..) }` block
// (clippy's suggestion) would move the failure to compile time and lose the
// named test + message, so the allow is the correct resolution, not a
// weakening.
#[allow(clippy::assertions_on_constants)]
fn pure_pq_audit_gate_flag_is_frozen_false() {
    assert!(
        !AUDIT_LANDED_PURE_PQ_FLAG,
        "AUDIT_LANDED_PURE_PQ_FLAG must remain false until the independent ml-kem/ml-dsa audit \
         lands (NF-2 / C-GM-AUDIT). Flipping it makes unaudited PQC a SOLE trust path."
    );
}
