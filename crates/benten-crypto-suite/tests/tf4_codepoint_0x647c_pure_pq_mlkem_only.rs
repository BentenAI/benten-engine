//! TF-4 — pre-G-CORE-9-FREEZE wire-format byte-pin for codepoint `0x647c`
//! (pure-PQ ML-KEM-768-only swap-matrix arm).
//!
//! **Pre-G-CORE-9-FREEZE 2026-05-24 ratification.** The pre-existing
//! G-CORE-3c (#1341) implementation reused codepoint `0x647b` for the
//! pure-PQ ML-KEM-768-only arm of the swap matrix
//! (the module-private `EncryptionArm::PurePqMlKem768Only` variant
//! inside `benten_crypto_suite::swap_matrix`; cite plain because the
//! enum is not `pub` so an intra-doc-link bracket form fails
//! `RUSTDOCFLAGS=-D warnings`),
//! conflating it with the future ML-KEM⊕HQC PQ⊕PQ end-state (also
//! `0x647b`). When the `AUDIT_LANDED_PURE_PQ_FLAG` flips at v1-GM,
//! deployments would silently disagree across this codepoint reuse —
//! a wire-format break.
//!
//! **Ben-ratified resolution (option (a)):** mint a new codepoint
//! `0x647c` for the pure-PQ-ML-KEM-only arm; keep `0x647b` strictly
//! reserved for the ML-KEM⊕HQC future. This pin asserts:
//!
//! 1. The named constant `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY`
//!    raw-value is exactly `0x647c` (the wire-format byte pin — this
//!    is what locks the codepoint at G-CORE-9 FREEZE).
//! 2. The constant is DISTINCT from `HYBRID_MLKEM768_HQC` (`0x647b`),
//!    not aliased.
//! 3. The cipher-suite dispatcher typed-rejects `0x647c` at the
//!    [`benten_crypto_suite::cipher_suite::CipherSuite::resolve`] level
//!    (the pure-PQ arm is reachable ONLY via the named
//!    [`benten_crypto_suite::swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`]
//!    constructor; gating preserves the C11b safety invariant).
//! 4. The [`benten_crypto_suite::swap_matrix::SwapMatrix`] PurePq arm
//!    surfaces `0x647c` via `cipher_suite_codepoint()` (the integration
//!    point — proves the swap-matrix routes to the new codepoint, not
//!    the old conflated `0x647b`).

use benten_crypto_suite::cipher_suite::CipherSuite;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;

/// **WIRE-FORMAT BYTE-PIN.** The raw value of
/// `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY` MUST be exactly
/// `0x647c`. G-CORE-9 FREEZE locks this value as wire-canonical.
#[test]
fn tf4_pure_pq_mlkem_only_codepoint_raw_is_0x647c() {
    assert_eq!(
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.raw(),
        0x647c,
        "pre-G-CORE-9-FREEZE wire-format byte-pin: \
         PURE_PQ_MLKEM768_ONLY MUST equal 0x647c (Ben-ratified \
         2026-05-24 — distinct codepoint from `0x647b` ML-KEM⊕HQC \
         to prevent wire-format collision when AUDIT_LANDED flips \
         at v1-GM)"
    );
}

/// `0x647c` MUST be a DISTINCT codepoint from `0x647b` (the
/// ML-KEM⊕HQC future end-state). Aliasing would re-introduce the
/// wire-format collision this codepoint mint is closing.
#[test]
fn tf4_pure_pq_distinct_from_ml_kem_hqc_future() {
    assert_ne!(
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.raw(),
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.raw(),
        "PURE_PQ_MLKEM768_ONLY (0x647c) MUST be distinct from \
         HYBRID_MLKEM768_HQC (0x647b future end-state) — aliasing \
         would defeat the whole point of the 2026-05-24 codepoint mint"
    );
    assert_eq!(
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.raw(),
        0x647b,
        "HYBRID_MLKEM768_HQC MUST remain at 0x647b (strictly reserved \
         for ML-KEM⊕HQC future end-state per CLAUDE.md baked-in #5)"
    );
}

/// The cipher-suite dispatcher MUST typed-reject `0x647c` at the
/// [`CipherSuite::resolve`] level. The pure-PQ arm is reachable ONLY
/// via the named [`SwapMatrix::try_pure_pq_sole_trust_path`]
/// constructor (which gates on `AUDIT_LANDED_PURE_PQ_FLAG`).
#[test]
fn tf4_cipher_suite_resolve_typed_rejects_0x647c() {
    let outcome = CipherSuite::resolve(CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY);
    assert!(matches!(
        outcome,
        Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647c })
    ));
}

/// The swap-matrix `try_pure_pq_sole_trust_path` constructor MUST
/// reject pre-audit with the named `AuditNotLandedPurePqRejected` arm
/// — proves the C11b gate is intact and the pure-PQ codepoint is
/// reachable ONLY via the named constructor (which itself is gated).
///
/// The internal cfg-gated test in `swap_matrix.rs` covers the codepoint
/// routing surface directly (the bypass constructor is `#[cfg(test)]`
/// scoped to that crate's unit-test build).
#[test]
fn tf4_swap_matrix_pure_pq_path_is_named_audit_gate() {
    use benten_crypto_suite::swap_matrix::{SwapMatrix, SwapMatrixError};
    let outcome = SwapMatrix::try_pure_pq_sole_trust_path();
    assert!(
        matches!(
            outcome,
            Err(SwapMatrixError::AuditNotLandedPurePqRejected { .. })
        ),
        "pure-PQ-sole-trust-path MUST be named-audit-gated pre-v1-GM \
         (codepoint 0x647c is reachable ONLY through this constructor; \
         this assertion guards the gate, the byte-pin tests above guard \
         the codepoint identity)"
    );
}
