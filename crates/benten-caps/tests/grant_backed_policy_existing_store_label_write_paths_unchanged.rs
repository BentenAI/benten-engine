//! G27-B — backward-compatibility regression guard.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.15 G27-B row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-B entry
//! ("backward-compat: existing store-label write paths unchanged").
//!
//! ## What this pin verifies
//!
//! G27-B threads `CapWriteContext::scope` through derivation. The
//! REGRESSION risk: callers that DO NOT populate `scope` (Phase-1
//! `crud('<label>')` zero-config callers + their existing tests +
//! the engine's own privileged-write call sites) must continue to
//! see the EXACT same label-derived `store:<label>:write` shape.
//!
//! Pin shape:
//! 1. Construct a `CapWriteContext` with `label = "post"` + `scope = ""`
//!    (the Phase-1 default — `CapWriteContext::default()` shape).
//! 2. Mint a grant for `"store:post:write"` (the derived shape).
//! 3. Assert `check_write(&ctx) == Ok(())` — the policy still
//!    derives the canonical CRUD scope from the label when `scope`
//!    is empty.
//!
//! Would-FAIL-if-no-op'd: G27-B implementer makes
//! `CapWriteContext::scope` REQUIRED (drops the label-derivation
//! fallback); every Phase-1 caller flips to deny. This pin catches
//! that regression at G27-B mini-review time.
//!
//! ## Coupling
//!
//! The opposite pin (`grant_backed_policy_derives_scope_from_write_context.rs`)
//! verifies the NEW behavior; this pin verifies the OLD behavior is
//! preserved as the fallback path. Both pins must pass together.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, GrantBackedPolicy, GrantReader};

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

/// RED-PHASE: G27-B backward-compat regression guard.
///
/// Verifies that the canonical Phase-1 `crud('<label>')` flow continues
/// to derive `store:<label>:write` from `CapWriteContext::label` when
/// `CapWriteContext::scope` is empty.
#[test]
fn grant_backed_policy_existing_store_label_write_paths_unchanged_when_scope_unset() {
    let grants = MockGrants::new(&["store:post:write"]);
    let policy = GrantBackedPolicy::new(grants);

    // Phase-1 caller shape: label populated, scope left empty (default).
    let ctx = CapWriteContext {
        label: "post".into(),
        // scope: "" (default)
        ..Default::default()
    };

    // Backward-compat: the label-derived `store:post:write` MUST
    // still match the granted scope when `CapWriteContext::scope` is
    // empty. Would-FAIL: G27-B implementer makes scope required;
    // every Phase-1 caller flips to deny.
    policy
        .check_write(&ctx)
        .expect("Phase-1 label-derived store:<label>:write shape MUST remain functional when CapWriteContext::scope is empty");
}

/// RED-PHASE: empty-label + empty-scope continues to be a fail-closed
/// deny (r6-sec-8 (a) inheritance from existing
/// `grant_backed_policy.rs::grant_backed_policy_denies_unstructured_empty_context`).
#[test]
fn grant_backed_policy_empty_label_empty_scope_continues_to_deny_unstructured() {
    let grants = MockGrants::new(&["store:post:write"]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = CapWriteContext::default();
    let err = policy
        .check_write(&ctx)
        .expect_err("empty/unstructured CapWriteContext must continue to deny");
    assert!(
        matches!(err, CapError::Denied { .. }),
        "G27-B backward-compat: empty-batch must still be denied (r6-sec-8 (a) inheritance); got {err:?}"
    );
}
