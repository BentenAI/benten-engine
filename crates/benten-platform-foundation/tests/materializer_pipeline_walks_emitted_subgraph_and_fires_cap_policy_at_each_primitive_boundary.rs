//! G23-B GREEN: the materializer walk's per-row cap gate is the
//! authoritative materialization-layer admit/deny boundary — invoked
//! for the content-CID decision and CONSUMED (Safe-1 #527 / Qual-1
//! #702 closure — Pattern F Bundle 5; §3.6b pim-2 end-to-end;
//! would-FAIL-if-no-op'd).
//!
//! The prior contract here ("fires cap-policy at EACH primitive
//! boundary") pinned the discarded-bool per-primitive fan-out that
//! #702/#527 identified as observability-theater: it invoked the gate
//! N times and `let _`-discarded every result, giving no production
//! enforcement (per-primitive cap-scope is enforced upstream by the T1
//! envelope check + schema-compile `derive_scope`) and no production
//! observability. That loop is removed; this pin now asserts the
//! substantive contract.

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

    // Recording cap_recheck — counts every invocation. After the
    // theater-loop removal the materializer fires this closure exactly
    // once: the authoritative content-CID per-row gate decision.
    let count = Arc::new(AtomicUsize::new(0));
    let recorder: MaterializerCapRecheck = {
        let count = Arc::clone(&count);
        Arc::new(move |_p: &Cid, _z: &str, _c: &Cid| -> bool {
            count.fetch_add(1, Ordering::SeqCst);
            true
        })
    };

    let mat = HtmlJsonMaterializer;
    let out = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: recorder,
            declared_requires: Vec::new(),
        })
        .unwrap();

    // OBSERVABLE CONSEQUENCE: the per-row gate is invoked exactly once
    // for the authoritative content-CID decision (NOT N-times in a
    // discarded-bool fan-out). WOULD-FAIL-IF-NO-OP: a dispatch that
    // skipped the cap-check entirely would leave the counter at 0; a
    // regression re-introducing the discarded-bool per-primitive
    // fan-out would push it back to >= primitive_count.
    let observed = count.load(Ordering::SeqCst);
    assert_eq!(
        observed, 1,
        "the per-row gate fires exactly once (authoritative content-CID \
         decision); the discarded-bool per-primitive fan-out is removed \
         per #527/#702. primitive_count={primitive_count}, observed={observed}"
    );

    // The admitting bool was CONSUMED end-to-end: exactly one
    // materialized row + zero denial frames.
    assert_eq!(out.materialized_row_cids().len(), 1);
    assert!(out.cap_denials().is_empty());
}
