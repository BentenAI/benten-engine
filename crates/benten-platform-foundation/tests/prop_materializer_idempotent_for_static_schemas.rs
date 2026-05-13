//! G23-B GREEN proptest: materializer idempotence for static schemas
//! (post-materializer safety per mat-r1-3 + cag-r1-1 + cag-r1-6).
//!
//! For ARBITRARY structurally-valid static schemas, the materializer
//! walk is idempotent: running it twice on the same inputs produces
//! byte-identical output. Stronger than mat-r1-3 which covers only
//! the canonical fixture.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_core::{Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerWalkInputs,
    allow_all_cap_recheck,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Generator: arbitrary schema bytes (single-Note shape with N fields).
/// Cases bounded to ~64 per MSRV 1.95 wall-clock budget (Phase-3 precedent).
fn any_static_schema_strategy() -> impl Strategy<Value = (String, Vec<String>, String)> {
    // Single-Note schema; field names are alphabetic 1-8 chars; body
    // string is alphabetic 1-32 chars.
    (
        "[a-z]{1,8}",
        proptest::collection::vec("[a-z]{1,8}", 1..=4),
        "[a-z ]{1,32}",
    )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64, // MSRV 1.95 wall-clock budget (Phase-3 precedent)
        .. ProptestConfig::default()
    })]
    #[test]
    fn prop_materializer_idempotent_for_static_schemas(
        (schema_name, field_names, body) in any_static_schema_strategy()
    ) {
        // Skip generations with duplicate field names (schema_compiler
        // rejects them).
        let mut unique_fields: Vec<String> = field_names;
        unique_fields.sort();
        unique_fields.dedup();
        if unique_fields.is_empty() {
            return Ok(());
        }
        // Skip cases where field name is "body" so we don't collide with
        // our Node content key.
        unique_fields.retain(|f| f != "body");
        // Always have a `body` field for content payload.
        unique_fields.insert(0, "body".into());

        let field_decls: Vec<String> = unique_fields
            .iter()
            .map(|f| format!(
                r#"{{ "label": "FieldScalar", "name": "{f}", "scalar": "text", "required": true, "default": null }}"#
            ))
            .collect();
        let schema_bytes = format!(
            r#"{{"label":"SchemaRoot","name":"{schema_name}","fields":[{}]}}"#,
            field_decls.join(",")
        );
        let spec = match benten_platform_foundation::compile_schema(schema_bytes.as_bytes()) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let engine = InMemoryMaterializerEngine::new();
        let mut props = BTreeMap::new();
        for f in &unique_fields {
            props.insert(f.clone(), Value::Text(body.clone()));
        }
        let cid = engine.put_node(Node::new(vec![schema_name.clone()], props));
        let alice = materializer_fixtures::actor_principal_alice_cid();

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
        prop_assert_eq!(out1.html_bytes(), out2.html_bytes());
        prop_assert_eq!(out1.json_bytes(), out2.json_bytes());
        prop_assert_eq!(out1.canonical_cid(), out2.canonical_cid());
    }
}
