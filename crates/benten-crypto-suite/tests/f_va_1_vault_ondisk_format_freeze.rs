//! **F-VA-1 — Layer-A vault on-disk DAG-CBOR format freeze; XChaCha20 (m-4).**
//!
//! ADDL R3 wave **W1-crypto-kat** (Tier-1 fan-out after W0-crypto-canary).
//! Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-5 F-VA-1 (merges CE-E1 + WF-A3/A4-vault
//!     + T-J3) + §4 P0 freeze-gating.
//!   - R0 §3.1 Layer-A (`${BENTEN_DATA_DIR}/vault.cbor`; payload CBOR
//!     `{ k_principal:[u8;32], user_did_signing_key, user_did_creation_time:u64 }`;
//!     16-byte salt; XChaCha20-Poly1305 24-byte nonce; `SymmetricAeadXNonce`).
//!   - R0 §4.1 frozen-surface rows: `SymmetricAead [u8;12]` + `SymmetricAeadXNonce
//!     [u8;24]` BOTH ship at v1-beta (U12/U32); vault uses the XNonce variant.
//!   - R0 §4.0 codepoint table: vault band `0x6100`.
//!   - CLAUDE.md baked-in #5 (codepoint-dispatched cipher-suite envelope;
//!     never-fork-primitives).
//!
//! # What this pins (FREEZE-GATING)
//!
//! The vault is a frozen on-disk byte-format. A miss here = an untested frozen
//! byte = a permanent cross-version vault-decode incompatibility. The pins lock:
//!   1. nonce width == 24 (XChaCha20-Poly1305 — m-4: a 12-byte ChaCha20 nonce
//!      hits the 2^32 random-nonce birthday bound under the vault's heavy reseal
//!      load: K_principal rotation re-seals + multi-device key-wrap re-seals);
//!   2. the vault codepoint is `SymmetricAeadXNonce` (vault band `0x6100`), NOT
//!      the 12-byte `SymmetricAead` variant;
//!   3. a 12-byte nonce presented under the XNonce codepoint is REJECTED at
//!      decode (codepoint discriminates nonce width — U2 strict-decode);
//!   4. the CBOR payload field-order is canonical (k_principal, then
//!      user_did_signing_key, then user_did_creation_time) — a re-serialize is
//!      byte-identical.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! At baseline NONE of {`VaultFile`, `EncryptedEnvelope`, `SymmetricAeadXNonce`,
//! `VaultCodepoint`} exist (`grep EncryptedEnvelope crates/benten-crypto-suite`
//! → ZERO at HEAD; the vault is a STUB per R0 §3.1 "current `K_principal` is a
//! STUB"). Per the wave-independence rule (each wave is self-contained for
//! parallel safety — NO cross-wave module dependency; W0 mints
//! `EncryptedEnvelope` but THIS wave must compile without depending on W0's new
//! module), this file commits a LOCAL `f_va_1_stub` module so it compiles green
//! at baseline behind `#[ignore]`. The R5 closing wave MUST:
//!   1. DELETE the local `f_va_1_stub` module,
//!   2. INSERT real imports against the LIVE vault + `EncryptedEnvelope` surface,
//!   3. UN-IGNORE each test,
//!   4. Verify the pinned vault bytes / nonce-width / codepoint PASS green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Each pin drives a PRODUCTION serialize/decode call site over a fixed
//! deterministic fixture + asserts an OBSERVABLE byte-level consequence. A stub
//! that emits a 12-byte nonce, or tags the vault `SymmetricAead`, or reorders
//! the CBOR fields, produces bytes ≠ the pinned reference. The stub deliberately
//! emits the WRONG shape (12-byte nonce, V1 layout) so the assertions FAIL until
//! R5 wires the real XNonce vault — never a silent-green pim-18 SHAPE-trap.
//!
//! # M-20 / Wave-0 DAG edge
//!
//! This family authors the vault against **V2 + BE + `EncryptedEnvelope`** from
//! the first commit. There is no surviving V1 vault golden vector.

#![allow(dead_code)]

// R5: wired to the LIVE vault surface (serde_ipld_dagcbor + XChaCha20-Poly1305).
// R11 MC-6: the frame header now persists the Argon2id salt + params.
use benten_crypto_suite::vault::{
    Argon2idParams, DAK_HKDF_INFO_TAG, DecodedVault, OWASP_DEFAULT, SYMMETRIC_AEAD_12B_CODEPOINT,
    VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, VAULT_XNONCE_LEN as FROZEN_XNONCE_LEN, VaultError,
    VaultPayload, decode_vault, decode_vault_strict, derive_dak, open_vault, serialize_vault,
};

/// A deterministic fixture salt (R11 MC-6: persisted in the frame header).
fn fixture_salt() -> [u8; 16] {
    [0x5Au8; 16]
}

/// Adapter: the real `serialize_vault` returns `Result`; the test calls a
/// `_for_test`-shaped fn returning the bytes (panics on the infallible path).
/// R11 MC-6: threads the fixture salt + OWASP params into the frame header.
fn serialize_vault_for_test(payload: &VaultPayload, dak: &[u8; 32]) -> Vec<u8> {
    serialize_vault(payload, &fixture_salt(), OWASP_DEFAULT, dak)
        .expect("vault serialize is infallible for the fixture")
}

fn decode_vault_for_test(bytes: &[u8], dak: &[u8; 32]) -> Result<DecodedVault, VaultError> {
    decode_vault(bytes, dak)
}

fn decode_vault_strict_for_test(codepoint: u16, nonce_len: usize) -> Result<(), VaultError> {
    decode_vault_strict(codepoint, nonce_len)
}

fn canonical_cbor_for_test(payload: &VaultPayload) -> Vec<u8> {
    payload.to_canonical_cbor()
}

/// A deterministic vault fixture (NOT a real keypair) so the format pins are
/// hermetic. R5 swaps in real `K_principal` + signing-key bytes without changing
/// the pin semantics.
fn fixture_payload() -> VaultPayload {
    VaultPayload {
        k_principal: [0x11u8; 32],
        // A fixed stand-in for `HybridSigningKeySerialized` (Ed25519⊕ML-DSA-65).
        user_did_signing_key: vec![0x22u8; 64],
        user_did_creation_time: 0x0000_0000_6543_2100u64,
    }
}

fn fixture_dak() -> [u8; 32] {
    [0x33u8; 32]
}

/// F-VA-1 (a) — the vault uses a 24-byte XChaCha20-Poly1305 nonce (m-4).
///
/// Drives the production serialize → decode round-trip + asserts the decoded
/// envelope's nonce width is exactly 24. would-FAIL-if-no-op'd: the stub emits a
/// 12-byte nonce, so the assertion fails until R5 wires the XNonce variant. A
/// 12-byte ChaCha20 nonce on the reseal-heavy vault site hits the 2^32
/// birthday bound — this pin is the load-bearing m-4 defense.
#[test]
fn vault_uses_24_byte_xchacha20_nonce() {
    let payload = fixture_payload();
    let dak = fixture_dak();
    let bytes = serialize_vault_for_test(&payload, &dak);
    let decoded = decode_vault_for_test(&bytes, &dak).expect("vault decode MUST succeed");
    assert_eq!(
        decoded.nonce.len(),
        FROZEN_XNONCE_LEN,
        "Layer-A vault MUST use a 24-byte XChaCha20-Poly1305 nonce (m-4): the \
         vault is reseal-heavy (K_principal rotation + multi-device key-wrap), \
         and a 12-byte random nonce hits the 2^32 birthday bound. would-FAIL \
         while the stub emits a 12-byte nonce."
    );
}

/// F-VA-1 (b) — the vault wire codepoint is `SymmetricAeadXNonce` (vault band
/// `0x6100`), NOT the 12-byte `SymmetricAead` sibling.
///
/// would-FAIL-if-no-op'd: the stub tags the vault with the 12-byte sibling
/// codepoint, so this fails until R5 dispatches the XNonce variant.
#[test]
fn vault_envelope_carries_xnonce_codepoint_0x6100() {
    let payload = fixture_payload();
    let dak = fixture_dak();
    let bytes = serialize_vault_for_test(&payload, &dak);
    let decoded: DecodedVault =
        decode_vault_for_test(&bytes, &dak).expect("vault decode MUST succeed");
    assert_eq!(
        decoded.codepoint, VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT,
        "the vault MUST dispatch through the SymmetricAeadXNonce codepoint \
         (0x6100 vault band); would-FAIL while the stub tags it with the \
         12-byte SymmetricAead sibling 0x{SYMMETRIC_AEAD_12B_CODEPOINT:04x}"
    );
    assert_ne!(
        decoded.codepoint, SYMMETRIC_AEAD_12B_CODEPOINT,
        "the vault codepoint MUST NOT be the 12-byte SymmetricAead sibling"
    );
}

/// F-VA-1 (c) — strict-decode: an XNonce-tagged vault carrying a 12-byte nonce
/// is REJECTED (U2 — codepoint discriminates nonce width; no silent coercion).
///
/// Negative-control. would-FAIL-if-no-op'd: the stub silently accepts the
/// width mismatch (the bug R5 closes); the real strict decode returns
/// `NonceWidthMismatch`.
#[test]
fn xnonce_codepoint_with_12_byte_nonce_is_rejected() {
    let outcome = decode_vault_strict_for_test(
        VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT,
        /* nonce_len */ 12,
    );
    assert!(
        matches!(outcome, Err(VaultError::NonceWidthMismatch)),
        "a vault tagged SymmetricAeadXNonce (0x6100) carrying a 12-byte nonce \
         MUST be typed-rejected (U2 strict-decode; codepoint discriminates \
         nonce width — NO silent coercion to a 12-byte read). would-FAIL while \
         the stub accepts the mismatch; got {outcome:?}"
    );
}

/// R13 F-12 — vault format-version policy pin (NAMED-carry Row D-72). The
/// production `parse_vault_frame` currently rejects ANY `bytes[1] !=
/// ENVELOPE_FORMAT_VERSION_V2` UNIFORMLY with `MalformedCbor` — it does NOT
/// yet distinguish `got > V2` ("newer vault; the reader is stale, please
/// upgrade") from `got < V2` ("stale vault; reject"). There is no extant V3,
/// so a differentiated policy is deferred to v1-Composing (Row D-72). This
/// pin LOCKS the current uniform-reject baseline so the future differentiated
/// policy is a DELIBERATE change against a documented pin, not a silent drift.
/// would-FAIL if a future edit changed the not-V2 rejection shape without
/// updating this pin + Row D-72.
#[test]
fn f_va_12_non_v2_version_byte_uniformly_rejects_baseline() {
    let payload = fixture_payload();
    let dak = fixture_dak();
    let bytes = serialize_vault_for_test(&payload, &dak);
    // Sanity: the honest frame decodes.
    assert!(
        decode_vault_for_test(&bytes, &dak).is_ok(),
        "F-12: the honest V2 vault frame MUST decode (positive control)."
    );
    // byte[1] is the envelope format-version byte (parse_vault_frame :357).
    // Flip it to a STALE (< V2) value and a NEWER (> V2) value; BOTH must
    // currently reject with the SAME MalformedCbor (uniform policy).
    for injected in [0x00u8, 0x01u8, 0x03u8, 0xFFu8] {
        let mut tampered = bytes.clone();
        tampered[1] = injected;
        let outcome = decode_vault_for_test(&tampered, &dak);
        assert!(
            matches!(outcome, Err(VaultError::MalformedCbor)),
            "F-12 (Row D-72): a vault whose format-version byte is 0x{injected:02x} \
             (NOT V2) MUST currently reject UNIFORMLY with MalformedCbor — the \
             got>V2 (\"newer, upgrade\") vs got<V2 (\"stale, reject\") \
             differentiated policy is DEFERRED (no extant V3). If this baseline \
             changes, update Row D-72. got {outcome:?}"
        );
    }
}

/// The FROZEN canonical DAG-CBOR golden hex for `fixture_payload()`
/// (F4-038). Computed ONCE from the canonical encoder (definite-length map
/// of 3 pairs; field order k_principal ‖ user_did_signing_key ‖
/// user_did_creation_time; bstr values; the u64 creation-time `0x65432100`
/// encodes as the 5-byte CBOR uint `1a 65 43 21 00` — BIG-ENDIAN per M-19).
/// This is an ABSOLUTE byte vector, NOT a self-referential re-encode — any
/// field reorder / encoding / endianness drift flips this pin.
/// R5 confirms-or-deliberately-updates against the real
/// `serde_ipld_dagcbor` encoder (M-20).
const VAULT_PAYLOAD_GOLDEN_HEX: &str = "a36b6b5f7072696e636970616c5820111111111111111111111111111111111111111111111111111111111111111174757365725f6469645f7369676e696e675f6b657958402222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222276757365725f6469645f6372656174696f6e5f74696d651a65432100";

/// Lowercase-hex-encode a byte slice (no external dep).
fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// F-VA-1 (d) — the CBOR payload is canonical + re-serialize is byte-identical
/// + the EXACT field-order bytes are frozen (F4-038 golden-hex; field-order
/// freeze: k_principal, user_did_signing_key, user_did_creation_time).
///
/// would-FAIL-if-no-op'd: a field-reorder, a non-big-endian integer, or any
/// encoding drift produces bytes ≠ the frozen `VAULT_PAYLOAD_GOLDEN_HEX`. The
/// golden-hex literal is an ABSOLUTE frozen vector (not `enc(x)==enc(x)`).
#[test]
fn vault_cbor_payload_canonical_and_reserializes_byte_identical() {
    let payload = fixture_payload();
    let first = canonical_cbor_for_test(&payload);
    let second = canonical_cbor_for_test(&payload);
    assert!(
        !first.is_empty(),
        "canonical CBOR of the vault payload MUST be non-empty (a frozen \
         format has bytes)"
    );
    assert_eq!(
        first, second,
        "canonical CBOR re-serialize MUST be byte-identical (deterministic \
         field order: k_principal, user_did_signing_key, \
         user_did_creation_time); would-FAIL on a non-canonical encoder"
    );

    // F4-038 golden-hex: freeze the EXACT field-order bytes. A field reorder
    // (e.g. user_did_creation_time first) or a little-endian creation-time
    // produces a different hex string → this pin flips. This is the
    // freeze-gating field-order assertion the prior self-equality arm lacked.
    assert_eq!(
        to_hex(&first),
        VAULT_PAYLOAD_GOLDEN_HEX,
        "vault payload canonical-DAG-CBOR bytes MUST match the frozen golden-hex \
         (field order k_principal ‖ user_did_signing_key ‖ user_did_creation_time; \
         big-endian creation-time per M-19); a reorder/encoding/endianness drift flips this"
    );
}

/// F-VA-1 (e) — R11 MC-6 frame-header freeze: the vault frame persists the
/// 16-byte Argon2id salt + `{m_cost,t_cost,p_cost}` (each u32 BE) in the header
/// BEFORE the nonce, at the fixed offsets
/// `magic(1) | V2(1) | codepoint(2) | salt(16) | m_cost(4) | t_cost(4) |
///  p_cost(4) | nonce_len(1) | nonce(24) | ct`.
///
/// would-FAIL-if-reverted: if the frame dropped salt+params, the header bytes
/// at offsets 4..32 would not equal the sealed salt/params and `nonce_len`
/// would not sit at offset 32.
#[test]
fn vault_frame_persists_salt_and_params_in_header() {
    let payload = fixture_payload();
    let dak = fixture_dak();
    let salt = fixture_salt();
    let params = OWASP_DEFAULT;
    let bytes = serialize_vault(&payload, &salt, params, &dak)
        .expect("vault serialize is infallible for the fixture");

    // Fixed header offsets (R11 MC-6).
    assert_eq!(
        &bytes[4..20],
        &salt[..],
        "salt persisted at header offset 4..20"
    );
    assert_eq!(
        u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
        params.m_cost,
        "m_cost (u32 BE) persisted at offset 20"
    );
    assert_eq!(
        u32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
        params.t_cost,
        "t_cost (u32 BE) persisted at offset 24"
    );
    assert_eq!(
        u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
        params.p_cost,
        "p_cost (u32 BE) persisted at offset 28"
    );
    assert_eq!(
        bytes[32] as usize, FROZEN_XNONCE_LEN,
        "nonce_len byte sits at offset 32 (after the 32-byte header) and == 24"
    );

    // The decode path surfaces the header salt+params.
    let decoded = decode_vault(&bytes, &dak).expect("decode succeeds");
    assert_eq!(decoded.salt, salt);
    assert_eq!(decoded.params, params);
}

/// F-VA-1 (f) — R11 MC-6 self-containment: `vault.cbor` bytes + password ALONE
/// re-derive the DAK (salt+params sourced from the frame header) and decrypt.
/// This is the load-bearing property MC-6 restores — before it, the salt+params
/// lived only in an un-persisted in-RAM struct.
///
/// would-FAIL-on-revert: without salt+params in the frame, `open_vault` could
/// not re-derive the DAK from the bytes alone.
#[test]
fn vault_opens_from_bytes_and_password_alone() {
    let payload = fixture_payload();
    // Runtime-built inputs (CodeQL hard-coded-crypto hygiene).
    let password: Vec<u8> = (0u8..20)
        .map(|i| i.wrapping_mul(9).wrapping_add(2))
        .collect();
    let salt: [u8; 16] = core::array::from_fn(|i| (i as u8).wrapping_add(0x40));
    let params: Argon2idParams = OWASP_DEFAULT;

    let dak = derive_dak(&password, &salt, params, DAK_HKDF_INFO_TAG);
    let bytes = serialize_vault(&payload, &salt, params, dak.expose())
        .expect("vault serialize is infallible for the fixture");

    // Open with the frame bytes + password ALONE — no external salt.
    let decoded: DecodedVault = open_vault(&bytes, &password, DAK_HKDF_INFO_TAG)
        .expect("vault.cbor bytes + password alone MUST decrypt (MC-6)");
    assert_eq!(decoded.payload, payload);
    assert_eq!(
        decoded.salt, salt,
        "the recovered salt matches the sealed salt"
    );
    assert_eq!(decoded.params, params);

    // A wrong password fails closed.
    let mut wrong = password.clone();
    wrong[0] ^= 0xAA;
    assert!(matches!(
        open_vault(&bytes, &wrong, DAK_HKDF_INFO_TAG),
        Err(VaultError::AeadFailed)
    ));
}
