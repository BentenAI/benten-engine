//! S-3 closure — frozen const VALUE pins for `benten-id`.
//!
//! The multicodec prefixes below are the FIRST TWO BYTES of every `did:key`
//! this project emits or parses. They are recorded by type only in
//! `docs/public-api/benten-id.txt`, and no test anywhere asserts their value.
//!
//! The UCAN decode bounds are the confirmed decode-bound tautology: the only
//! test that exercises `MAX_UCAN_PROOF_DEPTH`
//! (`crates/benten-engine/tests/typed_call_engine_dispatch.rs`) builds its
//! over-deep fixture as `MAX_UCAN_PROOF_DEPTH + 8`, so widening the bound
//! widens the fixture and the test stays green.

use benten_id::did::{
    DID_AGENT_PREFIX, DID_BENTEN_METHOD, DID_BENTEN_PREFIX, DID_KEY_PREFIX, ED25519_MULTICODEC,
    HYBRID_KEM_MULTICODEC, HYBRID_SIG_MULTICODEC, MAX_DID_KEY_STRING_LEN, MLDSA65_PUB_MULTICODEC,
    MLKEM768_PUB_MULTICODEC, X25519_PUB_MULTICODEC,
};
use benten_id::keypair::{ENVELOPE_ALG, ENVELOPE_VERSION};
use benten_id::keyset::{
    KEM_CP_HYBRID_X25519_MLKEM768, KEYSET_DOC_VERSION, MAX_KEYSET_DOC_BYTES,
    SIG_CP_LAMPS_MLDSA65_ED25519,
};
use benten_id::ucan::{MAX_UCAN_ENVELOPE_BYTES, MAX_UCAN_PER_LINK_BYTES, MAX_UCAN_PROOF_DEPTH};
use benten_id::vc::{MAX_VC_ENVELOPE_BYTES, VC_CONTEXT_V1, VC_TYPE_BASE};

// ---------------------------------------------------------------------------
// Group 1 — did:key multicodec prefixes.
//
// WHAT BREAKS: every DID string this project has ever emitted. These are
// cross-ecosystem identifiers; a change breaks interop with every external
// did:key resolver as well as every stored Benten identity.
// ---------------------------------------------------------------------------

#[test]
fn did_key_multicodec_prefixes_are_frozen() {
    assert_eq!(
        ED25519_MULTICODEC,
        [0xed, 0x01],
        "Ed25519 public-key multicodec (registered) — the first 2 bytes of every classical did:key"
    );
    assert_eq!(
        MLDSA65_PUB_MULTICODEC,
        [0x91, 0x24],
        "ML-DSA-65 public-key multicodec"
    );
    assert_eq!(
        MLKEM768_PUB_MULTICODEC,
        [0x8c, 0x24],
        "ML-KEM-768 public-key multicodec"
    );
    assert_eq!(
        X25519_PUB_MULTICODEC,
        [0xec, 0x01],
        "X25519 public-key multicodec (registered)"
    );
    assert_eq!(
        HYBRID_SIG_MULTICODEC,
        [0xef, 0x01],
        "Benten hybrid signature multicodec"
    );
    assert_eq!(
        HYBRID_KEM_MULTICODEC,
        [0xf0, 0x01],
        "Benten hybrid KEM multicodec"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — DID string prefixes.
//
// WHAT BREAKS: parsing and emission of every DID form. These are parsed from
// untrusted input, so a change is both an interop break and a parser-behaviour
// change.
// ---------------------------------------------------------------------------

#[test]
fn did_string_prefixes_are_frozen() {
    assert_eq!(DID_KEY_PREFIX, "did:key:z", "did:key multibase-z prefix");
    assert_eq!(DID_BENTEN_METHOD, "did:benten:", "did:benten method prefix");
    assert_eq!(
        DID_BENTEN_PREFIX, "did:benten:z",
        "did:benten multibase-z prefix"
    );
    assert_eq!(DID_AGENT_PREFIX, "did:agent:", "did:agent prefix");
}

// ---------------------------------------------------------------------------
// Group 3 — keyset document + keypair envelope.
//
// WHAT BREAKS: KEYSET_DOC_VERSION and ENVELOPE_VERSION are wire version bytes.
// The two codepoints are the frozen v1-beta defaults; they duplicate the
// crypto-suite registry values, so a one-sided edit is a cross-crate wire split.
// ---------------------------------------------------------------------------

#[test]
fn keyset_and_keypair_envelope_constants_are_frozen() {
    assert_eq!(KEYSET_DOC_VERSION, 1, "keyset document wire version");
    assert_eq!(
        SIG_CP_LAMPS_MLDSA65_ED25519, 0x0001,
        "LAMPS composite signature codepoint — must match crypto-suite registry"
    );
    assert_eq!(
        KEM_CP_HYBRID_X25519_MLKEM768, 0x647a,
        "hybrid KEM codepoint — must match crypto-suite registry"
    );
    assert_eq!(ENVELOPE_VERSION, 1, "keypair envelope wire version");
    assert_eq!(ENVELOPE_ALG, "Ed25519", "keypair envelope algorithm tag");
}

// ---------------------------------------------------------------------------
// Group 4 — decode bounds over untrusted input (the DoS cluster).
//
// WHAT BREAKS: every one of these caps a decode of attacker-supplied bytes
// (META #629). Their own tests build fixtures FROM the constants, so widening
// is currently invisible. Pinning the value is the only thing that makes a
// widening deliberate.
// ---------------------------------------------------------------------------

#[test]
fn identity_decode_bounds_are_frozen() {
    assert_eq!(
        MAX_DID_KEY_STRING_LEN, 4_096,
        "cap on an untrusted did:key string before base58 decode"
    );
    assert_eq!(
        MAX_KEYSET_DOC_BYTES,
        8 * 1024,
        "cap on an untrusted keyset document"
    );
    assert_eq!(MAX_KEYSET_DOC_BYTES, 8_192, "literal value");

    assert_eq!(
        MAX_UCAN_PROOF_DEPTH, 32,
        "UCAN proof-chain depth cap over untrusted CBOR — widening re-opens unbounded recursion"
    );
    assert_eq!(
        MAX_UCAN_PER_LINK_BYTES,
        16 * 1024,
        "per-proof-link byte cap"
    );
    assert_eq!(MAX_UCAN_PER_LINK_BYTES, 16_384, "literal value");
    assert_eq!(
        MAX_UCAN_ENVELOPE_BYTES,
        MAX_UCAN_PROOF_DEPTH * MAX_UCAN_PER_LINK_BYTES,
        "envelope cap is defined as depth * per-link"
    );
    assert_eq!(
        MAX_UCAN_ENVELOPE_BYTES, 524_288,
        "literal value (32 * 16384)"
    );

    assert_eq!(
        MAX_VC_ENVELOPE_BYTES,
        16 * 1024,
        "cap on an untrusted verifiable-credential envelope"
    );
    assert_eq!(MAX_VC_ENVELOPE_BYTES, 16_384, "literal value");
}

// ---------------------------------------------------------------------------
// Group 5 — W3C VC vocabulary.
//
// WHAT BREAKS: external interop with W3C VC verifiers.
// ---------------------------------------------------------------------------

#[test]
fn vc_vocabulary_is_frozen() {
    assert_eq!(
        VC_CONTEXT_V1, "https://www.w3.org/2018/credentials/v1",
        "W3C VC v1 JSON-LD context URI — externally specified"
    );
    assert_eq!(
        VC_TYPE_BASE, "VerifiableCredential",
        "W3C VC base type — externally specified"
    );
}
