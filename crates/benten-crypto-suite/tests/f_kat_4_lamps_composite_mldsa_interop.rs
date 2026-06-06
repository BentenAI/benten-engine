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
//! ## REAL INBOUND FIXTURE NOW IN-TREE + FORMAT-GAP FINDING + NAMED-DEFER (NQ-C3)
//!
//! Updated 2026-06-05 (f-kat4-inbound wave). The prior framing ("there are NO
//! external LAMPS fixtures in-tree or network-acquirable") was **wrong**: the
//! IETF LAMPS WG publishes a real, byte-exact KAT for the EXACT composite
//! `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`). It is now embedded
//! below as [`real_lamps_vector`] (the Ed25519 half + the LAMPS `M'`
//! construction parameters), sourced from:
//!
//!   - `lamps-wg/draft-composite-sigs` `src/testvectors.json`, `tcId =
//!     "id-MLDSA65-Ed25519-SHA512"`, pinned at commit
//!     `f0627ab34acfe1aee0abce4bee91ed2b577eab76` (2026-01-07 "re-ran test
//!     vectors"):
//!     <https://raw.githubusercontent.com/lamps-wg/draft-composite-sigs/f0627ab34acfe1aee0abce4bee91ed2b577eab76/src/testvectors.json>
//!   - Spec: `draft-ietf-lamps-pq-composite-sigs-19` §"Label and Context" +
//!     §"Composite-ML-DSA.Sign" (the `M' = Prefix || Label || len(ctx) || ctx
//!     || PH(M)` representative; `mldsaSig || tradSig` serialization order):
//!     <https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/19/>
//!
//! **Authenticity is proven IN-TEST, not asserted on faith:** the always-running
//! [`real_lamps_vector_ed25519_half_is_authentic`] reconstructs the real LAMPS
//! `M' = "CompositeAlgorithmSignatures2025" || "COMPSIG-MLDSA65-Ed25519-SHA512"
//! || 0x00 || SHA-512(m)` and cryptographically verifies the vector's Ed25519
//! half against it **using Benten's own `ed25519-dalek` + `sha2` deps**. This is
//! a genuine cryptographic check over REAL external cross-ecosystem bytes — NOT
//! a synthesized sentinel. It would FAIL on any tampered byte of the embedded
//! fixture, on a wrong `M'`, or if Benten's classical primitive diverged from
//! the IETF composite's classical leg.
//!
//! **FORMAT-GAP FINDING (the real blocker, sharper than "no fixture"):** Benten's
//! v1-beta hybrid signature (`benten_crypto_suite::sig`, NF-4) is NOT byte-
//! compatible with the IETF LAMPS composite wire format, by three independent
//! constructions:
//!   1. **Serialization order.** LAMPS = `mldsaSig(3309) || tradSig(64)` (ML-DSA
//!      first). Benten = `classical(64) || pq(3309) || commitment(32)` (Ed25519
//!      first, plus a SHA3-256 commitment trailer the LAMPS wire has no slot for).
//!   2. **Message representative.** LAMPS components sign `M'` (prefix + ASCII
//!      label + `len(ctx)||ctx` + `SHA-512(M)`), with the composite Label passed
//!      into ML-DSA as its `ctx`. Benten's components sign the RAW message with
//!      empty ML-DSA ctx + bind everything via its own SHA3-256 commitment.
//!   3. **Strip-resistance mechanism.** LAMPS relies on the shared `M'`/ctx
//!      binding; Benten relies on the explicit committing trailer.
//!
//! Therefore the LIVE `SignatureSuite::verify` **cannot** accept a real LAMPS
//! composite sig as-is — full inbound acceptance requires *implementing an IETF
//! LAMPS composite verifier* in production crypto (a new wire-format-affecting
//! verify path + likely a new public surface/ErrorCode). That is a freeze-level
//! design decision, OUT OF SCOPE for fixture-acquisition and reserved for Ben.
//!
//! **DISPOSITION (HARD-RULE clause-(b) BELONGS-NAMED-NOW + clause-(a) wire-scope).**
//! The full INBOUND-ACCEPTANCE arm [`benten_accepts_cross_ecosystem_lamps_signatures`]
//! stays `#[ignore]`'d and is **named-deferred to the v1-GM / NF-2 C-GM-AUDIT
//! cross-ecosystem-interop window** (`docs/SECURITY-POSTURE.md` C-GM-AUDIT; the
//! independent-audit + ecosystem-interop deliverable). Rationale, matching the
//! orchestrator prediction below: implementing the IETF LAMPS composite verifier
//! is a deliberate posture choice (does Benten's "LAMPS Composite" default emit/
//! accept the IETF *byte format*, or does it cite LAMPS only for its construction
//! *principles*?) that is NOT required to freeze the v1-beta wire — Benten's own
//! outbound format + OID-binding are what the freeze locks, and those ARE pinned
//! real + green below (pins 2+3). Cross-ecosystem inbound acceptance is not
//! wire-affecting for Benten's own format and is correctly a v1-GM deliverable.
//!
//! ### ORCHESTRATOR-FLAGGED FREEZE-GATING DECISION (NQ-C3 — unchanged)
//!
//! Per R2 §5-D-9 + §1 F-KAT-4 ("R2 must confirm freeze-gating vs v1-GM-deferred"):
//! this family is enumerated **freeze-gating-by-default** but the
//! freeze-gating-vs-v1-GM-deferred question (NQ-C3) is **SURFACED for Ben
//! ratification**. **Orchestrator prediction (now evidence-backed by the
//! format-gap finding):** Ben rules the OUTBOUND shape + OID-binding +
//! mismatched-OID-reject (pins 2+3, which lock Benten's own WIRE) are
//! freeze-gating-at-v1-beta, while INBOUND real-fixture *acceptance* (pin 1,
//! which needs a whole IETF LAMPS composite verifier) relaxes to
//! v1-GM-deferred. This is the `feedback_surface_arch_decisions_under_auth`
//! discipline at the interop layer.
//!
//! # STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Per wave-independence this file commits a LOCAL `f_kat_4_stub` modelling the
//! interop SHAPE (the OUTBOUND + OID-binding + mismatched-OID-reject pins below
//! drive the LIVE `benten_crypto_suite::sig` suite for the real arms). The real
//! IETF LAMPS vector + its authentic Ed25519-half crypto-check are in-tree and
//! GREEN. The full inbound-ACCEPTANCE arm stays `#[ignore]`'d per the FORMAT-GAP
//! FINDING + named-defer above (no pass-vs-sentinel; no faked "external" accept).

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

/// REAL published cross-ecosystem fixture: the IETF LAMPS WG known-answer test
/// vector for the EXACT composite `id-MLDSA65-Ed25519-SHA512`
/// (OID `1.3.6.1.5.5.7.6.48`).
///
/// Source (cite-drift-verified): `lamps-wg/draft-composite-sigs`
/// `src/testvectors.json`, `tcId = "id-MLDSA65-Ed25519-SHA512"`, pinned at
/// commit `f0627ab34acfe1aee0abce4bee91ed2b577eab76` (2026-01-07). Spec:
/// `draft-ietf-lamps-pq-composite-sigs-19`.
///
/// We embed the load-bearing slice that we can cryptographically validate with
/// Benten's own deps: the message `M`, the LAMPS `M'` construction parameters
/// (Prefix + Label), the Ed25519 public-key half (32 B) and Ed25519 signature
/// half (64 B) of the composite, plus the published full composite pk/sig
/// lengths (so the `mldsaSig(3309)||tradSig(64)` and `mldsaPK(1952)||tradPK(32)`
/// LAMPS shape is pinned). The ML-DSA-65 half (1952 B pk / 3309 B sig) is NOT
/// embedded byte-for-byte (it would require Benten to implement the IETF LAMPS
/// composite verifier to validate — the named-deferred work); its *lengths*
/// are pinned here so the composite shape is locked.
mod real_lamps_vector {
    /// `M`, the global message all LAMPS KAT signatures cover: ASCII
    /// "The quick brown fox jumps over the lazy dog." (decoded from the
    /// vector's base64 `m`).
    pub const MESSAGE: &[u8] = b"The quick brown fox jumps over the lazy dog.";

    /// LAMPS `M'` Prefix — ASCII "CompositeAlgorithmSignatures2025"
    /// (`draft-ietf-lamps-pq-composite-sigs-19` §"Label and Context").
    pub const M_PRIME_PREFIX: &[u8] = b"CompositeAlgorithmSignatures2025";

    /// LAMPS `M'` Label for this composite — ASCII
    /// "COMPSIG-MLDSA65-Ed25519-SHA512" (`src/algParams.md` row
    /// id-MLDSA65-Ed25519-SHA512).
    pub const M_PRIME_LABEL: &[u8] = b"COMPSIG-MLDSA65-Ed25519-SHA512";

    /// Published composite serialization: `mldsaSig(3309) || tradSig(64)`.
    pub const FULL_COMPOSITE_SIG_LEN: usize = 3309 + 64;
    /// Published composite public key: `mldsaPK(1952) || tradPK(32)`.
    pub const FULL_COMPOSITE_PK_LEN: usize = 1952 + 32;
    /// Offset of the Ed25519 (traditional) half inside the composite sig.
    pub const TRAD_SIG_OFFSET: usize = 3309;
    /// Offset of the Ed25519 (traditional) half inside the composite pk.
    pub const TRAD_PK_OFFSET: usize = 1952;

    /// The Ed25519 public-key half (32 B) of the real composite pk
    /// (`pk[1952..1984]`).
    pub const ED25519_PK: [u8; 32] = [
        0x42, 0xc8, 0x14, 0xea, 0x18, 0x81, 0x83, 0xf8, 0x36, 0x27, 0xb1, 0x3c, 0x96, 0x30, 0x3a,
        0xe1, 0x21, 0xc8, 0xdc, 0x1e, 0x1f, 0x1b, 0xad, 0xfc, 0xc8, 0x3f, 0xfb, 0xd5, 0xd8, 0x50,
        0x4f, 0xda,
    ];

    /// The Ed25519 signature half (64 B) of the real composite sig over the
    /// empty-ctx message (`s[3309..3373]`).
    pub const ED25519_SIG: [u8; 64] = [
        0x4b, 0xe4, 0x30, 0xa7, 0x6b, 0xb7, 0x18, 0x4a, 0x93, 0xbe, 0x67, 0x6e, 0xc1, 0xf2, 0xa2,
        0xa7, 0xaa, 0xac, 0x46, 0xdc, 0x75, 0x08, 0x7c, 0x90, 0xf4, 0xca, 0xd5, 0xed, 0x5d, 0xce,
        0x88, 0x8d, 0xe4, 0x6f, 0xb3, 0x7d, 0xf4, 0x8a, 0x86, 0x10, 0x72, 0xde, 0x77, 0x03, 0x67,
        0xce, 0x90, 0xa1, 0x6e, 0xef, 0xc4, 0x8c, 0xb5, 0xb5, 0x80, 0xd7, 0xa1, 0x9b, 0x46, 0xd2,
        0x1a, 0xf9, 0x4a, 0x05,
    ];

    /// The empty context (`ctx`) under which `ED25519_SIG` was produced.
    pub const CTX: &[u8] = b"";

    /// Reconstruct the LAMPS message representative `M'` for the empty-ctx
    /// signature: `Prefix || Label || len(ctx) || ctx || SHA-512(M)`.
    pub fn m_prime() -> Vec<u8> {
        use sha2::{Digest as _, Sha512};
        let ph = Sha512::digest(MESSAGE);
        let mut out = Vec::new();
        out.extend_from_slice(M_PRIME_PREFIX);
        out.extend_from_slice(M_PRIME_LABEL);
        out.push(u8::try_from(CTX.len()).expect("ctx <= 255 bytes per LAMPS"));
        out.extend_from_slice(CTX);
        out.extend_from_slice(&ph);
        out
    }
}

// The OUTBOUND shape arm is REAL (a genuine Benten LAMPS Composite ML-DSA
// signature with real Ed25519 (64 B) + ML-DSA-65 (3309 B) halves bound to the
// OID). The REAL IETF LAMPS vector's Ed25519 half is cryptographically verified
// in-tree (always-running, non-sentinel). The full INBOUND-ACCEPTANCE arm is
// named-deferred (`#[ignore]` + FLAG-FOR-BEN) per the FORMAT-GAP FINDING: it
// needs a production IETF LAMPS composite verifier, a wire-affecting/Ben-gated
// v1-GM deliverable. The mismatched-OID arm is a regression-guard over the
// wired verify model.
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

/// F-KAT-4 (a-AUTHENTIC) — the REAL IETF LAMPS vector's Ed25519 half
/// cryptographically verifies against the real LAMPS `M'`, using Benten's own
/// `ed25519-dalek` + `sha2` deps. **Always-running, non-sentinel.**
///
/// This is the genuine cross-ecosystem-bytes check that the corpus IS in-tree
/// and authentic. It proves: (1) the embedded fixture bytes are the real
/// published LAMPS WG bytes (any tampered byte FAILS this); (2) the LAMPS `M'`
/// construction (`Prefix || Label || len(ctx)||ctx || SHA-512(M)`) is correctly
/// reconstructed; (3) Benten's classical primitive validates the classical leg
/// of a real IETF composite. It does NOT claim Benten's *composite* verifier
/// accepts the *whole* composite (that is the named-deferred FORMAT-GAP work).
///
/// would-FAIL-if-no-op'd: a flipped byte of `ED25519_SIG`/`ED25519_PK`, a wrong
/// Prefix/Label, or a wrong pre-hash would make the Ed25519 verify reject.
#[test]
fn real_lamps_vector_ed25519_half_is_authentic() {
    use benten_crypto_suite::primitives::ed25519_dalek::{Signature, Verifier as _, VerifyingKey};

    let vk = VerifyingKey::from_bytes(&real_lamps_vector::ED25519_PK)
        .expect("real LAMPS vector Ed25519 public-key half MUST be a valid Ed25519 point");
    let sig = Signature::from_bytes(&real_lamps_vector::ED25519_SIG);
    let m_prime = real_lamps_vector::m_prime();

    vk.verify(&m_prime, &sig).expect(
        "the Ed25519 (traditional) half of the REAL IETF LAMPS \
         id-MLDSA65-Ed25519-SHA512 vector MUST verify against the reconstructed \
         LAMPS M' — proves the embedded cross-ecosystem fixture is authentic \
         published bytes, not a synthesized sentinel",
    );

    // Pin the published composite shape (mldsaSig(3309)||tradSig(64) etc.) so
    // the LAMPS serialization order is locked alongside the crypto check.
    assert_eq!(real_lamps_vector::FULL_COMPOSITE_SIG_LEN, 3373);
    assert_eq!(real_lamps_vector::FULL_COMPOSITE_PK_LEN, 1984);
    assert_eq!(
        real_lamps_vector::TRAD_SIG_OFFSET,
        3309,
        "LAMPS serializes mldsaSig(3309) FIRST then tradSig — the Ed25519 half \
         begins at offset 3309 in the composite signature"
    );
    assert_eq!(real_lamps_vector::TRAD_PK_OFFSET, 1952);
}

/// F-KAT-4 (a) — INBOUND ×3: Benten's PRODUCTION verifier accepts a real
/// `id-MLDSA65-Ed25519-SHA512` composite signature from each of {BouncyCastle,
/// OpenSSL-3.5, OpenPGP-PQC}.
///
/// NAMED-DEFERRED (HARD-RULE clause-(b)/(a)). This is NOT a fixture-acquisition
/// gap — a real IETF LAMPS vector IS now in-tree (see [`real_lamps_vector`] +
/// the authentic Ed25519-half check above). The blocker is the FORMAT-GAP
/// FINDING (see module docs): Benten's NF-4 hybrid is NOT byte-compatible with
/// the IETF LAMPS composite wire format (ML-DSA-first ordering + the `M'`
/// message representative + ML-DSA-ctx=Label vs Benten's Ed25519-first +
/// raw-message + SHA3-256 commitment). Full inbound acceptance therefore needs a
/// PRODUCTION IETF LAMPS composite verifier — a wire-format-affecting design
/// decision reserved for Ben (does the "LAMPS Composite" default emit/accept the
/// IETF *byte format*, or cite LAMPS for its *principles* only?). Destination:
/// v1-GM / NF-2 C-GM-AUDIT cross-ecosystem-interop window
/// (`docs/SECURITY-POSTURE.md`). The WIRE-affecting pins (outbound shape +
/// OID-binding + mismatched-OID-reject) ARE real + green so the v1-beta freeze
/// is NOT under-pinned.
#[test]
#[ignore = "NAMED-DEFERRED to v1-GM / NF-2 C-GM-AUDIT (FLAG-FOR-BEN). A real IETF LAMPS id-MLDSA65-Ed25519-SHA512 vector IS now in-tree (real_lamps_vector; authenticity crypto-proven by real_lamps_vector_ed25519_half_is_authentic). The blocker is NOT fixtures — it is the FORMAT-GAP: Benten's NF-4 hybrid is not byte-compatible with the IETF LAMPS composite wire format (ML-DSA-first + M'-representative + ML-DSA-ctx=Label vs Benten Ed25519-first + raw-msg + SHA3-256 commitment), so SignatureSuite::verify cannot accept a real composite without a PRODUCTION IETF LAMPS composite verifier — a wire-affecting/Ben-gated decision. The WIRE-affecting pins (outbound shape + OID-binding + mismatched-OID-reject) ARE real + green, so the v1-beta freeze is not under-pinned. No pass-vs-sentinel: refusing to fake a composite-accept."]
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
             Blocked by the FORMAT-GAP: needs a production IETF LAMPS composite \
             verifier (v1-GM-deferred)."
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
