//! G23-B GREEN: materializer end-to-end SubgraphSpec → HTML+JSON output
//! walk (exit-criterion 2; LOAD-BEARING substantive).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 1.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-B primary
//!   must-pass-test.
//! - Plan §1 deliverable 2 — materializer pipeline lands as IVM-view-shaped
//!   subgraph composition (Ben D-4F-2 ratified).

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
fn materializer_pipeline_walks_composed_subgraph_to_html_output() {
    let schema_bytes = schema_fixtures::canonical_note_type_schema_bytes();
    let spec =
        benten_platform_foundation::compile_schema(schema_bytes).expect("Note schema compiles");
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Build a content Node matching the schema's field set + insert via
    // InMemoryMaterializerEngine. The materializer reads it through the
    // engine's `read_node_as` seam per Class B β.
    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("the quick brown fox".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let content_cid = engine.put_node(Node::new(vec!["Note".into()], props));

    let mat = HtmlJsonMaterializer;
    let out = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect("walk succeeds for canonical Note");

    // HTML side: contains the article wrapper for the lowercased schema
    // name + the body field's content.
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        html.contains("<article class=\"benten-note\">"),
        "HTML article wrapper present: html={html}"
    );
    assert!(
        html.contains("the quick brown fox"),
        "field content rendered into HTML: html={html}"
    );
    assert!(
        html.contains("benten-field-body"),
        "field div class present: html={html}"
    );

    // JSON side: carries the schema-derived cap-scope (sec-3.5-r1-4
    // schema-derived; lowercased per canonical projection).
    let json = std::str::from_utf8(out.json_bytes()).unwrap();
    assert!(
        json.contains("\"body\":\"the quick brown fox\""),
        "JSON projection contains body field: json={json}"
    );
    assert!(
        json.contains("\"scope\""),
        "JSON projection carries schema-derived scope array: json={json}"
    );

    // No cap-denials.
    assert!(
        out.cap_denials().is_empty(),
        "allow-all gate yields no denials"
    );
    // Exactly one row admitted (the content Node).
    assert_eq!(
        out.materialized_row_cids().len(),
        1,
        "exactly one row materialised end-to-end"
    );
}
