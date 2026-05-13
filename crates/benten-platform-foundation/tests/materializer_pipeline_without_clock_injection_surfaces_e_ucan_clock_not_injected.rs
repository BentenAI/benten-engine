//! G23-B GREEN: materializer wallclock fail-closed inheritance per
//! sec-3.5-r1-7 + threat-model T11 (LOAD-BEARING substantive).
//!
//! Negative arm: materializer constructed against an engine adapter
//! with NO clock injected MUST fail-closed with
//! `E_UCAN_CLOCK_NOT_INJECTED` (NOT silently default to `now()`) per
//! Compromise #24 + Phase-3 G16-B-B closure floor.
//!
//! SUBSTANCE companion arm: re-running the SAME walk against an engine
//! WITH clock injected MUST succeed — proves the failure is
//! clock-driven, not unrelated.

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

fn make_note() -> Node {
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    Node::new(vec!["Note".into()], props)
}

#[test]
fn materializer_pipeline_without_clock_injection_surfaces_e_ucan_clock_not_injected() {
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Negative arm: no-clock engine — fail-closed.
    let engine_no_clock = InMemoryMaterializerEngine::without_clock();
    let cid_nc = engine_no_clock.put_node(make_note());
    let mat = HtmlJsonMaterializer;
    let err = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine_no_clock,
            spec: &spec,
            content_cid: cid_nc,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect_err("materializer MUST fail-closed with no clock injected per sec-3.5-r1-7");
    assert!(
        matches!(err, MaterializerError::UcanClockNotInjected),
        "expected MaterializerError::UcanClockNotInjected, got {err:?} — fail-closed floor breached"
    );
    assert_eq!(err.code(), ErrorCode::UcanClockNotInjected);

    // SUBSTANCE arm: same walk against a clock-injected engine
    // succeeds. Proves the failure was clock-driven, not unrelated.
    let engine_ok = InMemoryMaterializerEngine::new();
    let cid_ok = engine_ok.put_node(make_note());
    let _ok = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine_ok,
            spec: &spec,
            content_cid: cid_ok,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect("with clock injected, walk succeeds");
}
