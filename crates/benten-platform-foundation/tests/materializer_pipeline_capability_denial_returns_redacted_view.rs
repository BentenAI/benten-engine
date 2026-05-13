//! G23-B GREEN: cap-denial returns redacted view at Node-granularity
//! (LOAD-BEARING substantive). Closes r2-test-landscape §2.5 row 3 +
//! ratification #7.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, Value};
use benten_errors::ErrorCode;
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerWalkInputs,
    deny_all_cap_recheck,
};
use std::collections::BTreeMap;

#[test]
fn materializer_pipeline_capability_denial_returns_redacted_view() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let unauth = materializer_fixtures::actor_principal_unauthorized_cid();

    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("secret body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));

    let mat = HtmlJsonMaterializer;
    let out = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: unauth,
            // Gate denies all.
            cap_recheck: deny_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect("materializer returns Ok(redacted) NOT Err for cap-deny per ratification #7");

    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        !html.contains("secret body"),
        "redacted view MUST NOT leak field content under cap-deny: html={html}"
    );
    assert!(
        html.contains("[redacted]"),
        "redacted view MUST surface placeholder so UI can render an explanation"
    );

    // The materializer surfaces the cap-deny as a structured frame, but
    // returns success (not Err) per ratification #7.
    assert_eq!(
        out.cap_denials().len(),
        1,
        "exactly one Node-level cap-denial (the Note body)"
    );
    assert_eq!(
        out.cap_denials()[0].code(),
        ErrorCode::MaterializerCapDenied,
        "denial frame carries E_MATERIALIZER_CAP_DENIED (G23-B NEW)"
    );

    // JSON projection — `redacted: true` flag + scope array.
    let json = std::str::from_utf8(out.json_bytes()).unwrap();
    assert!(
        json.contains("\"redacted\":true"),
        "JSON projection surfaces redaction sentinel: json={json}"
    );
}
