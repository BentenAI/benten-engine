//! Edge-case test: the thinness contract — an Engine configured with
//! `NoAuthBackend` + `without_ivm()` + `without_versioning()` must still
//! produce a usable graph database.
//!
//! Per `docs/ARCHITECTURE.md` "The Thinness Test": A developer should be able
//! to use benten-core + benten-graph + benten-engine with NoAuthBackend,
//! versioning disabled, and no IVM subscribers — and get a pure content-
//! addressed graph database with no Benten-specific conventions. If that
//! configuration requires anything from benten-eval, benten-ivm, or
//! benten-caps, the engine is too thick.
//!
//! This is the edge-case pair: `without_ivm().without_caps().without_versioning()`
//! is the *thinnest* configuration. It must still create/read Nodes, create/read
//! Edges, and run basic CRUD. If it requires capabilities or views or a
//! version chain to function, the engine has leaked concerns into the core.
//!
//! R3 contract: `Engine::builder()` does not exist today (the spike's Engine
//! is `Engine::open(path)` directly). R5 (G7-A) introduces EngineBuilder,
//! `without_ivm`, `without_caps`, `without_versioning`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Node, Value};
use tempfile::tempdir;

#[test]
fn thinness_no_ivm_no_caps_no_versioning_still_works() {
    // The thinnest-viable Engine: NoAuth + no IVM + no versioning.
    // Must still support content-addressed Node create/read — the
    // "pure CID KV store" use case per `docs/ARCHITECTURE.md`.
    let dir = tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("thin.redb"))
        .without_ivm()
        .without_caps() // default is already NoAuth, but explicit is better
        .without_versioning()
        .build()
        .expect("thinnest configuration must build");

    // Create a Node.
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text("thin"));
    let node = Node::new(vec!["Doc".into()], props);

    let cid = engine.create_node(&node).expect("thin engine must create");
    let fetched = engine
        .get_node(&cid)
        .expect("thin engine must read")
        .expect("node must be present");
    assert_eq!(fetched, node);
}

#[test]
fn thin_engine_rejects_caps_dependent_apis() {
    // Honest-no boundary: an Engine without caps cannot grant capabilities.
    // The API must refuse explicitly rather than pretending to grant
    // something that won't be enforced.
    let dir = tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("thin.redb"))
        .without_caps()
        .build()
        .unwrap();

    let result = engine.grant_capability(
        &benten_core::Cid::from_blake3_digest([0u8; 32]),
        "store:post:write",
    );
    assert!(
        result.is_err(),
        "grant_capability on no-caps engine must fail honestly, not silently no-op"
    );
}

#[test]
fn thin_engine_rejects_view_reads() {
    // Honest-no boundary: an Engine without IVM cannot serve view reads.
    // Must return a typed error, not an empty page masquerading as data.
    let dir = tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("thin.redb"))
        .without_ivm()
        .build()
        .unwrap();

    let result = engine.read_view("system:ivm:content_listing");
    assert!(
        result.is_err(),
        "read_view on no-IVM engine must error, not return empty"
    );
}

#[test]
fn thin_engine_has_no_ivm_subscribers_when_disabled() {
    // R4 triage (m7): renamed from `thin_engine_survives_workspace_lint_budget`.
    // The v1 name implied CI lint-budget enforcement; the body only counts
    // subscribers. The lint-budget aspect is deferred to a future phase CI
    // check — this test just asserts the subscriber-count contract.
    //
    // If R5 accidentally pulls in all of `benten-ivm` at a build level even
    // under `.without_ivm()`, the subscriber-count assertion fires before
    // any runtime path exercises the IVM machinery.
    let dir = tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("thin.redb"))
        .without_ivm()
        .without_caps()
        .without_versioning()
        .build()
        .unwrap();

    // No IVM subscriber was registered.
    assert_eq!(
        engine.ivm_subscriber_count(),
        0,
        "without_ivm must mean zero subscribers, not just zero-exposed-API"
    );
}
