//! G23-B GREEN: materializer source uses `read_node_as` ONLY (never
//! `read_node`) — grep-assert + runtime-trace pair per cag-r1-9 +
//! CLAUDE.md baked-in #18 Class B β (LOAD-BEARING).
//!
//! ## SHAPE arm (grep)
//! materializer.rs source contains zero `.read_node(` call sites; the
//! MaterializerEngine trait names `read_node_as`. The fact that this
//! pin would PASS if the trait simply didn't exist means we additionally
//! grep for the affirmative `read_node_as` mention.
//!
//! ## SUBSTANCE arm (runtime trace)
//! A walk drives the InMemoryMaterializerEngine adapter; its trace
//! counter MUST increment `read_node_as_count` to at least 1 and MUST
//! NOT increment `bare_read_node_count` (which is unconditionally 0 by
//! construction — the trait surface has no `read_node` method).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Cid, Node, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, Materializer, MaterializerEngine, MaterializerError,
    MaterializerWalkInputs, allow_all_cap_recheck,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Trace-recording adapter — counts `read_node_as` invocations + asserts
/// (by trait surface) that NO bare `read_node` path exists.
struct TraceEngine {
    inner: benten_platform_foundation::InMemoryMaterializerEngine,
    read_node_as_count: AtomicUsize,
}

impl TraceEngine {
    fn new() -> Self {
        Self {
            inner: benten_platform_foundation::InMemoryMaterializerEngine::new(),
            read_node_as_count: AtomicUsize::new(0),
        }
    }
}

impl MaterializerEngine for TraceEngine {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        self.read_node_as_count.fetch_add(1, Ordering::SeqCst);
        self.inner.read_node_as(principal, cid)
    }
}

#[test]
fn materializer_uses_read_node_as_only_never_read_node() {
    // GREP-SHAPE arm.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/materializer.rs"))
        .expect("materializer.rs source readable from tests/");

    // No `.read_node(` call sites in source (match leading `.` so we
    // don't pick up the substring inside `.read_node_as(`).
    let bare_calls: Vec<_> = src.match_indices(".read_node(").collect();
    assert!(
        bare_calls.is_empty(),
        "materializer.rs MUST NOT call .read_node( directly — found {} call \
         sites at offsets {:?}; route ALL reads through read_node_as per \
         CLAUDE.md #18 Class B β + cag-r1-9",
        bare_calls.len(),
        bare_calls.iter().map(|(o, _)| *o).collect::<Vec<_>>(),
    );
    let bare_keyword: Vec<_> = src.match_indices(" read_node(").collect();
    assert!(
        bare_keyword.is_empty(),
        "materializer.rs MUST NOT reference free `read_node(` either"
    );

    // At least one `read_node_as` mention (the seam is named).
    let as_mentions: Vec<_> = src.match_indices("read_node_as").collect();
    assert!(
        !as_mentions.is_empty(),
        "materializer.rs MUST reference read_node_as at least once (the cap-attributed seam)"
    );

    // RUNTIME-SUBSTANCE arm.
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let engine = TraceEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.inner.put_node(Node::new(vec!["Note".into()], props));

    let mat = HtmlJsonMaterializer;
    let _ = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();

    let count = engine.read_node_as_count.load(Ordering::SeqCst);
    assert!(
        count >= 1,
        "runtime trace MUST observe at least 1 read_node_as event for the walk; got {count}"
    );
}
