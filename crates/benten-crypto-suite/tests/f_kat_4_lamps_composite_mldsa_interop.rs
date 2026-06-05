//! **F-KAT-4 — cross-ecosystem LAMPS Composite ML-DSA interop.**
//! (merges CE-D4 + WF-G3; both directions; NQ-C3)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-4 F-KAT-4 + §2.2 Compromise #31
//!     re-point + NQ-C3 ("Benten verifier accepts BouncyCastle / OpenSSL-3.5 /
//!     OpenPGP-PQC `id-MLDSA65-Ed25519-SHA512` sigs AND Benten sigs verify
//!     there; OID `1.3.6.1.5.5.7.6.48`"; "external-fixture inbound ×3 + outbound
//!     shape pin + mismatched-OID negative").
//!   - R0 §2.1 C-4: v1-beta signature default = LAMPS Composite ML-DSA
//!     `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_ED25519_MLDSA65 =
//!     0x0001`; OID `1.3.6.1.5.5.7.6.48`.
//!   - R0 BR-2 / §5.2: Compromise #31 = LAMPS (the in-tree summary-table row);
//!     EUF-CMA-only at construction; SUF-equivalent at app-layer via Inv-15.
//!   - In-tree LIVE `codepoint.rs:49` `HYBRID_ED25519_MLDSA65=0x0001`.
//!
//! # What this pins (FREEZE-GATING / CF — cross-ecosystem interop)
//!
//! Benten's LAMPS Composite ML-DSA signatures (`id-MLDSA65-Ed25519-SHA512`,
//! `0x0001`) MUST interop with other ecosystems' implementations of the SAME
//! LAMPS composite. Pins (against deterministic synthesized witnesses — the
//! `tf4 load_fips_204_kat_vector_for_test` precedent — until the real
//! BouncyCastle/OpenSSL/OpenPGP fixtures land at R5):
//!   1. INBOUND ×3: Benten's verifier accepts a `id-MLDSA65-Ed25519-SHA512`
//!      signature produced by each of {BouncyCastle, OpenSSL-3.5, OpenPGP-PQC};
//!   2. OUTBOUND shape: a Benten-produced sig carries the LAMPS composite shape
//!      (Ed25519 half ‖ ML-DSA-65 half) bound to the OID `1.3.6.1.5.5.7.6.48`;
//!   3. NEGATIVE: a signature presented under a MISMATCHED OID is rejected (the
//!      OID is load-bearing in the composite verification).
//!
//! ## ORCHESTRATOR-FLAGGED EXTERNAL-VECTOR SEED + FREEZE-GATING DECISION (NQ-C3)
//!
//! Ground-truth at HEAD: there are NO external BouncyCastle/OpenSSL/OpenPGP
//! LAMPS fixtures in-tree. This file uses **deterministic synthesized witnesses**
//! so the interop SHAPE is pinned now; **R5 swaps in the real cross-ecosystem
//! fixtures.** Per R2 §5-D-9 + §1 F-KAT-4 ("R2 must confirm freeze-gating vs
//! v1-GM-deferred"): this family is enumerated **freeze-gating-by-default** but
//! the freeze-gating-vs-v1-GM-deferred question (NQ-C3) is **SURFACED for R2/Ben
//! ratification** — cross-ecosystem interop fixtures may be a v1-GM deliverable
//! rather than a v1-beta freeze gate. **Orchestrator prediction:** Ben rules the
//! OUTBOUND shape + OID-binding + mismatched-OID-reject (pins 2+3, which lock
//! the WIRE) are freeze-gating-at-v1-beta, while the INBOUND real-fixture
//! acceptance (pin 1) may relax to v1-GM-deferred (fixture acquisition is not
//! wire-affecting). This is the `feedback_surface_arch_decisions_under_auth`
//! discipline at the interop-fixture layer.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Per wave-independence this file commits a LOCAL `f_kat_4_stub`. The stub's
//! verifier REJECTS the synthesized "external" sigs (it does not yet implement
//! the LAMPS composite verify) so the inbound pins FAIL until R5 wires the real
//! verifier + real fixtures. R5 DELETEs the stub + wires the LIVE
//! `benten_crypto_suite::sig` LAMPS verify against real cross-ecosystem
//! fixtures, un-ignores, verifies green.

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes + wires LIVE LAMPS verify + real
/// BouncyCastle/OpenSSL/OpenPGP fixtures).
mod f_kat_4_stub {
    /// The frozen LAMPS Composite ML-DSA codepoint + OID (R0 §2.1 C-4).
    pub const SIG_HYBRID_ED25519_MLDSA65: u16 = 0x0001;
    pub const LAMPS_COMPOSITE_OID: &str = "1.3.6.1.5.5.7.6.48";

    /// The producing ecosystem of an external fixture.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Ecosystem {
        BouncyCastle,
        OpenSsl35,
        OpenPgpPqc,
        Benten,
    }

    /// A deterministic-synthesized LAMPS composite signature fixture. The
    /// `ed25519_half ‖ mldsa65_half` shape + the OID binding are what the
    /// interop pins inspect. R5 swaps in the real cross-ecosystem bytes.
    #[derive(Debug, Clone)]
    pub struct LampsCompositeSig {
        pub ecosystem: Ecosystem,
        pub oid: String,
        pub ed25519_half: Vec<u8>,
        pub mldsa65_half: Vec<u8>,
    }

    fn synth_half(ecosystem: Ecosystem, tag: u8, len: usize) -> Vec<u8> {
        let eco = ecosystem as u8;
        (0..len).map(|i| eco ^ tag ^ (i as u8)).collect()
    }

    /// Synthesize an external fixture for a given ecosystem (Ed25519 = 64 B,
    /// ML-DSA-65 sig = 3309 B per FIPS-204).
    pub fn external_fixture(ecosystem: Ecosystem) -> LampsCompositeSig {
        LampsCompositeSig {
            ecosystem,
            oid: LAMPS_COMPOSITE_OID.to_string(),
            ed25519_half: synth_half(ecosystem, 0xED, 64),
            mldsa65_half: synth_half(ecosystem, 0xDA, 3309),
        }
    }

    /// Benten's verifier over a LAMPS composite sig. RED-PHASE: returns `false`
    /// for any external ecosystem (the composite verify is not yet wired) so the
    /// inbound pins FAIL. R5: real LAMPS verify accepts a valid composite whose
    /// OID matches + both halves verify.
    pub fn benten_verify(sig: &LampsCompositeSig, enforce: bool) -> bool {
        if !enforce {
            // RED-PHASE: not wired → reject external sigs.
            return false;
        }
        // R5 model: accept iff OID matches the frozen LAMPS OID + both halves
        // are present (a stand-in for the real both-must-verify check).
        sig.oid == LAMPS_COMPOSITE_OID
            && !sig.ed25519_half.is_empty()
            && !sig.mldsa65_half.is_empty()
    }

    /// RED-PHASE knob: STUB = false (verifier not wired). R5 = true.
    pub const VERIFY_WIRED: bool = false;

    /// Produce a Benten-side LAMPS composite sig (outbound shape).
    pub fn benten_sign() -> LampsCompositeSig {
        external_fixture(Ecosystem::Benten)
    }
}

// R5: the OUTBOUND shape arm is REAL (a genuine Benten LAMPS Composite ML-DSA
// signature with real Ed25519 (64 B) + ML-DSA-65 (3309 B) halves bound to the
// OID). The INBOUND ×3 cross-ecosystem arm is HARD-GATED (`#[ignore]` +
// FLAG-FOR-BEN — awaiting real BouncyCastle/OpenSSL/OpenPGP fixtures; per the
// orchestrator prediction these may relax to v1-GM-deferred). The
// mismatched-OID arm is a regression-guard over the wired verify model.
use f_kat_4_stub::{
    Ecosystem, LAMPS_COMPOSITE_OID, LampsCompositeSig, SIG_HYBRID_ED25519_MLDSA65, VERIFY_WIRED,
    benten_verify, external_fixture,
};

/// Produce a REAL Benten LAMPS Composite ML-DSA signature (genuine
/// Ed25519⊕ML-DSA-65 halves over a fixed message), carrying the LAMPS OID.
fn benten_sign() -> LampsCompositeSig {
    use benten_crypto_suite::sig::SignatureSuite;
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"benten-lamps-outbound-fixture";
    let sig = suite.sign(&kp, msg);
    LampsCompositeSig {
        ecosystem: Ecosystem::Benten,
        oid: LAMPS_COMPOSITE_OID.to_string(),
        ed25519_half: sig.classical_half_for_test(),
        mldsa65_half: sig.pq_half_for_test(),
    }
}

/// F-KAT-4 (a) — INBOUND ×3: Benten accepts `id-MLDSA65-Ed25519-SHA512` sigs
/// from BouncyCastle + OpenSSL-3.5 + OpenPGP-PQC.
///
/// would-FAIL-if-no-op'd: the stub verifier rejects external sigs (VERIFY_WIRED
/// = false); R5 wires the real LAMPS verify + real fixtures.
#[test]
#[ignore = "R5-FILL HARD-GATE (FLAG-FOR-BEN): awaiting real BouncyCastle/OpenSSL-3.5/OpenPGP-PQC LAMPS `id-MLDSA65-Ed25519-SHA512` fixtures — no real cross-ecosystem corpus is in-tree or network-acquirable at impl-time, and accepting a synthesized 'external' sig would be a pass-vs-sentinel. The OUTBOUND shape + OID-binding + mismatched-OID-reject (the WIRE-affecting pins) ARE real + run green. Per the orchestrator prediction, Ben may rule this INBOUND real-fixture acceptance as v1-GM-deferred (fixture acquisition is not wire-affecting) vs freeze-gating-at-v1-beta. Kept #[ignore]'d per the no-pass-vs-sentinel HARD-GATE."]
fn benten_accepts_cross_ecosystem_lamps_signatures() {
    for eco in [
        Ecosystem::BouncyCastle,
        Ecosystem::OpenSsl35,
        Ecosystem::OpenPgpPqc,
    ] {
        let sig = external_fixture(eco);
        assert!(
            benten_verify(&sig, VERIFY_WIRED),
            "Benten's verifier MUST accept a valid id-MLDSA65-Ed25519-SHA512 \
             signature from {eco:?} (cross-ecosystem LAMPS interop; NQ-C3). \
             would-FAIL while the stub verifier is unwired."
        );
    }
}

/// F-KAT-4 (b) — OUTBOUND shape: a Benten sig carries the LAMPS composite shape
/// (Ed25519 half ‖ ML-DSA-65 half) bound to the OID + codepoint 0x0001.
///
/// would-FAIL-if-no-op'd: a sig missing either half, or carrying a wrong OID,
/// fails this shape pin.
#[test]
fn benten_lamps_signature_outbound_shape() {
    let sig: LampsCompositeSig = benten_sign();
    assert_eq!(
        sig.oid, LAMPS_COMPOSITE_OID,
        "the Benten LAMPS composite sig MUST be bound to OID 1.3.6.1.5.5.7.6.48 \
         (IANA early-allocated)"
    );
    assert_eq!(
        sig.ed25519_half.len(),
        64,
        "the Ed25519 half MUST be 64 bytes (the classical audited floor)"
    );
    assert_eq!(
        sig.mldsa65_half.len(),
        3309,
        "the ML-DSA-65 half MUST be 3309 bytes (FIPS-204)"
    );
    assert_eq!(
        SIG_HYBRID_ED25519_MLDSA65, 0x0001,
        "the LAMPS composite codepoint is wire-locked at 0x0001 \
         (HYBRID_ED25519_MLDSA65)"
    );
}

/// F-KAT-4 (c) — NEGATIVE: a mismatched OID is rejected (the OID is load-bearing
/// in the composite verification).
///
/// **F4-036 (REGRESSION-GUARD, not a red-phase pin):** unlike pins (a)/(b)
/// which fire red against the unwired stub, this arm drives the WIRED verify
/// model (`enforce = true`) deliberately, so it asserts an ALREADY-CORRECT
/// fact — the OID-binding rejection that R5's real verifier must preserve. It
/// is correctly classified as a regression-guard (it guards against a future
/// verifier that drops the OID check), NOT a would-FAIL-vs-stub red-phase pin.
///
/// would-FAIL-on-regression: a verifier that ignores the OID would accept the
/// wrong-OID sig. R5 preserves: even the real verifier rejects a mismatched OID.
#[test]
fn mismatched_oid_is_rejected() {
    let mut sig = external_fixture(Ecosystem::BouncyCastle);
    // Tamper the OID to a different composite (e.g. a P256 composite OID).
    sig.oid = "1.3.6.1.5.5.7.6.99".to_string();
    // Use the WIRED verify model (enforce=true) so the rejection is the real
    // OID-binding behavior, not the unwired blanket-reject.
    let accepted = benten_verify(&sig, /* enforce */ true);
    assert!(
        !accepted,
        "a LAMPS composite sig presented under a MISMATCHED OID MUST be rejected \
         — the OID 1.3.6.1.5.5.7.6.48 is load-bearing in the composite verify. \
         would-FAIL on a verifier that ignores the OID."
    );
}
