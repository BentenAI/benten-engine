//! Phase 4-Foundation R6-FP-BF wave: closure pin for R6 R1 BLOCKER
//! schema-mat-r6-1 (REF_TARGET edge for FieldRef).
//!
//! Substantive arm: compile a SchemaRoot with a `FieldRef` whose
//! `ref_target_kind` is `"PluginDid"`; assert the emitted Subgraph
//! carries a `REF_TARGET` edge from the FieldRef's anchor primitive to
//! a type-descriptor primitive whose `schema_ref_target_kind` property
//! is `"PluginDid"`.
//!
//! WOULD-FAIL-IF-NO-OP per pim-2 §3.6b: removing the `add_edge_labeled`
//! call in `emit_vocabulary_edges` (FieldRef arm) drops the edge count
//! to zero.

#![allow(clippy::unwrap_used)]

#[test]
fn schema_compiler_emits_ref_target_edge_for_field_ref() {
    use benten_platform_foundation::schema_compiler::{compile, vocab::VocabEdge};

    const BYTES: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Note",
        "fields": [
            {
                "label": "FieldRef",
                "name": "author",
                "ref_target_kind": "PluginDid",
                "required": false,
                "default": null
            }
        ]
    }"#;

    let spec = compile(BYTES).expect("schema with FieldRef must compile");
    let sg = spec.as_subgraph();

    let ref_edges: Vec<&(String, String, String)> = sg
        .edges()
        .iter()
        .filter(|(_, _, label)| label == VocabEdge::RefTarget.as_str())
        .collect();

    assert_eq!(
        ref_edges.len(),
        1,
        "exactly 1 REF_TARGET edge expected for `author` FieldRef"
    );

    let target_id = &ref_edges[0].1;
    let target_node = sg
        .nodes()
        .iter()
        .find(|n| n.id == *target_id)
        .expect("REF_TARGET edge target must exist");
    let kind = target_node
        .property("schema_ref_target_kind")
        .and_then(|v| match v {
            benten_core::Value::Text(s) => Some(s.clone()),
            _ => None,
        })
        .expect("REF_TARGET target must carry `schema_ref_target_kind`");
    assert_eq!(
        kind, "PluginDid",
        "REF_TARGET target's ref_target_kind must be `PluginDid`"
    );
}
