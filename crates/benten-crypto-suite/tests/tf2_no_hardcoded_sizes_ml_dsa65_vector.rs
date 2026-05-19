//! TF-2 pin (c) + S3 — concrete ML-DSA-65-dimensioned synthetic vector
//! through EVERY size-touching surface; FAILS on any 32 B-key / 64 B-sig
//! (Ed25519-shaped) assumption.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core Wave R3-A,
//! agent R3-A2, family TF-2. **This is the explicit pim-18
//! SHAPE-not-SUBSTANCE trap flagged in r2-test-landscape §4-A for TF-2**:
//! a weak test could "assert agility" while silently assuming ≤64 B
//! (crypto-agility-r1-5 / r1-triage row 31). This file refuses to be that
//! weak test — it drives a **concrete ~1952 B-key / ~3309 B-sig**
//! synthetic ML-DSA-65 vector through struct / DAG-CBOR / napi / TS /
//! redb / CID-derivation / fixtures and asserts NO Ed25519-shaped size
//! assumption survives anywhere on the production path.
//!
//! Pin sources: r2-test-landscape TF-2 RED-phase shape (3) + S3 + §4-A
//! trap row; plan §4 CI "Signature-agility codepoint conformance" line
//! (`a concrete synthetic ML-DSA-65 key (~1952B) + sig (~3309B) round-trips
//! through every size-touching surface … and FAILS on any 32B-key/64B-sig
//! assumption, mandatory at G-CORE-2 merge`) + §1.A.FROZEN item 10
//! (the JS analog widening); CLAUDE.md #5 ("Never hardcode key/sig/
//! ciphertext sizes").
//!
//! # RED-PHASE STATUS (pim-12 §3.6e)
//!
//! `benten-crypto-suite` is a STUB at R3-A; the intended G-CORE-2 surface
//! does not exist, so these compile-but-fail at the `use` line. All
//! `#[ignore]`-staged `RED-PHASE: un-ignore at G-CORE-2`.
//!
//! # §3.5g cross-language rule-mirror note (carried into the G-CORE-2 brief)
//!
//! The "no-hardcoded-sizes" property is a Rust↔TS mirrored rule. The
//! intended Rust surface here (`SizeTouchingSurfaces`) MUST have an atomic
//! TS-side parity assertion in `packages/engine/` /
//! `bindings/napi` over `TypedCallInputShapes` / `TypedCallOutputShapes`
//! (`ed25519_*` / `keypair_*` / `did_resolve` arms) + `ManifestSignature`
//! per §1.A.FROZEN item 10 (the #1204 parity gate is asserted against the
//! PQ-hybrid-capable shape, NOT an Ed25519-shaped baseline). The TS-side
//! file is NOT writable at R3-A (the napi binding for the new crate is a
//! G-CORE-2 deliverable); it is flagged SHAPE-only-pending-production in
//! the report and carried as a G-CORE-2 §3.5g brief obligation rather than
//! shipped hollow (pim-18 — flag, do not fake).

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// RED-PHASE failure point: intended G-CORE-2 size-agility surface.
use benten_crypto_suite::sig::{HybridSignature, SignatureSuite};
use benten_crypto_suite::sizes::{SizeTouchingSurfaces, SyntheticVector};

/// The Ed25519-shaped sizes that MUST NOT appear as hardcoded assumptions
/// anywhere on the production path. ML-DSA-65 dwarfs them.
const ED25519_PUBKEY_LEN: usize = 32;
const ED25519_SIG_LEN: usize = 64;

/// ML-DSA-65 (FIPS-204 Category-3) canonical dimensions. The synthetic
/// vector MUST be exactly these — a test that assumes ≤64 B would FAIL
/// the moment it tries to hold this vector.
const ML_DSA_65_PUBKEY_LEN: usize = 1952;
const ML_DSA_65_SIG_LEN: usize = 3309;

/// Build the canonical concrete synthetic vector once. NOT a sentinel —
/// it is genuinely ML-DSA-65-dimensioned.
fn ml_dsa65_synthetic() -> SyntheticVector {
    let v = SyntheticVector::ml_dsa65_for_test();
    // Self-check the fixture is honest: if G-CORE-2 ever silently shrinks
    // this to an Ed25519 shape, THIS assertion fails first.
    assert_eq!(
        v.pq_pubkey_len(),
        ML_DSA_65_PUBKEY_LEN,
        "synthetic vector PQ pubkey MUST be ML-DSA-65-dimensioned (~1952 B), \
         not an Ed25519-shaped (32 B) stand-in (pim-18 SHAPE-trap)"
    );
    assert_eq!(
        v.pq_sig_len(),
        ML_DSA_65_SIG_LEN,
        "synthetic vector PQ sig MUST be ML-DSA-65-dimensioned (~3309 B)"
    );
    assert!(
        v.pq_pubkey_len() > ED25519_PUBKEY_LEN && v.pq_sig_len() > ED25519_SIG_LEN,
        "ML-DSA-65 dims MUST strictly exceed Ed25519 dims (the whole point \
         of the no-hardcoded-sizes class)"
    );
    v
}

/// Surface 1 — struct / in-memory: the `HybridSignature` struct MUST hold
/// the full ML-DSA-65-dimensioned PQ half without truncation. would-FAIL
/// if any `[u8; 64]`-shaped field silently clips it.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_struct_surface_holds_full_ml_dsa65_sig() {
    let v = ml_dsa65_synthetic();
    let sig: HybridSignature = v.as_hybrid_signature();
    assert_eq!(
        sig.pq_half_for_test().len(),
        ML_DSA_65_SIG_LEN,
        "HybridSignature struct surface MUST hold the full ~3309 B ML-DSA-65 \
         half — would-FAIL if a fixed [u8;64] field truncates it"
    );
}

/// Surface 2 — DAG-CBOR: serialize→deserialize the ML-DSA-65-dimensioned
/// hybrid signature through the production canonical DAG-CBOR codec and
/// assert byte-exact round-trip (no size-clamped field).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_dag_cbor_surface_round_trips_ml_dsa65() {
    let v = ml_dsa65_synthetic();
    let sig = v.as_hybrid_signature();

    let encoded = SizeTouchingSurfaces::dag_cbor_encode(&sig);
    let decoded = SizeTouchingSurfaces::dag_cbor_decode(&encoded);
    assert_eq!(
        decoded.pq_half_for_test().len(),
        ML_DSA_65_SIG_LEN,
        "DAG-CBOR round-trip MUST preserve the full ML-DSA-65 sig length; \
         would-FAIL on any 64 B clamp in the codec path"
    );
    assert_eq!(
        decoded, sig,
        "DAG-CBOR round-trip MUST be byte-exact for the ML-DSA-65 hybrid sig"
    );
}

/// Surface 3 — redb persistence: persist→load the ML-DSA-65-dimensioned
/// signature through the production redb-backed store path and assert no
/// truncation. would-FAIL if a fixed-width column/key clips it.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_redb_surface_persists_full_ml_dsa65() {
    let v = ml_dsa65_synthetic();
    let sig = v.as_hybrid_signature();

    let handle = SizeTouchingSurfaces::redb_store_for_test();
    handle.put_signature("k", &sig);
    let loaded = handle.get_signature("k").expect("must round-trip");
    assert_eq!(
        loaded.pq_half_for_test().len(),
        ML_DSA_65_SIG_LEN,
        "redb persistence MUST preserve the full ML-DSA-65 sig; \
         would-FAIL on any 64 B fixed-width assumption in the store layer"
    );
}

/// Surface 4 — CID derivation: deriving a CID over canonical bytes that
/// contain the ML-DSA-65-dimensioned signature MUST consume the full
/// length (the hash input is not length-clamped). would-FAIL if the
/// canonical-bytes builder assumes a 64 B sig region.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_cid_derivation_consumes_full_ml_dsa65() {
    let v = ml_dsa65_synthetic();
    let sig = v.as_hybrid_signature();

    let cid_full = SizeTouchingSurfaces::cid_over_signed_bytes(&sig);
    // Mutating a byte BEYOND the Ed25519 64 B boundary MUST change the
    // CID — proving the hash consumed the full ML-DSA-65 region, not
    // just the first 64 B.
    let mut clipped_view = sig.clone();
    clipped_view.flip_pq_byte_for_test(ED25519_SIG_LEN + 1000);
    let cid_mutated = SizeTouchingSurfaces::cid_over_signed_bytes(&clipped_view);
    assert_ne!(
        cid_full, cid_mutated,
        "a byte change at offset >64 B MUST change the CID — proves the \
         canonical-bytes/CID path consumed the FULL ~3309 B ML-DSA-65 sig, \
         not an Ed25519-shaped 64 B prefix (pim-18 substantive guard)"
    );
}

/// Surface 5 — napi/TS boundary shape: the napi-marshalled representation
/// MUST round-trip the ML-DSA-65-dimensioned bytes without an Ed25519-
/// shaped buffer assumption. The Rust side of the cross-language mirror
/// (§3.5g); the TS-side parity assertion is a G-CORE-2 §3.5g brief
/// obligation (flagged SHAPE-only-pending-production in the report —
/// the new crate's napi binding does not exist at R3-A).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_napi_boundary_shape_round_trips_ml_dsa65() {
    let v = ml_dsa65_synthetic();
    let sig = v.as_hybrid_signature();

    let marshalled = SizeTouchingSurfaces::napi_marshal(&sig);
    let unmarshalled = SizeTouchingSurfaces::napi_unmarshal(&marshalled);
    assert_eq!(
        unmarshalled.pq_half_for_test().len(),
        ML_DSA_65_SIG_LEN,
        "napi boundary MUST NOT impose an Ed25519-shaped (64 B) buffer; \
         the ML-DSA-65 sig MUST round-trip intact (§3.5g Rust side; the \
         TS-side parity assertion is the atomic G-CORE-2 mirror obligation)"
    );
}

/// Surface 6 — fixtures: the committed test/golden fixtures for the
/// signature surface MUST be regenerable at ML-DSA-65 dimensions (no
/// fixture frozen at an Ed25519 shape that would silently pin the wrong
/// size class forever).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_fixtures_are_ml_dsa65_dimensioned() {
    let fixture = SizeTouchingSurfaces::load_signature_fixture("hybrid_v1_default");
    assert_eq!(
        fixture.pq_pubkey_len(),
        ML_DSA_65_PUBKEY_LEN,
        "the committed hybrid-default fixture MUST be ML-DSA-65-dimensioned; \
         an Ed25519-shaped fixture would silently pin the wrong size class"
    );
    assert_eq!(fixture.pq_sig_len(), ML_DSA_65_SIG_LEN);
}

/// Cross-surface invariant: the suite MUST NOT expose ANY public constant
/// or API that hardcodes a signature/pubkey size. This is the structural
/// anti-Ed25519-assumption pin — the seam reports sizes dynamically from
/// the codepoint, never from a `const … = 64`.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_no_public_hardcoded_size_constant() {
    // The size MUST be reported by the codepoint-dispatched suite, not a
    // compile-time constant. If G-CORE-2 ever introduces a public
    // `pub const SIG_LEN`, the seam is no longer size-agile.
    let suite = SignatureSuite::v1_default();
    let reported = suite.signature_byte_len_for(suite.default_codepoint());
    assert_eq!(
        reported,
        ED25519_SIG_LEN + ML_DSA_65_SIG_LEN,
        "the hybrid sig length MUST be the SUM of both halves, reported \
         dynamically from the codepoint — never a hardcoded Ed25519 64 B"
    );
    assert!(
        !SignatureSuite::exposes_static_size_constant(),
        "the integration crate MUST NOT expose a hardcoded public size \
         constant — sizes are codepoint-derived (CLAUDE.md #5)"
    );
}
