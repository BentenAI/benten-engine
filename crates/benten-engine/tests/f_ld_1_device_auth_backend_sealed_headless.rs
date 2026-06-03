//! F-LD-1 — `DeviceAuthBackend` sealed trait + headless unlock (RED-PHASE).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-1: "sealed per #7; default Argon2id+XChaCha20+optional-keyring-core;
//!     headless via `BENTEN_VAULT_PASSWORD`/IPC."
//!   - R0.3 plan §3.4 (`4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md:494`):
//!     "`DeviceAuthBackend` trait (sealed per #7) with `unlock(prompt) ->
//!     SecretBox<[u8;32]>` + `lock()` + `supports_biometric()` +
//!     `supports_remote_unlock()`. Default impl = ... works in all three
//!     deployment shapes ... incl. headless full-peers (password-via-env-var
//!     `BENTEN_VAULT_PASSWORD` ...; no Tauri / no keyring dependency)."
//!   - CLAUDE.md baked-in #7 sealed-discipline; `kvbackend_conformance.rs` shape.
//!
//! ## RED-PHASE (pim-12 §3.6e)
//!
//! Every test compiles GREEN at baseline behind `#[ignore]` using a
//! SELF-CONTAINED stub-shim (no dependency on another wave's module). At R5
//! the stub is deleted, the real `benten_engine`/`benten-membership-set`
//! Layer-D `DeviceAuthBackend` is `use`d, and the tests un-ignore.
//!
//! Each test drives a PRODUCTION-shaped call site + asserts an OBSERVABLE
//! consequence + is would-FAIL-if-no-op'd (pim-2 §3.6b + pim-18 §3.6f).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM (R5 deletes this, inserts the real Layer-D types).
// ---------------------------------------------------------------------------
//
// Models the §3.4 `DeviceAuthBackend` sealed trait + the headless default
// impl. The unlock derives a 32-byte key from a password (stand-in for
// Argon2id+HKDF); the real impl swaps the KDF but the trait shape + the
// env-var headless path + object-safety + sealed-ness are what this family
// freezes.
mod shim {
    /// The "key material" the unlock yields. The real type is
    /// `secrecy::SecretBox<[u8;32]>`; the shim uses a plain array (F-VA-4
    /// owns the SecretBox/zeroize pin).
    pub type UnlockedKey = [u8; 32];

    #[derive(Debug)]
    pub enum DeviceAuthError {
        /// Headless: env-var/IPC password not supplied.
        NoPasswordSource,
        /// Wrong password (vault decrypt failed).
        VaultDecryptFailed,
        Locked,
    }

    /// Sealed marker — the real seal is a private supertrait in
    /// `benten_engine`. Tests in THIS crate can construct the default impl;
    /// no external crate can impl `DeviceAuthBackend`.
    mod sealed {
        pub trait Sealed {}
    }

    /// `DeviceAuthBackend` (sealed per CLAUDE.md #7). Object-safe so the
    /// engine can hold `Box<dyn DeviceAuthBackend>` (the EP-1 Tier-2 sealed
    /// seam roster shape).
    pub trait DeviceAuthBackend: sealed::Sealed {
        fn unlock(&self, prompt: Option<&str>) -> Result<UnlockedKey, DeviceAuthError>;
        fn lock(&mut self);
        fn supports_biometric(&self) -> bool;
        fn supports_remote_unlock(&self) -> bool;
        fn is_unlocked(&self) -> bool;
    }

    /// The Benten-vended default headless impl: Argon2id+XChaCha20 stand-in.
    /// `password_source` models reading `BENTEN_VAULT_PASSWORD` (or IPC).
    pub struct HeadlessDeviceAuth {
        /// The password the vault was sealed under (stand-in for the
        /// on-disk vault verifier).
        sealed_under: Vec<u8>,
        /// What the headless source supplies (`BENTEN_VAULT_PASSWORD`).
        password_source: Option<Vec<u8>>,
        unlocked: Option<UnlockedKey>,
    }

    impl HeadlessDeviceAuth {
        pub fn new(sealed_under: &[u8], password_source: Option<&[u8]>) -> Self {
            Self {
                sealed_under: sealed_under.to_vec(),
                password_source: password_source.map(<[u8]>::to_vec),
                unlocked: None,
            }
        }

        /// Stand-in KDF (real impl = Argon2id + HKDF). Deterministic +
        /// password-dependent so a wrong password yields a different key
        /// AND the verifier rejects it.
        fn derive(password: &[u8]) -> UnlockedKey {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-dak-v1-shim");
            h.update(password);
            *h.finalize().as_bytes()
        }
    }

    impl sealed::Sealed for HeadlessDeviceAuth {}

    impl DeviceAuthBackend for HeadlessDeviceAuth {
        fn unlock(&self, _prompt: Option<&str>) -> Result<UnlockedKey, DeviceAuthError> {
            let pw = self
                .password_source
                .as_ref()
                .ok_or(DeviceAuthError::NoPasswordSource)?;
            // Headless: NO TTY prompt; the password rides the env-var/IPC
            // source. Verify against the seal.
            if pw != &self.sealed_under {
                return Err(DeviceAuthError::VaultDecryptFailed);
            }
            Ok(Self::derive(pw))
        }

        fn lock(&mut self) {
            self.unlocked = None;
        }

        fn supports_biometric(&self) -> bool {
            // Headless full-peer: no biometric surface (Tauri/Composing owns it).
            false
        }

        fn supports_remote_unlock(&self) -> bool {
            true
        }

        fn is_unlocked(&self) -> bool {
            self.unlocked.is_some()
        }
    }
}

use shim::{DeviceAuthBackend, DeviceAuthError, HeadlessDeviceAuth};

/// F-LD-1 headless-unlock round-trip: a full-peer with NO TTY unlocks via the
/// `BENTEN_VAULT_PASSWORD`-style source. would-FAIL-if-no-op'd: if `unlock`
/// ignored the password source and returned a constant key, the
/// wrong-password arm below would (wrongly) also succeed.
#[test]
#[ignore = "RED-PHASE: F-LD-1 — DeviceAuthBackend headless env-var unlock round-trip; un-ignore at R5"]
fn f_ld_1_headless_unlock_via_env_var_password_round_trips() {
    let password = b"correct-horse-battery-staple";
    let backend = HeadlessDeviceAuth::new(password, Some(password));

    let key = backend
        .unlock(None)
        .expect("headless unlock with matching env-var password MUST succeed");

    // Observable: a real 32-byte key is produced (not the all-zero default).
    assert_ne!(key, [0u8; 32], "headless unlock MUST derive real key material");
    assert!(backend.supports_remote_unlock());
    assert!(!backend.supports_biometric(), "headless full-peer has no biometric");
}

/// F-LD-1 headless missing-source: a full-peer with neither env-var nor IPC
/// password → typed `NoPasswordSource` (NOT a silent default key, NOT a TTY
/// prompt hang). would-FAIL-if-no-op'd: a fallback-to-default-key impl returns
/// Ok here.
#[test]
#[ignore = "RED-PHASE: F-LD-1 — headless with no password source typed-rejects; un-ignore at R5"]
fn f_ld_1_headless_no_password_source_typed_rejects() {
    let backend = HeadlessDeviceAuth::new(b"sealed-pw", None);
    let err = backend
        .unlock(None)
        .expect_err("no password source MUST typed-reject, never default-key");
    assert!(
        matches!(err, DeviceAuthError::NoPasswordSource),
        "MUST surface NoPasswordSource; got {err:?}"
    );
}

/// F-LD-1 wrong-password: the headless source supplies a password that does
/// not match the seal → typed `VaultDecryptFailed`. Pairs with F-VA-3
/// (constant-time): here we pin only the typed-reject outcome.
#[test]
#[ignore = "RED-PHASE: F-LD-1 — wrong headless password typed-rejects; un-ignore at R5"]
fn f_ld_1_wrong_headless_password_typed_rejects() {
    let backend = HeadlessDeviceAuth::new(b"sealed-pw", Some(b"wrong-pw"));
    let err = backend
        .unlock(None)
        .expect_err("wrong password MUST typed-reject");
    assert!(
        matches!(err, DeviceAuthError::VaultDecryptFailed),
        "MUST surface VaultDecryptFailed; got {err:?}"
    );
}

/// F-LD-1 object-safety + sealed-trait conformance (the EP-1 Tier-2 sealed
/// seam shape; `kvbackend_conformance.rs` precedent). The engine holds the
/// backend as `Box<dyn DeviceAuthBackend>` — this compiles ONLY if the trait
/// is object-safe. would-FAIL-if-no-op'd: a non-object-safe trait (generic
/// method / `Self`-return) fails to coerce here.
#[test]
#[ignore = "RED-PHASE: F-LD-1 — DeviceAuthBackend object-safe + sealed; un-ignore at R5"]
fn f_ld_1_device_auth_backend_is_object_safe_and_dyn_dispatchable() {
    let pw = b"dyn-dispatch-pw";
    let boxed: Box<dyn DeviceAuthBackend> = Box::new(HeadlessDeviceAuth::new(pw, Some(pw)));
    // Drive through the trait object (the engine's storage shape).
    let key = boxed.unlock(Some("dyn prompt")).expect("dyn unlock MUST work");
    assert_ne!(key, [0u8; 32]);
    assert!(boxed.supports_remote_unlock());
}

// NOTE (sealed-trait grep-defense, R5): the real un-ignored family adds a
// compile-fence asserting NO external crate can impl `DeviceAuthBackend`
// (the `sealed::Sealed` private supertrait). That assertion is a
// trybuild/compile-fail arm the R5 wave wires against the real
// `benten_engine` seal — it cannot be expressed against this in-file shim
// (the shim's `Sealed` is reachable within this test crate by construction).
