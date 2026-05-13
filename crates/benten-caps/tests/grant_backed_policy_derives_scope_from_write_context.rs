//! G27-B — `GrantBackedPolicy::derive_write_scope` lift.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.15 G27-B row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-B entry
//! + CRATES-DEEP-DIVE §4 "Schema / scope derivation hardcoded".
//!
//! ## The lift
//!
//! Today `GrantBackedPolicy::derive_write_scope(label: &str) -> String`
//! at `crates/benten-caps/src/grant_backed.rs:215-220` hard-codes the
//! shape `format!("store:{label}:write")` (or `"store:write"` for an
//! empty label). The label is read off `WriteContext::label` — fine
//! for the Phase-1 `crud('<label>')` zero-config path but NOT
//! extensible to:
//! - non-CRUD scopes (sandbox / handler / view / system zones)
//! - plugin manifest grammar scopes (`private:<plugin_did>:*`,
//!   `requires:<plugin_did>:<path>`, `shares:<plugin_did>:<path>`)
//! - explicit scope-threading through `WriteContext::scope`.
//!
//! G27-B threads scope derivation through `WriteContext::scope` — if
//! the caller populates that field, the policy uses it verbatim; the
//! label-based default applies only when `scope` is empty.
//!
//! ## Pin shape (would-FAIL-if-no-op'd, pim-2 §3.6b)
//!
//! 1. Construct a `WriteContext` with `scope = "store:custom:write"`
//!    (NOT derivable from `label = "post"`).
//! 2. Mint a grant for `"store:custom:write"`.
//! 3. Run `check_write(&ctx)`; assert `Ok(())` (the explicit scope IS
//!    consulted, not the label-derived `"store:post:write"`).
//! 4. Would-FAIL: revert the lift; the policy falls back to the
//!    label-derived `"store:post:write"`, which doesn't match the
//!    granted `"store:custom:write"`, and the assertion flips to deny.
//!
//! ## RED-PHASE expectation
//!
//! G27-B R5 implementer threads `WriteContext::scope` through the
//! derivation; un-ignores this pin at wave-time per §3.6e.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};

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

/// RED-PHASE: G27-B — scope threads through `WriteContext::scope`.
///
/// At HEAD the policy hard-codes `store:{label}:write` and ignores
/// `WriteContext::scope`. The G27-B implementer threads the explicit
/// scope through; this pin un-ignores at G27-B wave-time.
#[test]
fn grant_backed_policy_derives_scope_from_write_context_scope_field() {
    let grants = MockGrants::new(&["store:custom:write"]);
    let policy = GrantBackedPolicy::new(grants);

    // Substantive arm: scope explicitly threaded; label hints at a
    // DIFFERENT scope shape so the regression fires if the policy
    // ever reverts to label-only derivation.
    let ctx = WriteContext {
        label: "post".into(),
        scope: "store:custom:write".into(),
        ..Default::default()
    };

    // Would-FAIL-if-no-op'd: revert G27-B; the policy derives
    // `store:post:write` from the label, which doesn't match the
    // granted `store:custom:write`, and `check_write` returns
    // `CapError::Denied`.
    policy
        .check_write(&ctx)
        .expect("RED-PHASE: G27-B — implementer must thread WriteContext::scope through derivation; explicit scope must override label-derived default");
}

/// Compile-time witness: the lift target field exists on `WriteContext`.
#[test]
fn write_context_scope_field_present_compile_witness() {
    let ctx = WriteContext {
        scope: "store:any:write".into(),
        ..Default::default()
    };
    let _: &str = ctx.scope.as_str();
}
