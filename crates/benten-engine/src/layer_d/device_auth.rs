//! Layer-D DAK substrate — the `DeviceAuthBackend` sealed trait + the
//! Benten-vended headless default impl (e2r §2; R0.7 §3.4).
//!
//! # `DeviceAuthBackend` (sealed per CLAUDE.md #7)
//!
//! The device-auth seam: `unlock(prompt) -> SecretBox<[u8;32]>` + `lock()` +
//! `supports_biometric()` + `supports_remote_unlock()`. It is a **sealed**
//! trait (Tier-2; Benten-internal) — a private supertrait
//! (`sealed::Sealed`) means no external crate can implement it. Object-safe
//! so the engine can hold `Box<dyn DeviceAuthBackend>`.
//!
//! # The headless default ([`HeadlessDeviceAuth`])
//!
//! The Benten-vended default = Argon2id + XChaCha20-Poly1305 (the real
//! [`benten_crypto_suite::vault`] stack). Works in all three deployment shapes
//! (baked-in #17), incl. headless full-peers: the password rides the
//! [`BENTEN_VAULT_PASSWORD`] env-var (or IPC) — NO TTY prompt, NO Tauri / no
//! keyring dependency. A missing password source typed-rejects
//! ([`DeviceAuthError::NoPasswordSource`]); a wrong password typed-rejects
//! ([`DeviceAuthError::VaultDecryptFailed`]) AFTER running the KDF + AEAD-open
//! to completion (F-VA-3 no-early-return constant-time property).
//!
//! # `BENTEN_VAULT_PASSWORD` (FREEZE — e2r §4.4)
//!
//! The env-var name is wire-frozen. A headless full-peer reads its vault
//! password from this env-var (or the IPC channel) and never blocks on a TTY.

use benten_crypto_suite::vault::{
    Argon2idParams, DAK_HKDF_INFO_TAG, OWASP_DEFAULT, UnlockedKeyMaterial, VaultPayload,
    decode_vault, derive_dak, serialize_vault,
};

/// The frozen headless password env-var name (e2r §4.4 FREEZE).
pub const BENTEN_VAULT_PASSWORD: &str = "BENTEN_VAULT_PASSWORD";

/// The key material a successful unlock yields — the crypto-suite
/// [`UnlockedKeyMaterial`] (the `K_principal` lives in a
/// `secrecy::SecretBox<[u8;32]>` inside: Debug-redacting + zeroize-on-Drop).
pub type UnlockedKey = UnlockedKeyMaterial;

/// Typed device-auth rejections (fail-closed; CLAUDE.md #5 — never a silent
/// default key).
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future device-auth failure
/// mode lands ADDITIVELY without a breaking SemVer bump on the frozen v1 API.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceAuthError {
    /// Headless: neither the [`BENTEN_VAULT_PASSWORD`] env-var nor the IPC
    /// channel supplied a password (NOT a silent default key, NOT a TTY hang).
    NoPasswordSource,
    /// The vault decrypt failed (wrong password / tampered vault). A SINGLE
    /// typed rejection across all wrong-password causes (F-VA-3 — no
    /// error-variant side-channel).
    VaultDecryptFailed,
    /// An operation was attempted while the backend is locked.
    Locked,
}

/// The sealed-marker module — the real seal is this private supertrait. No
/// external crate can name `sealed::Sealed`, so no external crate can
/// implement [`DeviceAuthBackend`].
mod sealed {
    /// Private supertrait — the seal.
    pub trait Sealed {}
}

/// `DeviceAuthBackend` (sealed per CLAUDE.md #7). Object-safe so the engine
/// can hold `Box<dyn DeviceAuthBackend>` (the EP-1 Tier-2 sealed-seam roster
/// shape).
pub trait DeviceAuthBackend: sealed::Sealed {
    /// Unlock the device vault, yielding the `K_principal` key material. The
    /// `prompt` is advisory (a UI hint); the headless default ignores it and
    /// reads the env-var / IPC source.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceAuthError::NoPasswordSource`] when no password is
    /// available headless, or [`DeviceAuthError::VaultDecryptFailed`] when the
    /// password is wrong (after running the KDF + AEAD-open to completion).
    fn unlock(&self, prompt: Option<&str>) -> Result<UnlockedKey, DeviceAuthError>;
    /// Drop the in-memory unlocked key material.
    fn lock(&mut self);
    /// Whether this backend supports a biometric unlock surface (the headless
    /// full-peer does not — Tauri/Composing owns biometrics).
    fn supports_biometric(&self) -> bool;
    /// Whether this backend supports a remote-unlock flow (Layer-D
    /// remote-permission-call).
    fn supports_remote_unlock(&self) -> bool;
    /// Whether the vault is currently unlocked.
    ///
    /// # Contract (C-06)
    ///
    /// This reflects the FRESHLY-CONSTRUCTED locked state. [`Self::unlock`]
    /// takes `&self` and RETURNS the recovered [`UnlockedKey`] BY VALUE — it
    /// does NOT mutate `self` to an "unlocked" state — so on the Benten-vended
    /// [`HeadlessDeviceAuth`] backend `is_unlocked()` is `false` from
    /// construction and stays `false` across `unlock` calls (only
    /// [`Self::lock`], which takes `&mut self`, ever touches the flag, and it
    /// sets it `false`). The unlocked key material lives in the value
    /// `unlock` hands back, NOT inside the backend; callers hold and zeroize
    /// that value themselves. A backend that owns persistent unlocked state
    /// (a future biometric/IPC backend with interior mutability) MAY report
    /// `true` here, but no `&self`-`unlock` default can flip this flag.
    fn is_unlocked(&self) -> bool;
}

/// The Benten-vended default headless impl: the real Argon2id +
/// XChaCha20-Poly1305 vault stack. The `password_source` models reading
/// [`BENTEN_VAULT_PASSWORD`] (or the IPC channel).
pub struct HeadlessDeviceAuth {
    /// The on-disk vault envelope bytes (XChaCha20-Poly1305-sealed under the
    /// DAK derived from the correct password).
    vault_bytes: Vec<u8>,
    /// The per-vault Argon2id salt.
    salt: [u8; 16],
    /// The Argon2id params the vault was sealed under.
    params: Argon2idParams,
    /// What the headless source supplies ([`BENTEN_VAULT_PASSWORD`] / IPC).
    /// `None` models no headless password source.
    password_source: Option<Vec<u8>>,
    /// The in-memory unlocked key, if any.
    unlocked: bool,
}

impl HeadlessDeviceAuth {
    /// Seal a fresh vault under `password` and build a headless backend whose
    /// password source supplies `password_source`. The real Argon2id +
    /// XChaCha20-Poly1305 stack is used.
    ///
    /// # Panics
    ///
    /// Panics only on an internal AEAD seal error (the OWASP params are valid).
    #[must_use]
    pub fn seal_and_build(
        k_principal: [u8; 32],
        password: &[u8],
        salt: [u8; 16],
        password_source: Option<&[u8]>,
    ) -> Self {
        let params = OWASP_DEFAULT;
        let dak = derive_dak(password, &salt, params, DAK_HKDF_INFO_TAG);
        let payload = VaultPayload {
            k_principal,
            user_did_signing_key: vec![0x22u8; 64],
            user_did_creation_time: 0,
        };
        let vault_bytes =
            serialize_vault(&payload, &dak).expect("vault seal is infallible for OWASP params");
        Self {
            vault_bytes,
            salt,
            params,
            password_source: password_source.map(<[u8]>::to_vec),
            unlocked: false,
        }
    }

    /// Attempt the vault unlock under `password`. ALWAYS runs the KDF
    /// ([`derive_dak`] Argon2id) → [`decode_vault`] (AEAD-open with a
    /// constant-time tag compare). There is NO input-dependent early return
    /// between the KDF and the AEAD-open (the F-VA-3 no-fast-fail-oracle
    /// property); a wrong password lands on the single
    /// [`DeviceAuthError::VaultDecryptFailed`].
    fn unlock_with_password(
        &self,
        password: &[u8],
    ) -> Result<UnlockedKeyMaterial, DeviceAuthError> {
        // Stage 1 — KDF (ALWAYS runs; no input-dependent skip).
        let dak = derive_dak(password, &self.salt, self.params, DAK_HKDF_INFO_TAG);
        // Stage 2 — AEAD-open (ALWAYS attempted; the AEAD tag compare is
        // constant-time inside `decode_vault`). Every failure cause (AEAD tag,
        // malformed, wrong-width, …) collapses to the single typed rejection —
        // no salt/params/tag error-variant side-channel.
        match decode_vault(&self.vault_bytes, &dak) {
            Ok(decoded) => Ok(UnlockedKeyMaterial::new(
                decoded.payload.k_principal,
                decoded.payload.user_did_signing_key,
            )),
            Err(_) => Err(DeviceAuthError::VaultDecryptFailed),
        }
    }
}

impl sealed::Sealed for HeadlessDeviceAuth {}

impl DeviceAuthBackend for HeadlessDeviceAuth {
    fn unlock(&self, _prompt: Option<&str>) -> Result<UnlockedKey, DeviceAuthError> {
        // Headless: NO TTY prompt; the password rides the env-var/IPC source.
        let pw = self
            .password_source
            .as_ref()
            .ok_or(DeviceAuthError::NoPasswordSource)?;
        self.unlock_with_password(pw)
    }

    fn lock(&mut self) {
        self.unlocked = false;
    }

    fn supports_biometric(&self) -> bool {
        // Headless full-peer: no biometric surface (Tauri/Composing owns it).
        false
    }

    fn supports_remote_unlock(&self) -> bool {
        true
    }

    fn is_unlocked(&self) -> bool {
        self.unlocked
    }
}

/// Expose the 32 bytes of `K_principal` from an [`UnlockedKey`] (greppable
/// access; routes through the crypto-suite's sole `expose_k_principal`
/// surface). Used by the F-LD-1 round-trip pins.
#[must_use]
pub fn expose_unlocked_key(key: &UnlockedKey) -> [u8; 32] {
    *key.expose_k_principal()
}
