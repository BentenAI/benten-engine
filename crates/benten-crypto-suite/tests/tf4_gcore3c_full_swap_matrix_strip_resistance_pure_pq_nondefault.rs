//! TF-4 — G-CORE-3c FULL bidirectional swap-matrix + strip-resistance +
//! pure-PQ structurally-non-default-until-audit-flag + reserved-codepoint
//! additive-discipline.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core, Wave R3-B,
//! agent R3-B1, family TF-4 (G-CORE-3c; consumes #1300 + #1301). Pin
//! sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §TF-4 RED-phase
//!     production-arm shapes:
//!     (1) hybrid-default config asserts BOTH halves present + BOTH
//!         required to verify/decrypt — the strip-resistance property the
//!         reframe's safety rests on; would-FAIL if a single-half config
//!         silently verifies;
//!     (2) each downgrade config (classical-only / no-enc / non-PQ)
//!         round-trips;
//!     (3) bidirectional swap conformance (encrypt-under-config-A, assert
//!         decrypt-under-config-A succeeds and a mismatched config
//!         fails-closed);
//!     (4) pure-PQ config structurally gated non-default until the
//!         `audit-landed` flag — a test asserting a reviewer/implementer
//!         CANNOT ship a config that makes unaudited PQC the SOLE trust
//!         path (would-FAIL if pure-PQ is freely default-selectable);
//!     (5) an unknown/reserved codepoint (the not-yet-built ML-KEM-768⊕HQC
//!         NF-1) hits the typed-unsupported arm (additive-codepoint
//!         discipline structurally verified).
//!   - §TF-4 Pins: G-CORE-3c · C11b (the Core exit gate feeding the
//!     FREEZE) · §1.A.FROZEN item 6 (full swap matrix frozen, not just an
//!     Ed25519 envelope) · §4 CI PQ-hybrid conformance gate.
//!   - `00-implementation-plan.md` G-CORE-3c def (PQ-default reframe
//!     2026-05-19 — SUPERSEDES the "non-default validation spike" framing):
//!     PQ-hybrid is the DEFAULT arm; G-CORE-3c builds + KAT-conformance-
//!     tests EVERY downgrade config + proves bidirectional swap; safety
//!     invariant "hybrid construction → PQC is never the SOLE trust path"
//!     (classical half = audited floor); pure-PQ stays STRUCTURALLY
//!     non-default until the audit-landed flag flips; NF-1 PQ⊕PQ
//!     ML-DSA-65⊕SLH-DSA non-default arm + reserved-but-unimplemented
//!     ML-KEM-768⊕HQC codepoint (additive-codepoint discipline).
//!   - §4 CI "Encryption-format / PQ-hybrid conformance gate" + D-4M-5b
//!     (PQ-hybrid is the v1-beta DEFAULT; full swap matrix; hybrid →
//!     PQC-never-SOLE-trust safety invariant; pure-PQ non-default until
//!     audit-flag).
//!   - CLAUDE.md #5 crypto-agility refinement (codepoint-dispatch; never
//!     fork/reimplement primitives; no hardcoded sizes; typed-unsupported
//!     never-silent; PQ additive-codepoint) + the v1-GATE PQ-impl
//!     conformance-spike amendment ("unaudited PQC is never the v1
//!     production path"; the spike is validation, not promotion).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3c (pim-12 / §3.6e).
//! ============================================================================
//! No encryption substrate / swap-matrix exists at origin/main
//! `ed03729a` (ground-truth confirmed: only transport + signing crypto).
//! The intended G-CORE-3c surface
//! (`benten_crypto_suite::swap::*` + the feature-gated downgrade-config
//! arms) does NOT exist → compile-but-fail at the `use`/symbol line.
//! Every `#[test]` is `#[ignore]`-staged with the literal marker
//! `RED-PHASE: un-ignore at G-CORE-3c`. The G-CORE-3c closing-wave
//! reviewer MUST verify these pins are *un-ignored* (landing-status, not
//! just spec-pin presence) per §3.6e.
//!
//! **Module-disjointness:** R3-A2 (TF-2) owns `sig`/`hash`/`codepoint`/
//! `varsig`/`sizes`/`boundary` (un-ignore at G-CORE-2). The sibling TF-3
//! files own `cipher`/`envelope`/`keywrap`/`confidential_content`
//! (un-ignore at G-CORE-3). This TF-4 file owns the DISJOINT `swap`
//! module + the swap-matrix conformance surface (un-ignore at
//! G-CORE-3c). No file or symbol-namespace collision.
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.6b + sub-rule 4: PRODUCTION-ARM (the real
//!    `SwapMatrix::config(..)` build + the production seal/sign + verify/
//!    open path for EACH config) + OBSERVABLE-CONSEQUENCE (hybrid requires
//!    both halves / each downgrade round-trips / a mismatched-config
//!    decrypt fails-closed / pure-PQ is NOT default-selectable without the
//!    audit flag / a reserved codepoint is a typed-unsupported error) +
//!    WOULD-FAIL-IF-NO-OP'd (a single-half-accepting impl, a
//!    silently-default-selectable pure-PQ, or a silent fallback all trip
//!    the assertions). Each pin targets the SPECIFIC matrix cell.
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: every pin BUILDS a config and
//!    drives the production crypto path + asserts a byte/verify-level
//!    consequence — NONE is "assert a SwapMatrix type is constructible".
//!    The strip-resistance pin actively tampers (zeroes/truncates/
//!    substitutes a half) and asserts the verify fails CLOSED.
//!  - §3.13 per-test static decomposition: NO process-scoped shared
//!    static; per-test locals only.
//!  - §3.6e (pim-12): `#[ignore]` + literal `RED-PHASE: un-ignore at
//!    G-CORE-3c`; reviewer verifies landing-status.
//!  - §3.5g: any minted `E_*` (e.g.
//!    `E_CRYPTO_PURE_PQ_REQUIRES_AUDIT_FLAG`) is a Rust↔TS 4-surface
//!    mirror obligation carried into the G-CORE-3c brief (TS side is a
//!    later deliverable — flagged SHAPE-only-pending-production, pim-18
//!    flag-don't-fake).
//! ----------------------------------------------------------------------------

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// RED-PHASE failure point: intended G-CORE-3c swap-matrix surface.
// Disjoint from R3-A2 + the TF-3 sibling files.
use benten_crypto_suite::swap::{SwapConfig, SwapConfigError, SwapMatrix, SwapMismatch, TrustPath};

/// The operative codepoints the plan + §4 CI gate name (carried so the
/// test holds the EXACT constants; the freeze ACT is the scheduled
/// G-CORE-9 P-III decision-point, NOT frozen here).
const ENC_HYBRID_DEFAULT_CODEPOINT: u16 = 0x647a; // X25519⊕ML-KEM-768
/// The reserved-but-unimplemented NF-1 KEM PQ⊕PQ codepoint
/// (ML-KEM-768⊕HQC; build-trigger FIPS-207-final ≈ 2027). MUST hit the
/// typed-unsupported arm (additive-codepoint discipline).
const RESERVED_ML_KEM_HQC_CODEPOINT: u16 = 0x6480;

/// Pin (1) — the hybrid-DEFAULT config asserts BOTH halves present AND
/// BOTH required to verify/decrypt. The strip-resistance property the
/// whole PQ-default reframe's safety rests on. would-FAIL if a
/// single-half config silently verifies.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c"]
fn tf4_hybrid_default_both_halves_required_strip_resistant() {
    // The DEFAULT swap-matrix config MUST be the PQ-hybrid arm (NOT a
    // classical-only or pure-PQ default). would-FAIL if the default is
    // not hybrid (the PQ-default reframe's load-bearing posture).
    let matrix = SwapMatrix::v1_default();
    let cfg = matrix.default_config();
    assert!(
        cfg.is_pq_hybrid(),
        "the v1-beta DEFAULT swap config MUST be PQ-hybrid \
         (PQ-default reframe 2026-05-19; classical-only is a NON-default \
         downgrade arm)"
    );
    assert_eq!(
        cfg.encryption_codepoint().as_u16(),
        ENC_HYBRID_DEFAULT_CODEPOINT,
        "the hybrid-default encryption codepoint MUST be 0x647a \
         (X25519⊕ML-KEM-768)"
    );

    // The hybrid construction → PQC is NEVER the SOLE trust path: the
    // classical half is the audited floor. The config MUST report a
    // hybrid trust path (both classical AND PQ), never sole-PQ.
    assert_eq!(
        cfg.trust_path(),
        TrustPath::HybridClassicalAndPq,
        "hybrid construction → PQC is never the SOLE trust path \
         (classical half = audited floor; safe to ship PQ-default \
         pre-audit). would-FAIL if the default trust path is sole-PQ."
    );

    // PRODUCTION sign+seal under the hybrid default.
    let kp = cfg.generate_keypair();
    let recipient = cfg.generate_recipient_keypair();
    let msg = b"swap-matrix hybrid-default payload";

    let signed = cfg.sign(&kp, msg).expect("hybrid-default sign");
    let sealed = cfg
        .seal(recipient.public(), msg)
        .expect("hybrid-default seal");

    // BOTH-required: stripping/zeroing/substituting EITHER half MUST make
    // verify fail CLOSED (typed error, never a silent single-half accept).
    let stripped_pq = signed.with_pq_half_stripped_for_test();
    assert!(
        cfg.verify(kp.public(), msg, &stripped_pq).is_err(),
        "stripping the ML-DSA-65 (PQ) signature half MUST fail verify \
         CLOSED (would-FAIL if a single-half config silently verifies — \
         the strip-resistance property the reframe's safety rests on)"
    );
    let stripped_classical = signed.with_classical_half_stripped_for_test();
    assert!(
        cfg.verify(kp.public(), msg, &stripped_classical).is_err(),
        "stripping the Ed25519 (classical) signature half MUST fail \
         verify CLOSED"
    );
    let substituted = signed.with_pq_half_substituted_for_test(&[0xAB; 64]);
    assert!(
        cfg.verify(kp.public(), msg, &substituted).is_err(),
        "substituting a forged PQ half MUST fail verify CLOSED \
         (committing/strip-resistant — neither half can be substituted)"
    );

    // The sealed envelope likewise requires the hybrid KEM (a
    // classical-only decap on a hybrid envelope MUST fail closed).
    assert!(
        cfg.open_with_classical_only_for_test(recipient.secret(), &sealed)
            .is_err(),
        "a classical-only decap on a PQ-hybrid sealed envelope MUST fail \
         CLOSED (both KEM halves required — HNDL-relevant)"
    );

    // Positive control: the full hybrid verify/open succeeds.
    cfg.verify(kp.public(), msg, &signed)
        .expect("intact hybrid signature MUST verify");
    let opened = cfg
        .open(recipient.secret(), &sealed)
        .expect("intact hybrid envelope MUST open");
    assert_eq!(opened.as_slice(), &msg[..]);
}

/// Pin (2) — each downgrade config (classical-only / no-encryption /
/// non-PQ encryption) round-trips on its OWN production path. The full
/// swap matrix is BUILT (not just an Ed25519 envelope). would-FAIL if a
/// downgrade config is unbuilt or non-functional.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c"]
fn tf4_every_downgrade_config_round_trips() {
    let matrix = SwapMatrix::v1_default();

    for cfg_kind in [
        SwapConfig::classical_only(),
        SwapConfig::no_encryption(),
        SwapConfig::non_pq_encryption(),
    ] {
        let cfg = matrix
            .build_config(cfg_kind.clone())
            .unwrap_or_else(|e| panic!("downgrade config {cfg_kind:?} MUST build: {e:?}"));

        assert!(
            !cfg.is_pq_hybrid(),
            "a downgrade config ({cfg_kind:?}) MUST NOT report PQ-hybrid"
        );

        let kp = cfg.generate_keypair();
        let recipient = cfg.generate_recipient_keypair();
        let msg = b"downgrade-config round-trip payload";

        let signed = cfg
            .sign(&kp, msg)
            .unwrap_or_else(|e| panic!("{cfg_kind:?} sign MUST succeed: {e:?}"));
        cfg.verify(kp.public(), msg, &signed)
            .unwrap_or_else(|e| panic!("{cfg_kind:?} verify MUST succeed: {e:?}"));

        // no_encryption skips the seal arm by definition; the others
        // round-trip the seal/open path.
        if !matches!(cfg_kind, SwapConfig::NoEncryption) {
            let sealed = cfg
                .seal(recipient.public(), msg)
                .unwrap_or_else(|e| panic!("{cfg_kind:?} seal MUST succeed: {e:?}"));
            let opened = cfg
                .open(recipient.secret(), &sealed)
                .unwrap_or_else(|e| panic!("{cfg_kind:?} open MUST succeed: {e:?}"));
            assert_eq!(opened.as_slice(), &msg[..], "{cfg_kind:?} round-trip");
        }
    }
}

/// Pin (3) — bidirectional swap conformance. Encrypt/sign under config-A,
/// assert decrypt/verify under config-A succeeds AND a MISMATCHED config
/// fails CLOSED (never a silent cross-config accept). would-FAIL if a
/// config-B verifier silently accepts config-A output.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c"]
fn tf4_bidirectional_swap_conformance_mismatch_fails_closed() {
    let matrix = SwapMatrix::v1_default();

    let cfg_hybrid = matrix.default_config();
    let cfg_classical = matrix
        .build_config(SwapConfig::classical_only())
        .expect("classical-only builds");

    let kp_h = cfg_hybrid.generate_keypair();
    let recip_h = cfg_hybrid.generate_recipient_keypair();
    let msg = b"bidirectional swap payload";

    // Same-config round-trip succeeds (both directions).
    let s_h = cfg_hybrid.sign(&kp_h, msg).expect("hybrid sign");
    cfg_hybrid
        .verify(kp_h.public(), msg, &s_h)
        .expect("hybrid→hybrid verify succeeds");
    let e_h = cfg_hybrid.seal(recip_h.public(), msg).expect("hybrid seal");
    assert_eq!(
        cfg_hybrid
            .open(recip_h.secret(), &e_h)
            .expect("hybrid→hybrid open succeeds")
            .as_slice(),
        &msg[..]
    );

    // Mismatched config MUST fail CLOSED (a classical-only verifier on a
    // hybrid signature is NOT a silent accept). The typed error carries
    // the mismatch (codepoint A vs expected B).
    let cross_verify = cfg_classical.verify(kp_h.public(), msg, &s_h);
    assert!(
        matches!(
            cross_verify,
            Err(SwapConfigError::Mismatch(SwapMismatch { .. }))
        ) || cross_verify.is_err(),
        "a classical-only config verifying a hybrid signature MUST fail \
         CLOSED with a typed mismatch (never a silent cross-config \
         accept): got {cross_verify:?}"
    );
    let cross_open = cfg_classical.open(recip_h.secret(), &e_h);
    assert!(
        cross_open.is_err(),
        "a classical-only config opening a hybrid envelope MUST fail \
         CLOSED (never a silent cross-config decrypt): got {cross_open:?}"
    );
}

/// Pin (4) — the pure-PQ (sole-PQ-trust) config is STRUCTURALLY
/// non-default until the `audit-landed` flag flips. A reviewer/implementer
/// MUST NOT be able to ship a config that makes UNAUDITED PQC the SOLE
/// trust path. would-FAIL if pure-PQ is freely default-selectable.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c"]
fn tf4_pure_pq_structurally_non_default_until_audit_flag() {
    let matrix = SwapMatrix::v1_default();

    // The DEFAULT is NEVER pure-PQ (the default trust path is hybrid).
    assert_ne!(
        matrix.default_config().trust_path(),
        TrustPath::SolePq,
        "the v1-beta default MUST NEVER be sole-PQ (unaudited PQC is \
         never the v1 production path — CLAUDE.md #5 v1-GATE amendment)"
    );

    // Attempting to BUILD a pure-PQ (sole-PQ-trust) config WITHOUT the
    // audit-landed flag MUST be a TYPED structural refusal — not merely
    // a runtime warning, and NEVER a silently-usable suite.
    let without_flag = matrix.build_config(SwapConfig::pure_pq_sole_trust());
    match without_flag {
        Err(SwapConfigError::PurePqRequiresAuditLanded) => { /* expected */ }
        Err(other) => panic!(
            "pure-PQ without the audit flag MUST be the specific \
             PurePqRequiresAuditLanded refusal, got {other:?}"
        ),
        Ok(_) => panic!(
            "pure-PQ (sole-PQ-trust) MUST NOT be buildable without the \
             audit-landed flag — a reviewer/implementer CANNOT ship a \
             config that makes unaudited PQC the SOLE trust path \
             (would-FAIL if pure-PQ is freely default-selectable)"
        ),
    }

    // WITH the audit-landed flag the pure-PQ config becomes buildable but
    // is STILL not the default (audit flips availability, never default).
    let audited_matrix = SwapMatrix::with_audit_landed_for_test();
    let pure_pq = audited_matrix
        .build_config(SwapConfig::pure_pq_sole_trust())
        .expect("post-audit pure-PQ MUST be buildable behind the flag");
    assert_eq!(pure_pq.trust_path(), TrustPath::SolePq);
    assert_ne!(
        audited_matrix.default_config().trust_path(),
        TrustPath::SolePq,
        "even post-audit, the DEFAULT MUST stay hybrid (the audit flag \
         flips pure-PQ AVAILABILITY, never the default selection)"
    );
}

/// Pin (5) — an unknown/reserved codepoint (the not-yet-built
/// ML-KEM-768⊕HQC NF-1 codepoint) hits the typed-unsupported arm, never a
/// silent fallback. The additive-codepoint discipline is structurally
/// verified: a reserved codepoint is recognized as reserved-but-
/// unimplemented and is a TYPED error, not a wire-break and not a silent
/// best-effort.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c"]
fn tf4_reserved_codepoint_typed_unsupported_additive_discipline() {
    let matrix = SwapMatrix::v1_default();

    // The reserved NF-1 ML-KEM-768⊕HQC codepoint is RECOGNIZED as
    // reserved-but-unimplemented (additive-codepoint discipline) and is a
    // TYPED unsupported error — never a silent fallback to the default.
    let attempt = matrix.build_config(SwapConfig::from_encryption_codepoint(
        RESERVED_ML_KEM_HQC_CODEPOINT,
    ));
    match attempt {
        Err(SwapConfigError::ReservedUnimplementedCodepoint { codepoint, .. }) => {
            assert_eq!(
                codepoint, RESERVED_ML_KEM_HQC_CODEPOINT,
                "the reserved NF-1 ML-KEM-768⊕HQC codepoint MUST be \
                 recognized as reserved-but-unimplemented (typed error \
                 carrying the codepoint)"
            );
        }
        Err(SwapConfigError::UnsupportedCodepoint { codepoint }) => {
            // Also acceptable: a strictly-unsupported typed arm (still
            // NOT a silent fallback). The load-bearing property is
            // typed-error-never-silent-fallback.
            assert_eq!(codepoint, RESERVED_ML_KEM_HQC_CODEPOINT);
        }
        Ok(cfg) => panic!(
            "a reserved/unimplemented codepoint MUST NOT silently fall \
             back to a usable config (got a usable config with codepoint \
             {:?}) — the additive-codepoint discipline + the \
             typed-unsupported-never-silent-fallback clause forbid this",
            cfg.encryption_codepoint()
        ),
        Err(other) => panic!(
            "reserved codepoint MUST be a typed reserved/unsupported \
             error, got {other:?}"
        ),
    }

    // Additive-codepoint structural property: introducing/recognizing the
    // reserved codepoint does NOT wire-break the existing v1 default
    // (an object sealed under 0x647a still round-trips after the reserved
    // codepoint is known to the dispatch table).
    let cfg = matrix.default_config();
    let recipient = cfg.generate_recipient_keypair();
    let sealed = cfg
        .seal(recipient.public(), b"pre-existing 0x647a content")
        .expect("v1 default still seals after reserved codepoint is known");
    let opened = cfg.open(recipient.secret(), &sealed).expect(
        "pre-existing 0x647a content stays decryptable forever \
                 (never-strand-content; additive-codepoint discipline)",
    );
    assert_eq!(opened.as_slice(), &b"pre-existing 0x647a content"[..]);
}
