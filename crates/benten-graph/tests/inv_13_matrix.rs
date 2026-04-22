//! Edge-case tests: Invariant-13 (immutability) 5-row firing matrix at the
//! graph storage layer.
//!
//! Covers rows 1, 2, and 3 (active-in-2a) of the plan §9.11 matrix:
//! - Row 1: `User` + content matches → `E_INV_IMMUTABILITY`.
//! - Row 2: `User` + content differs → `E_INV_IMMUTABILITY` (vacuous under
//!   content-addressing: the CID *is* the content, but we still assert the
//!   naming/routing of the error cleanly so that future mutable-id extensions
//!   don't silently change it).
//! - Row 3: `EnginePrivileged` + content matches → `Ok(cid_dedup)` with
//!   *no* audit-sequence advance and *no* ChangeEvent emitted (security-class
//!   portion is in the security writer's `inv_13_dedup_semantics.rs`; edge
//!   writer pins the dedup return-value behaviour).
//!
//! Rows 4 and 5:
//! - Row 4 (`SyncReplica`) lives in the security writer's file and is
//!   `#[ignore]`-gated for Phase 3 per R2 triage Gap 2.
//! - Row 5 (WAIT-resume stale-pin pre-check) lives in
//!   `wait_resume_stale_pin_rejects_before_write.rs` (also this partition).
//!
//! R3 red-phase contract: R5 (G2-A / G5-A) lands the matrix-aware `put_node`
//! path plus the `WriteAuthority` enum + `WriteContext::authority` field.
//! These tests compile; they fail because the matrix is not wired.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_errors::ErrorCode;
use benten_graph::{ChangeKind, NodeStore, RedbBackend, WriteAuthority, WriteContext};
use tempfile::tempdir;

fn node_with_title(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    Node::new(vec!["Doc".into()], props)
}

fn backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("inv13.redb")).expect("open redb");
    (dir, backend)
}

#[test]
fn inv_13_user_write_content_matches_rejects_with_immutability() {
    // Row 1: User writes node twice with identical content (CID matches).
    // Content-addressing guarantees this is the only path a "user re-put"
    // can reach — and Inv-13 must refuse it as an unauthorised second put.
    let (_dir, backend) = backend();
    let node = node_with_title("row1");

    let ctx_user = WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };

    // First put establishes the CID.
    let cid = backend
        .put_node_with_context(&node, &ctx_user)
        .expect("first put must succeed");

    // Second put with the same content + User authority must fire Inv-13.
    let err = backend
        .put_node_with_context(&node, &ctx_user)
        .expect_err("second User put to same CID must fire Inv-13");
    assert_eq!(
        err.code(),
        ErrorCode::InvImmutability,
        "Row 1 (User×match) must fire E_INV_IMMUTABILITY, got {:?}",
        err.code()
    );
    // Sanity: the CID was not altered.
    let fetched = backend.get_node(&cid).unwrap().unwrap();
    assert_eq!(fetched, node);
}

#[test]
fn inv_13_user_write_content_differs_vacuous_but_names_correctly() {
    // Row 2: Under content-addressed storage, "content differs at same CID"
    // is vacuous — different content yields a different CID by construction.
    // This test pins the *naming* of the code so that a future mutable-id
    // extension (or a hash collision) cannot silently succeed.
    //
    // Test strategy: force two writes with different content via User
    // authority; confirm they land at different CIDs (the normal path), AND
    // confirm that the test harness's `put_node_at_cid_for_test` backdoor —
    // which injects a CID/content mismatch — fires Inv-13 with the correct
    // code when invoked with User authority.
    let (_dir, backend) = backend();
    let n1 = node_with_title("alpha");
    let n2 = node_with_title("beta");

    let ctx_user = WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };

    let cid1 = backend.put_node_with_context(&n1, &ctx_user).unwrap();
    let cid2 = backend.put_node_with_context(&n2, &ctx_user).unwrap();
    assert_ne!(cid1, cid2, "different content must land at different CIDs");

    // Test-only backdoor: force a mismatched put to exercise the row-2
    // code-path semantics. The harness exists precisely so Inv-13's matrix
    // has a covered row-2 test even under content-addressing.
    let err = backend
        .put_node_at_cid_for_test(&cid1, &n2, &ctx_user)
        .expect_err("User + content-differs must fire Inv-13");
    assert_eq!(
        err.code(),
        ErrorCode::InvImmutability,
        "Row 2 (User×differs, test-only path) must fire E_INV_IMMUTABILITY"
    );
}

#[test]
fn inv_13_engine_privileged_content_matches_dedups_no_change_event() {
    // Row 3: EnginePrivileged + identical bytes → `Ok(cid_dedup)`.
    // Additional contract: the dedup path does NOT emit a ChangeEvent
    // (edge-writer's scope: the return value is Ok + the event stream is
    // empty. Security-writer pins the audit-sequence half).
    let (_dir, backend) = backend();
    let node = node_with_title("row3_dedup");

    // Seed once under User to establish the CID.
    let ctx_user = WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };
    let cid = backend.put_node_with_context(&node, &ctx_user).unwrap();

    // Subscribe before the dedup call so any event would be captured.
    let events_before = backend.drain_change_events_for_test();
    assert!(
        events_before
            .iter()
            .any(|e| e.cid == cid && e.kind == ChangeKind::Created),
        "sanity: seed put emitted a Created event"
    );

    // Re-put with EnginePrivileged authority — must dedup.
    let ctx_privileged = WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::EnginePrivileged,
        ..WriteContext::default()
    };
    let cid_dedup = backend
        .put_node_with_context(&node, &ctx_privileged)
        .expect("Row 3 (EnginePrivileged×match) must return Ok(cid_dedup)");
    assert_eq!(cid_dedup, cid, "dedup must return the existing CID");

    // Contract: no ChangeEvent was emitted for the dedup call.
    let events_after = backend.drain_change_events_for_test();
    assert!(
        events_after.is_empty(),
        "Row 3 dedup path must NOT emit a ChangeEvent; got {:?}",
        events_after
    );
}
