//! **F-KAT-2 — libcrux `check-secret-independence` CI gate (CE-D2).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-4 F-KAT-2 ("the libcrux hax/F*
//!     ML-KEM secret-independence CI gate is wired + green; absence = freeze
//!     blocker"; class = FG (CI)).
//!   - R0 §2.2 Q1: libcrux-ml-kem "verified secret-independence via hax/F*;
//!     `check-secret-independence` CI gate".
//!   - R0 §3.2 Wave-0 exit gate (m-2): "`check-secret-independence` in CI".
//!   - R0 §5.2 Compromise #30 (unaudited-PQ; CLOSES at v1-GM / C-GM-AUDIT) +
//!     #32 (ML-KEM-768 Decap CT side-channel — the libcrux CT-mitigation the
//!     secret-independence gate witnesses).
//!
//! # What this pins (FREEZE-GATING / CI — presence + enablement)
//!
//! The libcrux secret-independence gate (its hax/F*-derived constant-time /
//! secret-independence property check) MUST be WIRED into CI and ENABLED.
//! Its ABSENCE is a freeze blocker (an un-witnessed ML-KEM-768 Decap
//! side-channel posture — Compromise #32). Pins:
//!   1. the gate is REGISTERED (a named CI step / feature toggle exists);
//!   2. the gate is ENABLED (not declared-then-skipped);
//!   3. the gate's subject is the ML-KEM (libcrux) impl (not a no-op target).
//!
//! This is a CI-integration presence pin (the landscape's "FG (CI)" class), not
//! a runtime crypto round-trip. The discipline mirrors the in-tree presence-pin
//! shape used for other CI gates: assert the gate's registration record + its
//! enabled-flag, with a foil that would FAIL on a declared-but-disabled gate.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Ground-truth at HEAD: no `check-secret-independence` step exists (`grep -rn
//! check-secret-independence .github/ Cargo.toml` → ZERO; libcrux is not yet a
//! dep). Per wave-independence this file commits a LOCAL `f_kat_2_stub`
//! modelling the gate-registration record. The stub deliberately reports the
//! gate as REGISTERED-BUT-DISABLED so the enabled pin FAILS until R5 wires the
//! real CI gate. R5 DELETEs the stub + wires the real gate-registration probe
//! (a build-script/CI-manifest assertion against the actual libcrux feature +
//! `.github/workflows` step), un-ignores, verifies green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18)
//!
//! The pins assert the gate is registered AND enabled AND targets libcrux-ml-kem.
//! The stub's disabled-flag makes the enabled pin fail; a declared-then-skipped
//! gate (the classic CI no-op) is exactly what the enabled pin forbids.

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes + wires the real CI-gate registration
/// probe against `.github/workflows` + the libcrux feature).
mod f_kat_2_stub {
    /// A model of the CI secret-independence gate's registration record. R5's
    /// real probe reads the actual CI manifest + the libcrux feature flag.
    #[derive(Debug, Clone)]
    pub struct SecretIndependenceGate {
        pub registered: bool,
        pub enabled: bool,
        pub target_impl: &'static str,
    }

    /// Read the gate's registration. RED-PHASE: registered but DISABLED + the
    /// target is the placeholder — so the enabled + target pins FAIL until R5.
    pub fn read_secret_independence_gate() -> SecretIndependenceGate {
        SecretIndependenceGate {
            registered: false,        // R5: true (the CI step exists)
            enabled: false,           // R5: true (the step is not skipped)
            target_impl: "<unwired>", // R5: "libcrux-ml-kem"
        }
    }

    pub const EXPECTED_TARGET: &str = "libcrux-ml-kem";
}

use f_kat_2_stub::{EXPECTED_TARGET, read_secret_independence_gate};

/// F-KAT-2 (a) — the secret-independence CI gate is REGISTERED.
///
/// would-FAIL-if-no-op'd: the stub reports `registered=false`; R5 wires the real
/// CI step. Absence of the gate is a freeze blocker (Compromise #32).
#[test]
#[ignore = "R5-FILL HARD-GATE (FLAG-FOR-BEN): rides the SAME libcrux-add decision f_kat_1 surfaces — the `check-secret-independence` CI gate cannot be honestly REGISTERED + ENABLED + targeting-libcrux without first wiring libcrux-ml-kem into the build + adding the `.github/workflows` step. Reporting registered=true with no real CI step would be a pass-vs-sentinel. Ben decision: add libcrux-ml-kem now (then wire the CI gate + un-ignore) vs defer the libcrux swap + its secret-independence gate to v1-GM. Kept #[ignore]'d per the no-pass-vs-sentinel HARD-GATE (#32 freeze-blocker tracked)."]
fn secret_independence_gate_is_registered() {
    let gate = read_secret_independence_gate();
    assert!(
        gate.registered,
        "the libcrux check-secret-independence CI gate MUST be REGISTERED — its \
         absence is a freeze blocker (un-witnessed ML-KEM-768 Decap \
         side-channel posture; Compromise #32). would-FAIL while the stub \
         reports registered=false."
    );
}

/// F-KAT-2 (b) — the gate is ENABLED (not declared-then-skipped).
///
/// The classic CI no-op is a step that exists but is gated off. would-FAIL-if-
/// no-op'd: the stub reports `enabled=false`; R5 ensures the step runs.
#[test]
#[ignore = "R5-FILL HARD-GATE (FLAG-FOR-BEN): rides the libcrux-add decision (see the registered test) — kept #[ignore]'d until libcrux + the CI gate are wired."]
fn secret_independence_gate_is_enabled() {
    let gate = read_secret_independence_gate();
    assert!(
        gate.registered && gate.enabled,
        "the secret-independence gate MUST be ENABLED, not merely declared — a \
         declared-then-skipped CI step is a no-op that does not witness the \
         constant-time property. would-FAIL while the stub reports \
         enabled=false."
    );
}

/// F-KAT-2 (c) — the gate targets the libcrux ML-KEM impl (not a no-op target).
///
/// would-FAIL-if-no-op'd: the stub's `<unwired>` target; R5 points it at
/// `libcrux-ml-kem`.
#[test]
#[ignore = "R5-FILL HARD-GATE (FLAG-FOR-BEN): rides the libcrux-add decision (see the registered test) — kept #[ignore]'d until libcrux + the CI gate are wired."]
fn secret_independence_gate_targets_libcrux_ml_kem() {
    let gate = read_secret_independence_gate();
    assert_eq!(
        gate.target_impl, EXPECTED_TARGET,
        "the secret-independence gate MUST target the libcrux ML-KEM impl (the \
         hax/F*-verified one) — not a placeholder. would-FAIL while the stub \
         target is `{}`.",
        gate.target_impl
    );
}
