//! S-3 closure — frozen const VALUE pins for `benten-core`.
//!
//! The CID framing bytes are the deepest freeze surface in the project: every
//! content address ever produced embeds them. The existing assertions in
//! `src/lib.rs` are tautological — they compare encoder output against the same
//! constants the encoder used.
//!
//! Group 2 also closes V1-WIRE-FORMAT-INVENTORY row 9 (EncryptionClass wire
//! codepoints) at INTEGRATION-test level. That row's previously-named pin was
//! an in-`src` `#[cfg(test)]` unit test, which the required `frozen-bytes
//! corpus` CI job (integration targets only) cannot select.

use benten_core::encryption_class::codepoint::{CONFIDENTIAL, PUBLIC};
use benten_core::hlc::Hlc;
use benten_core::subgraph::Subgraph;
use benten_core::subgraph_spec::spec::Spec;
use benten_core::value::MAX_VALUE_DECODE_DEPTH;
use benten_core::{
    ATTRIBUTION_PROPERTY_KEY, BLAKE3_DIGEST_LEN, CID_LEN, CID_V1, LABEL_CURRENT,
    LABEL_NEXT_VERSION, MULTICODEC_DAG_CBOR, MULTIHASH_BLAKE3,
};

// ---------------------------------------------------------------------------
// Group 1 — CIDv1 framing bytes.
//
// WHAT BREAKS: EVERY content address in the system. A change re-frames every
// CID, so every stored node, every signature over a CID, every golden hex and
// every peer's view of shared content diverges. This is the single most
// load-bearing byte group in the freeze — and it previously had no value pin.
// ---------------------------------------------------------------------------

#[test]
fn cid_framing_bytes_are_frozen() {
    assert_eq!(CID_V1, 0x01, "CID version byte — multiformats CIDv1");
    assert_eq!(
        MULTICODEC_DAG_CBOR, 0x71,
        "dag-cbor multicodec — registered multiformats value"
    );
    assert_eq!(
        MULTIHASH_BLAKE3, 0x1e,
        "BLAKE3 multihash code — registered multiformats value"
    );
    assert_eq!(BLAKE3_DIGEST_LEN, 32, "BLAKE3-256 digest length");
    assert_eq!(
        CID_LEN,
        1 + 1 + 1 + 1 + BLAKE3_DIGEST_LEN as usize,
        "CID_LEN is version || codec || hash || len || digest"
    );
    assert_eq!(CID_LEN, 36, "literal CID length");
}

// ---------------------------------------------------------------------------
// Group 2 — encryption-class wire codepoints.
//
// WHAT BREAKS: the per-node confidentiality class byte. Reusing a codepoint for
// a different class is the discipline the #1341 incident established.
// ---------------------------------------------------------------------------

#[test]
fn encryption_class_codepoints_are_frozen() {
    assert_eq!(PUBLIC, 0x00, "EncryptionClass::Public wire codepoint");
    assert_eq!(
        CONFIDENTIAL, 0x01,
        "EncryptionClass::Confidential wire codepoint"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — decode bounds over untrusted CBOR (META #629).
//
// WHAT BREAKS: these bound nesting depth and input length before decode. They
// are the pre-decode boundary guard; widening re-opens stack exhaustion /
// unbounded allocation.
// ---------------------------------------------------------------------------

#[test]
fn core_decode_bounds_are_frozen() {
    assert_eq!(
        MAX_VALUE_DECODE_DEPTH, 64,
        "Value CBOR nesting-depth cap — widening re-opens stack exhaustion"
    );
    assert_eq!(
        Subgraph::MAX_DECODE_BYTES,
        16 * 1024 * 1024,
        "pre-decode input-length ceiling for Subgraph"
    );
    assert_eq!(Subgraph::MAX_DECODE_BYTES, 16_777_216, "literal value");
    assert_eq!(
        Spec::DEFAULT_MAX_DEPTH,
        64,
        "SubgraphSpec default walk-depth bound"
    );
}

// ---------------------------------------------------------------------------
// Group 4 — canonical graph vocabulary that participates in hashing.
//
// WHAT BREAKS: labels and property keys are hashed into node/subgraph CIDs
// (CLAUDE.md #5 — hashing covers labels and properties). A rename silently
// changes every affected CID.
// ---------------------------------------------------------------------------

#[test]
fn hashed_graph_vocabulary_is_frozen() {
    assert_eq!(
        LABEL_CURRENT, "CURRENT",
        "version-chain CURRENT edge label — hashed into CIDs"
    );
    assert_eq!(
        LABEL_NEXT_VERSION, "NEXT_VERSION",
        "version-chain NEXT_VERSION edge label — hashed into CIDs"
    );
    assert_eq!(
        ATTRIBUTION_PROPERTY_KEY, "attribution",
        "attribution property key — hashed into CIDs"
    );
}

// ---------------------------------------------------------------------------
// Group 5 — HLC skew tolerance.
//
// WHAT BREAKS: the clock-skew acceptance window for remote HLC timestamps.
// Widening it weakens the bound on how far a peer can push logical time.
// ---------------------------------------------------------------------------

#[test]
fn hlc_skew_tolerance_is_frozen() {
    assert_eq!(
        Hlc::DEFAULT_SKEW_TOLERANCE_MS,
        5 * 60 * 1000,
        "HLC default skew tolerance = 5 minutes"
    );
    assert_eq!(
        Hlc::DEFAULT_SKEW_TOLERANCE_MS,
        300_000,
        "literal value (ms)"
    );
}
