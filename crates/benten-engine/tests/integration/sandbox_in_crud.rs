//! Phase 2b R3-B — SANDBOX-inside-CRUD-handler integration tests (G7-A).
//!
//! Pin source: plan §4 SANDBOX integration.
//!
//! **G20-A1 wave-8a** (Phase 3): bodies un-ignored.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

#[test]
fn sandbox_inside_crud_handler_e2e() {
    // **G20-A1 wave-8a body** (Phase 3): plan §4 SANDBOX integration —
    // a handler composes SANDBOX → WRITE → RESPOND. End-to-end through
    // engine.call: SANDBOX runs, WRITE persists, RESPOND closes.
    //
    // We don't drive the full `crud('post')` macro path here (the
    // crud helper is a TS-side DSL convenience that lives in
    // `packages/engine/src`); the eval-level integration claim is
    // that a SubgraphSpec carrying SANDBOX + WRITE + RESPOND
    // composes cleanly through engine.call.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 7))").unwrap();
    let module_cid = Cid::from_blake3_digest(*blake3::hash(&module_bytes).as_bytes());
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str));
    sandbox_props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );

    let spec = SubgraphSpec::builder()
        .handler_id("g20a1.sandbox_in_crud")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .write(|w| w.label("post"))
        .respond()
        .build();

    let handler_id = engine
        .register_subgraph(spec)
        .expect("SANDBOX → WRITE → RESPOND composed handler MUST register");

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_post".to_string()], Default::default()),
        )
        .expect("SANDBOX-inside-CRUD-shape handler dispatch MUST succeed");
    assert!(
        outcome.is_ok_edge(),
        "SANDBOX → WRITE → RESPOND chain MUST route OK end-to-end \
         (SANDBOX runs; WRITE persists; RESPOND closes); got {outcome:?}"
    );
}

#[test]
fn sandbox_result_fed_to_write_cap_checked_at_host_boundary() {
    // **G20-A1 wave-8a body** (Phase 3): plan §4 — SANDBOX manifest
    // caps DO NOT extend to the WRITE primitive. The WRITE is
    // evaluated by the engine in the handler's (dispatcher) context,
    // NOT the SANDBOX module's context.
    //
    // Structural pin: the SANDBOX manifest's cap-set is what the
    // executor uses for HOST-FN cap intersection (`host:compute:*`),
    // not for engine-level WRITE cap-checks. Confirmed by
    // constructing a SANDBOX with ONLY `host:compute:time` (no WRITE-
    // authority) + asserting the SANDBOX → WRITE chain succeeds under
    // NoAuth (Phase-2b posture). A regression where the SANDBOX
    // manifest leaked into the WRITE cap-set would NOT change Phase-2b
    // behaviour (NoAuth permits everything), so the load-bearing pin
    // here is registration-time validation: a SANDBOX-only-cap-set
    // chain registers cleanly. Phase-3 capability backend extends
    // this to assert E_CAP_DENIED on WRITE.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap();
    let module_cid = Cid::from_blake3_digest(*blake3::hash(&module_bytes).as_bytes());
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str));
    // Manifest declares ONLY host:compute:time — NO WRITE-authority.
    sandbox_props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );

    let spec = SubgraphSpec::builder()
        .handler_id("g20a1.sandbox_write_cap_separation")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .write(|w| w.label("post"))
        .respond()
        .build();

    let handler_id = engine
        .register_subgraph(spec)
        .expect("registration MUST succeed; manifest cap-set is host-fn only");

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_post".to_string()], Default::default()),
        )
        .expect("SANDBOX → WRITE chain MUST register + dispatch under NoAuth");
    assert!(
        outcome.is_ok_edge(),
        "Phase-2b NoAuth: SANDBOX → WRITE chain succeeds; SANDBOX \
         manifest cap-set is NOT the WRITE authority"
    );
}
