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

// R4 triage (m12): the napi-export crate is a cdylib and its public-facing
// `napi` symbols cannot be linked from a regular integration test binary.
// These tests exercise the Rust-side `benten_napi::testing::*` helpers that
// B8 wires up behind the `in-process-test` feature. Run via:
//
//     cargo test -p benten-napi --features in-process-test --no-default-features
//
// The cfg-gate below keeps the workspace test-all invocation green by
// compiling an empty test binary when the feature is off.
#![cfg(feature = "in-process-test")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
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

// ---------------------------------------------------------------------------
// B8 boundary pins (added when B8 landed, 2026-07-29).
//
// The five tests above are the R3 contract: each proves that ONE hostile
// payload is refused. None of them proves the limit sits where the docs say
// it sits, and none of them would notice a decoder that refuses everything
// — a `deserialize_value_from_js_like` returning `Err(InputLimit)`
// unconditionally passes all five.
//
// Everything below closes that. Each test either pins the ACCEPT side of a
// boundary (so a too-strict limit fails) or asserts on the rejection
// MESSAGE (so deleting one of several checks that share `E_INPUT_LIMIT` is
// still detectable). Every one carries the mutation that must make it fail.
// ---------------------------------------------------------------------------

/// Non-vacuity anchor for the whole file.
///
/// MUTATION THAT MUST MAKE THIS FAIL: make `decode_value_bounded` return
/// `Err(input_limit(..))` unconditionally — the shape that would make all
/// five R3 tests above pass while enforcing nothing.
#[test]
fn napi_accepts_well_formed_payload_under_every_limit() {
    // {"a": 1}
    let payload = [0xa1_u8, 0x61, b'a', 0x01];
    let value = deserialize_value_from_js_like(&payload)
        .expect("a four-byte map is under every documented limit");
    match value {
        benten_core::Value::Map(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(m.get("a"), Some(&benten_core::Value::Int(1)));
        }
        other => panic!("expected Value::Map, got {other:?}"),
    }
}

/// The map-key ceiling is exactly 10 000: 10 000 decodes, 10 001 does not.
///
/// The reject half is already covered by `napi_rejects_oversized_value_map`;
/// this adds the half that catches a limit set too LOW, which no
/// rejection-only test can see.
///
/// MUTATION THAT MUST MAKE THIS FAIL: change `arg > NAPI_MAX_MAP_KEYS` to
/// `>=` in `input_limits::scan_dag_cbor`, or lower `NAPI_MAX_MAP_KEYS` by
/// one -> the 10 000-key map is rejected and the `expect` panics.
#[test]
fn napi_map_limit_boundary_is_exactly_10k() {
    let at_limit = make_giant_map(10_000);
    let value = deserialize_value_from_js_like(&at_limit)
        .expect("a 10 000-key map is AT the limit, not over");
    match value {
        benten_core::Value::Map(m) => assert_eq!(m.len(), 10_000),
        other => panic!("expected Value::Map, got {other:?}"),
    }

    let over = make_giant_map(10_001);
    let err = deserialize_value_from_js_like(&over).expect_err("10 001 keys is over");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("map_size"),
        "rejection must name the map-key limit, got: {}",
        err.message()
    );
}

/// The nesting ceiling is exactly 64 — the depth the canonical DAG-CBOR
/// decoder itself stops at (`benten_core::MAX_VALUE_DECODE_DEPTH`, byte-
/// pinned at 64 by `canonical_bytes_v1_const_values_core.rs`), NOT the 128
/// the B8 matrix and `docs/ERROR-CATALOG.md` name.
///
/// The literals below are deliberately spelled out rather than written as
/// `MAX_VALUE_DECODE_DEPTH` / `+ 1`: using the same constant the
/// implementation derives from would make this pass no matter what the
/// constant became, which is the assert-a-value-against-itself shape.
///
/// The ACCEPT arm is the important one and it is not decoration: it fails
/// if the napi ceiling is ever raised above the decoder's, which is the
/// exact way this check becomes dead code (the decoder would refuse first
/// and the napi guard would never fire). An earlier draft of B8 set the
/// napi cap to the documented 128 and this arm is what caught it.
///
/// MUTATION THAT MUST MAKE THIS FAIL: change `stack.len() > NAPI_MAX_DEPTH`
/// to `>=` in `input_limits::push_frame` -> depth 64 is rejected and the
/// `expect` panics. Or set `NAPI_MAX_DEPTH = 128` -> depth 65 is no longer
/// refused by the napi guard, the canonical decoder refuses it instead, and
/// the code becomes `E_SERIALIZE`.
#[test]
fn napi_depth_limit_boundary_is_exactly_64() {
    let at_limit = make_deep_list(64);
    deserialize_value_from_js_like(&at_limit)
        .expect("depth 64 is AT the limit and must round-trip through the canonical decoder");

    let over = make_deep_list(65);
    let err = deserialize_value_from_js_like(&over).expect_err("depth 65 is over");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("nesting_depth"),
        "rejection must name the depth limit, got: {}",
        err.message()
    );
}

/// The byte-string ceiling is exactly 16 MiB.
///
/// Both payloads are five wire bytes — a head declaring a length with no
/// body behind it. The one AT the limit gets past the size cap and dies in
/// the truncation arm with `E_SERIALIZE`; the one over it is refused by the
/// size cap with `E_INPUT_LIMIT`. That difference is what locates the
/// constant, and it costs no allocation to observe.
///
/// MUTATION THAT MUST MAKE THIS FAIL: lower `NAPI_MAX_BYTES` by one -> the
/// at-limit payload starts reporting `E_INPUT_LIMIT` and the `assert_eq`
/// below fails. Raise it by one -> `napi_rejects_oversized_bytes` above
/// starts reporting `E_SERIALIZE` and fails.
#[test]
fn napi_bytes_limit_boundary_is_exactly_16_mib() {
    // bytes(16 777 216) — exactly 16 MiB, payload absent.
    let at_limit = [0x5a_u8, 0x01, 0x00, 0x00, 0x00];
    let err = deserialize_value_from_js_like(&at_limit)
        .expect_err("the body is absent, so this is still an error — just not a LIMIT error");
    assert_eq!(
        err.code(),
        ErrorCode::Serialize,
        "16 MiB is AT the limit; it must fail as truncated input, not as an over-limit input. \
         Got: {}",
        err.message()
    );

    // bytes(16 777 217) — one over.
    let over = [0x5a_u8, 0x01, 0x00, 0x00, 0x01];
    let err = deserialize_value_from_js_like(&over).expect_err("one byte over the ceiling");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("bytes_len"),
        "rejection must name the bytes limit, got: {}",
        err.message()
    );
}

/// The text-string ceiling is exactly 1 MiB. Same head-only technique as
/// the byte-string boundary above.
///
/// MUTATION THAT MUST MAKE THIS FAIL: change `NAPI_MAX_TEXT_BYTES` in
/// either direction -> one of the two arms flips its code.
#[test]
fn napi_text_leaf_limit_is_exactly_one_mib() {
    // text(1 048 576) — exactly 1 MiB, payload absent.
    let at_limit = [0x7a_u8, 0x00, 0x10, 0x00, 0x00];
    let err = deserialize_value_from_js_like(&at_limit).expect_err("body absent");
    assert_eq!(
        err.code(),
        ErrorCode::Serialize,
        "1 MiB is AT the text limit, got: {}",
        err.message()
    );

    // text(1 048 577) — one over.
    let over = [0x7a_u8, 0x00, 0x10, 0x00, 0x01];
    let err = deserialize_value_from_js_like(&over).expect_err("one byte over");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("text_len"),
        "rejection must name the text limit, got: {}",
        err.message()
    );
}

/// The list-element ceiling is exactly 10 000, matching the `list_size`
/// default `docs/ERROR-CATALOG.md` publishes.
///
/// MUTATION THAT MUST MAKE THIS FAIL: change `arg > NAPI_MAX_LIST_ITEMS`
/// to `>=` -> the 10 000-element list is rejected and the `expect` panics.
#[test]
fn napi_list_limit_boundary_is_exactly_10k() {
    // array(10 000) followed by 10 000 x Int(0).
    let mut at_limit = vec![0x99_u8, 0x27, 0x10];
    at_limit.extend(std::iter::repeat_n(0x00_u8, 10_000));
    let value = deserialize_value_from_js_like(&at_limit).expect("10 000 elements is AT the limit");
    match value {
        benten_core::Value::List(items) => assert_eq!(items.len(), 10_000),
        other => panic!("expected Value::List, got {other:?}"),
    }

    // array(10 001) followed by 10 001 x Int(0).
    let mut over = vec![0x99_u8, 0x27, 0x11];
    over.extend(std::iter::repeat_n(0x00_u8, 10_001));
    let err = deserialize_value_from_js_like(&over).expect_err("10 001 elements is over");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("list_size"),
        "rejection must name the list limit, got: {}",
        err.message()
    );
}

/// A collection header that declares more children than the input can
/// possibly contain is refused as an attack, not walked.
///
/// This is the guard that makes the pre-allocation property hold for
/// collections: without it a decoder is free to size a buffer from a
/// number the payload never has to honour.
///
/// MUTATION THAT MUST MAKE THIS FAIL: delete the body of
/// `input_limits::check_declared_fits` -> the scanner walks off the end of
/// the input, and this reports `E_SERIALIZE` (truncated) instead of
/// `E_INPUT_LIMIT`.
#[test]
fn napi_rejects_declared_length_amplification() {
    // array(10 000) with nothing behind it — within the list-size cap, so
    // ONLY the amplification guard can refuse this one.
    let payload = [0x99_u8, 0x27, 0x10];
    let err = deserialize_value_from_js_like(&payload)
        .expect_err("a 3-byte input cannot hold 10 000 children");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("declared_length"),
        "rejection must name the declared-length guard, got: {}",
        err.message()
    );
}

/// The CBOR bomb is refused by the list-size cap, before the amplification
/// guard and long before anything is allocated.
///
/// Companion to `napi_rejects_recursive_cbor_bomb` above, which only pins
/// the code. Pinning the message is what keeps the two guards independently
/// observable: they both return `E_INPUT_LIMIT`, so the code alone cannot
/// tell which one fired.
///
/// MUTATION THAT MUST MAKE THIS FAIL: delete the `NAPI_MAX_LIST_ITEMS`
/// check -> the amplification guard refuses instead and the message says
/// `declared_length`.
#[test]
fn napi_rejects_recursive_cbor_bomb_reports_list_size() {
    let payload = make_cbor_bomb(1_000_000);
    let err = deserialize_value_from_js_like(&payload).expect_err("bomb");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("list_size"),
        "the bomb must be caught by the per-list cap, got: {}",
        err.message()
    );
}

/// A payload where every individual collection is legal but the total item
/// count is not.
///
/// 200 lists of 10 000 integers each: no list exceeds `list_size`, nothing
/// exceeds `nesting_depth`, no declared length is unsatisfiable, and the
/// wire is ~2 MB — comfortably under the payload ceiling. Only the
/// aggregate item cap stands between this and a multi-million-node
/// `Value` tree.
///
/// MUTATION THAT MUST MAKE THIS FAIL: delete the `NAPI_MAX_TOTAL_ITEMS`
/// check in `input_limits::scan_dag_cbor` -> this payload decodes
/// successfully and the `expect_err` panics.
#[test]
fn napi_rejects_item_count_blowup() {
    let mut payload = vec![0x98_u8, 0xc8]; // array(200)
    for _ in 0..200 {
        payload.extend_from_slice(&[0x99, 0x27, 0x10]); // array(10 000)
        payload.extend(std::iter::repeat_n(0x00_u8, 10_000));
    }
    assert!(
        payload.len() < 16 * 1024 * 1024,
        "must be under the payload ceiling"
    );

    let err = deserialize_value_from_js_like(&payload).expect_err("2 000 201 items");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("total_items"),
        "rejection must name the aggregate item cap, got: {}",
        err.message()
    );
}

/// A genuine Benten `CIDv1` string round-trips through the boundary.
///
/// Without this, every assertion in `napi_rejects_malformed_cid` is
/// satisfied by a parser that rejects unconditionally.
///
/// MUTATION THAT MUST MAKE THIS FAIL: make `parse_cid_string_bounded`
/// return `Err` unconditionally.
#[test]
fn napi_accepts_valid_cid_string() {
    let cid = benten_core::Cid::from_blake3_digest([7_u8; 32]);
    let text = cid.to_string();
    let parsed = deserialize_cid_from_js_like(text.as_bytes())
        .expect("a CID this boundary itself minted must parse");
    assert_eq!(parsed, cid);
}

/// A structurally perfect CID that carries the dag-pb multicodec (`0x70`)
/// instead of dag-cbor (`0x71`) is refused.
///
/// The R3 fixture for this case is the literal `b"bafkreiho7z4z4..."`,
/// which contains `.` — not in the base32 alphabet — so it can only ever
/// exercise the alphabet arm, never the multicodec arm it is labelled with.
/// These bytes are exact.
///
/// MUTATION THAT MUST MAKE THIS FAIL: delete the `MULTICODEC_DAG_CBOR`
/// check in `benten_core::Cid::from_bytes` -> this CID parses and the
/// `expect_err` panics.
#[test]
fn napi_rejects_dag_pb_multicodec_cid() {
    let mut raw = vec![0x01_u8, 0x70, 0x1e, 0x20];
    raw.extend_from_slice(&[7_u8; 32]);
    let text = format!(
        "b{}",
        data_encoding::BASE32_NOPAD.encode(&raw).to_lowercase()
    );

    let err = deserialize_cid_from_js_like(text.as_bytes())
        .expect_err("dag-pb is not a Benten multicodec");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    // Pin the ARM, not just the code: every malformed CID reports
    // E_INPUT_LIMIT, so the code alone cannot show the multicodec check is
    // what refused this one rather than, say, a length check.
    assert!(
        err.message().contains("multicodec"),
        "must be refused by the multicodec arm, got: {}",
        err.message()
    );
}

/// A structurally perfect CID that carries the sha2-256 multihash (`0x12`)
/// instead of BLAKE3 (`0x1e`) is refused. Exact bytes, for the same reason
/// as the multicodec case: the R3 fixture `b"bafyrei...sha256..."` cannot
/// reach the multihash arm.
///
/// MUTATION THAT MUST MAKE THIS FAIL: delete the `MULTIHASH_BLAKE3` check
/// in `benten_core::Cid::from_bytes` -> this CID parses and the
/// `expect_err` panics.
#[test]
fn napi_rejects_sha256_multihash_cid() {
    let mut raw = vec![0x01_u8, 0x71, 0x12, 0x20];
    raw.extend_from_slice(&[7_u8; 32]);
    let text = format!(
        "b{}",
        data_encoding::BASE32_NOPAD.encode(&raw).to_lowercase()
    );

    let err = deserialize_cid_from_js_like(text.as_bytes())
        .expect_err("sha2-256 is not the Benten multihash");
    assert_eq!(err.code(), ErrorCode::InputLimit);
    assert!(
        err.message().contains("multihash"),
        "must be refused by the multihash arm, got: {}",
        err.message()
    );
}
