//! **F-NQC4-1** — did:key hybrid-pubkey multicodec (CE-I2 + WF-H1, NQ-C4).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition; the hybrid
//! encode/decode arms wired GREEN at the `benten-id-hybrid-key` wave (the
//! B2 sealed-sender sender-auth prerequisite).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-NQC4-1**:
//!   "multicodec prefix for PQ-hybrid sig (Ed25519⊕ML-DSA-65) + KEM
//!    (X25519⊕ML-KEM-768) pubkeys in `did:key`; if no registered
//!    multiformats value, private-value-with-fallback reserved at
//!    G-CORE-9 round-trips; `did:agent:` optional allowlist alias.
//!    §10.6 NQ-C4, §4.1, U15; `did.rs:26,98`.  FG.  ~4-5 tests.
//!    Red-phase intent: `did_key_encode(hybrid_pk).prefix()==MULTICODEC`;
//!    round-trip; unknown multicodec → `UnknownMulticodec`; did:agent
//!    allowlist pin. SURFACES multiformats-registration question if no
//!    value."
//!
//! **OPEN-SPEC ARM RESOLVED (NQ-C4 / §5.D-9):** at G-CORE-9 the multiformats
//! registry had no assigned value for the PQ-hybrid pubkey shape, so a
//! reserved private-value-with-fallback was held. Registered COMPONENT codes
//! now exist (`mldsa-65-pub = 0x1211`, `ed25519-pub = 0xed`), so the v1 hybrid
//! `did:key` encoding uses the **two-registered-component-multikey** form
//! (`varint(0x1211) ‖ mldsaPK(1952) ‖ varint(0xed) ‖ tradPK(32)`, ML-DSA
//! FIRST) — #5-clean, no invented number. The single-byte `0xef`/`0xf0`
//! interim values are relegated to documented fallback-only. This wave wires
//! the REAL `benten_id::did::{Did::from_hybrid_public_key, Did::resolve_hybrid}`
//! and replaces the prior self-contained codec stub.
//!
//! **`did:agent:` (PIN-3) + the registration-doc-coupling (PIN-4) stay a
//! SEPARATE NAMED carry** — the `did:agent:` alias is an Inv-22 nature
//! concern orthogonal to sender-auth; it is NOT absorbed by this hybrid-key
//! wave (the arms remain `#[ignore]`'d with a precise carry reason below).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_crypto_suite::sig::SignatureSuite;
use benten_id::did::{Did, ED25519_MULTICODEC, MLDSA65_PUB_MULTICODEC};
use benten_id::errors::DidError;

/// Read the in-tree `did.rs` source for doc-coupling assertions.
fn did_rs() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/benten-id/src/did.rs"
    ))
    .expect("did.rs must be present")
}

/// Mint a real v1-beta-default hybrid (Ed25519⊕ML-DSA-65) verifying key.
fn hybrid_public_key() -> benten_crypto_suite::sig::PublicKey {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let pk = kp.public();
    assert!(
        pk.is_hybrid(),
        "v1-beta default suite MUST mint a hybrid (PQ-carrying) key"
    );
    pk
}

// ===========================================================================
// HYBRID ARMS (GREEN) — drive the REAL benten-id two-component-multikey
// encode/decode. These replace the prior self-contained stub codec.
// ===========================================================================

/// PIN 1 (registered-component-codec) — the REAL `did:key` hybrid encode
/// emits the two-component-multikey body `varint(0x1211) ‖ mldsaPK ‖
/// varint(0xed) ‖ tradPK` (ML-DSA FIRST, registered component codes), and
/// `resolve_hybrid` recovers a byte-identical composite key. Would-FAIL if
/// the codec is lossy, mis-orders the components, or shoe-horns the hybrid
/// key under a single Benten-private prefix.
#[test]
fn f_nqc4_1_hybrid_did_key_round_trips() {
    let pk = hybrid_public_key();

    let did = Did::from_hybrid_public_key(&pk);
    assert!(
        did.as_str().starts_with("did:key:z"),
        "hybrid did:key MUST carry the W3C `z` (base58btc) multibase prefix"
    );

    let recovered = did
        .resolve_hybrid()
        .expect("a freshly-encoded hybrid did:key MUST resolve");

    // Byte-identical round-trip: re-serialize both composites and compare.
    let want = pk
        .to_lamps_composite_bytes()
        .expect("hybrid pk serializes to LAMPS composite bytes");
    let got = recovered
        .to_lamps_composite_bytes()
        .expect("resolved hybrid pk serializes to LAMPS composite bytes");
    assert_eq!(
        got, want,
        "did:key hybrid encode/decode MUST round-trip the composite pubkey \
         bytes byte-for-byte (mldsaPK ‖ tradPK). A lossy or mis-ordered codec \
         regresses U15."
    );
}

/// PIN 1b — the hybrid did:key body uses the REGISTERED component multicodecs
/// in ML-DSA-first order (`0x1211` then `0xed`), NOT a Benten-private
/// single-byte squat. Decodes the base58btc body and asserts the leading
/// varint is `MLDSA65_PUB_MULTICODEC` and the second component prefix is
/// `ED25519_MULTICODEC`. Would-FAIL if the encoder fell back to `0xef`/`0xf0`.
#[test]
fn f_nqc4_1_hybrid_body_uses_registered_component_codecs_mldsa_first() {
    let pk = hybrid_public_key();
    let did = Did::from_hybrid_public_key(&pk);
    let body = did
        .as_str()
        .strip_prefix("did:key:z")
        .expect("did:key:z prefix");
    let decoded = bs58::decode(body).into_vec().expect("base58btc decode");

    // Leading varint == registered mldsa-65-pub (0x1211 = [0x91, 0x24]).
    assert_eq!(
        [decoded[0], decoded[1]],
        MLDSA65_PUB_MULTICODEC,
        "hybrid did:key MUST lead with the registered ML-DSA-65 component \
         multicodec (0x1211), ML-DSA FIRST — never a Benten-private prefix"
    );

    // The Ed25519 component (registered ed25519-pub = [0xed, 0x01]) follows
    // after the ML-DSA pubkey. Locate it via the upstream-sourced ML-DSA-65
    // pubkey length (no hardcoded size).
    let mldsa_len = benten_crypto_suite::sizes::ml_dsa_65_pubkey_len();
    let ed_prefix_at = MLDSA65_PUB_MULTICODEC.len() + mldsa_len;
    assert_eq!(
        [decoded[ed_prefix_at], decoded[ed_prefix_at + 1]],
        ED25519_MULTICODEC,
        "the SECOND hybrid component MUST be the registered Ed25519 \
         component multicodec (0xed)"
    );
}

/// PIN 2 — typed-reject on a corrupted hybrid did:key. Flipping the leading
/// component-codec byte MUST surface `DidError::UnknownMulticodec` (NEVER a
/// silent accept, NEVER a panic). Mirrors the `did.rs` typed-reject contract.
#[test]
fn f_nqc4_1_hybrid_unknown_component_codec_typed_reject() {
    let pk = hybrid_public_key();
    let did = Did::from_hybrid_public_key(&pk);
    let body = did
        .as_str()
        .strip_prefix("did:key:z")
        .expect("did:key:z prefix");
    let mut decoded = bs58::decode(body).into_vec().expect("base58btc decode");

    // Corrupt the leading component-codec byte to a value in no registry slot.
    decoded[0] = 0xff;
    let corrupted = Did::from_string_for_test_fixture(format!(
        "did:key:z{}",
        bs58::encode(&decoded).into_string()
    ));

    // `PublicKey` is not `Debug`, so match on the `Result` rather than
    // `expect_err` (which would require `T: Debug`).
    match corrupted.resolve_hybrid() {
        Err(DidError::UnknownMulticodec(0xff, _)) => {}
        Err(other) => panic!(
            "an unrecognized hybrid component multicodec MUST yield \
             UnknownMulticodec, never a silent fallback; got {other:?}"
        ),
        Ok(_) => panic!("a wrong component multicodec MUST be typed-rejected, not accepted"),
    }
}

/// PIN 2b — trailing-byte fail-closed. A well-formed hybrid body is EXACTLY
/// the two component multikeys; an extra appended byte MUST surface
/// `DidError::HybridTrailingBytes` (payload-stuffing defense).
#[test]
fn f_nqc4_1_hybrid_trailing_bytes_rejected() {
    let pk = hybrid_public_key();
    let did = Did::from_hybrid_public_key(&pk);
    let body = did
        .as_str()
        .strip_prefix("did:key:z")
        .expect("did:key:z prefix");
    let mut decoded = bs58::decode(body).into_vec().expect("base58btc decode");
    decoded.push(0x00); // one byte of slack

    let stuffed = Did::from_string_for_test_fixture(format!(
        "did:key:z{}",
        bs58::encode(&decoded).into_string()
    ));
    match stuffed.resolve_hybrid() {
        Err(DidError::HybridTrailingBytes { extra: 1 }) => {}
        Err(other) => panic!(
            "a hybrid body with slack MUST yield HybridTrailingBytes, never \
             a silent truncation-accept; got {other:?}"
        ),
        Ok(_) => panic!("trailing bytes after both components MUST be rejected, not accepted"),
    }
}

/// PIN 2c — truncated-body fail-closed. A body too short for the ML-DSA
/// component MUST surface `DidError::HybridBodyTooShort` (NEVER an
/// out-of-bounds panic).
#[test]
fn f_nqc4_1_hybrid_truncated_body_rejected() {
    let pk = hybrid_public_key();
    let did = Did::from_hybrid_public_key(&pk);
    let body = did
        .as_str()
        .strip_prefix("did:key:z")
        .expect("did:key:z prefix");
    let mut decoded = bs58::decode(body).into_vec().expect("base58btc decode");
    decoded.truncate(10); // far too short for mldsaPK

    let short = Did::from_string_for_test_fixture(format!(
        "did:key:z{}",
        bs58::encode(&decoded).into_string()
    ));
    match short.resolve_hybrid() {
        Err(DidError::HybridBodyTooShort { .. }) => {}
        Err(other) => panic!(
            "a too-short hybrid body MUST yield HybridBodyTooShort, never a \
             panic or other error; got {other:?}"
        ),
        Ok(_) => panic!("a truncated hybrid body MUST be typed-rejected, not accepted"),
    }
}

/// PIN 1c (doc-coupling) — `did.rs` defines a hybrid-SIG multicodec const +
/// the registered ML-DSA-65 component multicodec const. Would-FAIL if the
/// hybrid encode regressed to shoe-horning under the Ed25519 prefix.
#[test]
fn f_nqc4_1_did_rs_defines_hybrid_multicodecs() {
    let src = did_rs();
    assert!(
        src.contains("HYBRID_SIG_MULTICODEC") && src.contains("MLDSA65_PUB_MULTICODEC"),
        "did.rs MUST define the hybrid-SIG fallback const AND the registered \
         ML-DSA-65 component multicodec (NQ-C4 / U15)."
    );
    assert!(
        src.contains("HYBRID_KEM_MULTICODEC"),
        "did.rs MUST retain the hybrid-KEM fallback const (NQ-C4 / U15)."
    );
}

/// PIN 4-partial (doc-coupling, GREEN here) — the registered-component-codec
/// resolution of NQ-C4 is surfaced in `CRYPTO-CODEPOINTS.md`: the doc cites
/// the registered component multicodec values and marks `0xef`/`0xf0` as
/// fallback-only. (The broader `did:agent:` PIN-3 + the full PIN-4 carry stay
/// ignored below — this arm only pins the hybrid-key resolution this wave
/// lands.)
#[test]
fn f_nqc4_1_registered_component_codec_resolution_surfaced() {
    let codepoints = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/CRYPTO-CODEPOINTS.md"
    ))
    .expect("CRYPTO-CODEPOINTS.md must be present");
    assert!(
        codepoints.contains("multicodec") && codepoints.contains("hybrid"),
        "CRYPTO-CODEPOINTS.md MUST surface the hybrid-pubkey multicodec \
         registration status (NQ-C4)."
    );
    assert!(
        codepoints.contains("0x1211"),
        "CRYPTO-CODEPOINTS.md NQ-C4 section MUST cite the registered \
         ML-DSA-65 component multicodec (0x1211) the v1 hybrid did:key uses."
    );
}

// ===========================================================================
// SEPARATE NAMED CARRY — `did:agent:` alias (PIN-3) + the broader NQ-C4
// did:agent doc-coupling (PIN-4). Carried: NQ-C4 did:agent arm, separate from
// the hybrid-key wave. `did:agent` is an Inv-22 nature concern (nature DERIVED
// via method-parse; alias is an OPTIONAL allowlist hint, never authority-
// bearing) orthogonal to B2 sealed-sender SENDER-AUTH. NOT absorbed here.
// ===========================================================================

/// PIN 3 — `did:agent:` optional allowlist alias.
///
/// **CARRIED:** NQ-C4 `did:agent:` arm — separate from the hybrid-key wave
/// (Inv-22 nature-derived alias, orthogonal to sender-auth). Un-ignored by the
/// dedicated `did:agent:`/Inv-22 wave.
#[test]
#[ignore = "CARRIED: NQ-C4 did:agent arm, separate from the hybrid-key wave (Inv-22 nature concern, not sender-auth)"]
fn f_nqc4_1_did_agent_optional_allowlist_alias() {
    let src = did_rs();
    assert!(
        src.contains("did:agent"),
        "did.rs MUST name the `did:agent:` optional allowlist alias \
         (NQ-C4 + Inv-22: nature DERIVED via method-parse, alias is an \
         OPTIONAL allowlist hint, never a stored authoritative \
         discriminator)."
    );
    // Over-claim guard: the alias MUST NOT be described as carrying authority.
    assert!(
        !src.contains("did:agent grants") && !src.contains("did:agent authority"),
        "`did:agent:` MUST be an OPTIONAL alias, never authority-bearing \
         (Inv-22 nature-derived-not-stored)."
    );
}
