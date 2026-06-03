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
//!   - R0.3 plan §3.1; Compromise #34 (password-knowledge).
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

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — vault unlock with a no-early-return contract.
// ---------------------------------------------------------------------------
mod shim {
    use std::sync::atomic::{AtomicUsize, Ordering};

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
        /// AEAD tag over the real K_principal under the correct DAK.
        expected_tag: [u8; 32],
    }

    impl Vault {
        pub fn seal(password: &[u8], salt: [u8; 16]) -> Self {
            let dak = Self::kdf(password, &salt, &UnlockTrace::default());
            let tag = Self::aead_tag(&dak, &UnlockTrace::default());
            Self {
                salt,
                sealed_under_password: password.to_vec(),
                expected_tag: tag,
            }
        }

        /// Stand-in Argon2id+HKDF. ALWAYS runs to completion (no input-
        /// dependent early exit).
        fn kdf(password: &[u8], salt: &[u8; 16], trace: &UnlockTrace) -> [u8; 32] {
            trace.kdf_ran.fetch_add(1, Ordering::SeqCst);
            let mut h = blake3::Hasher::new();
            h.update(b"benten-vault-dak-v1:");
            h.update(salt);
            h.update(password);
            *h.finalize().as_bytes()
        }

        /// Stand-in AEAD tag computation over the DAK.
        fn aead_tag(dak: &[u8; 32], trace: &UnlockTrace) -> [u8; 32] {
            trace.aead_open_attempted.fetch_add(1, Ordering::SeqCst);
            let mut h = blake3::Hasher::new();
            h.update(b"benten-vault-aead-tag:");
            h.update(dak);
            *h.finalize().as_bytes()
        }

        /// Unlock. CONTRACT: ALWAYS runs KDF → AEAD-open, then a SINGLE
        /// constant-time tag compare. There is NO early return between the KDF
        /// and the AEAD-open regardless of whether the password is right or
        /// wrong (the no-fast-fail-oracle property).
        pub fn unlock(&self, password: &[u8], trace: &UnlockTrace) -> Result<[u8; 32], UnlockError> {
            // Stage 1: KDF (always runs).
            let dak = Self::kdf(password, &self.salt, trace);
            // Stage 2: AEAD-open (ALWAYS attempted — never skipped on a wrong pw).
            let candidate_tag = Self::aead_tag(&dak, trace);
            // Stage 3: constant-time compare of the full 32-byte tag.
            if constant_time_eq(&candidate_tag, &self.expected_tag) {
                Ok(dak)
            } else {
                Err(UnlockError::VaultDecryptFailed)
            }
        }

        pub fn sealed_under(&self) -> &[u8] {
            &self.sealed_under_password
        }
    }

    /// Constant-time 32-byte equality (no data-dependent short-circuit).
    pub fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
        let mut diff: u8 = 0;
        for i in 0..32 {
            diff |= a[i] ^ b[i];
        }
        diff == 0
    }
}

use shim::{Vault, UnlockError, UnlockTrace};
use std::sync::atomic::Ordering;

/// F-VA-3 STRUCTURAL no-early-return: a WRONG password still drives the unlock
/// path THROUGH the AEAD-open stage (the trace shows `aead_open_attempted`
/// incremented). would-FAIL-if-no-op'd: a fast-fail impl that rejected after
/// the KDF (before the AEAD-open) would leave `aead_open_attempted == 0`.
#[test]
#[ignore = "RED-PHASE: F-VA-3 — wrong password reaches AEAD-open (no early-return oracle); un-ignore at R5"]
fn f_va_3_wrong_password_reaches_aead_open_no_early_return() {
    let vault = Vault::seal(b"correct-password", [0x11; 16]);
    let trace = UnlockTrace::default();

    let err = vault
        .unlock(b"WRONG-password", &trace)
        .expect_err("a wrong password MUST reject");
    assert_eq!(err, UnlockError::VaultDecryptFailed);

    // The KDF ran AND the AEAD-open was attempted — proving NO early return
    // between them on the wrong-password path.
    assert_eq!(trace.kdf_ran.load(Ordering::SeqCst), 1, "KDF MUST run on wrong pw");
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
#[ignore = "RED-PHASE: F-VA-3 — correct password runs the same stages as wrong; un-ignore at R5"]
fn f_va_3_correct_password_runs_same_stages_as_wrong() {
    let vault = Vault::seal(b"correct-password", [0x22; 16]);

    let trace_ok = UnlockTrace::default();
    vault.unlock(b"correct-password", &trace_ok).expect("correct pw unlocks");

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
#[ignore = "RED-PHASE: F-VA-3 — all wrong-password causes yield one typed error; un-ignore at R5"]
fn f_va_3_all_wrong_passwords_yield_single_typed_error() {
    let vault = Vault::seal(b"correct-password", [0x33; 16]);
    for wrong in [&b"a"[..], &b"bb"[..], &b"completely-different-and-longer"[..]] {
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
#[ignore = "RED-PHASE: F-VA-3 — coarse timing-invariance (best-effort gross-oracle catch); un-ignore at R5"]
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
