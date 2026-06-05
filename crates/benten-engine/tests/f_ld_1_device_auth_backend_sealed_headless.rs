//! F-LD-1 — `DeviceAuthBackend` sealed trait + headless unlock (RED-PHASE).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-1: "sealed per #7; default Argon2id+XChaCha20+optional-keyring-core;
//!     headless via `BENTEN_VAULT_PASSWORD`/IPC."
//!   - R0.5 plan §3.4 (spec of record; minted against R0.3
//!     `4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md:494` — §3.4 is the
//!     stable Layer-D section R0.3→R0.5):
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

// R5: stub-shim DELETED; the real `benten_engine::layer_d::device_auth`
// sealed trait + headless Argon2id+XChaCha20-Poly1305 default impl are in
// use. The real `unlock` yields the crypto-suite `UnlockedKeyMaterial`
// (K_principal in a `secrecy::SecretBox`); `expose_unlocked_key` reads the 32
// bytes for the round-trip pins. The seal stores a fixed per-vault salt.
use benten_engine::layer_d::device_auth::{
    DeviceAuthBackend, DeviceAuthError, HeadlessDeviceAuth, expose_unlocked_key,
};

/// A fixed K_principal for the seal (the unlock recovers exactly this on the
/// correct-password path).
const K_PRINCIPAL: [u8; 32] = [0x5Au8; 32];
/// A fixed per-vault Argon2id salt.
const SALT: [u8; 16] = [0x11u8; 16];

/// F-LD-1 headless-unlock round-trip: a full-peer with NO TTY unlocks via the
/// `BENTEN_VAULT_PASSWORD`-style source. would-FAIL-if-no-op'd: if `unlock`
/// ignored the password source and returned a constant key, the
/// wrong-password arm below would (wrongly) also succeed.
#[test]
fn f_ld_1_headless_unlock_via_env_var_password_round_trips() {
    let password = b"correct-horse-battery-staple";
    let backend = HeadlessDeviceAuth::seal_and_build(K_PRINCIPAL, password, SALT, Some(password));

    let key = backend
        .unlock(None)
        .expect("headless unlock with matching env-var password MUST succeed");

    // Observable: the real K_principal is recovered (not the all-zero default).
    let k_bytes = expose_unlocked_key(&key);
    assert_ne!(
        k_bytes, [0u8; 32],
        "headless unlock MUST derive real key material"
    );
    assert_eq!(
        k_bytes, K_PRINCIPAL,
        "the correct-password unlock MUST recover the exact sealed K_principal"
    );
    assert!(backend.supports_remote_unlock());
    assert!(
        !backend.supports_biometric(),
        "headless full-peer has no biometric"
    );
}

/// F-LD-1 headless missing-source: a full-peer with neither env-var nor IPC
/// password → typed `NoPasswordSource` (NOT a silent default key, NOT a TTY
/// prompt hang). would-FAIL-if-no-op'd: a fallback-to-default-key impl returns
/// Ok here.
#[test]
fn f_ld_1_headless_no_password_source_typed_rejects() {
    let backend = HeadlessDeviceAuth::seal_and_build(K_PRINCIPAL, b"sealed-pw", SALT, None);
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
fn f_ld_1_wrong_headless_password_typed_rejects() {
    let backend =
        HeadlessDeviceAuth::seal_and_build(K_PRINCIPAL, b"sealed-pw", SALT, Some(b"wrong-pw"));
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
fn f_ld_1_device_auth_backend_is_object_safe_and_dyn_dispatchable() {
    let pw = b"dyn-dispatch-pw";
    let boxed: Box<dyn DeviceAuthBackend> = Box::new(HeadlessDeviceAuth::seal_and_build(
        K_PRINCIPAL,
        pw,
        SALT,
        Some(pw),
    ));
    // Drive through the trait object (the engine's storage shape).
    let key = boxed
        .unlock(Some("dyn prompt"))
        .expect("dyn unlock MUST work");
    assert_ne!(expose_unlocked_key(&key), [0u8; 32]);
    assert!(boxed.supports_remote_unlock());
}

// NOTE (sealed-trait grep-defense, R5): the real un-ignored family adds a
// compile-fence asserting NO external crate can impl `DeviceAuthBackend`
// (the `sealed::Sealed` private supertrait). That assertion is a
// trybuild/compile-fail arm the R5 wave wires against the real
// `benten_engine` seal — it cannot be expressed against this in-file shim
// (the shim's `Sealed` is reachable within this test crate by construction).
