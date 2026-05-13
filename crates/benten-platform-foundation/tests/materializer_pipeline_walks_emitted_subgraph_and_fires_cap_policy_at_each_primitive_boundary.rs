//! G23-B GREEN: materializer walk fires cap-policy at each primitive
//! boundary (§3.6b pim-2 end-to-end; would-FAIL-if-no-op'd).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Cid, Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerCapRecheck,
    MaterializerWalkInputs,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn materializer_pipeline_walks_emitted_subgraph_and_fires_cap_policy_at_each_primitive_boundary() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let primitive_count = spec.as_subgraph().primitive_count();
    assert!(primitive_count > 0, "schema emits at least one primitive");

    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Recording cap_recheck — counts every invocation. The materializer
    // walk fires this closure once per emitted primitive boundary +
    // once for the final content-CID read disposition.
    let count = Arc::new(AtomicUsize::new(0));
    let recorder: MaterializerCapRecheck = {
        let count = Arc::clone(&count);
        Arc::new(move |_p: &Cid, _z: &str, _c: &Cid| -> bool {
            count.fetch_add(1, Ordering::SeqCst);
            true
        })
    };

    let mat = HtmlJsonMaterializer;
    let _out = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: recorder,
            declared_requires: Vec::new(),
        })
        .unwrap();

    // OBSERVABLE CONSEQUENCE: every primitive boundary fired the
    // cap_recheck closure. WOULD-FAIL-IF-NO-OP: a dispatch that skipped
    // cap-checks would leave the counter at 0 (or 1, the per-row
    // disposition only).
    let observed = count.load(Ordering::SeqCst);
    assert!(
        observed >= primitive_count,
        "materializer walk MUST fire cap-policy at each primitive boundary; \
         primitive_count={primitive_count}, observed={observed}"
    );
}
