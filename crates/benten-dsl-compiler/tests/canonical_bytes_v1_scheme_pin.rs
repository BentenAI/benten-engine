//! Wire-format v1 closure-pin for #604 / #798 / #782 (canonical-byte
//! encoding of compiled subgraphs).
//!
//! Pre-v1 canonical-bytes normalization (refinement-audit-2026-05 wire-format
//! cluster item 1). The DSL compiler now emits the `benten_core::Subgraph-
//! Builder`-canonical property-key namespace (no underscore prefixes) and
//! uniform 2-char per-node id prefixes. The canonical bytes are a function
//! of these keys + ids (DAG-CBOR over the sorted `properties` BTreeMap +
//! `(id, kind)`-sorted node view), so this scheme is WIRE-STABLE and frozen
//! at v1.
//!
//! These assertions WOULD FAIL against the pre-fix scheme (underscore keys
//! `_target`/`_module`/`_body`/…, mixed-width id prefixes `r0`/`wait0`) and
//! WOULD FAIL if a future change perturbs the canonical encoding — that is
//! the point: post-v1 CID churn is catastrophic, so the encoding is pinned
//! here against a hard-coded expected CID.

#![allow(clippy::unwrap_used)]

use benten_dsl_compiler::{PrimitiveKind, compile_str};

/// A representative handler exercising every key-bearing primitive.
const PIN_SRC: &str = "handler 'wire-pin' { read('post') -> branch($x == 1) -> \
     transform({ a: $b }) -> call('h', { k: $v }) -> \
     sandbox('m', { fuel: 1000 }) -> emit('topic') -> subscribe('pat/*') -> \
     stream('label') -> iterate(x < 3) -> wait({ ttl: 5 }) -> respond }";

#[test]
fn dsl_emits_canonical_subgraphbuilder_key_namespace_no_underscore() {
    let c = compile_str(PIN_SRC).unwrap();
    // No emitted property key carries the legacy reserved underscore prefix.
    for p in &c.primitives {
        for k in p.properties.keys() {
            assert!(
                !k.starts_with('_'),
                "primitive {:?} emitted reserved-underscore key {k:?} — \
                 #604 scheme-(a) requires canonical SubgraphBuilder keys",
                p.kind
            );
        }
    }
    // Spot-check the canonical names that mirror SubgraphBuilder literals.
    let by_kind = |k: PrimitiveKind| {
        c.primitives
            .iter()
            .find(|p| p.kind == k)
            .expect("primitive present")
    };
    assert!(
        by_kind(PrimitiveKind::Call)
            .properties
            .contains_key("handler")
    );
    assert!(
        by_kind(PrimitiveKind::Sandbox)
            .properties
            .contains_key("module")
    );
    assert!(
        by_kind(PrimitiveKind::Transform)
            .properties
            .contains_key("body")
    );
    assert!(
        by_kind(PrimitiveKind::Branch)
            .properties
            .contains_key("predicate")
    );
    assert!(
        by_kind(PrimitiveKind::Emit)
            .properties
            .contains_key("topic")
    );
    assert!(
        by_kind(PrimitiveKind::Subscribe)
            .properties
            .contains_key("pattern")
    );
    // #782: ITERATE body is the DISTINCT `iter_body` (Text) — not the
    // shared `_body`, not falsely equated to SubgraphBuilder's `max` (Int).
    assert!(
        by_kind(PrimitiveKind::Iterate)
            .properties
            .contains_key("iter_body")
    );
    assert!(
        !by_kind(PrimitiveKind::Transform)
            .properties
            .contains_key("iter_body")
    );
}

#[test]
fn dsl_emits_uniform_two_char_node_id_prefixes() {
    let c = compile_str(PIN_SRC).unwrap();
    // The per-node id feeds canonical bytes via `(id, kind)` sort. #798
    // scheme-(a): every prefix is exactly 2 ASCII chars (then the index).
    // We re-derive ids the same way `emit` does (kind-prefixed, enumerated).
    let sg = &c.subgraph;
    for node in sg.nodes() {
        let id = &node.id;
        let digit_start = id.find(|ch: char| ch.is_ascii_digit()).unwrap_or(id.len());
        assert_eq!(
            digit_start, 2,
            "node id {id:?} prefix is not uniform-2-char (#798 scheme-a)"
        );
    }
}

#[test]
fn canonical_cid_is_wire_stable_and_pinned() {
    let c1 = compile_str(PIN_SRC).unwrap();
    let cid1 = c1.subgraph.cid().unwrap();

    // Re-compile from scratch → identical CID (deterministic + permutation-
    // stable via BTreeMap + sorted canonical view).
    let c2 = compile_str(PIN_SRC).unwrap();
    assert_eq!(
        c2.subgraph.cid().unwrap(),
        cid1,
        "recompiled handler must yield an identical canonical CID"
    );

    // Round-trip through canonical bytes → identical CID.
    let bytes = c1.subgraph.canonical_bytes().unwrap();
    let sg2 = benten_core::Subgraph::from_dagcbor(&bytes).unwrap();
    assert_eq!(
        sg2.cid().unwrap(),
        cid1,
        "canonical-bytes round-trip must preserve the CID"
    );

    // Hard-coded v1-frozen CID. This value is the canonical CID under the
    // post-fix scheme. If it ever changes the wire format has drifted —
    // catastrophic post-v1. Update ONLY via a deliberate, ratified
    // pre-v1 scheme change.
    const PINNED_V1_CID: &str = "bafyr4ib3prqi5s53m4gm4jjqhye3retmm3sollxk5kuhecbe6q6df6g6gi";
    assert_eq!(
        cid1.to_string(),
        PINNED_V1_CID,
        "canonical CID drifted from the v1-frozen value — wire-format \
         regression. If this is an intentional pre-v1 scheme change, \
         update PINNED_V1_CID and re-ratify."
    );
}
