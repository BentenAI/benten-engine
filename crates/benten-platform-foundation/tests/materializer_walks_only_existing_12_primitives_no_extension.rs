//! G23-B GREEN: materializer walks ONLY existing 12 primitives; no new
//! `PrimitiveKind` variant added (LOAD-BEARING; CLAUDE.md baked-in #1).
//!
//! ## Pair: grep-assert (SHAPE) + runtime-trace (SUBSTANCE) per §3.6f.
//! - GREP: materializer.rs source's `PrimitiveKind::` references all
//!   name one of the 12 canonical variants.
//! - RUNTIME: a real materializer walk's `dispatched_primitive_kinds()`
//!   set is a subset of the canonical 12.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::{Node, PrimitiveKind, Value};
use benten_platform_foundation::{
    HtmlJsonMaterializer, InMemoryMaterializerEngine, Materializer, MaterializerWalkInputs,
    allow_all_cap_recheck,
};
use std::collections::BTreeMap;

const CANONICAL_VARIANTS: &[&str] = &[
    "Read",
    "Write",
    "Transform",
    "Branch",
    "Iterate",
    "Wait",
    "Call",
    "Respond",
    "Emit",
    "Sandbox",
    "Subscribe",
    "Stream",
];

#[test]
fn materializer_walks_only_existing_12_primitives_no_extension() {
    // GREP arm: materializer.rs source's `PrimitiveKind::` mentions all
    // resolve to one of the 12 canonical names.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/materializer.rs"))
        .expect("materializer.rs source readable");

    for (idx, _m) in src.match_indices("PrimitiveKind::") {
        let tail = &src[idx + "PrimitiveKind::".len()..];
        let variant: String = tail
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        assert!(
            CANONICAL_VARIANTS.contains(&variant.as_str()),
            "materializer.rs references PrimitiveKind::{variant} — not in 12-primitive set"
        );
    }

    // RUNTIME arm: drive a real walk + check dispatched kinds.
    let spec = benten_platform_foundation::compile_schema(
        schema_fixtures::canonical_note_type_schema_bytes(),
    )
    .unwrap();
    let alice = materializer_fixtures::actor_principal_alice_cid();
    let engine = InMemoryMaterializerEngine::new();
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text("body".into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    let cid = engine.put_node(Node::new(vec!["Note".into()], props));

    let mat = HtmlJsonMaterializer;
    let out = mat
        .materialize_with_gate(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();

    let dispatched = out.dispatched_primitive_kinds();
    assert!(
        !dispatched.is_empty(),
        "walk dispatched at least one primitive"
    );
    for k in dispatched {
        // Every dispatched kind names one of the 12.
        match k {
            PrimitiveKind::Read
            | PrimitiveKind::Write
            | PrimitiveKind::Transform
            | PrimitiveKind::Branch
            | PrimitiveKind::Iterate
            | PrimitiveKind::Wait
            | PrimitiveKind::Call
            | PrimitiveKind::Respond
            | PrimitiveKind::Emit
            | PrimitiveKind::Sandbox
            | PrimitiveKind::Subscribe
            | PrimitiveKind::Stream => {}
            _ => panic!(
                "13th-or-later PrimitiveKind variant dispatched: {k:?} \
                 — CLAUDE.md baked-in #1 violation"
            ),
        }
    }
}
