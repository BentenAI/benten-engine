//! G23-B GREEN: Renderer / output-format pluggability validated
//! empirically by 2 impls (arch-r1-10 + D-4F-11 ratified).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerWalkInputs,
    PlaintextMaterializer, allow_all_cap_recheck,
};
use std::collections::BTreeMap;

#[test]
fn materializer_output_backend_pluggable_two_impls_compile_and_round_trip() {
    // BOTH impls MUST compile against the SAME trait signature.
    fn assert_materializer<M: Materializer>(_: &M) {}
    let html = HtmlJsonMaterializer;
    let plain = PlaintextMaterializer;
    assert_materializer(&html);
    assert_materializer(&plain);

    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("the body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));

    let inputs = || MaterializerWalkInputs {
        engine: &engine,
        spec: &spec,
        content_cid: cid,
        walk_principal: alice,
        cap_recheck: allow_all_cap_recheck(),
        declared_requires: Vec::new(),
    };

    let html_out = html.materialize_with_gate(inputs()).unwrap();
    let plain_out = plain.materialize_with_gate(inputs()).unwrap();

    // OUTPUTS DIFFER STRUCTURALLY — proves trait is not accidentally
    // HtmlJson-specific.
    let html_str = std::str::from_utf8(html_out.primary_bytes()).unwrap();
    let plain_str = std::str::from_utf8(plain_out.primary_bytes()).unwrap();
    assert!(
        html_str.contains('<') && html_str.contains('>'),
        "HtmlJsonMaterializer emits HTML tags: html_str={html_str}"
    );
    assert!(
        !plain_str.contains('<') && !plain_str.contains('>'),
        "PlaintextMaterializer MUST NOT emit HTML tags (else trait is HTML-coupled): plain_str={plain_str}"
    );

    // ROUND-TRIP: both impls re-produce identical output across runs
    // (per mat-r1-3 inherited determinism).
    let html_out2 = html.materialize_with_gate(inputs()).unwrap();
    let plain_out2 = plain.materialize_with_gate(inputs()).unwrap();
    assert_eq!(html_out.primary_bytes(), html_out2.primary_bytes());
    assert_eq!(plain_out.primary_bytes(), plain_out2.primary_bytes());

    // Structural-but-shape: both produce SOME field for "body".
    assert!(
        html_str.contains("body") && plain_str.contains("body"),
        "both impls surface the body field; format differs"
    );
    assert!(plain_str.contains("body: the body"));
}
