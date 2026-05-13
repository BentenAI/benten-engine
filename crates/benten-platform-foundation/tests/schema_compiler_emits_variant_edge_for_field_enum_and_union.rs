//! Phase 4-Foundation R6-FP-BF wave: closure pin for R6 R1 BLOCKER
//! schema-mat-r6-1 (VARIANT edge for FieldEnum + FieldUnion).
//!
//! Substantive arm: compile a SchemaRoot with both a `FieldEnum`
//! (`Status`: `{draft, published}`) and a `FieldUnion`
//! (`Content`: `{text-string, count-int}`); assert the emitted
//! Subgraph carries one `VARIANT` edge per variant, each terminating at
//! a type-descriptor primitive carrying `schema_variant_name` +
//! `schema_scalar_tag`.
//!
//! WOULD-FAIL-IF-NO-OP per pim-2 §3.6b: removing the `add_edge_labeled`
//! call in `emit_vocabulary_edges` (FieldEnum / FieldUnion arm) drops
//! the edge count to zero.

#![allow(clippy::unwrap_used)]

#[test]
fn schema_compiler_emits_variant_edge_for_field_enum_and_union() {
    use benten_platform_foundation::schema_compiler::{compile, vocab::VocabEdge};

    const BYTES: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Post",
        "fields": [
            {
                "label": "FieldEnum",
                "name": "status",
                "required": true,
                "default": null,
                "variants": [
                    {"name": "draft", "scalar": "text"},
                    {"name": "published", "scalar": "text"}
                ]
            },
            {
                "label": "FieldUnion",
                "name": "body",
                "required": true,
                "default": null,
                "variants": [
                    {"name": "text_body", "scalar": "text"},
                    {"name": "count_body", "scalar": "int"}
                ]
            }
        ]
    }"#;

    let spec = compile(BYTES).expect("schema with FieldEnum + FieldUnion must compile");
    let sg = spec.as_subgraph();

    let variant_edges: Vec<&(String, String, String)> = sg
        .edges()
        .iter()
        .filter(|(_, _, label)| label == VocabEdge::Variant.as_str())
        .collect();

    // 2 enum variants + 2 union variants = 4 VARIANT edges total.
    assert_eq!(
        variant_edges.len(),
        4,
        "expected 4 VARIANT edges (2 for FieldEnum + 2 for FieldUnion); got {} = {:?}",
        variant_edges.len(),
        variant_edges
    );

    // Collect (variant_name, scalar_tag) pairs from the targets.
    let mut pairs: Vec<(String, String)> = variant_edges
        .iter()
        .map(|(_, target_id, _)| {
            let node = sg
                .nodes()
                .iter()
                .find(|n| n.id == *target_id)
                .expect("VARIANT target must exist");
            let name = node
                .property("schema_variant_name")
                .and_then(|v| match v {
                    benten_core::Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .expect("VARIANT target must carry `schema_variant_name`");
            let tag = node
                .property("schema_scalar_tag")
                .and_then(|v| match v {
                    benten_core::Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .expect("VARIANT target must carry `schema_scalar_tag`");
            (name, tag)
        })
        .collect();
    pairs.sort();

    let mut expected: Vec<(String, String)> = vec![
        ("count_body".to_string(), "int".to_string()),
        ("draft".to_string(), "text".to_string()),
        ("published".to_string(), "text".to_string()),
        ("text_body".to_string(), "text".to_string()),
    ];
    expected.sort();

    assert_eq!(pairs, expected, "variant (name, scalar_tag) pairs mismatch");
}
