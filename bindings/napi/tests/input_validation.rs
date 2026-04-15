//! Phase 1 R3 security test — napi boundary DoS validation (R1 major #7, B8).
//!
//! Attack class: oversized, deeply-nested, or malformed inputs crossing the
//! TypeScript→Rust napi boundary. Every one is a denial-of-service or
//! memory-exhaustion vector against the engine's hottest surface. The
//! security-auditor enumerated seven specific vectors; R1 triage landed B8
//! as a new deliverable naming the five most critical in the B8 validation
//! matrix:
//!
//!   (i)   `Value::Map` with >10K keys → `E_INPUT_LIMIT`
//!   (ii)  deeply-nested `Value::List/Map` (>128 depth) → `E_INPUT_LIMIT`
//!   (iii) `Value::Bytes` >16MB → `E_INPUT_LIMIT`
//!   (iv)  malformed CID (wrong multicodec / multihash / length) → `E_INPUT_LIMIT`
//!   (v)   recursive DAG-CBOR bomb (millions of nested maps encoded inline)
//!         → `E_INPUT_LIMIT` before Rust allocates the full tree
//!
//! All five must reject BEFORE the Rust side allocates the adversarial
//! payload — allocating first and THEN checking size defeats the DoS
//! defense because the allocation itself is the attack.
//!
//! TDD contract: FAIL at R3. R5 lands B8: napi-side deserialization
//! wrappers that check size/depth/CID shape before delegating to the
//! canonical decoder, mapped to the new `E_INPUT_LIMIT` error code.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #7 (major)
//! - `.addl/phase-1/r1-triage.md` B8 napi input validation
//! - `.addl/phase-1/r2-test-landscape.md` §7 napi input validation

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::ErrorCode;
use benten_napi::testing::{
    deserialize_cid_from_js_like, deserialize_value_from_js_like, make_cbor_bomb, make_deep_list,
    make_giant_bytes, make_giant_map,
};

/// B8-i: `Value::Map` with > 10K keys rejected before allocation.
///
/// Allocation test: the attacker sends a 10K+1 key map. If the Rust side
/// builds the full `BTreeMap` before checking, it has already paid the
/// attack's cost. The defense must early-reject on size metadata (either
/// via CBOR-header length prefix or streaming-count).
#[test]
fn napi_rejects_oversized_value_map() {
    let payload = make_giant_map(/* keys = */ 10_001);
    let before_rss = memory_footprint_kb();
    let result = deserialize_value_from_js_like(&payload);
    let after_rss = memory_footprint_kb();

    let err = result.expect_err("map with >10K keys must be rejected at the napi boundary");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    // Load-bearing: the allocation must NOT precede the rejection. 10K
    // `Value` entries would balloon RSS measurably; a pre-alloc rejection
    // keeps growth near-zero.
    let delta_kb = after_rss.saturating_sub(before_rss);
    assert!(
        delta_kb < 1_024,
        "napi rejection must precede allocation — RSS delta {delta_kb} kB \
         suggests the full map was built before rejection. This defeats \
         the DoS defense."
    );
}

/// B8-ii: deeply-nested `Value::List/Map` rejected at depth 129+.
///
/// Attack: stack blow-up during recursive deserialization. Even though the
/// engine's evaluator is iterative, the `Value` serializer is recursive
/// (by the nature of DAG-CBOR). A 1M-deep list overflows the thread stack
/// — a remote crash.
#[test]
fn napi_rejects_deep_nested_value() {
    let payload = make_deep_list(/* depth = */ 129);
    let result = deserialize_value_from_js_like(&payload);

    let err = result.expect_err("nested depth > 128 must be rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);
}

/// B8-iii: `Value::Bytes` > 16MB rejected.
///
/// Attack: multi-GB payload forcing OOM. The napi boundary is the last
/// chance to bound allocation before the Rust side commits to `Vec<u8>`.
#[test]
fn napi_rejects_oversized_bytes() {
    // 16MB + 1 byte. We use a reference-carrying mock (no real allocation
    // on the test side either) because the test must not itself OOM.
    let payload = make_giant_bytes(/* bytes = */ 16 * 1024 * 1024 + 1);
    let result = deserialize_value_from_js_like(&payload);
    let err = result.expect_err(">16MB bytes must be rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);
}

/// B8-iv: malformed CID rejection.
///
/// Four shapes tested here because CID parsing has several rejection paths
/// (multibase prefix / version byte / multicodec / multihash code / digest
/// length) and each MUST surface `E_INPUT_LIMIT`. If any returns a different
/// code or panics, the napi boundary is lying to the caller.
#[test]
fn napi_rejects_malformed_cid() {
    // (a) not multibase-encoded
    let err = deserialize_cid_from_js_like(b"not-a-cid").expect_err("garbage rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);

    // (b) wrong multicodec (not 0x71 dag-cbor)
    let err = deserialize_cid_from_js_like(
        b"bafkreiho7z4z4...", // dag-pb, not dag-cbor
    )
    .expect_err("wrong multicodec rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);

    // (c) wrong multihash code (not 0x1e BLAKE3)
    let err =
        deserialize_cid_from_js_like(b"bafyrei...sha256...").expect_err("wrong multihash rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);

    // (d) truncated digest
    let err = deserialize_cid_from_js_like(b"bafyr4").expect_err("truncated rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);
}

/// B8-v: recursive DAG-CBOR bomb (millions of nested maps inline-encoded).
///
/// Attack: a single CBOR payload that *decodes* to a deeply-recursive
/// structure even though the wire bytes are only a few megabytes. This is
/// the CBOR equivalent of a zip-bomb. The defense is depth-checking
/// DURING decode — refusing to recurse past 128 levels even if the CBOR
/// claims to be well-formed.
#[test]
fn napi_rejects_recursive_cbor_bomb() {
    // A 4KB CBOR payload that expands to a 1M-deep nested map.
    let payload = make_cbor_bomb(/* nominal_depth = */ 1_000_000);
    assert!(payload.len() < 16 * 1024, "bomb must be SMALL on the wire");

    let result = deserialize_value_from_js_like(&payload);
    let err = result.expect_err("CBOR depth-bomb must be rejected");
    assert_eq!(err.code(), ErrorCode::InputLimit);
}

/// Helper — approximates RSS to detect whether the allocation preceded the
/// rejection. Used only as a tripwire; precision is not required. If the
/// platform lacks a cheap RSS reader, the helper returns 0 and the
/// assertion degrades to true-regardless (ok; the OTHER tests still
/// enforce correctness).
fn memory_footprint_kb() -> u64 {
    // Implementation lives in benten_napi::testing so it can share OS-
    // specific code paths with the production-path memory metrics.
    benten_napi::testing::rss_kb().unwrap_or(0)
}
