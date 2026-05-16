//! Phase-4-Foundation G24-A — admin UI v0 shell renders content via
//! the `Materializer` trait (G23-B consumer).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 3 (substantive); G23-B materializer pipeline consumer.
//!
//! Per Phase-4-Foundation G23-B materializer canary: content render
//! flows through a `Materializer` trait impl, not bespoke handcoded
//! "renderProperty(node)" logic. Admin UI's Content Types + Views
//! routes consume the materializer to project subgraph data to UI
//! shape via [`benten_platform_foundation::render_category_content_allow_all`].
//!
//! ## Substantive shape
//!
//! 1. Put a content Node via `Engine::create_node`.
//! 2. Render via `render_category_content_allow_all`, which delegates
//!    to `HtmlJsonMaterializer::materialize_with_gate` (the
//!    `Materializer` trait method).
//! 3. Assert the rendered output bytes reflect the engine-sourced
//!    Node — not a static mock.

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use benten_platform_foundation::{
    MaterializerEngine, MaterializerError, compile_schema, render_category_content_allow_all,
};
use std::collections::BTreeMap;

const CANONICAL_NOTE_SCHEMA: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "Note",
    "fields": [
        { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
    ]
}"#;

/// Thin adapter binding `MaterializerEngine` to a real `Engine` —
/// mirrors the production napi-bridge adapter shape.
struct EngineAdapter<'a>(&'a Engine);

impl<'a> MaterializerEngine for EngineAdapter<'a> {
    fn read_node_as(
        &self,
        principal: &benten_core::Cid,
        cid: &benten_core::Cid,
    ) -> Result<Option<Node>, MaterializerError> {
        self.0
            .read_node_as(principal, cid)
            .map_err(|e| MaterializerError::SchemaMismatch {
                reason: format!("engine read_node_as: {e}"),
            })
    }
}

fn principal_cid_for(name: &str) -> benten_core::Cid {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::text(name));
    Node::new(vec!["actor".to_string()], props).cid().unwrap()
}

#[test]
fn admin_ui_v0_shell_consumes_materializer_for_content_render() {
    let engine = Engine::open(":memory:").unwrap();

    // Put a content Node — the "distinguishing token" the rendered
    // output must echo back to prove materializer delegation:
    let mut props = BTreeMap::new();
    props.insert(
        "body".into(),
        Value::Text("admin-ui-distinguishing-token-G24A".into()),
    );
    let cid = engine
        .create_node(&Node::new(vec!["Note".into()], props))
        .unwrap();

    let spec = compile_schema(CANONICAL_NOTE_SCHEMA).unwrap();
    let adapter = EngineAdapter(&engine);
    let alice = principal_cid_for("alice");

    let out = render_category_content_allow_all(&adapter, &spec, cid, alice)
        .expect("render_category_content_allow_all must succeed");

    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        html.contains("admin-ui-distinguishing-token-G24A"),
        "Materializer output MUST flow to admin-UI consumer surface; saw {html}"
    );
    assert!(
        html.contains("benten-note"),
        "Materializer MUST tag with schema name class; saw {html}"
    );
    // The JSON projection side carries the canonical scope array.
    let json = std::str::from_utf8(out.json_bytes()).unwrap();
    assert!(
        json.contains("\"scope\""),
        "Materializer JSON projection MUST carry scope array; saw {json}"
    );
}
