//! Phase 4-Foundation R1 cap-r1-2 BLOCKER + cap-r1-10 (dual-gate substance
//! closure): `GrantBackedPolicy::check_read` MUST consult
//! `ReadContext::actor_cid` so a grant issued to user-B does not permit
//! user-A's read.
//!
//! # The bug (pre-fix)
//!
//! `check_read` wildcard-enumerated grants against `scope` alone via
//! `GrantReader::has_unrevoked_grant_for_scope(scope)`. The reader has
//! no actor binding — it returns `true` iff ANY peer holds the scope
//! under the same backend. user-A who lacks read-cap-X fires a read,
//! user-B in the same system holds X, the reader says "scope is
//! granted", and user-A's read is silently permitted.
//!
//! # The fix
//!
//! `check_read` now calls the principal-aware
//! `GrantReader::has_unrevoked_grant_for_scope_and_actor(scope,
//! actor_cid)`. The default trait impl collapses to scope-only when
//! `actor_cid` is `None`, preserving NoAuthBackend semantics for
//! callers that have not threaded an actor (Phase-1 / Phase-2
//! fixtures + the bare default policy). Implementations that DO bind
//! grantee to grant — notably the engine's `BackendGrantReader` —
//! override to filter by `grantee == actor_cid`.
//!
//! # Would-FAIL-if-no-op'd (§3.6b)
//!
//! Path B below DIRECTLY exercises the bug: with the post-fix code the
//! cross-principal read is denied; remove the `ctx.actor_cid`
//! threading from `check_read` (e.g. drop back to
//! `has_unrevoked_grant_for_scope(candidate)`) and Path B silently
//! permits — the test catches it.
//!
//! Cross-refs:
//! - `crates/benten-caps/src/grant_backed.rs::GrantReader::has_unrevoked_grant_for_scope_and_actor`
//! - `crates/benten-caps/src/grant_backed.rs::GrantBackedPolicy::check_read`
//! - `crates/benten-engine/src/builder.rs::BackendGrantReader` (concrete override)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, ReadContext};
use benten_core::{Cid, Node};

/// Actor-aware mock GrantReader: stores `(scope, grantee)` pairs so the
/// test can prove the `check_read` path actually consults `actor_cid`.
/// An "unbounded" grant (no grantee binding) is stored as
/// `(scope, None)` and answers `true` regardless of which actor asks.
struct ActorAwareMockGrants {
    /// (scope, optional grantee binding).
    grants: Vec<(String, Option<Cid>)>,
}

impl ActorAwareMockGrants {
    fn new(grants: Vec<(&str, Option<Cid>)>) -> Arc<Self> {
        Arc::new(Self {
            grants: grants
                .into_iter()
                .map(|(s, g)| (s.to_string(), g))
                .collect(),
        })
    }
}

impl GrantReader for ActorAwareMockGrants {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        // Scope-only fallback — used only when the policy calls the
        // base method (e.g. `check_write`, or `check_read` when
        // `actor_cid` is `None` AND the implementation has not
        // overridden the actor-aware method). For this test the
        // policy always calls the actor-aware override below.
        Ok(self.grants.iter().any(|(s, _)| s == scope))
    }

    fn has_unrevoked_grant_for_scope_and_actor(
        &self,
        scope: &str,
        actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        // The substantive check: a stored grant satisfies the lookup
        // iff scope matches AND either (a) the stored grantee is
        // `None` (unbounded), or (b) the requested actor is `Some`
        // and matches the stored grantee.
        //
        // When the caller threads `actor_cid = None` AND the stored
        // grant has `Some(grantee)`, the grant does NOT satisfy —
        // an actor-bound grant requires an actor on the lookup. This
        // mirrors the `BackendGrantReader` shape in `benten-engine`.
        for (s, g) in &self.grants {
            if s != scope {
                continue;
            }
            match (g, actor_cid) {
                (None, _) => return Ok(true), // unbounded
                (Some(stored), Some(asked)) if stored == asked => return Ok(true),
                _ => {}
            }
        }
        Ok(false)
    }
}

/// Synthesise a CID-shaped principal. Each call produces a distinct CID
/// keyed off the label string so user-A and user-B are observably
/// different identities.
fn principal(label: &str) -> Cid {
    let node = Node::new(vec![label.to_string()], Default::default());
    node.cid().unwrap()
}

/// Path A: user-A grants user-A read cap on scope X, user-A reads → permit.
///
/// This is the baseline positive: the actor-aware reader returns `true`
/// because `(scope, grantee) == (store:post:read, user_a_cid)` and the
/// caller's `actor_cid` matches.
#[test]
fn check_read_permits_when_actor_cid_matches_grantee() {
    let user_a = principal("user-a");

    let grants = ActorAwareMockGrants::new(vec![("store:post:read", Some(user_a))]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = ReadContext {
        label: "post".into(),
        actor_cid: Some(user_a),
        ..Default::default()
    };

    policy
        .check_read(&ctx)
        .expect("user-A with a user-A-bound read grant must be permitted");
}

/// Path B: user-A reads, but the only grant on scope X is bound to
/// user-B → DENY.
///
/// This is the BUG (cap-r1-2 BLOCKER) — pre-fix the scope-only reader
/// would silently permit because *some* grant on `store:post:read`
/// exists. Post-fix the principal-aware lookup refuses because
/// `grantee != actor_cid`. This is the load-bearing
/// would-FAIL-if-no-op'd assertion per §3.6b.
#[test]
fn check_read_denies_when_only_other_principal_holds_scope() {
    let user_a = principal("user-a");
    let user_b = principal("user-b");

    // user-B holds the read cap; user-A does NOT.
    let grants = ActorAwareMockGrants::new(vec![("store:post:read", Some(user_b))]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = ReadContext {
        label: "post".into(),
        actor_cid: Some(user_a),
        ..Default::default()
    };

    let err = policy
        .check_read(&ctx)
        .expect_err("user-A read must be denied — only user-B holds store:post:read");
    let CapError::DeniedRead { required, entity } = err else {
        panic!("expected DeniedRead, got {err:?}");
    };
    assert_eq!(
        required, "store:post:read",
        "denial must surface the derived required scope so the operator can see \
         which capability the read needed"
    );
    assert_eq!(
        entity, "post",
        "denial must surface the label the read targeted"
    );
}

/// Path C: an unbounded grant (no grantee binding) permits ANY actor.
///
/// This preserves NoAuthBackend semantics + every Phase-1 / Phase-2
/// fixture that issued grants without an actor binding. The
/// principal-aware reader returns `true` for `(scope, None)` regardless
/// of `actor_cid`, so the policy permits.
#[test]
fn check_read_permits_unbounded_grant_for_any_actor() {
    let user_a = principal("user-a");

    // No grantee binding — Phase-1 / NoAuthBackend / legacy fixture shape.
    let grants = ActorAwareMockGrants::new(vec![("store:post:read", None)]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = ReadContext {
        label: "post".into(),
        actor_cid: Some(user_a),
        ..Default::default()
    };

    policy
        .check_read(&ctx)
        .expect("unbounded grant (None grantee) must permit any actor");
}

/// Path D: `actor_cid = None` (no actor threaded) AND an actor-bound
/// grant exists for the scope → DENY.
///
/// Symmetric guarantee: a caller that fails to thread an actor cannot
/// piggyback on an actor-bound grant. The fallback "no actor at the
/// lookup" path is exclusively for unbounded grants. This rules out a
/// regression where dropping `ctx.actor_cid` (e.g. via a refactor) would
/// re-introduce the cross-principal permission bug under a different
/// surface shape.
#[test]
fn check_read_denies_when_actor_unset_against_actor_bound_grant() {
    let user_b = principal("user-b");

    let grants = ActorAwareMockGrants::new(vec![("store:post:read", Some(user_b))]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = ReadContext {
        label: "post".into(),
        actor_cid: None, // no actor on the lookup
        ..Default::default()
    };

    let err = policy
        .check_read(&ctx)
        .expect_err("actor-unset read must be denied against an actor-bound-only grant");
    assert!(
        matches!(err, CapError::DeniedRead { .. }),
        "expected DeniedRead, got {err:?}"
    );
}
