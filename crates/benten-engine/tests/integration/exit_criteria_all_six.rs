//! Phase 1 R3 integration — the six headline exit-criterion assertions,
//! Rust-side partner to the Vitest file `my-app/test/smoke.test.ts` produced
//! by `npx create-benten-app`.
//!
//! Each sub-test maps 1:1 to a numbered Vitest assertion in
//! `.addl/phase-1/00-implementation-plan.md` §1. If any of these fail,
//! Phase 1 is not done.
//!
//! **Status:** FAILING until G6-C (register_subgraph, mermaid, trace) +
//! G7 + G8 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// **Exit-criterion #1.** Registration succeeds.
#[test]
fn exit_1_registration_succeeds() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine
        .register_crud("post")
        .expect("registration succeeds; invariants 1/2/3/5/6/9/10/12 pass");
    assert!(!handler_id.is_empty(), "handler id must be non-empty");

    // No E_INV_* thrown: absence is verified by the register_crud Ok(..) branch above,
    // but also confirm registration is idempotent (re-registering returns the same id).
    let again = engine.register_crud("post").unwrap();
    assert_eq!(
        handler_id, again,
        "re-register returns same content-addressed id"
    );
}

/// **Exit-criterion #2.** Three creates + list reflects all three in createdAt order.
#[test]
fn exit_2_three_creates_list_returns_them() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine.register_crud("post").unwrap();

    for title in ["first", "second", "third"] {
        let mut p = BTreeMap::new();
        p.insert("title".into(), Value::Text(title.into()));
        engine
            .call(
                &handler_id,
                "post:create",
                Node::new(vec!["post".into()], p),
            )
            .unwrap();
    }

    let listed = engine
        .call(&handler_id, "post:list", Node::empty())
        .unwrap();
    let items = listed.as_list().expect("post:list is a List");
    assert_eq!(items.len(), 3, "exactly three posts");

    // createdAt order (deterministic HLC stamped at create time, monotonic).
    let titles: Vec<_> = items
        .iter()
        .map(|n| match n.properties.get("title") {
            Some(Value::Text(s)) => s.clone(),
            _ => panic!("title must be Text"),
        })
        .collect();
    assert_eq!(
        titles,
        vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string()
        ],
        "createdAt order preserved"
    );
}

/// **Exit-criterion #3.** Capability denial routes to ON_DENIED.
#[test]
fn exit_3_cap_denial_routes_on_denied() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine
        .register_crud_with_grants("post")
        .expect("grant-backed crud registers");

    let actor = engine.create_principal("alice").unwrap();
    engine.grant_capability(&actor, "store:post:write").unwrap();

    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("first".into()));
    let ok = engine
        .call(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], p.clone()),
        )
        .unwrap();
    assert!(ok.is_ok_edge());

    engine
        .revoke_capability(&actor, "store:post:write")
        .unwrap();
    let denied = engine
        .call(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], p),
        )
        .unwrap();
    assert!(denied.routed_through_edge("ON_DENIED"));
    assert_eq!(denied.error_code(), Some("E_CAP_DENIED"));
}

/// **Exit-criterion #4.** Trace has non-zero per-step timing and topological order.
/// IMPORTANT: topological order, not strict sequence — BRANCH/ITERATE/CALL admit
/// multiple valid traversal orders.
#[test]
fn exit_4_trace_non_zero_timing_and_topological_order() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine.register_crud("post").unwrap();

    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("traced".into()));
    let trace = engine
        .trace(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], p),
        )
        .expect("trace path exercises the evaluator without shortcut-execution");

    assert!(
        !trace.steps().is_empty(),
        "trace must produce at least one step"
    );
    for step in trace.steps() {
        assert!(
            step.duration_us() > 0,
            "every trace step must have non-zero timing; got {:?}",
            step
        );
    }

    // Topological order assertion: every step must appear only after all its
    // subgraph predecessors have appeared.
    let adj = engine
        .handler_predecessors(&handler_id)
        .expect("handler structure available for test");
    let mut seen = std::collections::HashSet::new();
    for step in trace.steps() {
        for pred in adj.predecessors_of(step.node_cid()) {
            assert!(
                seen.contains(pred),
                "topo-order violated: {} observed before predecessor {}",
                step.node_cid(),
                pred
            );
        }
        seen.insert(step.node_cid().clone());
    }
}

/// **Exit-criterion #5.** Mermaid output parses.
/// Rust-side partner to the TS `@mermaid-js/parser` check. We only assert the
/// string starts with `flowchart ` and contains one edge `-->`; the authoritative
/// Mermaid validation lives in the TS Vitest suite.
#[test]
fn exit_5_mermaid_output_parses_minimal_shape() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let mermaid = engine
        .handler_to_mermaid(&handler_id)
        .expect("toMermaid is pure");
    assert!(
        mermaid.starts_with("flowchart "),
        "must start with `flowchart <direction>`"
    );
    assert!(mermaid.contains("-->"), "must contain at least one edge");
    assert!(
        mermaid.contains("READ") || mermaid.contains("WRITE") || mermaid.contains("RESPOND"),
        "must reference at least one primitive label"
    );
}

/// **Exit-criterion #6.** TS <-> Rust CID round-trip. Rust-side partner; the
/// authoritative check lives in `bindings/napi/index.test.ts`. Here we simply
/// ensure the canonical fixture node hashes to the same CID on this host.
#[test]
fn exit_6_canonical_cid_matches_spike_fixture() {
    let expected = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";
    let actual = benten_core::testing::canonical_test_node()
        .cid()
        .unwrap()
        .to_base32();
    assert_eq!(
        actual, expected,
        "canonical CID must remain stable post-fork"
    );

    let (_dir, engine) = fresh_engine();
    let stored_cid = engine
        .create_node(&benten_core::testing::canonical_test_node())
        .unwrap();
    assert_eq!(
        stored_cid.to_base32(),
        expected,
        "TS<->Rust CID round-trip baseline holds"
    );
}
