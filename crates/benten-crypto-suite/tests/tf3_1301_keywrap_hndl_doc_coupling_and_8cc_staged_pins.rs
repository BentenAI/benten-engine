//! TF-3 pins (4) multi-device key-wrap PQ-hybrid arm DISTINCT from the
//! bulk-seal arm (HNDL-exposed, un-retrofittable) + (5) the
//! ENGINE-SPEC.md:312 trusted-engine-qualifier doc-coupling cite/sentinel
//! + the §8-CC-gated CID-addressing / non-decrypting-relay-verify
//! DEFERRED-WITH-NAMED-DESTINATION staged-pin arms (NOT droppable, NOT a
//! float; hard pre-G-CORE-9 deadline).
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core, Wave R3-B,
//! agent R3-B1, family TF-3 (#1301; consumes BOTH canaries). Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §TF-3 RED-phase shape
//!     (4) "the multi-device key-wrap PQ-hybrid arm distinct from the
//!     bulk-seal arm (multitenant-r1-4; HNDL-exposed — would-FAIL if the
//!     key-wrap path is not PQ-hybrid from format-version-1, since HNDL is
//!     un-retrofittable)" + (5) the trusted-engine-qualifier cite-pin
//!     (ENGINE-SPEC.md:312 surrounding 305-318 prose carries the
//!     "PQ-hybrid-sealed per-DID partition" wording post-G-CORE-3;
//!     §6.1/§6.2 companion-with-canary; would-FAIL if doc-coupling
//!     regresses).
//!   - §4-B "§8-CC still-open → TF-3's confidential-content arms are
//!     DEFERRED-WITH-NAMED-DESTINATION (not droppable, not float)":
//!     R3-B1 MUST (1) write the family's full structure including
//!     placeholder `#[ignore]`-RED-PHASE-staged-pin arms for the
//!     CID-addressing + relay-verify cases (§3.6e — un-ignored once §8-CC
//!     resolves, BEFORE G-CORE-9); (2) carry an explicit note these arms
//!     are gated on the §8-CC resolution and MUST land before the
//!     G-CORE-9 freeze (ordering joint mirrors the old NF-4 gating
//!     clause). NOT "defer the test" — a named-destination staged-pin
//!     with a hard pre-freeze deadline.
//!   - §4-D §3.6e RED-PHASE staged-pin obligations: TF-3 = "the §8-CC-gated
//!     CID-addressing/relay-verify arms — staged-pin with a pre-G-CORE-9
//!     deadline".
//!   - `00-implementation-plan.md` G-CORE-3 def (the multi-device
//!     key-wrap/recovery envelope is in-scope-and-PQ-hybrid, distinct
//!     from the bulk partition seal; its envelope shape — NOT the
//!     recovery protocol, which is G-COMP-3 / out of Core scope — is
//!     frozen at §1.A.FROZEN item 6) + the #1301 confidential-content
//!     design sub-decision (plaintext-CID vs encrypted-envelope-CID /
//!     STE vs ETS / what a non-decrypting relay can verify) + §5
//!     D-4M-13 (the surfaced §8-CC sub-fork) + the doc-coupling clause
//!     (multitenant-r1-3, §6.1/§6.2 companion-with-canary;
//!     ENGINE-SPEC.md:312 trusted-engine qualifier worded "PQ-hybrid-sealed
//!     per-DID partition").
//!   - CLAUDE.md baked-in #18 Principal-primitive confidentiality-half +
//!     #5 crypto-agility refinement (encryption PQ-hybrid from its
//!     format's FIRST version — HNDL is un-retrofittable, including the
//!     multi-device key-wrap/recovery path).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3 (pim-12 / §3.6e) for the key-wrap +
//! doc-coupling pins; the §8-CC-gated arms are STAGED-PIN with a HARD
//! pre-G-CORE-9 deadline (un-ignore when §8-CC resolves, which MUST
//! happen BEFORE G-CORE-9 freezes the #1301 envelope — §4-B).
//! ============================================================================
//! No encryption substrate exists at origin/main `ed03729a` (ground-truth
//! confirmed). The intended G-CORE-3 surface
//! (`benten_crypto_suite::{cipher, envelope, keywrap}` +
//! `benten_crypto_suite::confidential_content` for the §8-CC arms) does
//! NOT exist → compile-but-fail at the `use`/symbol line. Disjoint from
//! R3-A2's `sig`/`hash`/`codepoint` modules and from the sibling TF-3
//! round-trip file's pins (this file owns key-wrap + doc-coupling +
//! §8-CC).
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.6b + sub-rule 4: PRODUCTION-ARM (the real
//!    `KeyWrapEnvelope::wrap_for_device`/`unwrap` path, DISTINCT from
//!    `EncryptionEnvelope::seal`) + OBSERVABLE-CONSEQUENCE (the key-wrap
//!    envelope is PQ-hybrid from format-version-1 / a wrapped key is
//!    recoverable only by the target device / the doc sentinel text is
//!    present) + WOULD-FAIL-IF-NO-OP'd (a classical-only key-wrap path
//!    trips the HNDL assertion; a regressed doc trips the cite/sentinel).
//!    Each pin targets the SPECIFIC arm.
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: the key-wrap pin asserts the
//!    wrap path is a SEPARATE production surface that is PQ-hybrid (not
//!    "a KeyWrapEnvelope type exists"); the doc-coupling pin reads the
//!    actual ENGINE-SPEC.md file and asserts the SPECIFIC wording (not "a
//!    doc exists"). The §8-CC arms are explicitly flagged
//!    SHAPE-only-pending-design (the option-specific assertion CANNOT be
//!    written until §8-CC resolves — pim-18 flag-don't-fake: the test
//!    STRUCTURE lands; the option-arm body is a staged placeholder).
//!  - §3.13 per-test static decomposition: NO process-scoped shared
//!    static; per-test locals only.
//!  - §3.6e (pim-12): `#[ignore]` + literal marker on every test;
//!    reviewer verifies landing-status. The §8-CC arms carry the literal
//!    `RED-PHASE-STAGED-PIN(§8-CC): un-ignore when §8-CC resolves; MUST
//!    land before G-CORE-9` marker (the named destination = the G-CORE-3
//!    brief's explicit pre-flight design sub-decision + §8-CC + D-4M-13).
//!  - §3.5g / §3.5b: doc-coupling is itself the §6.1/§6.2
//!    companion-with-canary obligation; this file's doc-sentinel pin IS
//!    the cite-drift backstop the §3.5b adjacent-doc sweep relies on.
//! ----------------------------------------------------------------------------

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::path::PathBuf;

// RED-PHASE failure point: intended G-CORE-3 key-wrap + §8-CC surfaces.
// Disjoint from R3-A2 modules and from the sibling round-trip file.
use benten_crypto_suite::cipher::{CipherCodepoint, CipherSuite};
use benten_crypto_suite::envelope::EncryptionEnvelope;
use benten_crypto_suite::keywrap::{KeyWrapEnvelope, KeyWrapError, WrappedDeviceKey};

const ENC_HYBRID_DEFAULT_CODEPOINT: u16 = 0x647a;
const ENVELOPE_FORMAT_VERSION_1: u8 = 1;

/// Pin (4) — the multi-device key-wrap envelope is a SEPARATE production
/// surface from the bulk partition seal, AND it is PQ-hybrid from
/// format-version-1. would-FAIL if the key-wrap path is classical-only or
/// is the same code path as the bulk seal (HNDL is un-retrofittable: the
/// multi-device key-wrap/recovery path is harvest-now-decrypt-later
/// exposed even for at-rest "for-self" — it MUST be PQ-hybrid from v1).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_keywrap_is_distinct_surface_and_pq_hybrid_from_v1() {
    let suite = CipherSuite::v1_default();

    // The key-wrap envelope is its OWN production type (distinct from
    // `EncryptionEnvelope` — the bulk partition seal). would-FAIL if
    // G-CORE-3 collapses key-wrap into the bulk-seal path (the r2 §TF-3
    // shape (4) "distinct from the bulk-seal arm" obligation).
    let device_a = suite.generate_recipient_keypair();
    let device_b = suite.generate_recipient_keypair();

    // A per-DID partition data-encryption-key (the secret the multi-device
    // recovery path wraps for each owned device).
    let partition_dek = suite.generate_partition_dek();

    let wrapped_a: WrappedDeviceKey =
        KeyWrapEnvelope::wrap_for_device(&suite, device_a.public(), &partition_dek)
            .expect("production key-wrap for device A");
    let wrapped_b: WrappedDeviceKey =
        KeyWrapEnvelope::wrap_for_device(&suite, device_b.public(), &partition_dek)
            .expect("production key-wrap for device B");

    // HNDL property: the key-wrap envelope MUST be PQ-hybrid from
    // format-version-1 (codepoint 0x647a; same hybrid-KEM family as the
    // bulk seal). would-FAIL if the wrap path is classical-only.
    assert_eq!(
        wrapped_a.cipher_codepoint(),
        CipherCodepoint::from_u16(ENC_HYBRID_DEFAULT_CODEPOINT),
        "the multi-device key-wrap envelope MUST be PQ-hybrid from \
         format-version-1 (HNDL is un-retrofittable; classical-only \
         key-wrap is harvest-now-decrypt-later exposed)"
    );
    assert_eq!(wrapped_a.format_version(), ENVELOPE_FORMAT_VERSION_1);
    assert!(
        wrapped_a.is_pq_hybrid() && wrapped_b.is_pq_hybrid(),
        "every per-device wrapped key MUST be PQ-hybrid from v1"
    );

    // The key-wrap surface is NOT the bulk-seal surface: a `WrappedDeviceKey`
    // is structurally NOT a `SealedEnvelope` and cannot be opened by the
    // bulk-seal `open` (compile/type-level distinctness is asserted by the
    // separate types; the runtime arm asserts the unwrap recovers the DEK
    // only with the correct device secret).
    let recovered_a = KeyWrapEnvelope::unwrap(&suite, device_a.secret(), &wrapped_a)
        .expect("device A unwraps its own wrapped DEK");
    assert_eq!(
        recovered_a.as_bytes(),
        partition_dek.as_bytes(),
        "device A MUST recover the exact partition DEK"
    );

    // Cross-device negative: device B's secret MUST NOT unwrap device A's
    // wrapped key (the wrap is per-device-targeted; would-FAIL if the
    // wrap path ignores the recipient).
    let cross = KeyWrapEnvelope::unwrap(&suite, device_b.secret(), &wrapped_a);
    assert!(
        matches!(
            cross,
            Err(KeyWrapError::WrongRecipient) | Err(KeyWrapError::DecapsulationFailed)
        ),
        "device B MUST NOT unwrap device A's wrapped DEK (per-device \
         targeting; got {cross:?})"
    );
}

/// Pin (5) — the ENGINE-SPEC.md:312 trusted-engine-qualifier doc-coupling
/// cite/sentinel. The surrounding 305-318 multi-tenancy/Inv-11 prose MUST
/// carry the "PQ-hybrid-sealed per-DID partition" wording post-G-CORE-3
/// (multitenant-r1-3; §6.1/§6.2 companion-with-canary). would-FAIL if the
/// doc-coupling regresses (the unqualified "multi-tenancy = capability
/// scopes" claim must gain the trusted-engine qualifier — CLAUDE.md
/// baked-in #18 confidentiality-half framing + issue #1301).
///
/// RED-phase: at `ed03729a` ENGINE-SPEC.md:312 still carries the
/// unqualified claim (no encryption substrate exists), so this sentinel
/// FAILS until the G-CORE-3 wave lands the §6.1/§6.2 doc retense. That is
/// the canonical companion-with-canary RED-phase.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3"]
fn tf3_engine_spec_312_trusted_engine_qualifier_doc_sentinel() {
    // Resolve the workspace ENGINE-SPEC.md from CARGO_MANIFEST_DIR
    // (crates/benten-crypto-suite) → repo root → docs/ENGINE-SPEC.md.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/<crate> → repo root")
        .to_path_buf();
    let spec = repo_root.join("docs").join("ENGINE-SPEC.md");
    let body =
        std::fs::read_to_string(&spec).unwrap_or_else(|e| panic!("read {}: {e}", spec.display()));

    // The trusted-engine qualifier MUST be present post-G-CORE-3, worded
    // "PQ-hybrid-sealed per-DID partition" (NOT merely "encrypted
    // partition") per the plan G-CORE-3 doc-coupling clause.
    assert!(
        body.contains("PQ-hybrid-sealed per-DID partition"),
        "ENGINE-SPEC.md MUST carry the trusted-engine qualifier wording \
         \"PQ-hybrid-sealed per-DID partition\" post-G-CORE-3 \
         (multitenant-r1-3; §6.1/§6.2 companion-with-canary; CLAUDE.md \
         baked-in #18 confidentiality-half). would-FAIL until the \
         G-CORE-3 wave lands the §6.1/§6.2 doc retense."
    );

    // The unqualified "multi-tenancy = capability scopes" claim MUST gain
    // a trusted-engine qualifier (it is true ONLY on a cooperating
    // engine; misleading for every untrusted-host phase — baked-in #18).
    assert!(
        body.contains("trusted engine") || body.contains("trusted-engine"),
        "the multi-tenancy=capability-scopes claim MUST be qualified with \
         a trusted-engine caveat post-G-CORE-3 (capabilities give ZERO \
         confidentiality on an untrusted host; encryption is the \
         load-bearing substrate there — CLAUDE.md baked-in #18)"
    );
}

// ============================================================================
// §8-CC DEFERRED-WITH-NAMED-DESTINATION staged-pin arms.
//
// NAMED DESTINATION (HARD-RULE clause-(b), §4-B): the G-CORE-3 brief's
// explicit pre-flight #1301 confidential-content design sub-decision
// + the §8 surfaced sub-fork (§8-CC) + §5 D-list D-4M-13.
//
// These are NOT droppable and NOT a float. The §8-CC design-question
// (plaintext-CID-vs-encrypted-envelope-CID / sign-then-encrypt-vs-
// encrypt-then-sign / what-a-non-decrypting-relay-can-verify) is the ONE
// genuinely-NOT-resolved item. Its resolution SHAPES the P-III-frozen
// canonical-bytes/CID contract, so it MUST be settled BEFORE G-CORE-9
// freezes the #1301 envelope. R3-B1's obligation (§4-B): write the
// family's FULL STRUCTURE including these placeholder staged-pin arms;
// carry the explicit gating note. The option-specific assertion bodies
// are written once §8-CC resolves (a known design-fork already surfaced
// to Ben — NOT an R2/R3 decision). The ordering joint mirrors the old
// NF-4 gating clause: §8-CC MUST resolve → these arms un-ignore → BEFORE
// G-CORE-9.
//
// Marker convention (distinct from the plain RED-PHASE marker so the
// closing-wave sweep + reviewer can tell a §8-CC-gated staged-pin from a
// G-CORE-3 un-ignore):
//   RED-PHASE-STAGED-PIN(§8-CC): un-ignore when §8-CC resolves; MUST land
//   before G-CORE-9.
// ============================================================================

/// §8-CC staged-pin A — CID-addressing arm. Whether the content-CID
/// addresses the cleartext (plaintext-CID) or the sealed envelope
/// (encrypted-envelope-CID) changes what the CID-regression-guard
/// asserts (affects what a non-decrypting peer can dedup/verify; couples
/// the C1 cross-DID non-leak invariant + §8-F P-III canonical-bytes
/// freeze). The STRUCTURE lands now; the concrete option-arm assertion
/// is filled when §8-CC resolves (BEFORE G-CORE-9).
///
/// pim-18 flag-don't-fake: the assertion body is INTENTIONALLY a staged
/// placeholder that fails loud — writing a concrete option-arm now would
/// fake a decision §8-CC has not made. This is the named-destination
/// staged-pin, NOT a hollow sentinel and NOT a dropped test.
#[test]
#[ignore = "RED-PHASE-STAGED-PIN(§8-CC): un-ignore when §8-CC resolves; MUST land before G-CORE-9"]
fn tf3_8cc_cid_addressing_arm_staged_pin() {
    // Intended surface (filled at G-CORE-3 once §8-CC resolves):
    //   benten_crypto_suite::confidential_content::{
    //       ContentCidAddressing, CidAddressingMode,
    //   }
    //
    // The resolved §8-CC decision selects ONE of:
    //   (mode A) plaintext-CID  — CID addresses the cleartext; a
    //            non-decrypting peer CANNOT recompute/verify the CID;
    //   (mode B) envelope-CID   — CID addresses the sealed envelope; a
    //            non-decrypting relay CAN dedup/verify the envelope CID.
    // The concrete assertion (which mode + the CID-regression-guard
    // bytes) is written when §8-CC resolves. Until then this pin FAILS
    // LOUD by design — it is a named-destination staged-pin with a hard
    // pre-G-CORE-9 deadline (§4-B), NOT a droppable test.
    panic!(
        "§8-CC CID-addressing design-question UNRESOLVED. This is a \
         named-destination staged-pin (HARD-RULE clause-(b): destination \
         = the G-CORE-3 brief §1301-confidential-content sub-decision + \
         §8-CC + D-4M-13). It MUST be un-ignored + given its concrete \
         plaintext-CID-vs-envelope-CID assertion once §8-CC resolves, \
         which MUST happen BEFORE G-CORE-9 freezes the #1301 envelope. \
         Do NOT delete or weaken this pin."
    );
}

/// §8-CC staged-pin B — non-decrypting-relay-verify arm. Sign-then-encrypt
/// vs encrypt-then-sign changes what a non-decrypting relay can verify
/// (STE leaks the signer to a non-decrypting relay; ETS hides it but
/// complicates non-decrypting verification). The STRUCTURE lands now; the
/// concrete relay-verify assertion is filled when §8-CC resolves (BEFORE
/// G-CORE-9). Couples §8-F + the C1 cross-DID non-leak invariant.
#[test]
#[ignore = "RED-PHASE-STAGED-PIN(§8-CC): un-ignore when §8-CC resolves; MUST land before G-CORE-9"]
fn tf3_8cc_non_decrypting_relay_verify_arm_staged_pin() {
    // Intended surface (filled at G-CORE-3 once §8-CC resolves):
    //   benten_crypto_suite::confidential_content::{
    //       RelayVerifiable, SignEncryptOrder,
    //   }
    //
    // The resolved §8-CC decision selects STE or ETS and defines exactly
    // what a non-decrypting relay may verify (envelope integrity /
    // codepoint / size only — NEVER plaintext). The concrete assertion is
    // written when §8-CC resolves. Until then this pin FAILS LOUD by
    // design (named-destination staged-pin; hard pre-G-CORE-9 deadline).
    panic!(
        "§8-CC sign-then-encrypt-vs-encrypt-then-sign + \
         what-a-non-decrypting-relay-can-verify design-question \
         UNRESOLVED. Named-destination staged-pin (HARD-RULE clause-(b): \
         destination = G-CORE-3 brief §1301-confidential-content \
         sub-decision + §8-CC + D-4M-13). MUST be un-ignored + given its \
         concrete relay-verify assertion once §8-CC resolves, BEFORE \
         G-CORE-9 freezes the #1301 envelope. Do NOT delete or weaken \
         this pin."
    );
}
