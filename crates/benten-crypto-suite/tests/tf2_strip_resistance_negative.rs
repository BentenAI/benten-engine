//! TF-2 pins (G-CORE-3a F-1 / F-2 / F-3 / F-4 — the *negative*
//! strip-resistance + typed-reject side; complements
//! `tf2_hybrid_both_must_verify_strip_resistant.rs` which covers the
//! sig-side strip-resistance + the production happy path).
//!
//! ADDL R3-W1 (TDD RED-phase) test-writer — Phase-4-Meta-Core G-CORE-3a.
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3a row F-1 (typed
//!     `SignatureStripResistanceViolated`/`HybridHalfMissing` named arm —
//!     never silent-success; the load-bearing safety property of the
//!     entire PQ-default reframe) + F-2 (stripped-half hybrid encryption
//!     → typed error; never silent-success) + F-3 (recipient lacks one
//!     of the required hybrid keys → typed `RecipientLacksKeysForSuite`
//!     error) + F-4 (unknown codepoint at decrypt path → typed
//!     `UnsupportedCipherSuite`; NEVER silent fallback).
//!   - R2 §7 W1 row — **the named new file** `tf2_strip_resistance_negative.rs`.
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3a wave def
//!     (cipher_suite typed `UnsupportedCipherSuite`/`RecipientLacksKeysForSuite`).
//!   - CLAUDE.md baked-in #5 — typed-unsupported arm + never-silent-fallback
//!     P2P-mainstream choice (Veilid / MLS-RFC9420 / Nostr NIP-44).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56`:
//!   - The SIGNATURE strip-resistance NAMED-arm pin (F-1) uses the
//!     real-shipped `benten_crypto_suite::sig::SignatureSuite` / `VerifyError`
//!     surface — that file is LIVE since G-CORE-2-FP-1 + un-ignored at
//!     HEAD `c9c11c56`. F-1 here is the NAMED-typed-arm-presence
//!     negative-control (catches any future regression that collapses
//!     `HybridHalfMissing`/`StripResistanceViolated` to a generic `Err`).
//!     This pin can pass at HEAD — it's PASSIVE regression-guard for a
//!     pattern already-shipped.
//!   - The ENCRYPTION strip-resistance + RecipientLacksKeysForSuite +
//!     unknown-codepoint-decrypt arms (F-2/F-3/F-4) reference G-CORE-3a
//!     surfaces that don't exist yet → stub-shim discipline (per the
//!     `faa5475d` precedent). G-CORE-3a R5 implementer DELETES the
//!     `g_core_3a_stub` module + INSERTS real `use` + UN-IGNORES the
//!     F-2/F-3/F-4 tests.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! Negative-control pins for the never-silent-fallback contract. The
//! contract is load-bearing because (per CLAUDE.md #5): a silent fallback
//! on any of these surfaces would silently strand peers under a different
//! codepoint OR silently drop one of the hybrid halves OR silently
//! collapse "no key material" into success — each is a downgrade-attack
//! vector. The pins assert the typed arms are reachable + named.
//!
//! # §3.13 per-test-static decomposition
//!
//! No shared static; each test instantiates its own suite + keypair.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// F-1 uses the real-shipped sig surface; F-2/F-3/F-4 use stubs.
use benten_crypto_suite::cipher_suite::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;
use benten_crypto_suite::sig::{HybridSignature, SignatureSuite, VerifyError};

// G-CORE-3a R5: stub module DELETED; real production surface wired.
use benten_crypto_suite::aead::{AeadEnvelope, AeadError};
#[allow(unused_imports)]
use benten_crypto_suite::cipher_suite::WrappedKey;
use benten_crypto_suite::cipher_suite::{CipherSuite, RecipientKeypair};

/// Pin (F-1) typed `SignatureStripResistanceViolated` (or
/// `HybridHalfMissing`) arm is reachable — negative-control regression
/// pin.
///
/// `tf2_hybrid_both_must_verify_strip_resistant.rs` covers the
/// production strip-resistance happy + adversarial paths (ML-DSA half
/// stripped → fail; Ed25519 half stripped → fail; cross-message splice
/// → fail). This pin **names the typed arm itself** so any future
/// regression that collapses these specific variants to a generic
/// `Err(VerifyError)` is caught.
///
/// **Note:** this pin uses the REAL sig surface (LIVE at HEAD `c9c11c56`
/// per G-CORE-2-FP-1). It is `#[ignore]`-staged per the partition's
/// uniform pim-12 discipline; R5 G-CORE-3a un-ignores it together with
/// the F-2/F-3/F-4 pins (one un-ignore wave per file).
#[test]
fn tf2_f1_signature_strip_resistance_violated_arm_is_named() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"strip-resistance arm naming pin";
    let sig = suite.sign(&kp, msg);

    let stripped = sig.without_pq_half_for_test();
    let outcome = suite.verify(kp.public(), msg, &stripped);
    let is_strip_arm = matches!(
        outcome,
        Err(VerifyError::HybridHalfMissing(_) | VerifyError::StripResistanceViolated(_))
    );
    assert!(
        is_strip_arm,
        "the strip-resistance violation MUST surface via the NAMED typed \
         arms `HybridHalfMissing` or `StripResistanceViolated` (NOT \
         collapsed to a generic Err — the named arm is what the §6 P2P- \
         interop conformance lane gate 11 asserts on)"
    );
}

/// Pin (F-2) Stripped-half hybrid encryption → typed error; NEVER
/// silent-success.
///
/// Adversary presents an X-Wing-hybrid `WrappedKey` that has been
/// truncated/zeroed on the ML-KEM-768 (PQ) half OR the X25519
/// (classical) half. The unwrap path MUST fail closed with a typed
/// error; the X-Wing combiner is committing across both halves so the
/// derived shared secret won't match if either half is missing.
/// would-FAIL if an implementer's combiner silently falls back to
/// classical-half-only derivation.
#[test]
fn tf2_f2_stripped_half_hybrid_encryption_fails_closed_never_silent_success() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let kp: RecipientKeypair = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x12u8; 32];

    let wrapped = suite
        .wrap_key_material(&kp.public(), &k_root)
        .expect("wrap MUST succeed");

    let pq_stripped = wrapped.without_pq_half_for_test();
    let pq_outcome = suite.unwrap_key_material(&kp.secret(), &pq_stripped);
    assert!(
        pq_outcome.is_err(),
        "wrapping the X-Wing-hybrid with the ML-KEM-768 half stripped MUST \
         fail closed (NEVER a silent classical-only unwrap)"
    );

    let cl_stripped = wrapped.without_classical_half_for_test();
    let cl_outcome = suite.unwrap_key_material(&kp.secret(), &cl_stripped);
    assert!(
        cl_outcome.is_err(),
        "wrapping the X-Wing-hybrid with the X25519 half stripped MUST \
         fail closed (NEVER a silent PQ-only unwrap)"
    );
}

/// Pin (F-3) Recipient lacks one of the required hybrid keys → typed
/// `RecipientLacksKeysForSuite` error.
///
/// The hybrid suite requires BOTH an X25519 secret AND an ML-KEM-768
/// secret on the recipient side to unwrap. If the recipient presents
/// only one half (e.g. legacy classical-only recipient receiving a
/// hybrid-wrapped ciphertext), the suite MUST surface
/// `RecipientLacksKeysForSuite` — NEVER silent-Ok, NEVER silent-fallback.
/// Load-bearing per plan §3 G-CORE-3a "typed `UnsupportedCipherSuite`/
/// `RecipientLacksKeysForSuite` reject" line.
#[test]
fn tf2_f3_recipient_lacks_required_hybrid_key_typed_error() {
    let hybrid = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let hybrid_kp: RecipientKeypair = CipherSuite::generate_recipient_keypair_for_test(&hybrid);
    let k_root = [0x34u8; 32];
    let wrapped = hybrid
        .wrap_key_material(&hybrid_kp.public(), &k_root)
        .expect("wrap MUST succeed");

    let degenerate = RecipientKeypair::with_only_classical_half_for_test(&hybrid_kp);
    let outcome = hybrid.unwrap_key_material(&degenerate.secret(), &wrapped);
    assert!(
        matches!(outcome, Err(AeadError::RecipientLacksKeysForSuite)),
        "a recipient missing one of the required hybrid-suite key halves \
         MUST surface the NAMED `RecipientLacksKeysForSuite` typed error \
         (NEVER silent-fallback to classical-only unwrap; NEVER a generic \
         Err that the caller can't disambiguate); got {outcome:?}"
    );
}

/// Pin (F-4) Unknown codepoint at decrypt path → typed
/// `UnsupportedCipherSuite`; NEVER silent fallback to a default.
///
/// The never-silent-fallback contract clause from CLAUDE.md baked-in #5
/// is symmetric across encrypt and decrypt. An attacker (or a
/// future-version peer) presents a `WrappedKey` carrying an unknown
/// codepoint — the decrypt-path resolver MUST surface
/// `UnsupportedAlgorithm::CipherSuite` rather than silently picking a
/// default and (mis)decrypting.
#[test]
fn tf2_f4_unknown_codepoint_at_decrypt_typed_unsupported_never_silent_fallback() {
    let hybrid = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&hybrid);
    let k_root = [0x56u8; 32];
    let wrapped = hybrid
        .wrap_key_material(&kp.public(), &k_root)
        .expect("wrap MUST succeed");

    let unknown_codepoint = 0xCAFE_u16;
    let bogus = wrapped.with_codepoint_for_test(unknown_codepoint);
    let outcome = CipherSuite::resolve_for_wrapped_key(&bogus);
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::CipherSuite { codepoint }) if codepoint == unknown_codepoint),
        "unknown codepoint at decrypt path MUST surface typed \
         `UnsupportedAlgorithm::CipherSuite` (NEVER silent-fallback to the \
         hybrid default; that would be a downgrade vector + would silently \
         strand legitimate peers under reserved codepoints)"
    );
}

// =====================================================================
// R4-FP-1 extension (m-7 closure): Gate 6 PQ-hybrid envelope format-
// version discriminator.
// =====================================================================
//
// Per R4.1 triage m-7 (L4 wire-format-conformance, FIX-NOW): R2 §6
// Gate 6 (format-version discriminator in serialized envelope) is
// covered for DropBundle (W3) + SnapshotBlob (W5) but NOT for the
// PQ-hybrid envelope itself. This pin closes the gap.
//
// The Gate 6 contract: every serialized envelope MUST carry an
// explicit format-version byte (or codepoint) discriminator that
// future versions can dispatch on. Without it, a v2 envelope would
// be indistinguishable from a v1 envelope and a peer could silently
// misparse — the exact wire-break-via-version-drift failure class
// CLAUDE.md baked-in #5 (multiformats permanent commitment) defends
// against.
//
// The SHIPPED Varsig v1 header surface (`UcanVarsigV1Header` at
// `crates/benten-crypto-suite/src/varsig.rs`) is the load-bearing
// example: bytes [magic 0xb5 | version 0x01 | codepoint LE16 | ...].
// The G-CORE-3a `AeadEnvelope` (the PQ-hybrid envelope itself) must
// follow the SAME framing discipline — a magic byte, an explicit
// version byte, and a codepoint dispatch — so a future version-2
// envelope is distinguishable from a version-1 envelope without
// pre-coordination.

/// Gate 6 (R4-FP-1 m-7 closure) — the G-CORE-3a `AeadEnvelope`
/// serialization carries an explicit format-version discriminator
/// byte that a future-version-aware peer can dispatch on (NOT a
/// raw-concatenated payload that would be ambiguous against v2+).
#[test]
fn tf2_gate6_pq_hybrid_envelope_carries_explicit_format_version_discriminator() {
    // At G-CORE-3a un-ignore: the body wires against the real
    // `AeadEnvelope::to_wire_bytes()` (or equivalent serialization
    // surface name; final decision = G-CORE-3a implementer) and
    // asserts:
    //   (1) byte 0 is a stable magic byte (or a multiformats prefix
    //       byte equivalent to Varsig's 0xb5);
    //   (2) byte 1 (or byte 0 for a multicodec-style varint) is a
    //       FORMAT-VERSION DISCRIMINATOR that equals the v1-beta
    //       default version byte;
    //   (3) the wire MUST encode the cipher-suite codepoint AFTER the
    //       version byte — so a future v2 envelope (e.g. one carrying
    //       a sealed-AEAD-with-nonce-prefix-shape change) is
    //       distinguishable from v1 by examining byte 1 alone.
    //
    // The WOULD-FAIL arm: if the implementer ships `AeadEnvelope` as a
    // raw-concatenated `[wrapped_key | nonce | ciphertext]` (skipping
    // the version byte), a future v2 (e.g. one with an extra header
    // field or different nonce-derivation) is INDISTINGUISHABLE on the
    // wire from v1 — the exact silent-misparse-via-drift class Gate 6
    // defends against. The pin enforces a 1-byte version discriminator
    // up front.
    //
    // At R4-FP-1 author-time the AeadEnvelope serialization surface
    // does not yet exist; this pin stays LANDED at G-CORE-3a (pim-12 / §3.6e closure) flips
    // 0x647a to live AND ships AeadEnvelope::to_wire_bytes with a
    // format-version byte.
    //
    // Body sketch (un-ignore wires against the live surface):
    //
    //     let suite = CipherSuite::at_codepoint_for_test(
    //         CipherSuiteCodepoint::HYBRID_X25519_MLKEM768);
    //     let (recipient_pub, _) = generate_recipient_keypair_for_test();
    //     let k = b"32-bytes-of-uniformly-random-key";
    //     let wrapped = suite.wrap_key_material(&recipient_pub, k).unwrap();
    //     let env = suite.seal_aead(&wrapped, b"aad", b"plaintext").unwrap();
    //     let bytes = env.to_wire_bytes();
    //     // Assert byte 1 carries the format-version discriminator
    //     // for the v1-beta default (the exact byte value is a
    //     // G-CORE-3a design decision; the existence of the
    //     // discriminator at a stable position is the load-bearing
    //     // contract).
    //     assert!(bytes.len() >= 4,
    //         "AeadEnvelope wire-format MUST begin with magic + version + codepoint prefix");
    //     // Concretely (mirroring Varsig v1 shape):
    //     //   bytes[0] = 0xae (envelope magic) — sample; final = G-CORE-3a
    //     //   bytes[1] = 0x01 (v1-beta default format version)
    //     //   bytes[2..4] = codepoint LE16
    //     assert_eq!(bytes[1], 0x01,
    //         "Gate 6: AeadEnvelope MUST carry an explicit format-version \
    //          byte at byte-1 so a future v2 is distinguishable from v1 \
    //          without pre-coordination (multiformats-permanent-commitment \
    //          property per CLAUDE.md baked-in #5)");
    //
    // The stub body just `unimplemented!()` so an accidentally-un-ignored
    // test loud-fails (NOT silent-green) until the implementer wires it.
    let hybrid = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    // G-CORE-3a R5 wire: assert against the LIVE AeadEnvelope::to_wire_bytes
    // surface that the PQ-hybrid envelope carries an explicit
    // format-version discriminator byte.
    let k_root = [0x42u8; 32];
    let plaintext = b"Gate-6 PQ-hybrid envelope shape pin";
    let plaintext_cid = b"gate-6-plaintext-cid";
    let env = hybrid
        .seal_aead(&k_root, plaintext, plaintext_cid)
        .expect("hybrid seal_aead MUST succeed");
    let bytes = env.to_wire_bytes();
    assert!(
        bytes.len() >= 5,
        "Gate 6: AeadEnvelope wire-format MUST begin with magic + version + codepoint + nonce-len prefix (>= 5 bytes); got {}",
        bytes.len()
    );
    assert_eq!(
        bytes[0],
        benten_crypto_suite::aead::ENVELOPE_MAGIC,
        "Gate 6: byte 0 MUST be the envelope magic (the multiformats identifier)"
    );
    assert_eq!(
        bytes[1],
        benten_crypto_suite::aead::ENVELOPE_FORMAT_VERSION_V1,
        "Gate 6: byte 1 MUST carry the format-version discriminator (v1-beta = 0x01); a future v2 envelope is then distinguishable from v1 by inspecting this byte alone (multiformats-permanent-commitment property per CLAUDE.md baked-in #5)"
    );
    // M-19: codepoint serialized BIG-ENDIAN (migrated from LE at F-full Wave-0).
    let codepoint = u16::from_be_bytes([bytes[2], bytes[3]]);
    assert_eq!(
        codepoint,
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        "Gate 6: bytes 2-3 MUST carry the cipher-suite codepoint BE u16 (0x647a for the hybrid default; M-19)"
    );
    // Negative-control: stripping the magic byte MUST fail to parse as
    // a Benten envelope (distinguishability property).
    let raw_payload = &bytes[1..];
    let outcome = AeadEnvelope::from_wire_bytes(raw_payload);
    assert!(
        outcome.is_err(),
        "Gate 6 negative: bytes lacking the envelope magic MUST FAIL parse (silent acceptance makes the multiformats permanent-commitment vacuous; got {outcome:?}). \
         The DropBundle (W3) + SnapshotBlob \
         (W5) parallel pins are the named-companions; this closes the \
         PQ-hybrid-envelope itself case."
    );
}
