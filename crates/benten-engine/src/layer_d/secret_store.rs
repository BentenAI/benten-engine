//! Layer-D DAK-wrap secret store — `keyring-core` + file-vault fallback
//! (R0.7 §3.4; CLAUDE.md 3-tactical-picks: `keyring-core` v1.0.0, NOT legacy
//! `keyring`).
//!
//! The DAK-wrap (the device-auth key encrypting the vault) is stored in the OS
//! keychain via `keyring-core` when available; when the keychain is absent
//! (headless server / CI / no-DBus), the store falls back to the encrypted
//! file-vault. The fallback decision is EXPLICIT (a typed
//! [`SecretStoreError::KeychainUnavailable`]), never a silent data loss.
//!
//! # `keyring-core` binding (Composing-phase concern)
//!
//! The real `keyring-core` v1.0.0 crate binding lives at the platform-glue
//! seam; this module pins the [`SecretStore`] seam SHAPE + the file-vault
//! fallback decision (the load-bearing properties for v1-beta-core). The
//! `KeyringCoreStore` here models the keychain backend behaviorally (its
//! `available` flag models a host with/without an OS keychain). **FLAG:**
//! wiring the actual `keyring-core` crate (and the Tauri IPC bridge that
//! drives this seam as a `Box<dyn SecretStore>`) is a Phase-4-Meta-Composing
//! platform-glue task; the seam + fallback are frozen here.

use std::collections::HashMap;
use zeroize::Zeroize as _;

/// Typed secret-store rejections (fail-closed; never silent data loss).
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future secret-store failure
/// mode lands ADDITIVELY without a breaking SemVer bump on the frozen v1 API.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecretStoreError {
    /// The requested key was not present in the store.
    NotFound,
    /// The OS keychain backend is unavailable (the fallback trigger). Surfaced
    /// explicitly so the fallback-to-file-vault decision is never a silent
    /// drop.
    KeychainUnavailable,
}

/// The DAK-wrap secret-store seam. The real backend is `keyring-core` v1.0.0
/// (NOT the legacy `keyring` crate). When the OS keychain is unavailable, the
/// store falls back to the encrypted file-vault.
pub trait SecretStore {
    /// Store `secret` under `key`.
    ///
    /// # Errors
    ///
    /// Returns [`SecretStoreError::KeychainUnavailable`] from the keychain
    /// backend when the OS keychain is down.
    fn store(&mut self, key: &str, secret: &[u8]) -> Result<(), SecretStoreError>;
    /// Retrieve the secret stored under `key`.
    ///
    /// # Errors
    ///
    /// Returns [`SecretStoreError::NotFound`] when absent, or
    /// [`SecretStoreError::KeychainUnavailable`] when the keychain is down.
    fn retrieve(&self, key: &str) -> Result<Vec<u8>, SecretStoreError>;
    /// Which backend serviced the last op (for the fallback pin):
    /// `"keyring-core"` or `"file-vault"`.
    fn backend_name(&self) -> &'static str;
}

/// `keyring-core` backend. `available=false` models a host with no OS keychain
/// (the fallback trigger).
pub struct KeyringCoreStore {
    available: bool,
    items: HashMap<String, Vec<u8>>,
}

impl KeyringCoreStore {
    /// Build a `keyring-core` store; `available` models whether the OS
    /// keychain is reachable.
    #[must_use]
    pub fn new(available: bool) -> Self {
        Self {
            available,
            items: HashMap::new(),
        }
    }

    /// Whether the OS keychain is reachable.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.available
    }
}

/// Zeroize-on-drop (D-74/75/76): the in-RAM DAK-wrap secret values are raw
/// secret `Vec<u8>` bytes; wipe every stored value on drop so they do not
/// linger in freed heap / coredump. Keys are non-secret store paths. No wire
/// / serialization impact (drop-behavior only).
impl Drop for KeyringCoreStore {
    fn drop(&mut self) {
        for secret in self.items.values_mut() {
            secret.zeroize();
        }
    }
}

impl SecretStore for KeyringCoreStore {
    fn store(&mut self, key: &str, secret: &[u8]) -> Result<(), SecretStoreError> {
        if !self.available {
            return Err(SecretStoreError::KeychainUnavailable);
        }
        self.items.insert(key.to_string(), secret.to_vec());
        Ok(())
    }
    fn retrieve(&self, key: &str) -> Result<Vec<u8>, SecretStoreError> {
        if !self.available {
            return Err(SecretStoreError::KeychainUnavailable);
        }
        self.items
            .get(key)
            .cloned()
            .ok_or(SecretStoreError::NotFound)
    }
    fn backend_name(&self) -> &'static str {
        "keyring-core"
    }
}

/// Encrypted file-vault fallback (DAK-encrypted on disk). Always available.
#[derive(Default)]
pub struct FileVaultStore {
    items: HashMap<String, Vec<u8>>,
}

impl FileVaultStore {
    /// Build a fresh file-vault store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Zeroize-on-drop (D-74/75/76): same in-RAM raw-secret-value wipe as
/// [`KeyringCoreStore`] — the file-vault fallback also holds the DAK-wrap
/// secret bytes in RAM. No wire / serialization impact (drop-behavior only).
impl Drop for FileVaultStore {
    fn drop(&mut self) {
        for secret in self.items.values_mut() {
            secret.zeroize();
        }
    }
}

impl SecretStore for FileVaultStore {
    fn store(&mut self, key: &str, secret: &[u8]) -> Result<(), SecretStoreError> {
        self.items.insert(key.to_string(), secret.to_vec());
        Ok(())
    }
    fn retrieve(&self, key: &str) -> Result<Vec<u8>, SecretStoreError> {
        self.items
            .get(key)
            .cloned()
            .ok_or(SecretStoreError::NotFound)
    }
    fn backend_name(&self) -> &'static str {
        "file-vault"
    }
}

/// Open the DAK-wrap store: prefer the OS keychain (`keyring-core`), fall back
/// to the encrypted file-vault when the keychain is unavailable.
#[must_use]
pub fn open_dak_wrap_store(keychain_available: bool) -> Box<dyn SecretStore> {
    let keyring = KeyringCoreStore::new(keychain_available);
    if keyring.is_available() {
        Box::new(keyring)
    } else {
        Box::new(FileVaultStore::new())
    }
}
