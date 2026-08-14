//! S-3 closure — frozen const VALUE pins for `benten-drop`.
//!
//! `docs/public-api/benten-drop.txt` records these by type only, so
//! `ENVELOPE_FORMAT_VERSION: u8 = 2` -> `= 3` is an EMPTY baseline diff on a
//! frozen wire version byte.
//!
//! Existing coverage that does NOT substitute for these pins: the mirror test
//! `layer_c_cek_contexts_match_central_registry` (src/layer_c.rs) asserts
//! mirror-A == mirror-B, which catches divergence but not a coordinated sweep
//! that edits both copies. The pins below fix the VALUE — and, unlike that
//! in-`src` unit test, they are INTEGRATION tests, so they are selectable by
//! the required `frozen-bytes corpus` CI job.

use benten_drop::bundle::{DROP_BUNDLE_MAX_SIZE_BYTES, DROP_BUNDLE_VERSION_V1};
use benten_drop::layer_c::group_posture::{
    MEMBERSHIP_GROUP_CEK_CONTEXT, MEMBERSHIP_SET_GROUP_MULTI_STANZA, SELF_DESCRIBING_CID_LEN,
};
use benten_drop::layer_c::{
    AAD_VERSION, DROP_TO_RECIPIENT_SEALED_SENDER, ENVELOPE_FORMAT_VERSION, HYBRID_X25519_MLKEM768,
    LAYER_C_CEK_CONTEXT, LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT, LAYER_C_GROUP_CEK_CONTEXT,
    MAX_LAYER_C_GROUP_RECIPIENTS, SENDER_AUTH_DOMAIN, SENDER_AUTH_SIG_CODEPOINT,
};

// ---------------------------------------------------------------------------
// Group 1 — Layer-C wire header bytes.
//
// WHAT BREAKS: version/AAD-version bytes are read by every receiver. A change
// makes existing Drops typed-reject on updated peers and vice versa.
// ---------------------------------------------------------------------------

#[test]
fn layer_c_wire_version_bytes_are_frozen() {
    assert_eq!(
        ENVELOPE_FORMAT_VERSION, 2,
        "Layer-C envelope format version is a frozen wire byte"
    );
    assert_eq!(
        AAD_VERSION, 0x01,
        "Layer-C AAD version is committed INTO the AEAD tag — a change invalidates every \
         previously-sealed Drop"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — Layer-C codepoints.
//
// WHAT BREAKS: envelope dispatch. Reusing/moving one is the #1341 class.
// ---------------------------------------------------------------------------

#[test]
fn layer_c_codepoints_are_frozen() {
    assert_eq!(
        HYBRID_X25519_MLKEM768, 0x647a,
        "v1-beta default KEM codepoint"
    );
    assert_eq!(LAYER_C_DROP, 0x6500);
    assert_eq!(DROP_TO_RECIPIENT_SEALED_SENDER, 0x6510);
    assert_eq!(LAYER_C_DROP_MULTI_RECIPIENT, 0x6520);
    assert_eq!(MEMBERSHIP_SET_GROUP_MULTI_STANZA, 0x6610);

    // Derived from the crypto-suite SSOT via `const fn raw()`. Pinning the
    // RESULT catches an SSOT move that a derivation-only reading would let
    // through silently.
    assert_eq!(
        SENDER_AUTH_SIG_CODEPOINT, 0x0001,
        "sealed-sender origin-auth signature codepoint (LAMPS composite) — wire-locked"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — Layer-C key-derivation + signature domain tags.
//
// WHAT BREAKS: silent, CI-green key-schedule fork. These bytes feed BLAKE3 CEK
// derivation and the origin-auth signature preimage; they never reach the wire,
// so no golden hex moves and every round-trip stays self-consistent.
// ---------------------------------------------------------------------------

#[test]
fn layer_c_domain_and_cek_context_bytes_are_frozen() {
    assert_eq!(
        SENDER_AUTH_DOMAIN, b"benten/layer-c/sealed-sender-origin-auth/v1",
        "sealed-sender origin-auth signature domain"
    );
    assert_eq!(
        LAYER_C_CEK_CONTEXT, b"benten-drop:layer-c:cek",
        "single-recipient CEK derivation context — a change makes every prior Drop undecryptable"
    );
    assert_eq!(
        LAYER_C_GROUP_CEK_CONTEXT, b"benten-drop:layer-c:group-cek",
        "group CEK derivation context"
    );
    assert_eq!(
        MEMBERSHIP_GROUP_CEK_CONTEXT, b"benten-drop:membership-group-cek",
        "0x6610 multi-stanza group CEK derivation context"
    );
}

// ---------------------------------------------------------------------------
// Group 4 — structural bounds.
//
// WHAT BREAKS: SELF_DESCRIBING_CID_LEN is an AAD field WIDTH — a change shifts
// every subsequent AAD byte. MAX_LAYER_C_GROUP_RECIPIENTS is the u16 roster
// ceiling enforced before a `u16::try_from(...).expect(...)`.
// ---------------------------------------------------------------------------

#[test]
fn layer_c_structural_bounds_are_frozen() {
    assert_eq!(
        SELF_DESCRIBING_CID_LEN, 36,
        "self-describing CIDv1 length (0x01 0x71 0x1e 0x20 || 32-byte BLAKE3) — an AAD field width"
    );
    assert_eq!(
        MAX_LAYER_C_GROUP_RECIPIENTS,
        u16::MAX as usize,
        "group roster ceiling is the u16 wire ceiling"
    );
    assert_eq!(
        MAX_LAYER_C_GROUP_RECIPIENTS, 65_535,
        "group roster ceiling literal value"
    );
}

// ---------------------------------------------------------------------------
// Group 5 — Drop bundle envelope.
//
// WHAT BREAKS: DROP_BUNDLE_VERSION_V1 is the on-the-wire bundle version;
// readers typed-reject unknown versions. DROP_BUNDLE_MAX_SIZE_BYTES is a
// bloat/non-canonical-CBOR canary (NOT a decode bound) — pinned so the canary's
// sensitivity cannot be silently relaxed.
// ---------------------------------------------------------------------------

#[test]
fn drop_bundle_envelope_constants_are_frozen() {
    assert_eq!(
        DROP_BUNDLE_VERSION_V1, 1,
        "Drop bundle on-the-wire version byte"
    );
    assert_eq!(
        DROP_BUNDLE_MAX_SIZE_BYTES,
        4 * 1024,
        "4 KiB size-envelope canary over the ~2688-byte Spike-G measurement"
    );
    assert_eq!(DROP_BUNDLE_MAX_SIZE_BYTES, 4_096, "literal value");
}
