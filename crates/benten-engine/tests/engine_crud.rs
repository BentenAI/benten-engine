//! R4 triage M17 — CRUD surface coverage for `Engine` methods that the
//! landscape identified as untested at R3 close.
//!
//! These stubs drive the R5 G7/G8 landings for `Engine::update_node`,
//! `Engine::delete_node`, and the edge-level CRUD methods. TS-tier B3 napi
//! partner lives in `bindings/napi/index.test.ts`.
//!
//! Status: FAILING until R5 G7 + G8 land. Red-phase tests exercise the
//! public method shape so regressions surface before the method lands.

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

fn post(title: &str) -> Node {
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], p)
}

#[test]
fn engine_update_node_replaces_properties() {
    let (_dir, engine) = fresh_engine();
    let cid = engine.create_node(&post("first")).unwrap();

    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("first-updated".into()));
    let updated = Node::new(vec!["post".into()], p);
    let new_cid = engine.update_node(&cid, &updated).unwrap();

    // Updated node yields a new CID (content-addressed).
    assert_ne!(new_cid, cid, "update produces new CID (content-addressed)");
    let fetched = engine.get_node(&new_cid).unwrap().expect("updated node");
    assert_eq!(
        fetched.properties.get("title"),
        Some(&Value::Text("first-updated".into()))
    );
}

#[test]
fn engine_delete_node_removes_entry() {
    let (_dir, engine) = fresh_engine();
    let cid = engine.create_node(&post("first")).unwrap();
    engine.delete_node(&cid).unwrap();
    let fetched = engine.get_node(&cid).unwrap();
    assert!(fetched.is_none(), "deleted node must be unretrievable");
}

#[test]
fn engine_create_edge_and_read_back() {
    let (_dir, engine) = fresh_engine();
    let a = engine.create_node(&post("a")).unwrap();
    let b = engine.create_node(&post("b")).unwrap();

    let edge_cid = engine.create_edge(&a, &b, "RELATED_TO").unwrap();
    let edge = engine
        .get_edge(&edge_cid)
        .unwrap()
        .expect("edge retrievable");
    assert_eq!(edge.source, a);
    assert_eq!(edge.target, b);
    assert_eq!(edge.label, "RELATED_TO");
}

#[test]
fn engine_edges_from_returns_outbound() {
    let (_dir, engine) = fresh_engine();
    let a = engine.create_node(&post("a")).unwrap();
    let b = engine.create_node(&post("b")).unwrap();
    let c = engine.create_node(&post("c")).unwrap();

    let _ab = engine.create_edge(&a, &b, "RELATED_TO").unwrap();
    let _ac = engine.create_edge(&a, &c, "RELATED_TO").unwrap();

    let out = engine.edges_from(&a).unwrap();
    assert_eq!(out.len(), 2, "two outbound edges from a");
}

#[test]
fn engine_delete_edge_removes_entry() {
    let (_dir, engine) = fresh_engine();
    let a = engine.create_node(&post("a")).unwrap();
    let b = engine.create_node(&post("b")).unwrap();
    let edge_cid = engine.create_edge(&a, &b, "REL").unwrap();
    engine.delete_edge(&edge_cid).unwrap();
    assert!(engine.get_edge(&edge_cid).unwrap().is_none());
}
