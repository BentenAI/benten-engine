//! G23-B GREEN: T1 negative pin — materializer rejects subgraph with
//! cap-scope mismatch (declared envelope < runtime composition).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T1 defense + arch-r1-3 `E_MATERIALIZER_SCHEMA_MISMATCH` mint.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, Value};
use benten_errors::ErrorCode;
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerError,
    MaterializerWalkInputs, allow_all_cap_recheck,
};
use std::collections::BTreeMap;

#[test]
fn materializer_rejects_subgraph_whose_runtime_composition_exceeds_declared_cap_scope() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    // The Note schema emits cap-scope annotations like `read:Note.body`,
    // `read:Note.created_at`, etc. Declaring an envelope of ONLY
    // `read:Note` (no field-suffix) is narrower than the runtime
    // composition → T1 envelope-violation rejection at materializer entry.
    let declared_requires = vec!["read:Note".to_string()];

    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));
    let alice = materializer_fixtures::actor_principal_alice_cid();

    let mat = HtmlJsonMaterializer;
    let err = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires,
        })
        .expect_err(
            "T1 negative: subgraph whose runtime composition exceeds declared cap-scope \
             MUST be REJECTED at materializer entry",
        );
    assert!(
        matches!(&err, MaterializerError::SchemaMismatch { .. })
            && err.code() == ErrorCode::MaterializerSchemaMismatch,
        "T1 negative: must surface typed E_MATERIALIZER_SCHEMA_MISMATCH per \
         arch-r1-3 mint; got {err:?}"
    );

    // Defense-in-depth: the rejection happened BEFORE any READ for the
    // out-of-envelope scope was issued — InMemoryMaterializerEngine's
    // `read_node_as` is never called because the entry-validate-then-
    // dispatch shape fails-fast pre-fanout. We can't observe a
    // read-counter at this layer (the InMemory engine doesn't expose
    // one), but the SHAPE assertion above pins the disposition.
}
