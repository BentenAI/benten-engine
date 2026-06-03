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

/// SELF-CONTAINED stub-shim (R5 deletes this whole module + wires the LIVE
/// `benten_crypto_suite::vault` + `EncryptedEnvelope::SymmetricAeadXNonce`
/// surface).
mod f_va_1_stub {
    /// The frozen vault wire codepoint (R0 §4.0 vault band `0x6100`).
    pub const VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT: u16 = 0x6100;

    /// The 12-byte `SymmetricAead` (ChaCha20-Poly1305) sibling codepoint —
    /// a RATIFIED, frozen, shipping v1-beta variant (Ben ruling 3,
    /// 2026-06-02; "ship both" per R0.3 §4.1 rows `SymmetricAead [u8;12]` +
    /// `SymmetricAeadXNonce [u8;24]`). Its assigned home is the Layer-A
    /// vault band `0x6100..0x61FF` (R0.3 §4.0). It is NOT the vault's own
    /// codepoint (the vault uses the 24-byte XNonce variant at `0x6100`);
    /// here it serves as the foil for the nonce-width-discrimination pin.
    pub const SYMMETRIC_AEAD_12B_CODEPOINT: u16 = 0x6101;

    /// XChaCha20-Poly1305 nonce width (m-4). STUB emits the WRONG width so the
    /// red-phase pin fails until R5.
    pub const STUB_NONCE_LEN: usize = 12; // R5 makes the real vault emit 24.

    /// The frozen XNonce width the vault MUST emit.
    pub const FROZEN_XNONCE_LEN: usize = 24;

    /// Vault CBOR payload (R0 §3.1). Field order is FROZEN: k_principal,
    /// user_did_signing_key, user_did_creation_time.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct VaultPayload {
        pub k_principal: [u8; 32],
        pub user_did_signing_key: Vec<u8>,
        pub user_did_creation_time: u64,
    }

    /// A decoded vault envelope (the on-disk `EncryptedEnvelope::
    /// SymmetricAeadXNonce` after AEAD-open). Carries the wire codepoint + the
    /// actual nonce bytes used so the format-freeze pins can inspect them.
    #[derive(Debug, Clone)]
    pub struct DecodedVault {
        pub codepoint: u16,
        pub nonce: Vec<u8>,
        pub payload: VaultPayload,
    }

    /// Production serialize: build the on-disk vault envelope bytes for a fixed
    /// payload under a fixed DAK + salt. STUB returns an empty Vec (deliberately
    /// NOT a real serialization) so the byte-pins FAIL until R5.
    pub fn serialize_vault_for_test(_payload: &VaultPayload, _dak: &[u8; 32]) -> Vec<u8> {
        Vec::new()
    }

    /// Production decode of vault envelope bytes. STUB returns a `DecodedVault`
    /// with the WRONG nonce width + WRONG codepoint so the freeze pins fail.
    pub fn decode_vault_for_test(
        _bytes: &[u8],
        _dak: &[u8; 32],
    ) -> Result<DecodedVault, VaultError> {
        Ok(DecodedVault {
            codepoint: SYMMETRIC_AEAD_12B_CODEPOINT,
            nonce: vec![0u8; STUB_NONCE_LEN],
            payload: VaultPayload {
                k_principal: [0u8; 32],
                user_did_signing_key: Vec::new(),
                user_did_creation_time: 0,
            },
        })
    }

    /// Strict-decode: a vault tagged `SymmetricAeadXNonce` (`0x6100`) carrying a
    /// 12-byte nonce MUST be rejected (U2 — codepoint discriminates nonce
    /// width). STUB accepts it (the bug R5 closes); the negative pin asserts the
    /// real decode rejects.
    pub fn decode_vault_strict_for_test(
        codepoint: u16,
        nonce_len: usize,
    ) -> Result<(), VaultError> {
        // STUB: silently accepts the mismatch (RED-PHASE bug). R5 makes this
        // return Err(VaultError::NonceWidthMismatch) when codepoint==XNonce &&
        // nonce_len != 24.
        let _ = (codepoint, nonce_len);
        Ok(())
    }

    /// Re-serialize a decoded payload to canonical DAG-CBOR.
    ///
    /// **F4-038 golden-hex (per the R4-fix GOLDEN-HEX procedure):** the
    /// prior stub returned an empty Vec, so the format-freeze test could
    /// only assert self-equality (`first == second`) — it froze ZERO bytes
    /// and pinned nothing about the field ORDER. This stub now emits a
    /// REAL deterministic canonical DAG-CBOR encoding (definite-length,
    /// canonical map-key order, big-endian integers per M-19) so the
    /// `VAULT_PAYLOAD_GOLDEN_HEX` literal below freezes the exact field-order
    /// bytes. Any field reorder / encoding / endianness drift flips the pin.
    /// R5 confirms-or-deliberately-updates the frozen literal against the
    /// real `serde_ipld_dagcbor` encoder (M-20).
    ///
    /// Field order is FROZEN per R0.3 §3.1: k_principal, then
    /// user_did_signing_key, then user_did_creation_time. The canonical
    /// DAG-CBOR map-key order (length-first, then bytewise) happens to
    /// coincide with this declaration order for these three keys.
    pub fn canonical_cbor_for_test(payload: &VaultPayload) -> Vec<u8> {
        // Minimal hand-rolled canonical CBOR (no serde dep in the stub).
        fn uint(major: u8, n: u64) -> Vec<u8> {
            let m = major << 5;
            if n < 24 {
                vec![m | (n as u8)]
            } else if n < 0x100 {
                vec![m | 24, n as u8]
            } else if n < 0x1_0000 {
                let b = (n as u16).to_be_bytes();
                vec![m | 25, b[0], b[1]]
            } else if n < 0x1_0000_0000 {
                let b = (n as u32).to_be_bytes();
                vec![m | 26, b[0], b[1], b[2], b[3]]
            } else {
                let b = n.to_be_bytes();
                let mut v = vec![m | 27];
                v.extend_from_slice(&b);
                v
            }
        }
        fn tstr(s: &str) -> Vec<u8> {
            let mut out = uint(3, s.len() as u64);
            out.extend_from_slice(s.as_bytes());
            out
        }
        fn bstr(b: &[u8]) -> Vec<u8> {
            let mut out = uint(2, b.len() as u64);
            out.extend_from_slice(b);
            out
        }

        let mut out = uint(5, 3); // map of 3 pairs
        // Pair 1: "k_principal" => bstr(k_principal)
        out.extend(tstr("k_principal"));
        out.extend(bstr(&payload.k_principal));
        // Pair 2: "user_did_signing_key" => bstr(signing_key)
        out.extend(tstr("user_did_signing_key"));
        out.extend(bstr(&payload.user_did_signing_key));
        // Pair 3: "user_did_creation_time" => u64 (BE per M-19)
        out.extend(tstr("user_did_creation_time"));
        out.extend(uint(0, payload.user_did_creation_time));
        out
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum VaultError {
        NonceWidthMismatch,
        UnknownCodepoint,
        MalformedCbor,
    }
}

use f_va_1_stub::{
    DecodedVault, FROZEN_XNONCE_LEN, SYMMETRIC_AEAD_12B_CODEPOINT,
    VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, VaultError, VaultPayload, canonical_cbor_for_test,
    decode_vault_for_test, decode_vault_strict_for_test, serialize_vault_for_test,
};

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
#[ignore = "RED-PHASE: F-VA-1 — vault MUST use a 24-byte XChaCha20-Poly1305 nonce (m-4 reseal-birthday defense); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-VA-1 — vault envelope MUST carry the SymmetricAeadXNonce codepoint 0x6100 (NOT the 12-byte sibling); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-VA-1 — XNonce codepoint + 12-byte nonce MUST be typed-rejected (U2 strict-decode); un-ignore at R5"]
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
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
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
#[ignore = "RED-PHASE: F-VA-1 — vault CBOR payload canonical + re-serialize byte-identical + frozen golden-hex field-order pin; un-ignore at R5"]
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
