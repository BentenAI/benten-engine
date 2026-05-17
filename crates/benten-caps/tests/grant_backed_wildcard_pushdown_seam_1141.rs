//! refinement-audit-2026-05 #1141 closure-pin (Pattern F: Qual-1 #694
//! + Safe-2 #552 + Fwd-1 #928) — §3.6b end-to-end, would-FAIL-if-no-op'd.
//!
//! # What this pins
//!
//! #1141's structural close pushes the wildcard-ancestor enumeration
//! out of `GrantBackedPolicy::check_write` / `check_read`'s inlined
//! `for candidate in wildcard_variants(scope)` hot-path loop into a
//! single `GrantReader::has_unrevoked_grant_matching` seam (default
//! body preserves prior behavior; an indexed backend overrides it for
//! the Fwd-1 #928 single-lookup close).
//!
//! 1. **Single-seam (source-coupled, regression-defense).** A
//!    `GrantReader` that overrides ONLY `has_unrevoked_grant_matching`
//!    (returning a fixed answer) and PANICS from every exact-match
//!    method MUST drive the policy decision. If a future edit
//!    re-introduces an inline `wildcard_variants` loop in `check_write`
//!    / `check_read` that calls the exact-match methods directly
//!    (recreating the #694/#928 inlined cascade the pushdown kills),
//!    the panicking exact-match methods fire and this test FAILs.
//!
//! 2. **#552 deep-scope behavioral.** A >6-segment required scope whose
//!    only stored grant is a trailing-wildcard parent
//!    (`private:<did>:*`) MUST be admitted through the policy — the
//!    pre-#1141 `n > 6` branch silently dropped that parent, denying a
//!    legitimately-granted deep write (asymmetric drift vs
//!    `attenuation::check_attenuation`). If the silent-drop is
//!    restored, this FAILs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{
    CapError, CapWriteContext, CapabilityPolicy, GrantBackedPolicy, GrantReader, PendingOp,
};
use benten_core::Cid;

/// A reader that ONLY implements the #1141 single seam. Every
/// exact-match method panics — so if the policy reaches an exact-match
/// path (an inline `wildcard_variants` loop), the test panics, proving
/// the seam is no longer the single path.
struct SeamOnlyReader {
    answer: bool,
}

impl GrantReader for SeamOnlyReader {
    fn has_unrevoked_grant_for_scope(&self, _scope: &str) -> Result<bool, CapError> {
        panic!(
            "#1141 regression: check_write/check_read reached the exact-match \
             reader directly — the inline wildcard_variants loop was \
             re-introduced (the pushdown seam was bypassed)"
        );
    }

    fn has_unrevoked_grant_for_scope_and_actor(
        &self,
        _scope: &str,
        _actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        panic!(
            "#1141 regression: check_read reached the exact-match \
             actor-aware reader directly — pushdown seam bypassed"
        );
    }

    fn has_unrevoked_grant_matching(
        &self,
        _required_scope: &str,
        _actor_cid: Option<&Cid>,
    ) -> Result<bool, CapError> {
        Ok(self.answer)
    }
}

fn fake_cid() -> Cid {
    benten_core::Node::new(vec!["post".to_string()], Default::default())
        .cid()
        .unwrap()
}

#[test]
fn check_write_drives_decision_through_the_single_pushdown_seam() {
    let policy = GrantBackedPolicy::new(Arc::new(SeamOnlyReader { answer: true }));
    let ctx = CapWriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    // Permits iff the seam (not an exact-match path) was consulted.
    policy
        .check_write(&ctx)
        .expect("seam returning true must permit; exact-match panics if bypassed");
}

#[test]
fn check_write_denies_through_the_single_seam_when_seam_says_no() {
    let policy = GrantBackedPolicy::new(Arc::new(SeamOnlyReader { answer: false }));
    let ctx = CapWriteContext {
        label: "post".into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec!["post".into()],
        }],
        ..Default::default()
    };
    let err = policy
        .check_write(&ctx)
        .expect_err("seam returning false must deny through the single seam");
    assert!(matches!(err, CapError::Denied { .. }));
}

#[test]
fn check_read_drives_decision_through_the_single_pushdown_seam() {
    let policy = GrantBackedPolicy::new(Arc::new(SeamOnlyReader { answer: true }));
    let ctx = benten_caps::ReadContext {
        label: "post".into(),
        ..Default::default()
    };
    policy
        .check_read(&ctx)
        .expect("seam returning true must permit read; exact-match panics if bypassed");
}

/// #552: a deep (>6-segment) required scope whose ONLY stored grant is
/// a trailing-wildcard ancestor must be admitted end-to-end through the
/// real default-body seam (not a stub). Uses a map-backed reader so the
/// default `has_unrevoked_grant_matching` body (the #552-fixed
/// `wildcard_variants` enumeration) actually runs.
struct MapReader {
    grants: Vec<String>,
}
impl GrantReader for MapReader {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        Ok(self.grants.iter().any(|g| g == scope))
    }
}

#[test]
fn deep_scope_trailing_wildcard_grant_admitted_through_policy_seam() {
    // 8-segment private-namespace-shaped required scope; the actor
    // holds only the `private:<did>:*` trailing-wildcard parent.
    let required_label = "z6MkDeep:resource:sub:detail:leaf:tip";
    let stored_parent = "store:z6MkDeep:resource:sub:detail:leaf:tip:*".to_string();
    // Build the actual derived write scope the policy will compute:
    // `store:<label>:write`. We instead grant a trailing-* ancestor of
    // it that the pre-#1141 n>6 branch would have dropped.
    let policy = GrantBackedPolicy::new(Arc::new(MapReader {
        grants: vec![format!("store:{required_label}:*")],
    }));
    let _ = stored_parent;

    let ctx = CapWriteContext {
        label: required_label.into(),
        pending_ops: vec![PendingOp::PutNode {
            cid: fake_cid(),
            labels: vec![required_label.into()],
        }],
        ..Default::default()
    };
    // `store:<label>:write` has >6 colon-segments; only the
    // `store:<label>:*` trailing parent is stored. Pre-#1141 this was
    // silently denied (n>6 drop). Post-#1141 it is admitted.
    policy.check_write(&ctx).expect(
        "#552: deep-scope write with a trailing-wildcard parent grant must \
         be admitted (the n>6 silent-drop is fixed in the single seam)",
    );
}
