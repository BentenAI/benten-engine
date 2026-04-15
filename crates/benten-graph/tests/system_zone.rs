//! Phase 1 R3 security test — system-zone write forgery (R1 SC1).
//!
//! Attack class: a user-authored operation subgraph issues a WRITE targeting a
//! `system:CapabilityGrant` (or any other `system:`-prefixed label) Node. Without
//! Invariant 11 enforcement in the evaluator (Phase 2), the only Phase 1 defense
//! is the `WriteContext::is_privileged` flag at the `benten-graph` write-path
//! layer (N8). Engine-API paths (`grant_capability`, `create_view`,
//! `revoke_capability`) set `is_privileged = true`; user-facing CRUD paths do
//! not. Any WRITE with `is_privileged = false` to a label beginning with
//! `system:` must reject with `E_SYSTEM_ZONE_WRITE` before reaching the
//! underlying redb transaction.
//!
//! TDD contract: these tests FAIL at R3 (R1 triage added N8 as a new
//! deliverable). R5 lands `WriteContext`, the privileged-flag plumbing, the
//! system-label prefix check, and the `E_SYSTEM_ZONE_WRITE` error code.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #1 (critical)
//! - `.addl/phase-1/r1-triage.md` SC1 disposition
//! - `.addl/phase-1/r2-test-landscape.md` §2.2 `WriteContext` rows + §7 security
//! - `docs/ERROR-CATALOG.md` `E_SYSTEM_ZONE_WRITE`
//!
//! Invariant 11 full enforcement is Phase 2; this is the Phase 1 stopgap.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_graph::{ErrorCode, RedbBackend, WriteContext};

/// Construct a Node whose label begins with `system:` — a would-be forged
/// capability grant. Properties match a plausible `CapabilityGrant` shape so
/// the test exercises the exact attack surface the security auditor named.
fn forged_system_capability_grant() -> Node {
    let mut props = std::collections::BTreeMap::new();
    props.insert("scope".into(), Value::Text("admin:*".into()));
    props.insert("granted_to".into(), Value::Text("attacker:did:z6M…".into()));
    Node::new(vec!["system:CapabilityGrant".into()], props)
}

/// Attack: user-authored subgraph tries to WRITE a `system:`-labelled Node.
///
/// This is the canonical forgery scenario. A NoAuthBackend deployment (the
/// Phase 1 default) has no capability check to stop it, so the only defense
/// is the graph-layer system-label prefix check. The test asserts the write
/// is rejected with the stable `E_SYSTEM_ZONE_WRITE` code and that NO bytes
/// land in the store.
#[test]
fn user_operation_cannot_write_system_labeled_node() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let node = forged_system_capability_grant();

    // Default WriteContext — the user-operation path, `is_privileged = false`.
    let ctx = WriteContext::default();
    assert!(
        !ctx.is_privileged,
        "Default WriteContext must be unprivileged; privileged construction is \
         an explicit opt-in reserved for engine API paths."
    );

    let err = backend
        .put_node_with_context(&node, &ctx)
        .expect_err("system-labelled user write must be rejected");

    assert_eq!(
        err.code(),
        ErrorCode::SystemZoneWrite,
        "unprivileged write to system:* label must surface the stable \
         E_SYSTEM_ZONE_WRITE code, not a generic validation error"
    );

    // Integrity: the forged grant must not exist in the store after rejection.
    let cid = node.cid().unwrap();
    assert!(
        backend.get_node(&cid).unwrap().is_none(),
        "forged system Node must NOT be persisted on the rejected path"
    );
}

/// Positive control: the engine-API-privileged path (set only by
/// `grant_capability` / `create_view` / `revoke_capability`) MUST be able to
/// write `system:` Nodes. Otherwise the engine itself can't store its own
/// metadata.
#[test]
fn privileged_engine_path_can_write_system_labeled_node() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let node = forged_system_capability_grant();

    let ctx = WriteContext::privileged_for_engine_api();
    assert!(ctx.is_privileged);

    let cid = backend
        .put_node_with_context(&node, &ctx)
        .expect("privileged write to system:* must succeed");

    let fetched = backend.get_node(&cid).unwrap().expect("stored");
    assert_eq!(fetched, node);
}

/// Boundary: the `system:` prefix check must be a true prefix match. A label
/// that merely *contains* `system:` (e.g. `user:system:foo`) is NOT reserved.
/// This matters because a careless substring check would lock users out of
/// perfectly legitimate label names.
#[test]
fn system_prefix_check_is_prefix_not_substring() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();

    let mut props = std::collections::BTreeMap::new();
    props.insert("title".into(), Value::Text("ok".into()));
    let node = Node::new(vec!["user:system:notes".into()], props);

    let ctx = WriteContext::default();
    assert!(
        backend.put_node_with_context(&node, &ctx).is_ok(),
        "the `system:` zone is a label-prefix reservation; labels that \
         contain the substring but do not start with it are user-space and \
         must be accepted under unprivileged contexts"
    );
}

/// Boundary: empty label list is not a system label. Rejecting it here would
/// conflate unrelated validation failures with the system-zone defense.
#[test]
fn empty_label_list_is_not_system_zone() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();

    let mut props = std::collections::BTreeMap::new();
    props.insert("x".into(), Value::Int(1));
    let node = Node::new(Vec::new(), props);

    let ctx = WriteContext::default();
    // Writing a Node with no labels is not a system-zone violation. (It may be
    // rejected for other reasons in Phase 1 proper — this test only asserts it
    // does NOT fire E_SYSTEM_ZONE_WRITE.)
    if let Err(e) = backend.put_node_with_context(&node, &ctx) {
        assert_ne!(e.code(), ErrorCode::SystemZoneWrite);
    }
}
