//! TF-3 pins (1) #1301 envelope round-trip native + wasm32 + (2)
//! codepoint-dispatch fires + typed-unsupported arm (never silent
//! fallback) + (3) format-version discriminator forward-compat.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core, Wave R3-B,
//! agent R3-B1, family TF-3 (#1301 encryption-as-confidentiality
//! substrate; consumes BOTH canaries #989 + #1300). Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §TF-3 RED-phase
//!     production-arm shapes (1) envelope round-trip native + wasm32
//!     through the PRODUCTION seal/unseal path + PQ-hybrid combiner KAT;
//!     (2) codepoint-dispatch fires + typed-unsupported arm (never silent
//!     fallback); (3) format-version discriminator forward-compat.
//!   - §TF-3 Pins: G-CORE-3 · C2 · §1.A.FROZEN item 6 · D-4M-13 / §8-CC.
//!   - `00-implementation-plan.md` G-CORE-3 def: "#1301 v1
//!     format-version-1 IS the PQ-hybrid envelope — the X25519⊕ML-KEM-768
//!     hybrid KEM at frozen codepoint `0x647a` (combiner vendored ~30-LOC;
//!     X-Wing-style construction) … Format-version discriminator present
//!     from v1 … #1301 calls the integration crate's typed cipher-suite
//!     API, never instantiates a primitive."
//!   - §4 CI "Encryption-format / PQ-hybrid conformance gate" — round-trips
//!     the #1301 envelope across native + wasm32, asserts codepoint-dispatch
//!     fires, asserts the typed unsupported-algorithm arm (never silent
//!     fallback), asserts format-version discriminator present.
//!   - CLAUDE.md #5 crypto-agility refinement (codepoint-dispatch; typed
//!     unsupported-never-silent-fallback; the integration crate is the
//!     ONLY crypto-primitive call site — #1301 calls its typed API).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3 (pim-12 / §3.6e).
//! ============================================================================
//! `benten-crypto-suite` is a STUB at R3-A/R3-B; the intended G-CORE-3
//! cipher-suite/envelope surface (`benten_crypto_suite::cipher::*`,
//! `benten_crypto_suite::envelope::*`) does NOT exist at origin/main
//! `ed03729a` (ground-truth: no encryption substrate exists anywhere in
//! the synced tree — only transport + signing crypto). So every test here
//! **compiles-but-fails at the `use`/symbol-resolution line** until
//! G-CORE-3 lands #1301. Each `#[test]` is `#[ignore]`-staged with the
//! literal marker `RED-PHASE: un-ignore at G-CORE-3`. The G-CORE-3
//! closing-wave reviewer MUST verify these pins are *un-ignored*
//! (landing-status, not just spec-pin presence) per §3.6e.
//!
//! **Module-disjointness from R3-A2 (TF-2):** R3-A2 owns
//! `benten_crypto_suite::{sig, hash, codepoint, varsig, sizes, boundary}`
//! (un-ignore at G-CORE-2). This TF-3/TF-4 lane owns the DISJOINT
//! cipher-suite/envelope/key-wrap/swap-matrix surface
//! (`benten_crypto_suite::{cipher, envelope, keywrap, swap}`; un-ignore
//! at G-CORE-3 / G-CORE-3c). No file or symbol-namespace collision — this
//! is the crypto-suite's disjointness mechanism (the stub has NO
//! `[features]`; disjointness is by intended-module-namespace per the
//! R3-A2 convention, not by feature-gate).
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.5b HARDENED (pim-1): no public-shape change here (RED-phase test
//!    only); the post-shape-change adjacent-doc sweep is a G-CORE-3-wave
//!    obligation carried in the report (ENGINE-SPEC.md:312 trusted-engine
//!    qualifier + 305-318 prose — TF-3 cite/sentinel pin lives in the
//!    sibling `tf3_1301_keywrap_hndl_and_doc_coupling.rs`).
//!  - §3.6b + sub-rule 4: each pin is a PRODUCTION-ARM (the real
//!    `EncryptionEnvelope::seal`/`open` codepoint-dispatched path) +
//!    OBSERVABLE-CONSEQUENCE (cleartext recovered byte-identical / a
//!    typed error on an unknown codepoint / the version discriminator
//!    byte is present + rejects an unknown future version) +
//!    WOULD-FAIL-IF-NO-OP'd (a stub that returns plaintext, ignores the
//!    codepoint, or omits the version byte trips every assertion). Each
//!    pin targets the SPECIFIC arm (round-trip / native==wasm32 parity /
//!    KAT vector / codepoint-dispatch / typed-unsupported / format-version
//!    forward-compat), not an umbrella "encryption works".
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: every pin enumerates a real
//!    production call site (`cipher::CipherSuite::dispatch` /
//!    `envelope::EncryptionEnvelope::{seal,open}`) and asserts an
//!    observable byte-level consequence — NONE is "assert an Envelope
//!    type is constructible". The KAT pin drives a fixed test vector
//!    through the production combiner and asserts the EXACT expected
//!    ciphertext/shared-secret bytes (a constructible-type sentinel would
//!    pass without doing real crypto and is explicitly rejected here).
//!  - §3.13 per-test static decomposition: this file introduces NO
//!    process-scoped shared static. Every test owns fresh locals; the
//!    native-vs-wasm32 parity pin uses a per-test `let` vector (semantic
//!    local name `parity_payload`), never a single shared static under
//!    the parallel runner (TF-3 envelope round-trip is on the §4-D
//!    §3.13-on-surface list — explicitly honored).
//!  - §3.6e (pim-12): `#[ignore]` + literal `RED-PHASE: un-ignore at
//!    G-CORE-3` on every test; reviewer verifies landing-status.
//!  - §3.5g cross-language rule-mirror: any `E_*` ErrorCode minted for the
//!    encryption path (e.g. an `E_CRYPTO_UNSUPPORTED_CIPHER_CODEPOINT` /
//!    `E_CRYPTO_ENVELOPE_VERSION_UNKNOWN`) is a Rust↔TS 4-surface mirror
//!    obligation carried into the G-CORE-3 brief (the TS side is a
//!    G-CORE-3 deliverable, flagged SHAPE-only-pending-production in the
//!    report — pim-18 flag-don't-fake).
//!  - §3.5h / §3.5i / §3.5j / §3.5n / §3.6g-j / §3.11 / §3.5l / §3.5m:
//!    R3 produces test files + a well-formed JSON-free markdown report
//!    (no JSON artifact here); the mini-reviewer tree-state-freshness +
//!    workspace-pre-merge + ground-truth-verify obligations are carried
//!    into the G-CORE-3 closing-wave brief, not actionable in this
//!    RED-phase test file.
//! ----------------------------------------------------------------------------

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// RED-PHASE failure point: intended G-CORE-3 (#1301) cipher-suite +
// envelope surface. Does NOT exist at ed03729a → compile-but-fail here.
// Disjoint from R3-A2's `sig`/`hash`/`codepoint` modules.
use benten_crypto_suite::cipher::{CipherCodepoint, CipherSuite, UnsupportedCipher};
use benten_crypto_suite::envelope::{
    EncryptionEnvelope, EnvelopeError, EnvelopeFormatVersion, SealedEnvelope,
};

/// The operative encryption-hybrid default codepoint the plan + §4 CI gate
/// pin (X25519⊕ML-KEM-768 hybrid KEM; vendored ~30-LOC X-Wing-style
/// combiner). Named so the test carries the EXACT constant — the freeze
/// ACT stays the scheduled G-CORE-9 P-III decision-point, NOT frozen here.
const ENC_HYBRID_DEFAULT_CODEPOINT: u16 = 0x647a;

/// #1301 v1 format-version-1. The discriminator MUST be present from v1
/// (forward-compat: an unknown future version is a typed reject, never a
/// silent best-effort parse).
const ENVELOPE_FORMAT_VERSION_1: u8 = 1;

/// Pin (1) — the PRODUCTION seal→open round-trip on the #1301 envelope.
/// Seal a per-DID partition payload, open it, assert the cleartext is
/// recovered **byte-identical**. would-FAIL if the seal path is a no-op
/// passthrough (returns plaintext) or the open path cannot recover bytes.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_envelope_seal_open_round_trips_byte_identical() {
    // The v1 default cipher suite MUST be the PQ-hybrid X25519⊕ML-KEM-768
    // at codepoint 0x647a (branch-(i): hybrid-KEM-wrapped from the first
    // format version — NOT classical-then-retrofit).
    let suite = CipherSuite::v1_default();
    assert_eq!(
        suite.codepoint(),
        CipherCodepoint::from_u16(ENC_HYBRID_DEFAULT_CODEPOINT),
        "v1 #1301 default cipher suite MUST be the PQ-hybrid \
         X25519+ML-KEM-768 at codepoint 0x647a (branch-(i): \
         hybrid-from-first-format-version; HNDL is un-retrofittable)"
    );
    assert!(
        suite.is_pq_hybrid(),
        "the v1 #1301 default MUST be PQ-hybrid (would-FAIL if a \
         classical-then-retrofit default ships — the un-retrofittable \
         HNDL failure the crypto contract forbids)"
    );

    let recipient = suite.generate_recipient_keypair();
    let plaintext = b"per-DID partition canonical-bytes payload \x00\xff\x7f";

    // PRODUCTION seal path through the codepoint-dispatched envelope.
    let sealed: SealedEnvelope = EncryptionEnvelope::seal(&suite, recipient.public(), plaintext)
        .expect("production #1301 seal MUST succeed for the v1 default suite");

    // Observable consequence #1: the sealed bytes are NOT the plaintext
    // (would-FAIL if seal is a no-op passthrough — pim-18 substantive).
    assert_ne!(
        sealed.ciphertext_bytes(),
        &plaintext[..],
        "sealed envelope ciphertext MUST NOT equal the plaintext \
         (would-FAIL if #1301 seal is a no-op passthrough stub)"
    );

    // PRODUCTION open path.
    let recovered = EncryptionEnvelope::open(&suite, recipient.secret(), &sealed)
        .expect("production #1301 open MUST recover the cleartext");

    // Observable consequence #2: byte-identical recovery.
    assert_eq!(
        recovered.as_slice(),
        &plaintext[..],
        "open(seal(p)) MUST recover p byte-identically"
    );
}

/// Pin (1, native==wasm32 parity) — the #1301 envelope round-trip MUST be
/// target-agnostic: a payload sealed and opened produces a byte-identical
/// result regardless of target (the §4 CI gate round-trips across native
/// **and** wasm32). This test asserts the production seal/open path is
/// deterministic+target-portable (pure-Rust, wasm32-capable per the
/// crypto-agility contract). The §4 CI lane runs the SAME assertion under
/// `--target wasm32-unknown-unknown`; this in-tree pin is the native half +
/// the determinism predicate the wasm32 half relies on. would-FAIL if the
/// envelope encoding is endian/word-size sensitive (a hardcoded-size /
/// platform-int hazard) — that is exactly the no-hardcoded-sizes class.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_envelope_round_trip_is_target_portable_deterministic() {
    let suite = CipherSuite::v1_default();
    let recipient = suite.generate_recipient_keypair();
    // Per-test local (no shared static under the parallel runner — §3.13).
    let parity_payload: Vec<u8> = (0u16..=511).map(|i| (i % 256) as u8).collect();

    // Seal twice with a fixed ephemeral-randomness injection so the
    // envelope is deterministic for the parity assertion (the production
    // API MUST expose a deterministic-seal test seam — the §4 CI
    // native-vs-wasm32 parity gate structurally requires it; would-FAIL
    // if seal cannot be made deterministic for conformance vectors).
    let fixed_eph = suite.deterministic_ephemeral_for_test(&[0x5a; 32]);
    let sealed_a = EncryptionEnvelope::seal_with_ephemeral(
        &suite,
        recipient.public(),
        &parity_payload,
        &fixed_eph,
    )
    .expect("deterministic seal A");
    let sealed_b = EncryptionEnvelope::seal_with_ephemeral(
        &suite,
        recipient.public(),
        &parity_payload,
        &fixed_eph,
    )
    .expect("deterministic seal B");

    assert_eq!(
        sealed_a.to_canonical_bytes(),
        sealed_b.to_canonical_bytes(),
        "the #1301 envelope MUST be deterministic under a fixed ephemeral \
         (the §4 CI native-vs-wasm32 parity gate requires byte-equal \
         envelopes across targets; would-FAIL on any platform-int / \
         endian / hardcoded-size hazard in the encoding)"
    );

    let recovered = EncryptionEnvelope::open(&suite, recipient.secret(), &sealed_a)
        .expect("open the deterministic envelope");
    assert_eq!(recovered.as_slice(), parity_payload.as_slice());
}

/// Pin (1, KAT) — the PQ-hybrid combiner KAT. A FIXED known-answer test
/// vector (X25519 scalar + ML-KEM-768 decap key + ciphertext) drives the
/// production X-Wing-style combiner and asserts the EXACT expected derived
/// shared-secret bytes. would-FAIL if the combiner is a stub returning
/// zeros / the wrong concatenation order / a single-KEM-only secret
/// (pim-18: a constructible-type sentinel would pass without doing real
/// crypto — this drives bytes through the real combiner).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_pq_hybrid_combiner_kat_exact_shared_secret_bytes() {
    let suite = CipherSuite::v1_default();

    // The KAT vector is a fixture the G-CORE-3 wave commits alongside the
    // combiner (the §4 CI gate "PQ-hybrid combiner KAT vectors"). At
    // RED-phase the fixture loader is the intended surface; the EXACT
    // expected bytes are the conformance anchor (NOT a placeholder — the
    // G-CORE-3 implementer fills the fixture; the test asserts the
    // production combiner reproduces it exactly).
    let kat = benten_crypto_suite::cipher::kat::x_wing_hybrid_kem_vector_1();

    let derived = suite
        .combine_hybrid_shared_secret(&kat.x25519_dh, &kat.ml_kem_shared_secret)
        .expect("production X-Wing-style combiner MUST run");

    assert_eq!(
        derived.as_bytes(),
        kat.expected_combined_secret.as_slice(),
        "the vendored ~30-LOC X-Wing-style combiner MUST reproduce the \
         committed KAT shared-secret EXACTLY (would-FAIL on a zero-stub, \
         a single-KEM-only secret, or a wrong concatenation/transcript \
         order — this is the load-bearing hybrid-KEM correctness pin)"
    );

    // Strip-resistance at the KEM layer: the combined secret MUST change
    // if EITHER input half changes (neither the classical X25519 half nor
    // the PQ ML-KEM half can be ignored by the combiner).
    let mut tampered_x = kat.x25519_dh.clone();
    tampered_x[0] ^= 0x01;
    let derived_tx = suite
        .combine_hybrid_shared_secret(&tampered_x, &kat.ml_kem_shared_secret)
        .expect("combiner runs on tampered X25519 half");
    assert_ne!(
        derived_tx.as_bytes(),
        kat.expected_combined_secret.as_slice(),
        "flipping the X25519 half MUST change the combined secret \
         (would-FAIL if the combiner ignores the classical half)"
    );

    let mut tampered_pq = kat.ml_kem_shared_secret.clone();
    tampered_pq[0] ^= 0x01;
    let derived_tpq = suite
        .combine_hybrid_shared_secret(&kat.x25519_dh, &tampered_pq)
        .expect("combiner runs on tampered ML-KEM half");
    assert_ne!(
        derived_tpq.as_bytes(),
        kat.expected_combined_secret.as_slice(),
        "flipping the ML-KEM half MUST change the combined secret \
         (would-FAIL if the combiner ignores the PQ half — this is the \
         HNDL-relevant property: a classical-only effective KEM is \
         harvest-now-decrypt-later exposed)"
    );
}

/// Pin (2) — codepoint-dispatch FIRES + the typed-unsupported arm is a
/// typed error, NEVER a silent fallback. An unknown/reserved cipher
/// codepoint hits `UnsupportedCipher`, not a default best-effort path.
/// would-FAIL if dispatch silently falls back to the v1 default (the
/// exact fail-open the crypto contract + §4 CI gate forbid).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_codepoint_dispatch_typed_unsupported_never_silent_fallback() {
    // A codepoint that is structurally NOT the v1 default and NOT any
    // built downgrade — it must be a TYPED unsupported error.
    let reserved = CipherCodepoint::from_u16(0xFFFE);

    let dispatch_result = CipherSuite::try_from_codepoint(reserved);
    match dispatch_result {
        Err(UnsupportedCipher { codepoint }) => {
            assert_eq!(
                codepoint, reserved,
                "the typed-unsupported arm MUST carry the offending \
                 codepoint (typed error, never a silent fallback)"
            );
        }
        Ok(suite) => panic!(
            "codepoint-dispatch MUST NOT silently fall back to a usable \
             suite for an unknown/reserved codepoint (got a usable suite \
             with codepoint {:?}) — this is the fail-open the crypto \
             contract + §4 CI gate explicitly forbid",
            suite.codepoint()
        ),
    }

    // And the dispatch DID fire for a known codepoint (positive control:
    // the v1 default 0x647a resolves to the PQ-hybrid suite).
    let ok =
        CipherSuite::try_from_codepoint(CipherCodepoint::from_u16(ENC_HYBRID_DEFAULT_CODEPOINT))
            .expect("the v1 default codepoint MUST dispatch to a usable suite");
    assert!(ok.is_pq_hybrid());
}

/// Pin (2, decode-side) — an envelope on the wire carrying an unknown
/// cipher codepoint is REJECTED with a typed error at `open`, never
/// silently decrypted under the default suite. would-FAIL if `open`
/// ignores the envelope's codepoint and assumes the default.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_open_rejects_unknown_codepoint_envelope_typed_not_silent() {
    let suite = CipherSuite::v1_default();
    let recipient = suite.generate_recipient_keypair();
    let sealed = EncryptionEnvelope::seal(&suite, recipient.public(), b"payload").expect("seal");

    // Forge the on-wire codepoint to an unsupported value while leaving
    // the rest of the envelope intact (a non-decrypting tamper).
    let forged = sealed.with_forced_codepoint_for_test(CipherCodepoint::from_u16(0xFFFD));

    let err = EncryptionEnvelope::open(&suite, recipient.secret(), &forged)
        .expect_err("open MUST reject an unknown-codepoint envelope");
    match err {
        EnvelopeError::UnsupportedCipher(cp) => {
            assert_eq!(cp, CipherCodepoint::from_u16(0xFFFD));
        }
        other => panic!(
            "open on an unknown-codepoint envelope MUST be the typed \
             UnsupportedCipher error, never a silent decrypt-under-default \
             (got {other:?})"
        ),
    }
}

/// Pin (3) — the format-version discriminator is present from v1 AND an
/// unknown FUTURE version is a typed reject (forward-compat: never a
/// silent best-effort parse of an unrecognized envelope). would-FAIL if
/// the version byte is absent or an unknown version is parsed leniently.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_format_version_discriminator_present_and_forward_compat() {
    let suite = CipherSuite::v1_default();
    let recipient = suite.generate_recipient_keypair();
    let sealed = EncryptionEnvelope::seal(&suite, recipient.public(), b"v1 payload").expect("seal");

    // Observable: the envelope carries the version discriminator = 1.
    assert_eq!(
        sealed.format_version(),
        EnvelopeFormatVersion::from_u8(ENVELOPE_FORMAT_VERSION_1),
        "the #1301 envelope MUST carry the format-version discriminator \
         from v1 (would-FAIL if the discriminator byte is absent)"
    );

    // Forward-compat: an envelope claiming an unknown future version is a
    // typed reject, NEVER a silent lenient parse.
    let future = sealed.with_forced_format_version_for_test(EnvelopeFormatVersion::from_u8(99));
    let err = EncryptionEnvelope::open(&suite, recipient.secret(), &future)
        .expect_err("open MUST reject an unknown future format-version");
    assert!(
        matches!(err, EnvelopeError::UnknownFormatVersion(v) if v == EnvelopeFormatVersion::from_u8(99)),
        "an unknown future envelope version MUST be a typed reject \
         (forward-compat; never a silent best-effort parse): got {err:?}"
    );
}
