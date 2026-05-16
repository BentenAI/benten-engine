//! G24-A wave-completion sweep closes phase-4-backlog §4.13 mr-4 MAJOR.
//!
//! The PRIMARY SANDBOX-storage-mutation-host-fn defense lives at
//! `schema_compiler::compile` — schemas that embed `kv:write` /
//! `kv:delete` / `edges:add` / `edges:remove` SANDBOX references are
//! rejected at parse-time with `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`.
//! Pinned by `materializer_rejects_subgraph_with_unregistered_sandbox_host_fn`.
//!
//! THIS pin exercises the DEFENSE-IN-DEPTH arm at `materializer.rs:905-921`
//! — the materializer entry-point also re-checks a hand-authored
//! [`SchemaSubgraphSpec`] (one that bypassed the schema-compile path)
//! for banned SANDBOX host-fns. Before G24-A wave-completion sweep,
//! this arm had no integration test because constructing a
//! `SchemaSubgraphSpec` directly required `pub(crate) fn new` access.
//! G24-A landed `#[doc(hidden)] SchemaSubgraphSpec::for_test_from_handcoded_subgraph`
//! to make this defense-in-depth pin reachable.
//!
//! ## What this pin proves
//!
//! - Materializer entry rejects a hand-authored spec embedding a
//!   `Sandbox` primitive whose `sandbox_host_fn` is in the banned set
//!   (`kv:write`, `kv:delete`, `edges:add`, `edges:remove`).
//! - The rejection surfaces `E_MATERIALIZER_SCHEMA_MISMATCH` (the
//!   materializer-side ErrorCode; the upstream `schema_compiler::compile`
//!   path emits the schema-side `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`).
//! - Hand-authored spec WITHOUT a banned SANDBOX is accepted (positive
//!   control proving the rejection is targeted, not pathological).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_core::{Node, OperationNode, PrimitiveKind, Subgraph, Value};
use benten_errors::ErrorCode;
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerError,
    MaterializerWalkInputs, SchemaSubgraphSpec, allow_all_cap_recheck,
};
use std::collections::BTreeMap;

fn handcoded_spec_with_sandbox_host_fn(host_fn: &str) -> SchemaSubgraphSpec {
    let mut sg = Subgraph::new("defense_in_depth_handcoded_spec");
    sg.nodes.push(
        OperationNode::new("r0", PrimitiveKind::Read)
            .with_property("cap_scope", Value::Text("read:DefenseInDepth.body".into())),
    );
    sg.nodes.push(
        OperationNode::new("s0", PrimitiveKind::Sandbox)
            .with_property("sandbox_host_fn", Value::Text(host_fn.into())),
    );
    sg.edges.push(("r0".into(), "s0".into(), "feeds".into()));
    SchemaSubgraphSpec::for_test_from_handcoded_subgraph("DefenseInDepth", sg)
}

fn handcoded_clean_spec() -> SchemaSubgraphSpec {
    let mut sg = Subgraph::new("defense_in_depth_clean_spec");
    sg.nodes.push(
        OperationNode::new("r0", PrimitiveKind::Read)
            .with_property("cap_scope", Value::Text("read:DefenseInDepth.body".into())),
    );
    SchemaSubgraphSpec::for_test_from_handcoded_subgraph("DefenseInDepth", sg)
}

fn note_cid(engine: &InMemoryMaterializerEngine) -> benten_core::Cid {
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("payload".into()));
    engine.put_node(Node::new(vec!["DefenseInDepth".into()], props))
}

#[test]
fn materializer_rejects_handcoded_spec_referencing_kv_write_host_fn() {
    let engine = InMemoryMaterializerEngine::new();
    let cid = note_cid(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let spec = handcoded_spec_with_sandbox_host_fn("kv:write");
    let err = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect_err("kv:write host-fn MUST trip defense-in-depth materializer entry-check");
    assert_eq!(err.code(), ErrorCode::MaterializerSchemaMismatch);
    match err {
        MaterializerError::SchemaMismatch { reason } => {
            assert!(
                reason.contains("kv:write"),
                "diagnostic must name the banned host-fn: {reason}"
            );
        }
        other => panic!("expected SchemaMismatch, got: {other:?}"),
    }
}

#[test]
fn materializer_rejects_handcoded_spec_referencing_kv_delete_host_fn() {
    let engine = InMemoryMaterializerEngine::new();
    let cid = note_cid(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let spec = handcoded_spec_with_sandbox_host_fn("kv:delete");
    let err = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect_err("kv:delete host-fn MUST trip defense-in-depth materializer entry-check");
    assert!(matches!(err, MaterializerError::SchemaMismatch { .. }));
}

#[test]
fn materializer_rejects_handcoded_spec_referencing_edges_add_host_fn() {
    let engine = InMemoryMaterializerEngine::new();
    let cid = note_cid(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let spec = handcoded_spec_with_sandbox_host_fn("edges:add");
    let err = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect_err("edges:add host-fn MUST trip defense-in-depth materializer entry-check");
    assert!(matches!(err, MaterializerError::SchemaMismatch { .. }));
}

#[test]
fn materializer_rejects_handcoded_spec_referencing_edges_remove_host_fn() {
    // G24-B-FP-1 closure of §4.15: the 4th banned host-fn
    // (`edges:remove`) was named in the module-level banned-set + the
    // production runtime check at `materializer.rs:909`, but had no
    // dedicated sub-test prior. This mirrors the 3-variant shape +
    // proves the rejection arm is symmetric across all 4 banned
    // host-fns.
    let engine = InMemoryMaterializerEngine::new();
    let cid = note_cid(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let spec = handcoded_spec_with_sandbox_host_fn("edges:remove");
    let err = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect_err("edges:remove host-fn MUST trip defense-in-depth materializer entry-check");
    assert_eq!(err.code(), ErrorCode::MaterializerSchemaMismatch);
    match err {
        MaterializerError::SchemaMismatch { reason } => {
            assert!(
                reason.contains("edges:remove"),
                "diagnostic must name the banned host-fn: {reason}"
            );
        }
        other => panic!("expected SchemaMismatch, got: {other:?}"),
    }
}

#[test]
fn materializer_accepts_handcoded_spec_with_no_sandbox_node() {
    // Positive control: the rejection arm is targeted at banned-SANDBOX,
    // not pathologically rejecting all hand-authored specs.
    let engine = InMemoryMaterializerEngine::new();
    let cid = note_cid(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let spec = handcoded_clean_spec();
    let _out = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect("handcoded spec without SANDBOX must walk OK");
}
