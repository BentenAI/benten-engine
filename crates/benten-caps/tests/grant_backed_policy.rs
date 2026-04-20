//! Phase-1 R6 security-auditor — GrantBackedPolicy coverage gaps (r6-sec-8).
//!
//! Two gaps surfaced in the R6 pass:
//!
//! 1. **Empty-batch permissiveness.** A `WriteContext` arriving at the policy
//!    with no `pending_ops`, an empty `label`, and an empty `scope` used to
//!    return `Ok(())` by default. Under GrantBackedPolicy this is a
//!    fail-open — an unstructured context reaching the policy is an error
//!    mode, not a legitimate no-op. The policy now DENIES rather than
//!    permit-by-default.
//! 2. **Delete scope derivation.** `PendingOp::DeleteNode` / `DeleteEdge`
//!    now carry the captured labels (threaded via read-before-delete from
//!    `benten-graph::Transaction`). The policy derives the same
//!    `store:<label>:write` scope it uses for the create side, so an actor
//!    with only a read grant cannot delete through a transaction.
//!
//! Cross-refs:
//! - `.addl/phase-1/r6-*` (R6 security-auditor findings)
//! - `crates/benten-caps/src/grant_backed.rs` (`check_write` body)
//! - `crates/benten-caps/src/policy.rs` (`PendingOp::DeleteNode/DeleteEdge`)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{
    CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, PendingOp, WriteContext,
};
use benten_core::Cid;

/// Mock GrantReader: the scope→bool map is injected at construction time so
/// each test sets its own grant world. All `has_unrevoked_grant_for_scope`
/// calls are answered from the map; unknown scopes return `false`.
struct MockGrants {
    grants: Vec<String>,
}

impl MockGrants {
    fn new(scopes: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            grants: scopes.iter().map(|s| (*s).to_string()).collect(),
        })
    }
}

impl GrantReader for MockGrants {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        Ok(self.grants.iter().any(|g| g == scope))
    }
}

/// Synthetic CID for PendingOp construction.
fn fake_cid() -> Cid {
    // Any valid content-addressed body works here — the policy only inspects
    // labels / scopes, not the CID itself.
    let node = benten_core::Node::new(vec!["post".to_string()], Default::default());
    node.cid().unwrap()
}

/// r6-sec-8 (a): empty-batch deny.
///
/// The policy used to return `Ok(())` for a context with no pending ops,
/// empty label, empty scope. That was a permit-by-default fail-open under a
/// grant-backed policy. Regression test: construct exactly that shape and
/// assert `CapError::Denied`.
#[test]
fn grant_backed_policy_denies_unstructured_empty_context() {
    let grants = MockGrants::new(&["store:post:read", "store:post:write"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext::default();
    let err = policy
        .check_write(&ctx)
        .expect_err("empty/unstructured WriteContext must be denied");
    assert!(
        matches!(err, CapError::Denied { .. }),
        "empty-batch must be denied (not permit-by-default); got {err:?}"
    );
}

/// r6-sec-8 (a): empty pending_ops with a fallback label still gets checked.
#[test]
fn grant_backed_policy_permits_empty_batch_with_matching_fallback_label() {
    let grants = MockGrants::new(&["store:post:write"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "post".into(),
        ..Default::default()
    };
    policy
        .check_write(&ctx)
        .expect("matching fallback label + grant must permit");
}

/// r6-sec-8 (b): the canonical delete-denial scenario.
///
/// Subject holds `store:post:read` only. The evaluator issues a
/// `PendingOp::DeleteNode { cid, labels: ["post"] }`. Pre-r6-sec-8 the
/// policy ignored DeleteNode entirely (scope never checked, write went
/// through under a read-only grant). The fix derives
/// `store:post:write` from the captured labels and denies.
#[test]
fn grant_backed_policy_denies_unauthorized_delete() {
    let grants = MockGrants::new(&["store:post:read"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::DeleteNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };

    let err = policy
        .check_write(&ctx)
        .expect_err("delete without store:post:write must be denied");
    let CapError::Denied { required, .. } = err else {
        panic!("expected Denied, got {err:?}");
    };
    assert_eq!(
        required, "store:post:write",
        "delete must derive the write-scope family from the captured labels"
    );
}

/// Delete with matching write grant — the positive case that proves the
/// label-threading actually routes through the grant lookup.
#[test]
fn grant_backed_policy_permits_authorized_delete() {
    let grants = MockGrants::new(&["store:post:write"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::DeleteNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };

    policy
        .check_write(&ctx)
        .expect("delete with matching write grant must permit");
}

/// Idempotent-miss delete (empty labels vec) is a no-op at the policy level;
/// no scope is derived and no grant is required. This keeps the idempotent-
/// delete contract intact — deleting an already-absent CID must not require
/// a cap grant the caller wouldn't otherwise need.
#[test]
fn grant_backed_policy_permits_idempotent_miss_delete() {
    let grants = MockGrants::new(&[]); // no grants at all
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: String::new(),
        pending_ops: vec![PendingOp::DeleteNode {
            cid: fake_cid(),
            labels: Vec::new(),
        }],
        ..Default::default()
    };

    policy
        .check_write(&ctx)
        .expect("idempotent-miss delete must be a no-op at the policy");
}

/// Edge delete denial — mirrors the node-delete fix.
#[test]
fn grant_backed_policy_denies_unauthorized_edge_delete() {
    let grants = MockGrants::new(&["store:post:read"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "AUTHORED_BY".into(),
        pending_ops: vec![PendingOp::DeleteEdge {
            cid: fake_cid(),
            label: Some("AUTHORED_BY".into()),
        }],
        ..Default::default()
    };

    let err = policy
        .check_write(&ctx)
        .expect_err("edge delete without matching write grant must be denied");
    let CapError::Denied { required, .. } = err else {
        panic!("expected Denied, got {err:?}");
    };
    assert_eq!(required, "store:AUTHORED_BY:write");
}

/// r6b-dx-C1: wildcard capability must permit every matching concrete scope.
///
/// A subject holds `store:post:*` (the scope spelled in QUICKSTART + the
/// scaffolder README). The evaluator issues a WRITE whose primary label is
/// `"post"`; the policy derives required scope `store:post:write`. Before
/// the fix the grant never matched because the reader only returns exact-
/// scope equality and `store:post:*` ≠ `store:post:write`. After the fix,
/// the policy enumerates wildcard ancestors of the required scope and
/// succeeds as long as any ancestor is stored.
#[test]
fn grant_backed_policy_wildcard_permits_create_get_list_delete_under_same_label() {
    let grants = MockGrants::new(&["store:post:*"]);
    let policy = GrantBackedPolicy::new(grants);

    // Create path — derived required scope is `store:post:write`.
    let create_ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    policy
        .check_write(&create_ctx)
        .expect("wildcard `store:post:*` must permit `store:post:write`");

    // Delete path — same derived required scope via captured labels.
    let delete_ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::DeleteNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    policy
        .check_write(&delete_ctx)
        .expect("wildcard `store:post:*` must permit delete under the same label");

    // Read path — `check_read` must honour the same ancestor set.
    let read_ctx = benten_caps::ReadContext {
        label: "post".into(),
        ..Default::default()
    };
    policy
        .check_read(&read_ctx)
        .expect("wildcard `store:post:*` must permit `store:post:read`");
}

/// Bare `*` (the UCAN-spirit catch-all) is the broadest wildcard and must
/// satisfy any derived concrete scope.
#[test]
fn grant_backed_policy_bare_wildcard_permits_everything() {
    let grants = MockGrants::new(&["*"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    policy
        .check_write(&ctx)
        .expect("bare `*` wildcard must permit `store:post:write`");
}

/// Narrower wildcard at the wrong segment must NOT permit a mismatched
/// concrete scope. `store:comment:*` is unrelated to `store:post:write`.
#[test]
fn grant_backed_policy_wildcard_denies_wrong_label() {
    let grants = MockGrants::new(&["store:comment:*"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    let err = policy
        .check_write(&ctx)
        .expect_err("wildcard on a different label must NOT permit");
    assert!(matches!(err, CapError::Denied { .. }));
}
