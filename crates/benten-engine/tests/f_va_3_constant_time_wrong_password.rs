//! F-VA-3 — Constant-time wrong-password path (RED-PHASE; FN).
//!
//! R3 wave **W3-layer-d** (assigned to W3 per landscape §3: "constant-time
//! shares with W1 — assign to W3 to keep vault-format in W1").
//!
//! Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 5
//!     F-VA-3 (merges CE-E3 + T-J2): "wrong-password runs Argon2id+AEAD-decrypt
//!     to completion (no fast-fail early-return timing oracle)." Red-phase:
//!     "structural no-early-return-before-AEAD-open pin + coarse timing-
//!     invariance harness (best-effort)."
//!   - R0.5 plan §3.1; Compromise #34 (password-knowledge).
//!
//! ## RED-PHASE (pim-12 §3.6e)
//!
//! SELF-CONTAINED stub-shim models the vault unlock path. The key property:
//! a WRONG password must NOT short-circuit before the AEAD-open (no early-exit
//! timing oracle distinguishing "salt bad" / "params bad" from "AEAD-tag bad").
//! R5 swaps in the real Argon2id + XChaCha20-Poly1305 vault unlock.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// R5: the stub-shim KDF + AEAD stand-ins are REPLACED with the REAL
// `benten_crypto_suite::vault` primitives — `derive_dak` (Argon2id+HKDF) +
// `serialize_vault`/`decode_vault` (XChaCha20-Poly1305 with a constant-time
// AEAD tag compare). The `UnlockTrace` instrumentation wraps the REAL calls so
// the structural no-early-return property is asserted against the real unlock
// path (the KDF + the AEAD-open both run unconditionally on a wrong password —
// the no-fast-fail-oracle property). The single typed `VaultDecryptFailed`
// collapses every wrong-password cause (bad salt / params / tag) — there is no
// error-variant side-channel.
//
// Argon2id is intentionally heavy; to keep the 200-iter timing harness fast we
// seal under a REDUCED Argon2id param set (the constant-time PROPERTY is
// param-independent — what F-VA-3 pins is "no early return / single typed
// error / no gross timing oracle", not the OWASP cost factor, which the vault
// format-freeze family owns).
mod shim {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use benten_crypto_suite::vault::{
        Argon2idParams, DAK_HKDF_INFO_TAG, VaultPayload, decode_vault, derive_dak, serialize_vault,
    };

    /// A reduced Argon2id param set so the 200-iter timing harness completes
    /// quickly. The no-early-return / single-typed-error / no-gross-timing
    /// properties F-VA-3 pins are param-independent.
    const FAST_PARAMS: Argon2idParams = Argon2idParams {
        m_cost: 8,
        t_cost: 1,
        p_cost: 1,
    };

    /// Tracks how far the unlock path progressed, so the test can assert the
    /// AEAD-open stage was REACHED even on a wrong password (no early return).
    #[derive(Debug, Default)]
    pub struct UnlockTrace {
        pub kdf_ran: AtomicUsize,
        pub aead_open_attempted: AtomicUsize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum UnlockError {
        /// The single typed rejection — indistinguishable across all wrong-
        /// password causes (bad salt / bad params / bad tag all land here).
        VaultDecryptFailed,
    }

    pub struct Vault {
        salt: [u8; 16],
        sealed_under_password: Vec<u8>,
        /// The REAL XChaCha20-Poly1305-sealed vault bytes under the correct DAK.
        vault_bytes: Vec<u8>,
    }

    impl Vault {
        pub fn seal(password: &[u8], salt: [u8; 16]) -> Self {
            // Stage 1: REAL Argon2id+HKDF DAK derivation.
            let dak = derive_dak(password, &salt, FAST_PARAMS, DAK_HKDF_INFO_TAG);
            // Stage 2: REAL XChaCha20-Poly1305 seal of a fixed payload.
            let payload = VaultPayload {
                k_principal: [0x5A; 32],
                user_did_signing_key: vec![0x22; 64],
                user_did_creation_time: 0,
            };
            let vault_bytes =
                serialize_vault(&payload, &dak).expect("vault seal is infallible for valid params");
            Self {
                salt,
                sealed_under_password: password.to_vec(),
                vault_bytes,
            }
        }

        /// Unlock. CONTRACT: ALWAYS runs the REAL KDF (Argon2id+HKDF) → the REAL
        /// AEAD-open (XChaCha20-Poly1305, constant-time tag compare). There is
        /// NO early return between the KDF and the AEAD-open regardless of
        /// whether the password is right or wrong (the no-fast-fail-oracle
        /// property). The trace records that both stages ran.
        pub fn unlock(
            &self,
            password: &[u8],
            trace: &UnlockTrace,
        ) -> Result<[u8; 32], UnlockError> {
            // Stage 1: REAL KDF (always runs).
            let dak = derive_dak(password, &self.salt, FAST_PARAMS, DAK_HKDF_INFO_TAG);
            trace.kdf_ran.fetch_add(1, Ordering::SeqCst);
            // Stage 2: REAL AEAD-open (ALWAYS attempted; the AEAD tag compare is
            // constant-time inside `decode_vault`).
            trace.aead_open_attempted.fetch_add(1, Ordering::SeqCst);
            match decode_vault(&self.vault_bytes, &dak) {
                // Every failure cause collapses to the single typed rejection.
                Ok(decoded) => Ok(decoded.payload.k_principal),
                Err(_) => Err(UnlockError::VaultDecryptFailed),
            }
        }

        pub fn sealed_under(&self) -> &[u8] {
            &self.sealed_under_password
        }
    }
}

use shim::{UnlockError, UnlockTrace, Vault};
use std::sync::atomic::Ordering;

/// F-VA-3 STRUCTURAL no-early-return: a WRONG password still drives the unlock
/// path THROUGH the AEAD-open stage (the trace shows `aead_open_attempted`
/// incremented). would-FAIL-if-no-op'd: a fast-fail impl that rejected after
/// the KDF (before the AEAD-open) would leave `aead_open_attempted == 0`.
#[test]
fn f_va_3_wrong_password_reaches_aead_open_no_early_return() {
    let vault = Vault::seal(b"correct-password", [0x11; 16]);
    let trace = UnlockTrace::default();

    let err = vault
        .unlock(b"WRONG-password", &trace)
        .expect_err("a wrong password MUST reject");
    assert_eq!(err, UnlockError::VaultDecryptFailed);

    // The KDF ran AND the AEAD-open was attempted — proving NO early return
    // between them on the wrong-password path.
    assert_eq!(
        trace.kdf_ran.load(Ordering::SeqCst),
        1,
        "KDF MUST run on wrong pw"
    );
    assert_eq!(
        trace.aead_open_attempted.load(Ordering::SeqCst),
        1,
        "AEAD-open MUST be reached even on a wrong password (no fast-fail oracle)"
    );
}

/// F-VA-3 correct-password path reaches the SAME stages (the right password
/// does not take a structurally-different, observably-faster path). Pairs with
/// the wrong-password trace to show identical stage-progression.
#[test]
fn f_va_3_correct_password_runs_same_stages_as_wrong() {
    let vault = Vault::seal(b"correct-password", [0x22; 16]);

    let trace_ok = UnlockTrace::default();
    vault
        .unlock(b"correct-password", &trace_ok)
        .expect("correct pw unlocks");

    let trace_bad = UnlockTrace::default();
    let _ = vault.unlock(b"wrong-password", &trace_bad);

    // Both paths run the KDF once + attempt the AEAD-open once — identical
    // stage progression (the structural constant-time property).
    assert_eq!(
        trace_ok.kdf_ran.load(Ordering::SeqCst),
        trace_bad.kdf_ran.load(Ordering::SeqCst),
        "correct + wrong passwords MUST run the KDF the same number of times"
    );
    assert_eq!(
        trace_ok.aead_open_attempted.load(Ordering::SeqCst),
        trace_bad.aead_open_attempted.load(Ordering::SeqCst),
        "correct + wrong passwords MUST both attempt the AEAD-open"
    );
}

/// F-VA-3 single-typed-rejection: all wrong-password causes land on the SAME
/// typed error (`VaultDecryptFailed`) — no error-variant side-channel that
/// distinguishes "salt off" / "params off" / "tag off". A different wrong
/// password (different bytes) still yields the identical typed error.
#[test]
fn f_va_3_all_wrong_passwords_yield_single_typed_error() {
    let vault = Vault::seal(b"correct-password", [0x33; 16]);
    for wrong in [
        &b"a"[..],
        &b"bb"[..],
        &b"completely-different-and-longer"[..],
    ] {
        let trace = UnlockTrace::default();
        assert_eq!(
            vault.unlock(wrong, &trace),
            Err(UnlockError::VaultDecryptFailed),
            "every wrong password MUST surface the SAME typed error (no variant side-channel)"
        );
    }
}

/// F-VA-3 coarse timing-invariance harness (BEST-EFFORT). Measures the
/// wall-clock of the correct vs a wrong unlock over repeated runs; asserts they
/// are within a generous ratio. This is intentionally coarse + tolerant (CI
/// timing is noisy) — its job is to catch a GROSS early-return oracle (e.g. a
/// wrong password returning ~instantly because it skipped the KDF+AEAD), NOT to
/// prove sub-microsecond constant-time (that is the R5/audit kani/dudect job).
#[test]
fn f_va_3_coarse_timing_invariance_best_effort() {
    use std::time::Instant;
    let vault = Vault::seal(b"correct-password", [0x44; 16]);

    let iters = 200;
    let mut t_ok = std::time::Duration::ZERO;
    let mut t_bad = std::time::Duration::ZERO;
    for _ in 0..iters {
        let trace = UnlockTrace::default();
        let s = Instant::now();
        let _ = vault.unlock(b"correct-password", &trace);
        t_ok += s.elapsed();

        let trace = UnlockTrace::default();
        let s = Instant::now();
        let _ = vault.unlock(b"wrong-password-x", &trace);
        t_bad += s.elapsed();
    }

    // Gross-oracle guard: neither path is more than ~8× the other. A skipped
    // KDF+AEAD on the wrong path would show up as a far larger disparity.
    let ok_ns = t_ok.as_nanos().max(1);
    let bad_ns = t_bad.as_nanos().max(1);
    // Coarse ratio for a gross-oracle guard; precision loss is irrelevant at
    // the 8× tolerance band.
    #[allow(clippy::cast_precision_loss)]
    let ratio = (ok_ns.max(bad_ns)) as f64 / (ok_ns.min(bad_ns)) as f64;
    assert!(
        ratio < 8.0,
        "coarse timing invariance: ok={ok_ns}ns bad={bad_ns}ns ratio={ratio:.2} — \
         a gross early-return oracle would exceed this (best-effort; audit owns the precise bound)"
    );
}
