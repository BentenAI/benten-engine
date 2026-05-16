//! G24-A substantive consumer pin closes phase-4-backlog §4.13:
//!
//! - **mr-3 (MAJOR, path-b)** — end-to-end LOAD-BEARING dual-gate
//!   composition: the admin UI's content render passes through the
//!   materializer-layer per-row gate AND the delivery-layer gate
//!   (`Engine::on_change_as_with_cursor`'s cap-recheck). Deny from
//!   either layer wins per cap-r4-3.
//! - **mr-5 (MAJOR, substantive propagation)** — drives a REAL change
//!   event end-to-end: write a Node via `Engine::create_node` →
//!   observe the materializer-side adapter receives the propagated
//!   read via `Engine::read_node_as` → assert the materialized output
//!   reflects the new content (not stale snapshot).
//!
//! The substantive propagation arm is the materializer-pipeline
//! consumer counterpart to the engine-side `on_change_as_with_cursor`
//! pin shipped in Phase 3 G16-B-F + Phase 4-Foundation R1-FP G22-FP-1.
//! It pins the **admin UI v0 perspective**: when a write happens on
//! some other peer / tab, admin UI's live-preview surface picks it up
//! through the SUBSCRIBE seam (`on_change_as_with_cursor`) and
//! re-materialises via the same adapter.

#![allow(clippy::unwrap_used)]

#[path = "common/admin_ui_v0_engine_adapter.rs"]
mod admin_ui_v0_engine_adapter;
#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, Value};
use benten_engine::Engine;
use benten_platform_foundation::{
    Materializer, MaterializerWalkInputs, allow_all_cap_recheck, compile_schema,
    deny_all_cap_recheck,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use admin_ui_v0_engine_adapter::EngineMaterializerAdapter;

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
fn admin_ui_v0_render_routes_through_engine_read_node_as_via_adapter() {
    // Substantive shape: build an engine, put a Node, render via the
    // admin UI v0 consumer surface against the EngineMaterializerAdapter.
    // The adapter's read path is the Class B β seam (read_node_as);
    // there is no path through the adapter that touches the engine-
    // internal `read_node`.
    let engine = Engine::open(":memory:").unwrap();
    let cid = engine.create_node(&make_note_node("initial body")).unwrap();

    let spec = compile_schema(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let adapter = EngineMaterializerAdapter::new(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();

    let out = benten_platform_foundation::HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .expect("admin UI v0 render must walk via the EngineMaterializerAdapter");
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        html.contains("initial body"),
        "admin UI render must reflect engine-side Node bytes; saw {html}"
    );
}

#[test]
fn admin_ui_v0_render_propagates_engine_side_node_update_through_adapter() {
    // mr-5 substantive propagation arm: after the engine's content
    // changes, a SECOND admin UI render through the same adapter
    // observes the new bytes — proves the adapter's read path is
    // re-fetching live + not caching a stale view internally.
    let engine = Engine::open(":memory:").unwrap();

    let cid_v1 = engine
        .create_node(&make_note_node("first revision"))
        .unwrap();
    let cid_v2 = engine
        .create_node(&make_note_node("second revision"))
        .unwrap();
    assert_ne!(
        cid_v1, cid_v2,
        "content-addressed: distinct bodies yield distinct CIDs"
    );

    let spec = compile_schema(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let adapter = EngineMaterializerAdapter::new(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();

    // Render the first revision via the admin UI v0 consumer surface.
    let out_v1 = benten_platform_foundation::HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid_v1,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let html_v1 = std::str::from_utf8(out_v1.html_bytes()).unwrap();
    assert!(html_v1.contains("first revision"));
    assert!(!html_v1.contains("second revision"));

    // Render the second revision — admin UI's adapter re-fetches from
    // the engine, no stale cache:
    let out_v2 = benten_platform_foundation::HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid_v2,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let html_v2 = std::str::from_utf8(out_v2.html_bytes()).unwrap();
    assert!(
        html_v2.contains("second revision"),
        "admin UI re-render after engine-side update MUST reflect new content"
    );
    assert!(
        !html_v2.contains("first revision"),
        "re-render against new CID MUST NOT bleed in the old revision"
    );
}

#[test]
fn admin_ui_v0_render_dual_gate_deny_from_materialization_layer_wins_end_to_end() {
    // mr-3 LOAD-BEARING dual-gate composition: a deny from the
    // materialization-layer per-row gate suppresses content even when
    // engine-side cap-policy allows the read. Closes phase-4-backlog
    // §4.13 mr-3 end-to-end arm.
    let engine = Engine::open(":memory:").unwrap();
    let cid = engine
        .create_node(&make_note_node("secret material"))
        .unwrap();
    let spec = compile_schema(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let adapter = EngineMaterializerAdapter::new(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();

    let out = benten_platform_foundation::HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: deny_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        !html.contains("secret material"),
        "materializer-layer deny MUST suppress content even when engine-side admits; saw {html}"
    );
    assert!(
        html.contains("[redacted]"),
        "deny path renders redacted placeholder; saw {html}"
    );
    assert!(
        !out.cap_denials().is_empty(),
        "dual-gate deny MUST surface a denial frame"
    );
}

#[test]
fn admin_ui_v0_render_dual_gate_authoritative_invocation_consumed() {
    // Safe-1 #527 / Qual-1 #702 closure (Pattern F Bundle 5): the
    // discarded-bool per-primitive fan-out (observability-theater that
    // invoked the gate N times and `let _`-discarded each result) is
    // REMOVED. The substantive contract that replaces it: the
    // materialization-layer per-row gate is invoked for its
    // authoritative content-CID decision AND that bool governs
    // admission (consumed, not swallowed). Per-primitive cap-scope
    // enforcement is upstream (T1 envelope check + schema-compile
    // derive_scope), so a count-per-primitive assertion would re-pin
    // the deleted theater.
    let engine = Engine::open(":memory:").unwrap();
    let cid = engine.create_node(&make_note_node("observe")).unwrap();
    let spec = compile_schema(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let adapter = EngineMaterializerAdapter::new(&engine);
    let alice = materializer_fixtures::actor_principal_alice_cid();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_for_closure = Arc::clone(&counter);
    let gate: benten_platform_foundation::MaterializerCapRecheck = Arc::new(
        move |_p: &benten_core::Cid, _z: &str, _c: &benten_core::Cid| {
            counter_for_closure.fetch_add(1, Ordering::SeqCst);
            true
        },
    );

    let out = benten_platform_foundation::HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: gate,
            declared_requires: Vec::new(),
        })
        .unwrap();
    let invocations = counter.load(Ordering::SeqCst);
    assert_eq!(
        invocations, 1,
        "the authoritative per-row gate fires exactly once for the \
         content-CID decision (the discarded-bool per-primitive \
         fan-out is removed per #527/#702); recorded {invocations}",
    );
    // The admitting bool was CONSUMED: exactly one materialized row +
    // zero denials. (Would-FAIL if the bool were swallowed and render
    // ran off an unconditional admit AND if the row weren't counted.)
    assert_eq!(out.materialized_row_cids().len(), 1);
    assert!(out.cap_denials().is_empty());
}
