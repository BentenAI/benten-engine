//! Phase 4-Foundation R6-FP-BF wave: closure pin for R6 R1 BLOCKER
//! schema-mat-r6-1 (KEY_TYPE + VALUE_TYPE edges for FieldMap).
//!
//! Substantive arm: compile a SchemaRoot with a `FieldMap` whose
//! `key_scalar` is `text` + `value_scalar` is `int`; assert the emitted
//! Subgraph carries BOTH `KEY_TYPE` and `VALUE_TYPE` edges sourced from
//! the same anchor primitive, each terminating at a type-descriptor
//! primitive carrying the correct `schema_scalar_tag`.
//!
//! WOULD-FAIL-IF-NO-OP per pim-2 §3.6b: removing the `add_edge_labeled`
//! calls in `emit_vocabulary_edges` (FieldMap arm) drops the edge count
//! to zero.

#![allow(clippy::unwrap_used)]

#[test]
fn schema_compiler_emits_key_type_and_value_type_edges_for_field_map() {
    use benten_platform_foundation::schema_compiler::{compile, vocab::VocabEdge};

    const BYTES: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Index",
        "fields": [
            {
                "label": "FieldMap",
                "name": "by_tag",
                "key_scalar": "text",
                "value_scalar": "int",
                "required": true,
                "default": null
            }
        ]
    }"#;

    let spec = compile(BYTES).expect("schema with FieldMap must compile");
    let sg = spec.as_subgraph();

    let key_edges: Vec<&(String, String, String)> = sg
        .edges()
        .iter()
        .filter(|(_, _, label)| label == VocabEdge::KeyType.as_str())
        .collect();
    let value_edges: Vec<&(String, String, String)> = sg
        .edges()
        .iter()
        .filter(|(_, _, label)| label == VocabEdge::ValueType.as_str())
        .collect();

    assert_eq!(key_edges.len(), 1, "exactly 1 KEY_TYPE edge expected");
    assert_eq!(value_edges.len(), 1, "exactly 1 VALUE_TYPE edge expected");

    // Both edges share the same source (the FieldMap's anchor READ).
    assert_eq!(
        key_edges[0].0, value_edges[0].0,
        "KEY_TYPE + VALUE_TYPE must share the same source (the FieldMap anchor); \
         got key.from = {:?}, value.from = {:?}",
        key_edges[0].0, value_edges[0].0
    );

    // Targets carry the correct scalar tags.
    let key_target = sg
        .nodes()
        .iter()
        .find(|n| n.id == key_edges[0].1)
        .expect("KEY_TYPE target must exist");
    let value_target = sg
        .nodes()
        .iter()
        .find(|n| n.id == value_edges[0].1)
        .expect("VALUE_TYPE target must exist");

    let key_tag = key_target
        .property("schema_scalar_tag")
        .and_then(|v| match v {
            benten_core::Value::Text(s) => Some(s.clone()),
            _ => None,
        })
        .expect("KEY_TYPE target must carry `schema_scalar_tag`");
    let value_tag = value_target
        .property("schema_scalar_tag")
        .and_then(|v| match v {
            benten_core::Value::Text(s) => Some(s.clone()),
            _ => None,
        })
        .expect("VALUE_TYPE target must carry `schema_scalar_tag`");

    assert_eq!(
        key_tag, "text",
        "KEY_TYPE target's scalar tag must be `text`"
    );
    assert_eq!(
        value_tag, "int",
        "VALUE_TYPE target's scalar tag must be `int`"
    );
}
