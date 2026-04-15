//! Phase 1 R3 security test — engine-API-only mutation of system zone.
//!
//! Attack class companion to `system_zone.rs` (which tests the graph-layer
//! rejection). This file tests the ENGINE-level contract: the only way to
//! write a `system:`-labelled Node is through a dedicated privileged API —
//! `Engine::grant_capability`, `Engine::create_view`, `Engine::revoke_capability`.
//! Any attempt to reach the system zone through the generic
//! `Engine::create_node` / `transaction` / subgraph-CALL paths must be
//! rejected with `E_SYSTEM_ZONE_WRITE`.
//!
//! This is the upper boundary of the N8 defense: the graph-layer
//! (`system_zone.rs`) is the last line of defense; the engine-level API is
//! the first line, and user code never has a legitimate reason to call it.
//!
//! TDD contract: FAIL at R3. R5 lands the three privileged engine APIs +
//! the `is_privileged` plumbing through them.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #1 (critical)
//! - `.addl/phase-1/r1-triage.md` SC1 N8 disposition
//! - `.addl/phase-1/r2-test-landscape.md` §2.6 N7 + §7 system-zone tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{ErrorCode, Node, Value};
use benten_engine::{Engine, EngineError};

fn system_labeled_node() -> Node {
    let mut props = std::collections::BTreeMap::new();
    props.insert("scope".into(), Value::Text("anything".into()));
    Node::new(vec!["system:IVMView".into()], props)
}

/// `Engine::create_node` is the user-facing CRUD entry point. Using it to
/// write a `system:`-labelled Node must fail — this is the primary
/// privilege-escalation vector in NoAuthBackend deployments.
#[test]
fn create_node_rejects_system_labeled_write() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let node = system_labeled_node();
    let err = engine
        .create_node(&node)
        .expect_err("create_node must reject system:* labels");

    match err {
        EngineError::Graph(ge) => {
            assert_eq!(ge.code(), ErrorCode::SystemZoneWrite);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

/// Privileged engine APIs are the ONLY way to mutate the system zone.
/// `Engine::grant_capability` writes a `system:CapabilityGrant` Node and
/// MUST succeed.
#[test]
fn grant_capability_only_via_engine_api() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    // The privileged API path.
    let cid = engine
        .grant_capability("post:write", "test-subject")
        .expect("engine.grant_capability must succeed as privileged path");

    // Readable back — the grant exists in the store.
    let fetched = engine.get_node(&cid).unwrap().expect("grant persisted");
    assert!(fetched.labels.iter().any(|l| l == "system:CapabilityGrant"));
}

/// `Engine::create_view` writes a `system:IVMView` Node — privileged path.
#[test]
fn create_view_only_via_engine_api() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let view_cid = engine
        .create_view("content_listing", Default::default())
        .expect("engine.create_view is the privileged view-creation path");
    let fetched = engine.get_node(&view_cid).unwrap().unwrap();
    assert!(fetched.labels.iter().any(|l| l == "system:IVMView"));
}

/// `Engine::revoke_capability` is the third privileged API. Revoking also
/// writes to the system zone (records the revocation as a Node).
#[test]
fn revoke_capability_only_via_engine_api() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let grant_cid = engine
        .grant_capability("post:write", "test-subject")
        .unwrap();
    // Privileged revocation — must succeed.
    engine
        .revoke_capability(grant_cid, "post:write")
        .expect("engine.revoke_capability is a privileged path");
}

/// Transaction path: user-path transactions cannot smuggle system-zone
/// writes by batching them with legitimate writes. If the privilege flag is
/// per-operation rather than per-transaction (it is, per the R1 triage),
/// the system-zone write in the batch MUST fire `E_SYSTEM_ZONE_WRITE`
/// while the legitimate writes in the same transaction ROLL BACK.
#[test]
fn transaction_cannot_smuggle_system_zone_write() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let mut legit_props = std::collections::BTreeMap::new();
    legit_props.insert("title".into(), Value::Text("legit".into()));
    let legit = Node::new(vec!["Post".into()], legit_props);

    let system_node = system_labeled_node();

    let result = engine.transaction(|tx| {
        tx.create_node(&legit)?;
        tx.create_node(&system_node)?; // this must reject
        Ok(())
    });

    let err = result.expect_err("transaction must abort on system-zone write");
    match err {
        EngineError::Graph(ge) => assert_eq!(ge.code(), ErrorCode::SystemZoneWrite),
        other => panic!("unexpected error: {other:?}"),
    }

    // Atomicity: the legit write must ALSO be rolled back.
    let legit_cid = legit.cid().unwrap();
    assert!(
        engine.get_node(&legit_cid).unwrap().is_none(),
        "transaction aborting on system-zone violation must roll back ALL \
         writes in the batch, not partial-commit"
    );
}
