//! G23-B GREEN: dual-gate end-to-end LOAD-BEARING pim-2 §3.6b pin —
//! would-FAIL-if-no-op'd.
//!
//! Closes r2-test-landscape §2.5 row 7 + sec-3.5-r1-1 composition pin 4
//! of 4. Drives the PRODUCTION
//! `Materializer::materialize_with_gate` entry point against an engine
//! containing 2 rows (one admitted, one denied) + a `MaterializerCapRecheck`
//! that admits some CIDs and denies others.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Cid, Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerCapRecheck,
    MaterializerWalkInputs, allow_all_cap_recheck,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn make_note_node(body: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text(body.into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    Node::new(vec!["Note".into()], props)
}

#[test]
fn materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Put 2 Notes; admit only the first via the gate.
    let engine = InMemoryMaterializerEngine::new();
    let admitted_cid = engine.put_node(make_note_node("admitted body"));
    let denied_cid = engine.put_node(make_note_node("denied body"));

    let admitted_set: BTreeSet<Cid> = std::iter::once(admitted_cid).collect();
    let admitted_arc = Arc::new(admitted_set);
    let cap_recheck: MaterializerCapRecheck = {
        let set = Arc::clone(&admitted_arc);
        Arc::new(move |_p, _zone, cid| set.contains(cid))
    };

    let mat = HtmlJsonMaterializer;

    // Walk against the admitted CID — succeeds, content present.
    let out_admitted = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: admitted_cid,
            walk_principal: alice,
            cap_recheck: Arc::clone(&cap_recheck),
            declared_requires: Vec::new(),
        })
        .unwrap();
    assert_eq!(
        out_admitted.materialized_row_cids().len(),
        1,
        "admitted walk yields 1 row (the admitted CID)"
    );
    assert_eq!(out_admitted.materialized_row_cids()[0], admitted_cid);
    let html_admitted = std::str::from_utf8(out_admitted.html_bytes()).unwrap();
    assert!(html_admitted.contains("admitted body"));

    // Walk against the denied CID — gate denies, redacted output.
    let out_denied = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: denied_cid,
            walk_principal: alice,
            cap_recheck: Arc::clone(&cap_recheck),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let html_denied = std::str::from_utf8(out_denied.html_bytes()).unwrap();
    assert!(
        !html_denied.contains("denied body"),
        "denied content MUST NOT leak"
    );
    assert!(html_denied.contains("[redacted]"));
    assert_eq!(out_denied.cap_denials().len(), 1);
    assert_eq!(
        out_denied.materialized_row_cids().len(),
        0,
        "denied walk yields 0 admitted rows (NOT 1 = gate-bypass; NOT 2 = arm-no-op)"
    );

    // Smoke-check: allow-all gate sees BOTH rows when iterated.
    let out_admitted_allow = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: admitted_cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let out_denied_allow = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: denied_cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    assert_eq!(
        out_admitted_allow.materialized_row_cids().len(),
        1,
        "allow-all on admitted CID yields 1 row"
    );
    assert_eq!(
        out_denied_allow.materialized_row_cids().len(),
        1,
        "allow-all on denied CID yields 1 row — proves the 0 above is gate-driven, NOT empty-view"
    );
}
