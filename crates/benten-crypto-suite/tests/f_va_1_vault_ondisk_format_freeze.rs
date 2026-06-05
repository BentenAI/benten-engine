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
use benten_crypto_suite::vault::{
    DecodedVault, SYMMETRIC_AEAD_12B_CODEPOINT, VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT,
    VAULT_XNONCE_LEN as FROZEN_XNONCE_LEN, VaultError, VaultPayload, decode_vault,
    decode_vault_strict, serialize_vault,
};

/// Adapter: the real `serialize_vault` returns `Result`; the test calls a
/// `_for_test`-shaped fn returning the bytes (panics on the infallible path).
fn serialize_vault_for_test(payload: &VaultPayload, dak: &[u8; 32]) -> Vec<u8> {
    serialize_vault(payload, dak).expect("vault serialize is infallible for the fixture")
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
