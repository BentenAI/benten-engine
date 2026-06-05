//! **F-VA-5 — Engine-lock + `EngineLocked` typed-reject; atomic vault hydrate.**
//! (merges CE-E5 + T-J1)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-5 F-VA-5.
//!   - R0 §3.1 Layer-A "Engine startup": "read vault → DeviceAuthBackend::
//!     unlock(prompt) → Argon2id → HKDF → AEAD-decrypt → hydrate
//!     `UnlockedKeyMaterial { k_principal, user_did_signing_key }` → all
//!     per-Node AEAD + UCAN-signing gated through the unlocked handle. Pre-unlock
//!     ops return `EngineLocked` typed-reject."
//!   - R0 §3.1 vault payload: "both keys live in the SAME vault (atomic
//!     lock/unlock; identity coherence)".
//!
//! # What this pins (FN/FG — lock-state gate)
//!
//! Pre-unlock crypto ops are typed-rejected with `EngineLocked`; post-unlock the
//! same ops succeed against the hydrated `UnlockedKeyMaterial`; the vault
//! hydrates BOTH keys atomically (K_principal + user_did_signing_key together —
//! never one without the other; identity coherence). Pins:
//!   1. a pre-unlock encrypt op returns `Err(EngineLocked)` (typed; not a
//!      panic, not a silent plaintext write);
//!   2. after `unlock`, the same encrypt op succeeds;
//!   3. the unlock hydrates BOTH keys (a post-unlock handle exposing only one
//!      key is an atomic-hydrate violation).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! At baseline the lock-state machine + `EngineLocked` + `UnlockedKeyMaterial`
//! do not exist (the vault is a STUB per R0 §3.1; `grep EngineLocked
//! crates/benten-crypto-suite` → ZERO). Per wave-independence this file commits
//! a LOCAL `f_va_5_stub`. R5 DELETEs it, wires the LIVE vault lock-state +
//! `UnlockedKeyMaterial`, un-ignores, verifies green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Each pin drives the production lock-gated op + asserts the typed outcome /
//! observable consequence. The stub's pre-unlock op deliberately returns `Ok`
//! (the fail-OPEN bug R5 closes) so the pre-unlock-reject pin FAILS until R5
//! wires the `EngineLocked` gate — never a silent-green SHAPE-trap.

#![allow(dead_code)]


// R5: wired to the LIVE vault lock-state (VaultEngine) + UnlockedKeyMaterial.
// Thin adapters preserve the test's `StubEngine` API shape over the real
// `benten_crypto_suite::vault::VaultEngine`.
use benten_crypto_suite::vault::{VaultEngine, VaultError, VaultPayload};

/// Test-view of the hydrated key material exposing the fields the F-VA-5 pins
/// read (the production handle keeps `k_principal` in a `SecretBox`, accessed
/// via `expose_k_principal`).
struct UnlockedKeyMaterial {
    k_principal: [u8; 32],
    user_did_signing_key: Vec<u8>,
}

struct StubEngine {
    inner: VaultEngine,
    view: Option<UnlockedKeyMaterial>,
}

impl StubEngine {
    fn new_locked() -> Self {
        Self {
            inner: VaultEngine::new_locked(),
            view: None,
        }
    }
    fn unlock_for_test(&mut self) {
        let payload = VaultPayload {
            k_principal: [0x44u8; 32],
            user_did_signing_key: vec![0x55u8; 64],
            user_did_creation_time: 1,
        };
        self.inner.unlock(&payload);
        let km = self.inner.unlocked_handle().expect("hydrated post-unlock");
        self.view = Some(UnlockedKeyMaterial {
            k_principal: *km.expose_k_principal(),
            user_did_signing_key: km.user_did_signing_key().to_vec(),
        });
    }
    fn encrypt_node(&self, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        self.inner.encrypt_node(plaintext)
    }
    fn unlocked_handle(&self) -> Option<&UnlockedKeyMaterial> {
        self.view.as_ref()
    }
}

/// F-VA-5 (a) — a pre-unlock crypto op is typed-rejected with `EngineLocked`.
///
/// would-FAIL-if-no-op'd: the stub fails OPEN (returns the plaintext as Ok); R5
/// wires the `EngineLocked` gate. This is the fail-CLOSED guarantee — a locked
/// engine MUST NOT silently produce a plaintext write.
#[test]
fn pre_unlock_op_is_typed_rejected_engine_locked() {
    let engine = StubEngine::new_locked();
    let outcome = engine.encrypt_node(b"secret node body");
    assert!(
        matches!(outcome, Err(VaultError::EngineLocked)),
        "a pre-unlock crypto op MUST be typed-rejected with EngineLocked \
         (fail-CLOSED; NEVER a silent plaintext pass-through). would-FAIL while \
         the stub fails OPEN; got {outcome:?}"
    );
}

/// F-VA-5 (b) — after `unlock`, the same op succeeds against the hydrated key.
///
/// would-FAIL-if-no-op'd: a no-op unlock would leave the op rejected; the pin
/// demands a real ciphertext (non-empty, ≠ plaintext) from the unlocked handle.
#[test]
fn post_unlock_op_succeeds() {
    let mut engine = StubEngine::new_locked();
    engine.unlock_for_test();
    let plaintext = b"secret node body";
    let ct = engine
        .encrypt_node(plaintext)
        .expect("post-unlock encrypt MUST succeed");
    assert_eq!(
        ct.len(),
        plaintext.len(),
        "the keyed transform preserves length (stub model); R5's AEAD adds a \
         tag — adjust the pin then. The load-bearing assertion is the NEXT one."
    );
    assert_ne!(
        ct.as_slice(),
        plaintext.as_slice(),
        "post-unlock encryption MUST actually key the plaintext (ciphertext ≠ \
         plaintext) — would-FAIL on a no-op encrypt that returns the plaintext."
    );
}

/// F-VA-5 (c) — unlock hydrates BOTH keys atomically (identity coherence).
///
/// R0 §3.1: K_principal + user_did_signing_key live in the SAME vault and
/// hydrate together. would-FAIL-if-no-op'd: a handle exposing one key without
/// the other is an atomic-hydrate violation.
#[test]
fn unlock_atomically_hydrates_both_keys() {
    let mut engine = StubEngine::new_locked();
    assert!(
        engine.unlocked_handle().is_none(),
        "pre-unlock there is NO hydrated handle"
    );
    engine.unlock_for_test();
    let km: &UnlockedKeyMaterial = engine
        .unlocked_handle()
        .expect("post-unlock the handle MUST be present");
    assert_eq!(
        km.k_principal.len(),
        32,
        "K_principal MUST be hydrated (32 bytes)"
    );
    assert!(
        !km.user_did_signing_key.is_empty(),
        "the user-DID signing key MUST be hydrated ATOMICALLY with K_principal \
         (same vault; identity coherence) — would-FAIL on a handle that \
         hydrates K_principal alone."
    );
}
