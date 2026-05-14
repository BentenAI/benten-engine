//! Phase 4-Foundation R6-FP-BF wave: closure pin for R6 R1 BLOCKER
//! schema-mat-r6-1 (the 5 labeled vocabulary edges un-wired pre-fix-pass —
//! historical narrative said "5 of 6" but the canonical post-vocabulary-
//! reduction vocabulary at HEAD is 5 labeled edges total, with
//! object-to-field as implicit-via-recursion; see
//! `docs/SCHEMA-DRIVEN-RENDERING.md §2.2`).
//!
//! Substantive arm: compile a SchemaRoot with a `FieldList` whose
//! `item_scalar` is `text`; assert the emitted Subgraph carries an
//! `ITEM_TYPE` edge from the list's anchor primitive to a terminal
//! type-descriptor primitive whose `schema_scalar_tag` property is
//! `"text"`. The edge label MUST exactly match
//! `VocabEdge::ItemType.as_str()` per D-4F-NEW-TYPED-FIELD-NODE-VOCAB.
//!
//! WOULD-FAIL-IF-NO-OP per pim-2 §3.6b: removing the `add_edge_labeled`
//! call in `emit_vocabulary_edges` (FieldList arm) collapses the
//! `ITEM_TYPE` edge count to zero + this assertion fails.

#![allow(clippy::unwrap_used)]

#[test]
fn schema_compiler_emits_item_type_edge_for_field_list() {
    use benten_platform_foundation::schema_compiler::{compile, vocab::VocabEdge};

    const BYTES: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "TodoList",
        "fields": [
            {
                "label": "FieldList",
                "name": "items",
                "item_scalar": "text",
                "required": true,
                "default": null
            }
        ]
    }"#;

    let spec = compile(BYTES).expect("schema with FieldList must compile");
    let sg = spec.as_subgraph();

    let item_type_edges: Vec<&(String, String, String)> = sg
        .edges()
        .iter()
        .filter(|(_, _, label)| label == VocabEdge::ItemType.as_str())
        .collect();

    assert_eq!(
        item_type_edges.len(),
        1,
        "exactly 1 ITEM_TYPE edge expected (FieldList `items` with `item_scalar=text`); \
         got {} edges = {:?}",
        item_type_edges.len(),
        item_type_edges
    );

    // The edge's target Node carries `schema_scalar_tag = "text"`.
    let target_id = &item_type_edges[0].1;
    let target_node = sg
        .nodes()
        .iter()
        .find(|n| n.id == *target_id)
        .expect("ITEM_TYPE edge target must exist");
    let tag = target_node
        .property("schema_scalar_tag")
        .and_then(|v| match v {
            benten_core::Value::Text(s) => Some(s.clone()),
            _ => None,
        })
        .expect("ITEM_TYPE target must carry `schema_scalar_tag` property");
    assert_eq!(tag, "text", "ITEM_TYPE target's scalar tag must be `text`");
}
