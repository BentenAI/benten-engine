//! Boundary-hardening closure pins for issue #554 (Safe-2 / META #629
//! DoS-via-unbounded-decode — `benten-core` slice).
//!
//! The byte-size ceiling (`Subgraph::MAX_DECODE_BYTES`) bounds *total
//! allocation* but NOT recursion *depth*: a pure nested-array CBOR payload
//! costs ~1 wire byte per nesting level, so a payload far under the 16 MiB
//! ceiling can still drive `Value::deserialize` into a stack-overflow abort.
//! These tests exercise the real production decode paths (`Value` via
//! `serde_ipld_dagcbor::from_slice`, the same call the `Node` /
//! `Subgraph::load_verified` / sync-handshake / napi paths use) and assert
//! the structural depth guard rejects an adversarial payload with a typed
//! error instead of recursing.
//!
//! Each test would FAIL if the `ValueVisitor` depth guard
//! (`MAX_VALUE_DECODE_DEPTH`) were reverted: without the cap the decode of
//! an over-deep payload either succeeds (no error → assertion fails) or, at
//! the genuinely adversarial depths, aborts the process — either way the
//! "returns a typed error at a modest over-cap depth" assertion does not
//! hold.

use std::collections::BTreeMap;

use benten_core::{MAX_VALUE_DECODE_DEPTH, Node, Subgraph, Value};

/// Build a DAG-CBOR payload that is `levels` nested single-element definite
/// arrays around an integer-0 leaf. CBOR: `0x81` = "array of length 1";
/// `0x00` = "unsigned integer 0". `levels` bytes of `0x81` + one `0x00`
/// byte encodes `levels` structural nesting levels in `levels + 1` bytes —
/// the wire cost is ~1 byte/level, which is exactly why a byte ceiling
/// cannot subsume a depth ceiling.
fn nested_array_payload(levels: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(levels + 1);
    buf.extend(core::iter::repeat_n(0x81u8, levels));
    buf.push(0x00); // innermost leaf: integer 0
    buf
}

/// Build a DAG-CBOR payload of `levels` nested single-entry maps. CBOR:
/// `0xA1` = "map of 1 pair"; `0x60` = "empty text string" (the key);
/// innermost value is `0x00` (integer 0).
fn nested_map_payload(levels: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(levels * 2 + 1);
    for _ in 0..levels {
        buf.push(0xA1); // map(1)
        buf.push(0x60); // key: text string of length 0
    }
    buf.push(0x00); // innermost value: integer 0
    buf
}

#[test]
fn value_decode_accepts_payload_at_the_depth_ceiling() {
    // A payload nested exactly to the ceiling must still decode (the guard
    // rejects strictly *beyond* the budget — an inclusive bound). Pins that
    // the cap is not off-by-one-strict in a way that would reject
    // legitimate shapes, and that benten's guard (not the codec's looser
    // 256) is the deterministic gate at exactly this value.
    let bytes = nested_array_payload(MAX_VALUE_DECODE_DEPTH);
    let decoded: Result<Value, _> = serde_ipld_dagcbor::from_slice(&bytes);
    assert!(
        decoded.is_ok(),
        "a payload nested exactly to MAX_VALUE_DECODE_DEPTH ({}) must \
         decode; the depth guard must reject only *beyond* the budget",
        MAX_VALUE_DECODE_DEPTH
    );
}

#[test]
fn value_decode_rejects_one_past_the_ceiling_with_bentens_typed_error() {
    // Exactly one level past the ceiling — proves benten's guard is the
    // active gate at MAX_VALUE_DECODE_DEPTH (the codec's intrinsic limit is
    // looser ~256, so a hit at +1 can ONLY be benten's guard). Tiny on the
    // wire, so `Subgraph::MAX_DECODE_BYTES` does NOT fire here — only the
    // structural depth guard can reject this. Without the guard this
    // payload decodes successfully and the assertion below fails.
    let over = MAX_VALUE_DECODE_DEPTH + 1;
    let bytes = nested_array_payload(over);
    assert!(
        bytes.len() < 1024,
        "the adversarial payload is tiny on the wire ({} bytes) — proving \
         a byte ceiling cannot subsume the depth ceiling",
        bytes.len()
    );
    let decoded: Result<Value, _> = serde_ipld_dagcbor::from_slice(&bytes);
    let err =
        decoded.expect_err("payload one level past the ceiling MUST be rejected (issue #554)");
    // `serde_ipld_dagcbor::DecodeError` Display is its Debug; benten's
    // custom serde error surfaces as `Msg("DAG-CBOR nesting depth ...")`.
    // A codec-origin depth rejection would surface as `DepthLimit` — so
    // asserting the message text distinguishes "benten's guard fired"
    // (correct) from "the codec's looser guard fired" (would mean benten's
    // guard regressed / was reverted).
    let msg = format!("{err}");
    assert!(
        msg.contains("nesting depth") && msg.contains("#554"),
        "rejection at ceiling+1 must be BENTEN's depth-guard typed error \
         (not the codec's `DepthLimit`), got: {msg}"
    );
}

#[test]
fn value_decode_rejects_over_deep_nested_maps_with_bentens_typed_error() {
    let over = MAX_VALUE_DECODE_DEPTH + 1;
    let bytes = nested_map_payload(over);
    let decoded: Result<Value, _> = serde_ipld_dagcbor::from_slice(&bytes);
    let err = decoded.expect_err("over-deep nested-map payload MUST be rejected (issue #554)");
    let msg = format!("{err}");
    assert!(
        msg.contains("nesting depth") && msg.contains("#554"),
        "rejection must be benten's depth-guard typed error, got: {msg}"
    );
}

#[test]
fn value_decode_rejects_adversarial_deep_payload_without_stack_overflow() {
    // A genuinely adversarial depth that would blow the thread stack if the
    // visitor recursed (each level sinks a Rust stack frame). The guard must
    // reject *before* recursing. Reaching this assertion at all proves the
    // process did not abort (a stack overflow would terminate the test
    // binary, not return an error).
    let adversarial = 200_000;
    let bytes = nested_array_payload(adversarial);
    let decoded: Result<Value, _> = serde_ipld_dagcbor::from_slice(&bytes);
    assert!(
        decoded.is_err(),
        "an adversarially-deep payload MUST be rejected pre-recursion \
         (issue #554 stack-overflow boundary guard)"
    );
}

/// Build a `Value` that is `levels` nested single-element lists around an
/// integer leaf — the in-memory analogue of `nested_array_payload`.
fn deep_value(levels: usize) -> Value {
    let mut v = Value::Int(0);
    for _ in 0..levels {
        v = Value::List(vec![v]);
    }
    v
}

#[test]
fn node_load_verified_rejects_authentic_but_adversarial_deep_property() {
    // #554's trust-boundary case: a malicious peer mints a *genuine* Node
    // (correct CID) whose property value is adversarially deep, then serves
    // `(real_cid, real_bytes)`. The hash-first defense PASSES (the bytes
    // really do hash to the claimed CID) — only the structural depth guard
    // protects the decode. Encoding is the trusted/own-data direction (no
    // cap there, correctly); the cap is on the untrusted decode.
    let mut props = BTreeMap::new();
    props.insert(
        "payload".to_string(),
        deep_value(MAX_VALUE_DECODE_DEPTH + 5),
    );
    let node = Node::new(vec!["Adversarial".to_string()], props);
    let bytes = node
        .to_canonical_bytes()
        .expect("encoding own (over-deep) data is the trusted direction and must succeed");
    let cid = node.cid().expect("cid of own data");

    // Authentic bytes + matching CID: hash-first check passes; the decode
    // then walks the over-deep property and MUST be rejected by the depth
    // guard. Without the guard this returns Ok(node) and the test fails.
    let result = Node::load_verified(&cid, &bytes);
    let err = result
        .expect_err("authentic-but-adversarial deep Node MUST be rejected at decode (issue #554)");
    let msg = format!("{err}");
    assert!(
        msg.contains("nesting depth") || msg.contains("serialization failed"),
        "expected the depth-guard rejection wrapped into CoreError::Serialize, got: {msg}"
    );
}

#[test]
fn subgraph_load_verified_enforces_byte_size_ceiling() {
    // #1152 size-cap arm: `Subgraph::load_verified` rejects payloads above
    // `Subgraph::MAX_DECODE_BYTES` *before* invoking the decoder, bounding
    // the unbounded-allocation vector at the untrusted sync/napi boundary.
    let oversized = vec![0u8; Subgraph::MAX_DECODE_BYTES + 1];
    let err = Subgraph::load_verified(&oversized)
        .expect_err("a payload above MAX_DECODE_BYTES MUST be rejected pre-decode (META #629)");
    let msg = format!("{err}");
    assert!(
        msg.contains("boundary") || msg.contains("ceiling") || msg.contains("#629"),
        "expected the byte-ceiling boundary-guard rejection, got: {msg}"
    );

    // A payload at exactly the ceiling is NOT rejected by the size guard
    // (it fails decode for being non-canonical garbage instead — proving
    // the size check is `>` not `>=` and does not over-reject).
    let at_ceiling = vec![0u8; Subgraph::MAX_DECODE_BYTES];
    let err2 = Subgraph::load_verified(&at_ceiling).expect_err("garbage still fails to decode");
    let msg2 = format!("{err2}");
    assert!(
        !(msg2.contains("boundary") && msg2.contains("ceiling")),
        "a payload exactly at the ceiling must pass the size guard (it then \
         fails as non-canonical), got size-guard rejection: {msg2}"
    );
}

#[test]
fn shallow_legitimate_payload_still_round_trips() {
    // Regression guard: the depth cap must not break legitimate shallow
    // payloads. A 3-level list-of-map-of-list is well within budget.
    let v = Value::List(vec![Value::Map(
        [(
            "k".to_string(),
            Value::List(vec![Value::Int(1), Value::Text("ok".into())]),
        )]
        .into_iter()
        .collect(),
    )]);
    let bytes = serde_ipld_dagcbor::to_vec(&v).expect("encode");
    let decoded: Value =
        serde_ipld_dagcbor::from_slice(&bytes).expect("shallow decode must succeed");
    assert_eq!(
        decoded, v,
        "shallow legitimate payload must round-trip unchanged"
    );
}
