//! G23-B GREEN: canonical-bytes determinism across runs (mat-r1-3).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerWalkInputs,
    allow_all_cap_recheck,
};
use std::collections::BTreeMap;

#[test]
fn materializer_canonical_bytes_determinism_across_runs() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("deterministic".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));

    let mat = HtmlJsonMaterializer;
    let mk = || MaterializerWalkInputs {
        engine: &engine,
        spec: &spec,
        content_cid: cid,
        walk_principal: alice,
        cap_recheck: allow_all_cap_recheck(),
        declared_requires: Vec::new(),
    };
    let out1 = mat.materialize_with_gate(mk()).unwrap();
    let out2 = mat.materialize_with_gate(mk()).unwrap();
    let out3 = mat.materialize_with_gate(mk()).unwrap();

    assert_eq!(
        out1.html_bytes(),
        out2.html_bytes(),
        "HTML bytes stable across 2 runs"
    );
    assert_eq!(
        out2.html_bytes(),
        out3.html_bytes(),
        "HTML bytes stable across 3+ runs"
    );
    assert_eq!(
        out1.json_bytes(),
        out2.json_bytes(),
        "JSON bytes stable across runs"
    );

    // Content-addressing: canonical CID over the output bytes is stable.
    let cid1 = out1.canonical_cid();
    let cid2 = out2.canonical_cid();
    let cid3 = out3.canonical_cid();
    assert_eq!(cid1, cid2, "canonical CID stable across runs");
    assert_eq!(cid2, cid3, "canonical CID stable across 3+ runs");
}
