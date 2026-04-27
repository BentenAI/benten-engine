//! Phase-2b G12-D — `SubgraphSpec.primitives` widening coverage.
//!
//! Per plan §3.2 G12-D + D6-RESOLVED (BTreeMap+Value, no CBOR passthrough).
//!
//! Pins:
//! 1. `subgraph_spec_primitives_carries_per_primitive_props` — a
//!    `SubgraphSpec` with mixed primitive kinds (WRITE + READ +
//!    SUBSCRIBE) round-trips through registration preserving each
//!    primitive's properties bag.
//! 2. `wider_subgraph_spec_cid_stable_when_primitives_unset` — empty
//!    `primitives` Vec produces the same handler CID as a fresh empty
//!    spec on the legacy WRITE-only construction path (forward-compat
//!    pin: widening doesn't drift the CID for the empty case).
//! 3. `wider_subgraph_spec_old_handlers_still_load` — handlers built via
//!    the legacy `.write(|w| ...)` builder method still register and
//!    dispatch correctly post-widening (back-compat pin).
//! 4. `subgraph_for_spec_walks_primitives_not_just_write_specs` — a
//!    `SubgraphSpec` that declares a non-WRITE primitive (READ) via the
//!    widened `.primitive_with_props` entry point + a WRITE constructs
//!    a runnable Subgraph that includes both nodes, NOT just WRITE.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// Pin 1 — D6-RESOLVED widening preserves per-primitive properties.
///
/// Build a `SubgraphSpec` with three primitives: a WRITE (its config
/// folded into the bag under `WRITE_PROP_*` keys), a READ (empty bag),
/// and a SUBSCRIBE primitive whose declared `pattern` config rides in
/// the bag. Read each entry back via `spec.primitives()` and assert
/// every kind + property survives.
#[test]
fn subgraph_spec_primitives_carries_per_primitive_props() {
    let mut sub_props = BTreeMap::new();
    sub_props.insert("pattern".into(), Value::text("post:*"));

    let spec = SubgraphSpec::builder()
        .handler_id("widening:mixed")
        .write(|w| w.label("post").property("title", Value::text("hello")))
        .primitive("r0", PrimitiveKind::Read)
        .primitive_with_props(PrimitiveSpec {
            id: "sub0".into(),
            kind: PrimitiveKind::Subscribe,
            properties: sub_props.clone(),
        })
        .build();

    let prims = spec.primitives();
    assert_eq!(prims.len(), 3, "three primitives recorded");

    // WRITE entry — kind + folded WriteSpec config.
    assert!(matches!(prims[0].kind, PrimitiveKind::Write));
    assert_eq!(prims[0].id, "w0");
    // Label is folded into the bag under `WRITE_PROP_LABEL` ("_label").
    assert_eq!(
        prims[0].properties.get("_label"),
        Some(&Value::text("post")),
        "WriteSpec.label folded into PrimitiveSpec.properties under _label"
    );
    // User properties from `.property("title", ...)` ride under
    // `WRITE_PROP_USER_PROPERTIES` ("_user_properties") as a typed Value::Map.
    let user_props = prims[0]
        .properties
        .get("_user_properties")
        .expect("WriteSpec.properties folded under _user_properties");
    if let Value::Map(m) = user_props {
        assert_eq!(m.get("title"), Some(&Value::text("hello")));
    } else {
        panic!("_user_properties must be Value::Map, got {user_props:?}");
    }

    // READ entry — empty bag, structurally registered.
    assert!(matches!(prims[1].kind, PrimitiveKind::Read));
    assert_eq!(prims[1].id, "r0");
    assert!(
        prims[1].properties.is_empty(),
        "READ with no declared config carries an empty BTreeMap (uniform shape)"
    );

    // SUBSCRIBE entry — per-primitive pattern in the bag.
    assert!(matches!(prims[2].kind, PrimitiveKind::Subscribe));
    assert_eq!(prims[2].id, "sub0");
    assert_eq!(
        prims[2].properties.get("pattern"),
        Some(&Value::text("post:*")),
        "SUBSCRIBE.pattern preserved in per-primitive properties bag"
    );

    // Back-compat synthesised view: `spec.write_specs()` derives the
    // legacy Vec<WriteSpec> from the widened primitives storage.
    let writes = spec.write_specs();
    assert_eq!(
        writes.len(),
        1,
        "exactly one WriteSpec synthesised (one kind=Write entry)"
    );
    assert_eq!(writes[0].label_ref(), "post");
    assert_eq!(
        writes[0].properties_ref().get("title"),
        Some(&Value::text("hello"))
    );
}

/// Pin 2 — empty `primitives` Vec is forward-compat-stable.
///
/// A `SubgraphSpec` with NO primitives declared registers cleanly under
/// the widened shape AND dispatches via the empty-spec `noop_read+respond`
/// fallback in `Engine::subgraph_for_spec`. Asserts the empty case round-
/// trips through registration without surfacing an error or constructing
/// an unexpectedly-shaped subgraph.
#[test]
fn wider_subgraph_spec_cid_stable_when_primitives_unset() {
    let (_dir, engine) = fresh_engine();

    let spec = SubgraphSpec::builder()
        .handler_id("widening:empty-primitives")
        .build();

    // Empty primitives → registration succeeds; the engine constructs a
    // noop_read+respond fallback subgraph (preserves the prior empty-
    // write_specs semantics from the pre-widening code path).
    let handler_cid = engine
        .register_subgraph(spec.clone())
        .expect("empty SubgraphSpec must register cleanly under widened shape");

    // Re-register the same empty spec → identical handler CID
    // (content-addressed; the empty-primitives encoding is stable).
    let (_dir2, engine2) = fresh_engine();
    let handler_cid2 = engine2
        .register_subgraph(spec)
        .expect("re-registration succeeds");
    assert_eq!(
        handler_cid, handler_cid2,
        "empty-primitives SubgraphSpec produces stable handler CID across registrations"
    );
}

/// Pin 3 — pre-widening `.write(|w| ...)` builder usage still works.
///
/// The vast majority of existing handler-construction sites use the
/// `.write(|w| w.label(...).property(...))` builder. Post-widening, the
/// builder folds each WriteSpec into a `PrimitiveSpec` of kind=Write +
/// `WRITE_PROP_*` bag. Asserts the legacy build call shape produces a
/// registerable + dispatchable handler with the same on-disk effect as
/// before (one WRITE produces one node).
#[test]
fn wider_subgraph_spec_old_handlers_still_load() {
    let (_dir, engine) = fresh_engine();

    let spec = SubgraphSpec::builder()
        .handler_id("widening:legacy-write-builder")
        .write(|w| w.label("post").property("title", Value::text("legacy")))
        .build();

    // Legacy build → exactly one PrimitiveSpec of kind=Write under the
    // widened storage.
    assert_eq!(spec.primitives().len(), 1);
    assert!(matches!(spec.primitives()[0].kind, PrimitiveKind::Write));

    let handler_cid = engine
        .register_subgraph(spec)
        .expect("legacy .write() builder registers under widened storage");

    // Dispatch through call(): the WRITE produces a node with the
    // configured label.
    let outcome = engine
        .call(&handler_cid, "create", Node::empty())
        .expect("call dispatches through the widened-spec subgraph");
    assert!(
        outcome.is_ok_edge(),
        "legacy WRITE-only handler dispatches via OK edge: {outcome:?}"
    );
    assert_eq!(
        outcome.successful_write_count(),
        1,
        "exactly one WRITE produced one node"
    );
    assert!(
        outcome.created_cid().is_some(),
        "WRITE outcome carries the created node CID"
    );
}

/// Pin 4 — `Engine::subgraph_for_spec` walks `spec.primitives` (not just
/// the legacy WRITE-only path).
///
/// Closes the G12-A mini-review carry: pre-G12-D, `subgraph_for_spec`
/// iterated `spec.write_specs` only, silently dropping non-WRITE
/// primitives declared via `.primitive(id, kind)`. Post-widening, the
/// function walks the full `spec.primitives` storage and constructs an
/// OperationNode for every entry.
///
/// This test registers a handler with READ + WRITE primitives declared
/// in that order, then exercises BOTH the trace path (which fires the
/// first READ, hitting `ON_NOT_FOUND` because the empty input doesn't
/// carry a target_cid) AND the Mermaid render (which materialises the
/// FULL constructed Subgraph regardless of dispatch-time edge routing).
/// The Mermaid output proves the Subgraph the engine materialised
/// includes BOTH the READ and WRITE nodes — i.e. `subgraph_for_spec`
/// walked `spec.primitives` and didn't silently drop the READ.
#[test]
fn subgraph_for_spec_walks_primitives_not_just_write_specs() {
    let (_dir, engine) = fresh_engine();

    let spec = SubgraphSpec::builder()
        .handler_id("widening:read-then-write")
        .primitive("r0", PrimitiveKind::Read)
        .write(|w| w.label("post"))
        .build();

    // Pre-flight: spec has TWO primitives declared, only one is WRITE.
    assert_eq!(spec.primitives().len(), 2);
    assert_eq!(spec.write_specs().len(), 1);
    // Capture the actual builder-assigned primitive ids (the WRITE id
    // depends on insertion order; with R0 inserted first, the WRITE
    // becomes `w1` because the builder formats `w{primitives.len()}`).
    let read_id = spec.primitives()[0].id.clone();
    let write_id = spec.primitives()[1].id.clone();
    assert_eq!(read_id, "r0");
    assert_eq!(write_id, "w1");

    let handler_cid = engine
        .register_subgraph(spec)
        .expect("READ+WRITE spec registers under widened walk");

    // The Mermaid render reconstructs the registered Subgraph in full
    // (one labelled node per primitive + edges per add_edge call), so it
    // surfaces every node `subgraph_for_spec` materialised regardless of
    // runtime edge routing. Pre-G12-D the READ node would be silently
    // omitted because the function walked `spec.write_specs` only.
    let mermaid = engine
        .handler_to_mermaid("widening:read-then-write")
        .expect("mermaid render succeeds for registered handler");

    assert!(
        mermaid.contains(&read_id),
        "constructed Subgraph must contain the READ node (id={read_id}) declared via \
         `.primitive(...)`. mermaid:\n{mermaid}"
    );
    assert!(
        mermaid.contains(&write_id),
        "constructed Subgraph must contain the WRITE node (id={write_id}) declared via \
         `.write(|w| ...)`. mermaid:\n{mermaid}"
    );

    // Belt-and-suspenders: the trace must show the READ step actually
    // fired at dispatch time (proving the READ node is wired into the
    // root edge, not just present in the node set).
    let trace = engine
        .trace(&handler_cid, "create", Node::empty())
        .expect("trace dispatches successfully");

    let mut saw_read = false;
    let mut step_summary = Vec::new();
    for step in trace.steps() {
        if let Some(prim) = step.primitive() {
            step_summary.push(format!("{}({})", prim, step.node_id().unwrap_or("?")));
            // `primitive()` returns the engine-side lowercased Debug-format
            // label (e.g. "read" / "write") — see `primitive_kind_label`
            // in `crates/benten-engine/src/engine.rs`.
            if prim == "read" {
                saw_read = true;
            }
        }
    }
    assert!(
        saw_read,
        "trace must show the READ from `spec.primitives` fired (root edge wires \
         to the first declared primitive). observed steps: {step_summary:?}"
    );
}
