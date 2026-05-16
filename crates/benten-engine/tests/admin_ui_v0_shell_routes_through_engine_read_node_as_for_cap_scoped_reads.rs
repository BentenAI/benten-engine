//! Phase-4-Foundation G24-A — admin UI v0 routing through
//! `Engine::read_node_as` for cap-scoped reads.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 2 (substantive runtime trace); closes CLAUDE.md baked-in #18
//! (Class B β `read_node_as` consumer; admin UI v0 is the first
//! non-trusted-principal consumer of the seam shipped at PR #184).
//!
//! ## Substantive shape (the runtime-trace half of pim-18 §3.6f)
//!
//! 1. Put a content Node via `Engine::create_node`.
//! 2. Render via the admin UI v0 consumer surface threaded through
//!    a [`MaterializerEngine`] adapter — the adapter is a runtime
//!    *substitute* for the engine seam that the admin UI consumes.
//! 3. The adapter tracks invocation count of its `read_node_as`
//!    method; the assertion is that the consumer actually calls
//!    through it (not bypasses).
//! 4. The adapter records the principal CID passed in — proves the
//!    admin-UI-DID (NOT a trusted-handle stand-in) was threaded as
//!    walk-principal.
//!
//! Companion grep-assert at
//! `admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`.

#![allow(clippy::unwrap_used)]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use benten_platform_foundation::{
    MaterializerEngine, MaterializerError, compile_schema, render_category_content_allow_all,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

const CANONICAL_NOTE_SCHEMA: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "Note",
    "fields": [
        { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
    ]
}"#;

/// Adapter that records every `read_node_as` invocation — proves the
/// admin UI v0 consumer surface routes through the cap-scoped seam.
struct TracingAdapter<'a> {
    engine: &'a Engine,
    /// Each entry: (principal_cid, target_cid).
    invocations: Mutex<Vec<(Cid, Cid)>>,
}

impl<'a> TracingAdapter<'a> {
    fn new(engine: &'a Engine) -> Self {
        Self {
            engine,
            invocations: Mutex::new(Vec::new()),
        }
    }
}

impl<'a> MaterializerEngine for TracingAdapter<'a> {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        self.invocations.lock().unwrap().push((*principal, *cid));
        self.engine
            .read_node_as(principal, cid)
            .map_err(|e| MaterializerError::SchemaMismatch {
                reason: format!("engine read_node_as: {e}"),
            })
    }
}

fn principal_cid_for(name: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::text(name));
    Node::new(vec!["actor".to_string()], props).cid().unwrap()
}

#[test]
fn admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads() {
    let engine = Engine::open(":memory:").unwrap();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("traced body".into()));
    let content_cid = engine
        .create_node(&Node::new(vec!["Note".into()], props))
        .unwrap();

    let admin_ui_did = principal_cid_for("admin-ui-v0-plugin-did");
    let adapter = TracingAdapter::new(&engine);
    let spec = compile_schema(CANONICAL_NOTE_SCHEMA).unwrap();

    let _out = render_category_content_allow_all(&adapter, &spec, content_cid, admin_ui_did)
        .expect("render must succeed");

    let invocations = adapter.invocations.lock().unwrap();
    assert!(
        !invocations.is_empty(),
        "Admin UI v0 consumer MUST invoke Engine::read_node_as via adapter — \
         class B β seam discipline per CLAUDE.md #18; trace shows ZERO invocations"
    );
    // Every invocation threads admin_ui_did as the principal — proves
    // the consumer is honouring the Class B β walk-principal contract
    // (NOT silently substituting a trusted-handle / None).
    for (p, _c) in invocations.iter() {
        assert_eq!(
            p, &admin_ui_did,
            "Every read MUST thread the admin-UI plugin DID as principal — \
             saw {p}, expected {admin_ui_did}"
        );
    }
}
