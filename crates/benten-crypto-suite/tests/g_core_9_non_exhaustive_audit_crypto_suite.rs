//! R10-council fix wave (F-02 / F-11) — benten-crypto-suite
//! `#[non_exhaustive]` audit arm-coverage pins.
//!
//! Companion to `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs`
//! and `crates/benten-drop/tests/g_core_9_non_exhaustive_audit_drop.rs`. The
//! crypto-suite audit lives here because benten-engine does NOT depend on
//! benten-crypto-suite's swap-matrix surface directly in the audit test.
//!
//! The 5 `Swap*` matrix enums (`SwapKeypair` / `SwapPublicKey` /
//! `SwapRecipientKeypair` / `SwapRecipientPublic` / `SwapRecipientSecret`) plus
//! `DischargeDisposition` are SemVer-frozen `#[non_exhaustive]` (§11
//! SemVer-readiness) so a future arm lands ADDITIVELY. Because these tests are a
//! SEPARATE crate from the library, the `_` catch-all stays reachable ONLY while
//! the attribute is present — REMOVING `#[non_exhaustive]` turns the catch-all
//! into a `unreachable_patterns` warning that `-D warnings` promotes to a build
//! break (§11 HALT-AND-SURFACE). That is the arm-coverage pin.
//!
//! Carve-out preserved: the drop-side `benten_drop::layer_c::BindingContext` is
//! INTENTIONALLY exhaustive-by-design (wire-keying: one codepoint per variant),
//! audited on the drop side, NOT covered here.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_crypto_suite::discharge::DischargeDisposition;
use benten_crypto_suite::swap_matrix::{
    SwapKeypair, SwapMatrix, SwapPublicKey, SwapRecipientKeypair, SwapRecipientPublic,
    SwapRecipientSecret,
};

#[test]
fn swap_keypair_audit_arm_coverage_non_exhaustive() {
    fn audit(k: &SwapKeypair) -> &'static str {
        match k {
            SwapKeypair::Hybrid(_) => "Hybrid",
            SwapKeypair::PurePq(_) => "PurePq",
            _ => "Unknown",
        }
    }
    let kp = SwapMatrix::v1_beta_default().generate_keypair_for_test();
    assert_eq!(audit(&kp), "Hybrid");
}

#[test]
fn swap_public_key_audit_arm_coverage_non_exhaustive() {
    fn audit(p: &SwapPublicKey) -> &'static str {
        match p {
            SwapPublicKey::Hybrid(_) => "Hybrid",
            SwapPublicKey::PurePq(_) => "PurePq",
            _ => "Unknown",
        }
    }
    let pk = SwapMatrix::v1_beta_default()
        .generate_keypair_for_test()
        .public();
    assert_eq!(audit(&pk), "Hybrid");
}

#[test]
fn swap_recipient_keypair_audit_arm_coverage_non_exhaustive() {
    fn audit(k: &SwapRecipientKeypair) -> &'static str {
        match k {
            SwapRecipientKeypair::Cipher(_) => "Cipher",
            SwapRecipientKeypair::None => "None",
            SwapRecipientKeypair::PurePqMlKem(_) => "PurePqMlKem",
            _ => "Unknown",
        }
    }
    let rk = SwapMatrix::v1_beta_default().generate_recipient_keypair_for_test();
    assert_eq!(audit(&rk), "Cipher");
}

#[test]
fn swap_recipient_public_audit_arm_coverage_non_exhaustive() {
    fn audit(p: &SwapRecipientPublic<'_>) -> &'static str {
        match p {
            SwapRecipientPublic::Cipher(_) => "Cipher",
            SwapRecipientPublic::None => "None",
            SwapRecipientPublic::PurePqMlKem(_) => "PurePqMlKem",
            _ => "Unknown",
        }
    }
    let rk = SwapMatrix::v1_beta_default().generate_recipient_keypair_for_test();
    assert_eq!(audit(&rk.public()), "Cipher");
}

#[test]
fn swap_recipient_secret_audit_arm_coverage_non_exhaustive() {
    fn audit(s: &SwapRecipientSecret<'_>) -> &'static str {
        match s {
            SwapRecipientSecret::Cipher(_) => "Cipher",
            SwapRecipientSecret::None => "None",
            SwapRecipientSecret::PurePqMlKem(_) => "PurePqMlKem",
            _ => "Unknown",
        }
    }
    let rk = SwapMatrix::v1_beta_default().generate_recipient_keypair_for_test();
    assert_eq!(audit(&rk.secret()), "Cipher");
}

#[test]
fn discharge_disposition_audit_arm_coverage_non_exhaustive() {
    fn audit(d: DischargeDisposition) -> &'static str {
        match d {
            DischargeDisposition::Deleted => "Deleted",
            DischargeDisposition::CratePrivate => "CratePrivate",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(DischargeDisposition::CratePrivate), "CratePrivate");
}
