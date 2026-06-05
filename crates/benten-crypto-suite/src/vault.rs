//! Layer-A vault — the at-rest `K_principal` + user-DID-signing-key store
//! (G-CORE-3 #1301; F-full Layer-A).
//!
//! # On-disk format (FROZEN; R0.5 §3.1)
//!
//! `${BENTEN_DATA_DIR}/vault.cbor` — a DAG-CBOR-encoded
//! [`EncryptedEnvelope`](crate::envelope::EncryptedEnvelope) at the vault
//! band codepoint [`VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT`] (`0x6100`). The
//! sealed payload is the canonical DAG-CBOR [`VaultPayload`]
//! `{ k_principal:[u8;32], user_did_signing_key, user_did_creation_time:u64 }`.
//!
//! # XChaCha20-Poly1305 24-byte nonce (m-4)
//!
//! The vault is **reseal-heavy** (K_principal rotation + multi-device
//! key-wrap re-seal), so a 12-byte random ChaCha20 nonce would hit the
//! 2^32 birthday bound. The vault uses the 24-byte XChaCha20-Poly1305 nonce
//! variant (`SymmetricAeadXNonce`) — the codepoint discriminates the nonce
//! width (a 12-byte nonce presented under the XNonce codepoint is
//! strict-rejected; U2).
//!
//! # DAK derivation (RFC 9106; F-VA-2)
//!
//! `DAK = HKDF-SHA256( Argon2id(pw, salt; m=19456, t=2, p=1),
//!   info = "benten-dak-v1" )`. The Argon2id params + the HKDF
//! domain-separation info-tag are part of the frozen derivation: a param /
//! info-tag drift changes the DAK and breaks every existing vault. The
//! info-tag is the codepoint slot for a future Argon2id-v2 param set.
//!
//! # Memory hygiene (F-VA-4; Compromise #36/#39)
//!
//! The unlocked `K_principal` lives in [`secrecy::SecretBox<[u8;32]>`] inside
//! [`UnlockedKeyMaterial`] — its Debug redacts (never leaks the bytes) + it
//! zeroizes on Drop.
//!
//! # Lock-state (F-VA-5)
//!
//! Pre-unlock crypto ops are typed-rejected with [`VaultError::EngineLocked`]
//! (fail-closed; never a silent plaintext write). `unlock` atomically
//! hydrates BOTH keys (K_principal + user_did_signing_key; identity coherence).

use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::structural_kdf::StructuralKdfKey;

/// The Layer-A vault wire codepoint (R0.5 §4.0 vault band `0x6100`; the
/// 24-byte `SymmetricAeadXNonce` variant).
pub const VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT: u16 = 0x6100;

/// The 12-byte `SymmetricAead` (ChaCha20-Poly1305) sibling codepoint — a
/// RATIFIED frozen v1-beta variant in the vault band `0x6100..0x61FF`. NOT
/// the vault's own codepoint (the vault uses the 24-byte XNonce variant).
pub const SYMMETRIC_AEAD_12B_CODEPOINT: u16 = 0x6101;

/// XChaCha20-Poly1305 nonce width (m-4) — the frozen vault nonce length.
pub const VAULT_XNONCE_LEN: usize = 24;

/// The frozen DAK HKDF info-tag (codepoint slot for a future Argon2id-v2
/// param set per R0.5 §3.1).
pub const DAK_HKDF_INFO_TAG: &[u8] = b"benten-dak-v1";

/// RFC-9106 / OWASP Argon2id params (R0.5 §2.2 tactical pick).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argon2idParams {
    /// Memory cost (KiB).
    pub m_cost: u32,
    /// Time cost (iterations).
    pub t_cost: u32,
    /// Parallelism.
    pub p_cost: u32,
}

/// The frozen v1-beta default Argon2id params (OWASP).
pub const OWASP_DEFAULT: Argon2idParams = Argon2idParams {
    m_cost: 19456,
    t_cost: 2,
    p_cost: 1,
};

/// Derive the Device-Authentication Key (DAK):
/// `HKDF-SHA256( Argon2id(pw, salt; params), info )`.
///
/// Deterministic for the same `(password, salt, params, info_tag)`. A param
/// change or an info-tag change yields a different DAK (both are load-bearing
/// in the derivation). Per CLAUDE.md baked-in #5 the Argon2id + HKDF
/// primitives are wrapped from the vetted upstream `argon2` + `hkdf` crates.
///
/// # Panics
///
/// Panics only on an internal Argon2id param-construction error (the OWASP +
/// stronger fixtures are valid by construction; an invalid caller-supplied
/// param set is a programming error).
#[must_use]
pub fn derive_dak(
    password: &[u8],
    salt: &[u8; 16],
    params: Argon2idParams,
    info_tag: &[u8],
) -> [u8; 32] {
    // Step 1 — Argon2id(pw, salt; m/t/p) → 32-byte seed.
    let argon_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(32))
        .expect("Argon2id params valid (m/t/p within RFC-9106 bounds)");
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut seed = [0u8; 32];
    argon
        .hash_password_into(password, salt, &mut seed)
        .expect("Argon2id hash into 32-byte buffer is infallible for valid params");

    // Step 2 — intermediate HKDF-SHA256 over the Argon2id seed with the
    // codepoint label info-tag → the DAK (domain separation + the
    // future-v2-param-set slot).
    let hk = Hkdf::<Sha256>::new(None, &seed);
    let mut dak = [0u8; 32];
    hk.expand(info_tag, &mut dak)
        .expect("HKDF-SHA256 expand to 32 B is infallible");
    seed.zeroize();
    dak
}

/// Vault CBOR payload (R0.5 §3.1). Field order is FROZEN:
/// `k_principal`, then `user_did_signing_key`, then `user_did_creation_time`.
///
/// The canonical DAG-CBOR map-key order (length-first, then bytewise) for
/// these three keys coincides with this declaration order (11 < 20 < 22).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultPayload {
    /// The principal's at-rest content-encryption root key.
    ///
    /// DAG-CBOR-encoded as a byte string (not an array of integers) per the
    /// frozen on-disk layout — `serde_bytes` selects the byte-string major
    /// type so the 32-byte key serializes as `0x5820 || 32 bytes`.
    #[serde(with = "serde_bytes")]
    pub k_principal: [u8; 32],
    /// The user-DID hybrid signing key (Ed25519⊕ML-DSA-65 serialized).
    ///
    /// DAG-CBOR-encoded as a byte string (`serde_bytes`).
    #[serde(with = "serde_bytes")]
    pub user_did_signing_key: Vec<u8>,
    /// Vault creation time (seconds).
    pub user_did_creation_time: u64,
}

impl VaultPayload {
    /// Serialize to canonical DAG-CBOR bytes (the frozen on-disk payload
    /// encoding). Deterministic; a re-serialize is byte-identical.
    ///
    /// # Panics
    ///
    /// Panics only on an internal serializer error (the payload shape is
    /// always serializable).
    #[must_use]
    pub fn to_canonical_cbor(&self) -> Vec<u8> {
        serde_ipld_dagcbor::to_vec(self).expect("VaultPayload serializes to DAG-CBOR")
    }

    /// Decode from canonical DAG-CBOR bytes.
    ///
    /// # Errors
    ///
    /// Returns [`VaultError::MalformedCbor`] on a decode failure.
    pub fn from_canonical_cbor(bytes: &[u8]) -> Result<Self, VaultError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|_| VaultError::MalformedCbor)
    }
}

/// A decoded vault envelope — the on-disk
/// [`EncryptedEnvelope`](crate::envelope::EncryptedEnvelope) after AEAD-open.
/// Carries the wire codepoint + the nonce bytes used so the format-freeze
/// pins can inspect them.
#[derive(Debug, Clone)]
pub struct DecodedVault {
    /// The vault wire codepoint (`0x6100` XNonce).
    pub codepoint: u16,
    /// The XChaCha20-Poly1305 24-byte nonce.
    pub nonce: Vec<u8>,
    /// The decoded canonical payload.
    pub payload: VaultPayload,
}

/// Serialize a vault on-disk envelope for a fixed payload under a fixed DAK.
///
/// The envelope is XChaCha20-Poly1305-sealed (24-byte nonce) at the vault
/// codepoint `0x6100`, over the canonical DAG-CBOR payload. Deterministic
/// only in the sense the format is fixed; the nonce is random per seal.
///
/// # Errors
///
/// Returns [`VaultError`] on an internal AEAD error.
pub fn serialize_vault(payload: &VaultPayload, dak: &[u8; 32]) -> Result<Vec<u8>, VaultError> {
    use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
    use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

    let pt = payload.to_canonical_cbor();
    let key = Key::from_slice(dak);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce_bytes = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let aad = vault_aad();
    let ct = cipher
        .encrypt(
            nonce,
            chacha20poly1305::aead::Payload {
                msg: &pt,
                aad: &aad,
            },
        )
        .map_err(|_| VaultError::AeadFailed)?;

    // On-disk layout: magic 0xae | V2 | codepoint BE | nonce_len | nonce | ct.
    let mut out = Vec::with_capacity(5 + nonce_bytes.len() + ct.len());
    out.push(crate::envelope::ENVELOPE_MAGIC);
    out.push(crate::envelope::ENVELOPE_FORMAT_VERSION_V2);
    out.extend_from_slice(&VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT.to_be_bytes());
    out.push(u8::try_from(nonce_bytes.len()).unwrap_or(u8::MAX));
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decode + AEAD-open a vault envelope under a DAK. Enforces the XNonce
/// codepoint + 24-byte nonce width (strict-decode; U2).
///
/// # Errors
///
/// Returns [`VaultError`] on a malformed / wrong-width / unknown-codepoint /
/// AEAD-failure input.
pub fn decode_vault(bytes: &[u8], dak: &[u8; 32]) -> Result<DecodedVault, VaultError> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

    if bytes.len() < 5 {
        return Err(VaultError::MalformedCbor);
    }
    if bytes[0] != crate::envelope::ENVELOPE_MAGIC {
        return Err(VaultError::MalformedCbor);
    }
    if bytes[1] != crate::envelope::ENVELOPE_FORMAT_VERSION_V2 {
        return Err(VaultError::MalformedCbor);
    }
    let codepoint = u16::from_be_bytes([bytes[2], bytes[3]]);
    if codepoint != VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT {
        return Err(VaultError::UnknownCodepoint);
    }
    let nonce_len = bytes[4] as usize;
    // U2 strict-decode: the XNonce codepoint discriminates the nonce width.
    if nonce_len != VAULT_XNONCE_LEN {
        return Err(VaultError::NonceWidthMismatch);
    }
    if bytes.len() < 5 + nonce_len {
        return Err(VaultError::MalformedCbor);
    }
    let nonce_bytes = &bytes[5..5 + nonce_len];
    let ct = &bytes[5 + nonce_len..];

    let key = Key::from_slice(dak);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(nonce_bytes);
    let aad = vault_aad();
    let pt = cipher
        .decrypt(
            nonce,
            chacha20poly1305::aead::Payload { msg: ct, aad: &aad },
        )
        .map_err(|_| VaultError::AeadFailed)?;
    let payload = VaultPayload::from_canonical_cbor(&pt)?;
    Ok(DecodedVault {
        codepoint,
        nonce: nonce_bytes.to_vec(),
        payload,
    })
}

/// Strict-decode check: a vault tagged `SymmetricAeadXNonce` (`0x6100`)
/// carrying a 12-byte nonce MUST be rejected (U2 — codepoint discriminates
/// nonce width; no silent coercion).
///
/// # Errors
///
/// Returns [`VaultError::NonceWidthMismatch`] when `codepoint` is the XNonce
/// vault codepoint but `nonce_len != 24`.
pub fn decode_vault_strict(codepoint: u16, nonce_len: usize) -> Result<(), VaultError> {
    if codepoint == VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT && nonce_len != VAULT_XNONCE_LEN {
        return Err(VaultError::NonceWidthMismatch);
    }
    Ok(())
}

/// The vault AEAD AAD (domain-separates the vault seal from every other
/// envelope; binds the vault codepoint).
fn vault_aad() -> Vec<u8> {
    let mut aad = Vec::new();
    aad.extend_from_slice(b"benten-vault:");
    aad.extend_from_slice(&VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT.to_be_bytes());
    aad
}

/// The hydrated key material (R0.5 §3.1) — BOTH keys, atomically. The
/// `k_principal` lives in a [`secrecy::SecretBox`] (Debug-redacting +
/// zeroize-on-Drop); the signing key is a plain `Vec<u8>` (its bytes are not
/// themselves a long-lived at-rest secret in the same coredump sense, but the
/// whole struct is dropped together).
pub struct UnlockedKeyMaterial {
    k_principal: SecretBox<[u8; 32]>,
    user_did_signing_key: Vec<u8>,
}

impl UnlockedKeyMaterial {
    /// Construct from the hydrated payload (production unlock path).
    #[must_use]
    pub fn new(k_principal: [u8; 32], user_did_signing_key: Vec<u8>) -> Self {
        Self {
            k_principal: SecretBox::new(Box::new(k_principal)),
            user_did_signing_key,
        }
    }

    /// Construct directly from a 32-byte `K_principal` for the F-LB-1
    /// structural-KDF bridge pins (the vault source the structural-KDF chain
    /// seeds from).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn from_vault_bytes_for_test(k_principal_bytes: [u8; 32]) -> Self {
        Self::new(k_principal_bytes, vec![0x22u8; 64])
    }

    /// Expose the `K_principal` bytes (explicit, greppable access; the SOLE
    /// access path — mirrors `secrecy::ExposeSecret`).
    #[must_use]
    pub fn expose_k_principal(&self) -> &[u8; 32] {
        self.k_principal.expose_secret()
    }

    /// The 32-byte `K_principal` length (F-VA-5 atomic-hydrate pin).
    #[must_use]
    pub fn k_principal_len(&self) -> usize {
        self.k_principal.expose_secret().len()
    }

    /// The user-DID signing key (hydrated ATOMICALLY with K_principal).
    #[must_use]
    pub fn user_did_signing_key(&self) -> &[u8] {
        &self.user_did_signing_key
    }

    /// The structural-KDF root key seed — the structural-KDF chain seeds from
    /// the vault `K_principal` (F-LB-1 keying-root binding).
    #[must_use]
    pub fn structural_kdf_root_key(&self) -> StructuralKdfKey {
        StructuralKdfKey::from_bytes_for_test(self.k_principal.expose_secret())
    }
}

impl core::fmt::Debug for UnlockedKeyMaterial {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Redacts — never renders the key bytes (Compromise #36).
        f.debug_struct("UnlockedKeyMaterial")
            .field("k_principal", &"SecretBox<[u8; 32]>")
            .field("user_did_signing_key", &"<redacted>")
            .finish()
    }
}

/// A minimal lock-state engine modelling the production vault gate (F-VA-5).
/// Pre-unlock crypto ops are typed-rejected with [`VaultError::EngineLocked`].
pub struct VaultEngine {
    unlocked: Option<UnlockedKeyMaterial>,
}

impl Default for VaultEngine {
    fn default() -> Self {
        Self::new_locked()
    }
}

impl VaultEngine {
    /// A fresh locked engine (no hydrated handle).
    #[must_use]
    pub fn new_locked() -> Self {
        Self { unlocked: None }
    }

    /// Unlock atomically hydrates BOTH keys from a decoded vault payload.
    pub fn unlock(&mut self, payload: &VaultPayload) {
        self.unlocked = Some(UnlockedKeyMaterial::new(
            payload.k_principal,
            payload.user_did_signing_key.clone(),
        ));
    }

    /// A lock-gated production crypto op. Pre-unlock returns
    /// [`VaultError::EngineLocked`] (fail-CLOSED; never a silent plaintext
    /// pass-through). Post-unlock seals the plaintext under K_principal.
    ///
    /// # Errors
    ///
    /// Returns [`VaultError::EngineLocked`] when the engine is locked.
    pub fn encrypt_node(&self, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        match &self.unlocked {
            Some(km) => {
                // A real keyed transform (XOR keystream stand-in over
                // K_principal; the observable consequence is ciphertext ≠
                // plaintext + a non-empty keyed output). The production path
                // routes the structural-KDF + AEAD; this gate's contract is
                // the lock-state, not the cipher.
                let k = km.expose_k_principal();
                let out: Vec<u8> = plaintext
                    .iter()
                    .enumerate()
                    .map(|(i, b)| b ^ k[i % 32])
                    .collect();
                Ok(out)
            }
            None => Err(VaultError::EngineLocked),
        }
    }

    /// The hydrated handle (None while locked).
    #[must_use]
    pub fn unlocked_handle(&self) -> Option<&UnlockedKeyMaterial> {
        self.unlocked.as_ref()
    }
}

/// Typed vault error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum VaultError {
    /// A pre-unlock crypto op was attempted (fail-closed; F-VA-5).
    #[error("engine is locked — unlock the vault before crypto ops (fail-closed)")]
    EngineLocked,

    /// The DAK derivation failed (wrong password).
    #[error("wrong password — DAK does not open the vault")]
    WrongPassword,

    /// A vault tagged `SymmetricAeadXNonce` carried a non-24-byte nonce.
    #[error(
        "vault nonce width mismatch (XNonce codepoint requires a 24-byte nonce; U2 strict-decode)"
    )]
    NonceWidthMismatch,

    /// The vault codepoint is unknown.
    #[error("unknown vault codepoint")]
    UnknownCodepoint,

    /// The vault CBOR payload is malformed.
    #[error("malformed vault CBOR")]
    MalformedCbor,

    /// The AEAD seal/open failed.
    #[error("vault AEAD seal/open failed")]
    AeadFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dak_deterministic_and_param_sensitive() {
        let pw = b"correct horse battery staple";
        let salt = [0x5Au8; 16];
        let a = derive_dak(pw, &salt, OWASP_DEFAULT, DAK_HKDF_INFO_TAG);
        let b = derive_dak(pw, &salt, OWASP_DEFAULT, DAK_HKDF_INFO_TAG);
        assert_eq!(a, b);
        let stronger = Argon2idParams {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        };
        assert_ne!(a, derive_dak(pw, &salt, stronger, DAK_HKDF_INFO_TAG));
        assert_ne!(a, derive_dak(pw, &salt, OWASP_DEFAULT, b"benten-dak-v2"));
    }

    #[test]
    fn vault_round_trips_with_24_byte_nonce() {
        let payload = VaultPayload {
            k_principal: [0x11u8; 32],
            user_did_signing_key: vec![0x22u8; 64],
            user_did_creation_time: 0x0000_0000_6543_2100,
        };
        let dak = [0x33u8; 32];
        let bytes = serialize_vault(&payload, &dak).unwrap();
        let decoded = decode_vault(&bytes, &dak).unwrap();
        assert_eq!(decoded.nonce.len(), VAULT_XNONCE_LEN);
        assert_eq!(decoded.codepoint, VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn strict_decode_rejects_12_byte_nonce() {
        assert!(matches!(
            decode_vault_strict(VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT, 12),
            Err(VaultError::NonceWidthMismatch)
        ));
    }

    #[test]
    fn engine_locked_pre_unlock() {
        let engine = VaultEngine::new_locked();
        assert!(matches!(
            engine.encrypt_node(b"secret"),
            Err(VaultError::EngineLocked)
        ));
    }

    #[test]
    fn unlock_hydrates_both_keys() {
        let payload = VaultPayload {
            k_principal: [0x44u8; 32],
            user_did_signing_key: vec![0x55u8; 64],
            user_did_creation_time: 1,
        };
        let mut engine = VaultEngine::new_locked();
        assert!(engine.unlocked_handle().is_none());
        engine.unlock(&payload);
        let km = engine.unlocked_handle().unwrap();
        assert_eq!(km.k_principal_len(), 32);
        assert!(!km.user_did_signing_key().is_empty());
        let ct = engine.encrypt_node(b"secret node body").unwrap();
        assert_ne!(ct.as_slice(), b"secret node body".as_slice());
    }

    #[test]
    fn debug_does_not_leak_key() {
        let km = UnlockedKeyMaterial::new([0xDEu8; 32], vec![0x11u8; 64]);
        let rendered = format!("{km:?}");
        assert!(!rendered.contains("222")); // 0xDE decimal
    }
}
