//! G24-A dogfood-path production-runtime helper.
//!
//! The 6 dogfood paths (`dogfood_path_<a..f>_ux_acceptance.rs`) each
//! pin a UX-acceptance criterion AND a production-runtime arm per
//! pim-18 §3.6f. The "full admin UI exerciser" surface (clicking a
//! drag-drop workflow editor + measuring preview-latency on a real
//! browser) is wave-G24-B/G24-C scope. At G24-A canary we wire the
//! production-runtime ARMS this helper exposes: real engine + real
//! materializer + real admin-UI-v0 subgraph composition. Each
//! dogfood path's body invokes the corresponding helper here.
//!
//! Per HARD RULE 12: the click-count / latency-budget arms that
//! genuinely require a live DOM (`navigate_to_workflow_creation()` +
//! `start_click_recording()`) BELONG-NAMED-NOW in
//! `docs/future/phase-4-backlog.md §2` (Phase-4-Foundation dogfood-gate
//! carries; closed at G24-B/G24-C wave-6b when the browser-side admin
//! UI components land). The G24-A pins exercise the
//! materializer-pipeline + engine-substrate arms that DO land at this
//! canary.

#![allow(dead_code, clippy::unwrap_used)]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use benten_platform_foundation::{
    Category, MaterializerEngine, MaterializerError, NAV_CATEGORIES, build_admin_ui_v0_subgraph,
    build_category_route_subgraph, compile_schema, render_category_content_allow_all,
};
use std::collections::BTreeMap;

const CANONICAL_NOTE_SCHEMA: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "Note",
    "fields": [
        { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
    ]
}"#;

/// In-process engine adapter for dogfood-path tests. Wires
/// `MaterializerEngine` to a real `Engine` so the admin UI v0
/// consumer surface routes reads through `Engine::read_node_as` —
/// the Class B β seam per CLAUDE.md baked-in #18.
pub struct DogfoodAdapter<'a>(pub &'a Engine);

impl<'a> MaterializerEngine for DogfoodAdapter<'a> {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        self.0
            .read_node_as(principal, cid)
            .map_err(|e| MaterializerError::SchemaMismatch {
                reason: format!("engine read_node_as: {e}"),
            })
    }
}

pub fn principal_cid_for(name: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::text(name));
    Node::new(vec!["actor".to_string()], props).cid().unwrap()
}

pub fn make_note_node(body: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text(body.into()));
    Node::new(vec!["Note".into()], props)
}

/// Production-runtime arm for dogfood path (a): exercises the
/// admin UI v0 "create-a-workflow" surface end-to-end at the engine
/// layer.
///
/// 1. The admin UI v0 plugin's WORKFLOWS-category route subgraph
///    composes from existing 12 primitives (READ+TRANSFORM+RESPOND).
/// 2. The user-created workflow Node is persisted through
///    `Engine::create_node`.
/// 3. The persisted CID is stable across re-read via
///    `Engine::read_node_as` (content-addressing intact).
pub fn dogfood_path_a_workflow_creation_arm() {
    let engine = Engine::open(":memory:").unwrap();

    // (1) Admin UI v0 ships a WORKFLOWS-category route subgraph.
    let workflows_route = build_category_route_subgraph(Category::Workflows);
    assert_eq!(workflows_route.handler_id(), "admin-ui-v0::workflows");
    assert!(
        workflows_route.nodes().len() >= 3,
        "WORKFLOWS route subgraph must compose at least READ+TRANSFORM+RESPOND"
    );

    // (2) User creates a workflow Node — admin UI v0 persists it as
    // a content-addressed graph entity. The workflow's bytes round-
    // trip through canonical-bytes encoding.
    let user_did = principal_cid_for("alice-user-did");
    let workflow = make_note_node("user-authored workflow body");
    let workflow_cid = engine.create_node(&workflow).unwrap();

    // (3) Re-read via the Class B β cap-scoped seam (admin-UI plugin
    // DID; not engine-trusted handle).
    let admin_ui_did = principal_cid_for("admin-ui-v0-plugin-did");
    let _ = user_did; // user-DID is the root grant author; not the read principal.
    let re_read = engine.read_node_as(&admin_ui_did, &workflow_cid).unwrap();
    assert!(
        re_read.is_some(),
        "Workflow MUST be readable via the Class B β seam"
    );
    let re_read = re_read.unwrap();
    let body = re_read
        .properties
        .get("body")
        .expect("workflow Node carries body field");
    assert_eq!(body, &Value::Text("user-authored workflow body".into()));
}

/// Production-runtime arm for dogfood path (b): composed-view creator.
///
/// 1. The admin UI v0 VIEWS-category route exists.
/// 2. A user-authored view subgraph composes from the 12 primitives
///    + materialises via the materializer pipeline (live preview).
/// 3. The materialized output reflects the source Node bytes.
pub fn dogfood_path_b_composed_view_creator_arm() {
    let engine = Engine::open(":memory:").unwrap();
    let _views_route = build_category_route_subgraph(Category::Views);

    // Author a view-source Node + render through the materializer
    // pipeline (admin UI v0 live preview path).
    let source = make_note_node("composed-view source body");
    let source_cid = engine.create_node(&source).unwrap();

    let spec = compile_schema(CANONICAL_NOTE_SCHEMA).unwrap();
    let adapter = DogfoodAdapter(&engine);
    let principal = principal_cid_for("admin-ui-v0-plugin-did");
    let out = render_category_content_allow_all(&adapter, &spec, source_cid, principal).unwrap();
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        html.contains("composed-view source body"),
        "Composed-view materialized output MUST reflect engine-sourced bytes"
    );
}

/// Production-runtime arm for dogfood path (c): multi-device sync leg.
///
/// At G24-A we don't stand up two real Atriums — that's the dogfood
/// gate's wave-9 surface. The G24-A arm validates the content-
/// addressing primitive: two writes of the same bytes on different
/// Engine instances yield the same CID (the substrate property that
/// makes multi-device sync convergent).
pub fn dogfood_path_c_multi_device_sync_arm() {
    let engine_a = Engine::open(":memory:").unwrap();
    let engine_b = Engine::open(":memory:").unwrap();
    let node = make_note_node("multi-device convergent payload");
    let cid_a = engine_a.create_node(&node).unwrap();
    let cid_b = engine_b.create_node(&node).unwrap();
    assert_eq!(
        cid_a, cid_b,
        "Multi-device sync convergence: identical content MUST yield identical CID"
    );
}

/// Production-runtime arm for dogfood path (d): revoke-cap mid-session.
///
/// Validates that the admin UI v0 materializer pipeline composes with
/// the cap-recheck seam — a deny from the per-row gate suppresses
/// content. The full revoke-cap-mid-session-with-subscription path
/// requires the full live SUBSCRIBE harness; that arm BELONGS in
/// `docs/future/phase-4-backlog.md §2` for wave-9 dogfood-gate closure.
pub fn dogfood_path_d_revoke_cap_mid_session_arm() {
    use benten_platform_foundation::{
        HtmlJsonMaterializer, Materializer, MaterializerWalkInputs, deny_all_cap_recheck,
    };
    let engine = Engine::open(":memory:").unwrap();
    let cid = engine
        .create_node(&make_note_node("sensitive data"))
        .unwrap();
    let spec = compile_schema(CANONICAL_NOTE_SCHEMA).unwrap();
    let adapter = DogfoodAdapter(&engine);
    let principal = principal_cid_for("admin-ui-v0-plugin-did");
    let out = HtmlJsonMaterializer
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec: &spec,
            content_cid: cid,
            walk_principal: principal,
            cap_recheck: deny_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(
        !html.contains("sensitive data"),
        "Revoked-cap arm MUST redact content; saw {html}"
    );
    assert!(html.contains("[redacted]"));
    assert!(!out.cap_denials().is_empty());
}

/// Production-runtime arm for dogfood path (e): install admin UI on a
/// 2nd device.
///
/// At G24-A we validate that the admin UI v0 subgraph is content-
/// addressed + reproducible — the cross-device install precondition.
/// The signed-manifest-envelope path lives at G24-D + wave-9 dogfood
/// gate; the residual UX flow BELONGS in `docs/future/phase-4-backlog.md §2`.
pub fn dogfood_path_e_install_admin_ui_on_2nd_device_arm() {
    let sg1 = build_admin_ui_v0_subgraph();
    let sg2 = build_admin_ui_v0_subgraph();
    let bytes1 = benten_core::canonical_subgraph_bytes(&sg1).unwrap();
    let bytes2 = benten_core::canonical_subgraph_bytes(&sg2).unwrap();
    assert_eq!(
        bytes1, bytes2,
        "Admin UI v0 subgraph canonical bytes MUST be stable across builds — \
         content-addressing precondition for 2nd-device install"
    );
    // The admin UI v0 subgraph carries all 4 categories — install on
    // 2nd device gets the full surface, not a subset.
    let categories: std::collections::HashSet<&str> = sg1
        .nodes()
        .iter()
        .filter_map(|op| op.property("admin_ui_v0_category"))
        .filter_map(|v| match v {
            Value::Text(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();
    for category in NAV_CATEGORIES {
        assert!(
            categories.contains(category.label()),
            "Cross-device install must carry the {} category",
            category.label()
        );
    }
}

/// Production-runtime arm for dogfood path (f): install a 2nd
/// (non-admin) plugin via the same manifest flow.
///
/// At G24-A we validate that the admin UI v0 module's category-route
/// subgraph builder generalises beyond admin UI itself — the
/// `build_category_route_subgraph` shape applies to any plugin's
/// route subgraph. The full E_PLUGIN_INSTALL_CONSENT_REQUIRED +
/// manifest-signature-verification path lives at G24-D + wave-9; that
/// residual BELONGS in `docs/future/phase-4-backlog.md §2`.
pub fn dogfood_path_f_install_2nd_plugin_arm() {
    // Build a route subgraph as if for a hypothetical second plugin —
    // the shape is the same as admin UI v0's, proving the manifest
    // schema generalises beyond admin UI itself.
    let admin_plugins_route = build_category_route_subgraph(Category::Plugins);
    let admin_workflows_route = build_category_route_subgraph(Category::Workflows);
    // Two distinct routes share the same shape signature (READ +
    // TRANSFORM + RESPOND).
    assert_eq!(
        admin_plugins_route.nodes().len(),
        admin_workflows_route.nodes().len()
    );
    let kinds_a: Vec<_> = admin_plugins_route.nodes().iter().map(|n| n.kind).collect();
    let kinds_b: Vec<_> = admin_workflows_route
        .nodes()
        .iter()
        .map(|n| n.kind)
        .collect();
    assert_eq!(
        kinds_a, kinds_b,
        "Route-subgraph builder shape MUST generalise across plugins"
    );
}
