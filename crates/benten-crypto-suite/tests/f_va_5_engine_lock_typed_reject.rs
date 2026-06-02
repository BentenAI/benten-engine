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

/// SELF-CONTAINED stub-shim (R5 deletes this whole module + wires the LIVE
/// vault lock-state + `UnlockedKeyMaterial`).
mod f_va_5_stub {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum VaultError {
        /// The typed pre-unlock rejection (R0 §3.1).
        EngineLocked,
        WrongPassword,
    }

    /// The hydrated key material (R0 §3.1) — BOTH keys, atomically.
    #[derive(Debug, Clone)]
    pub struct UnlockedKeyMaterial {
        pub k_principal: [u8; 32],
        pub user_did_signing_key: Vec<u8>,
    }

    /// A minimal engine handle with a lock state, modelling the production
    /// surface. STUB's `encrypt_node` fails OPEN pre-unlock (the bug); R5 makes
    /// it return `Err(EngineLocked)`.
    pub struct StubEngine {
        unlocked: Option<UnlockedKeyMaterial>,
        /// RED-PHASE knob: STUB = false (fails open). R5 = true (gate live).
        enforce_lock: bool,
    }

    impl StubEngine {
        pub fn new_locked() -> Self {
            Self {
                unlocked: None,
                // RED-PHASE: lock NOT enforced. R5 flips this to true.
                enforce_lock: false,
            }
        }

        /// Unlock hydrates BOTH keys atomically from the (fixed) vault.
        pub fn unlock_for_test(&mut self) {
            self.unlocked = Some(UnlockedKeyMaterial {
                k_principal: [0x44u8; 32],
                user_did_signing_key: vec![0x55u8; 64],
            });
        }

        /// A lock-gated production op. Pre-unlock MUST be `Err(EngineLocked)`.
        pub fn encrypt_node(&self, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
            match &self.unlocked {
                Some(km) => {
                    // Post-unlock: a trivial keyed transform (NOT real AEAD;
                    // R5 wires the structural-KDF + AEAD path). The observable
                    // consequence is a non-empty keyed ciphertext.
                    let mut out = Vec::with_capacity(plaintext.len());
                    for (i, b) in plaintext.iter().enumerate() {
                        out.push(b ^ km.k_principal[i % 32]);
                    }
                    Ok(out)
                }
                None => {
                    if self.enforce_lock {
                        Err(VaultError::EngineLocked)
                    } else {
                        // RED-PHASE fail-OPEN bug: returns plaintext-as-Ok.
                        Ok(plaintext.to_vec())
                    }
                }
            }
        }

        pub fn unlocked_handle(&self) -> Option<&UnlockedKeyMaterial> {
            self.unlocked.as_ref()
        }
    }
}

use f_va_5_stub::{StubEngine, UnlockedKeyMaterial, VaultError};

/// F-VA-5 (a) — a pre-unlock crypto op is typed-rejected with `EngineLocked`.
///
/// would-FAIL-if-no-op'd: the stub fails OPEN (returns the plaintext as Ok); R5
/// wires the `EngineLocked` gate. This is the fail-CLOSED guarantee — a locked
/// engine MUST NOT silently produce a plaintext write.
#[test]
#[ignore = "RED-PHASE: F-VA-5 — pre-unlock crypto op MUST return Err(EngineLocked) (fail-closed); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-VA-5 — post-unlock the gated op succeeds against the hydrated key; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-VA-5 — unlock MUST atomically hydrate BOTH K_principal AND user_did_signing_key; un-ignore at R5"]
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
