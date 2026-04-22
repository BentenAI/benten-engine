//! Edge-case tests: engine thinness survives the arch-1 dep-break reshape
//! (R2 landscape §2.6.3, phil post-arch-1).
//!
//! Arch-1 (plan G1-B) moves `EvalError` / `HostError` plumbing so that
//! `benten-eval` no longer depends on `benten-graph`. The reshape must NOT
//! leak concerns into the thin-engine configurations:
//!   - `Engine::builder().without_ivm().without_caps().without_versioning()`
//!     still creates, reads, and deletes Nodes.
//!   - The thin config does not inadvertently pull `benten-eval` types into
//!     `PrimitiveHost` signatures for any method the thin-engine path calls.
//!   - Phase-2a's new `WriteAuthority` + `ExecutionStateEnvelope` types are
//!     invisible to the pure-KV-store use case.
//!
//! Concerns pinned:
//! - Thinnest config builds successfully post arch-1.
//! - `create_node` / `get_node` / `delete_node` on the thinnest config work.
//! - `grant_capability` on `.without_caps()` still fails honestly (no silent
//!   accept-but-ignore).
//! - `read_view` on `.without_ivm()` still fails with `E_SUBSYSTEM_DISABLED`.
//! - The thin config does NOT accidentally expose WAIT / resume methods
//!   (those require eval + attribution infrastructure; thin configs skip the
//!   whole tower).
//!
//! R3 red-phase contract: R5 (G1-B) completes the dep-break. These tests
//! compile; they fail if any post-arch-1 signature requires `benten-eval` or
//! `benten-graph` types for the thin path.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use tempfile::tempdir;

fn thin_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("thin_after_arch1.redb"))
        .without_ivm()
        .without_caps()
        .without_versioning()
        .build()
        .expect("thinnest config must still build post-arch-1");
    (dir, engine)
}

#[test]
fn thinness_preserved_after_arch_1_dep_break_create_and_read() {
    let (_dir, engine) = thin_engine();

    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text("thin"));
    let node = Node::new(vec!["Doc".into()], props);

    let cid = engine.create_node(&node).expect("create must succeed");
    let fetched = engine
        .get_node(&cid)
        .expect("get must succeed")
        .expect("node must be present");
    assert_eq!(fetched, node);
}

#[test]
fn thinness_preserved_after_arch_1_dep_break_delete_roundtrip() {
    let (_dir, engine) = thin_engine();

    let mut props = BTreeMap::new();
    props.insert("tag".into(), Value::text("delete_me"));
    let node = Node::new(vec!["Tmp".into()], props);
    let cid = engine.create_node(&node).unwrap();

    engine.delete_node(&cid).expect("delete must succeed");
    let post_delete = engine.get_node(&cid).unwrap();
    assert!(
        post_delete.is_none(),
        "deleted node must not be readable back in thin config"
    );
}

#[test]
fn thin_engine_still_refuses_grants_post_arch_1() {
    let (_dir, engine) = thin_engine();
    let actor = Cid::from_blake3_digest([0u8; 32]);
    let result = engine.grant_capability(&actor, "store:post:write");
    let err = result.expect_err("grant_capability on no-caps thin engine must fail honestly");
    assert_eq!(err.code(), ErrorCode::SubsystemDisabled);
}

#[test]
fn thin_engine_still_refuses_view_reads_post_arch_1() {
    let (_dir, engine) = thin_engine();
    // Phase 2a R3 consolidation: `read_view` takes one arg in Phase 1; the
    // R3 writer double-threaded `Value::unit()` from the `call_with_suspension`
    // idiom. Drop the second arg (compiles against the Phase-1 API).
    let result = engine.read_view("nonexistent_view");
    let err = result.expect_err("read_view on no-ivm thin engine must fail honestly");
    // SubsystemDisabled vs UnknownView — SubsystemDisabled is the "no IVM at
    // all" path; UnknownView is the "IVM on but view not registered" path.
    // Thin config hits the former.
    assert_eq!(
        err.code(),
        ErrorCode::SubsystemDisabled,
        "thin without_ivm() must fire SubsystemDisabled, got {:?}",
        err.code()
    );
}

#[test]
fn thin_engine_wait_api_is_either_unavailable_or_no_op() {
    // Phase-2a WAIT requires attribution + caps + eval wiring. A thin engine
    // must either:
    //   (a) not expose `call_with_suspension` at all (type-level gating), or
    //   (b) expose it but return a typed error (SubsystemDisabled).
    //
    // Whichever shape lands is fine for this test as long as it's not a
    // silent no-op that would let a developer think a suspend happened.
    let (_dir, engine) = thin_engine();
    let result = engine.call_with_suspension_on_thin_config_for_test(
        "nonexistent_handler",
        "run",
        Value::unit(),
    );
    match result {
        Ok(_) => panic!("thin config must not silently complete a WAIT call"),
        Err(e) => {
            assert!(
                matches!(
                    e.code(),
                    ErrorCode::SubsystemDisabled
                        | ErrorCode::NotImplemented
                        | ErrorCode::DuplicateHandler
                        | ErrorCode::NotFound
                ),
                "thin WAIT refusal must be typed, got {:?}",
                e.code()
            );
        }
    }
}
