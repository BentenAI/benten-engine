//! G27-B — non-CRUD scope round-trip pin.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.15 G27-B row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-B entry
//! + D-4F-1 FULL ratification (plugin manifest scope grammar).
//!
//! ## What this pin verifies
//!
//! G27-B threads `WriteContext::scope` through the derivation; the
//! immediate consequence is that NON-CRUD scope shapes flow through
//! the policy correctly. Today's hard-coded `store:{label}:write`
//! shape can only express CRUD-flavored writes; plugin manifest
//! grammar introduces:
//! - `private:<plugin_did>:*`             — private-namespace cap
//! - `requires:<plugin_did>:<path>`       — manifest `requires` half
//! - `shares:<plugin_did>:<path>`         — manifest `shares` half
//! - sandbox / handler / view zones       — non-CRUD primitive scopes
//!
//! ## Pin shape — substantive round-trips for 3 non-CRUD shapes
//!
//! For each non-CRUD scope shape:
//! 1. Mint a grant carrying the scope STRING exactly.
//! 2. Construct a `WriteContext` with `scope` populated to the
//!    matching scope STRING (NOT a label that derives to it).
//! 3. Assert `check_write(&ctx) == Ok(())` — the policy consults the
//!    explicit scope + matches the grant.
//!
//! Would-FAIL-if-no-op'd: the policy ignores `WriteContext::scope` +
//! tries to derive from label; the label is empty (no CRUD label
//! applicable for `private:` / `requires:` / `shares:` shapes), so
//! the derivation falls back to `"store:write"`, which doesn't match
//! any of the granted scopes; the assertion flips to deny.

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

/// RED-PHASE: G27-B — non-CRUD scope round-trips (plugin manifest grammar).
///
/// Verifies `WriteContext::scope`-threaded derivation supports
/// non-CRUD scope shapes that the hard-coded `store:{label}:write`
/// derivation cannot express.
#[test]
#[ignore = "RED-PHASE: G27-B — un-ignore at G27-B wave; verifies non-CRUD scopes flow through WriteContext::scope correctly"]
fn grant_backed_policy_non_crud_scope_round_trip_private_namespace() {
    let plugin_did = "did:key:zPluginDidPlaceholder";
    let scope = format!("private:{plugin_did}:notes");
    let grants = MockGrants::new(&[scope.as_str()]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: String::new(), // no CRUD label applicable
        scope: scope.clone(),
        ..Default::default()
    };

    policy.check_write(&ctx).expect(
        "RED-PHASE: G27-B — private-namespace scope must round-trip via WriteContext::scope",
    );
}

#[test]
#[ignore = "RED-PHASE: G27-B — un-ignore at G27-B wave; manifest `requires` scope shape"]
fn grant_backed_policy_non_crud_scope_round_trip_manifest_requires() {
    let plugin_did = "did:key:zPluginDidPlaceholder";
    let scope = format!("requires:{plugin_did}:calendar/events");
    let grants = MockGrants::new(&[scope.as_str()]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: String::new(),
        scope: scope.clone(),
        ..Default::default()
    };

    policy
        .check_write(&ctx)
        .expect("RED-PHASE: G27-B — manifest `requires` scope must round-trip");
}

#[test]
#[ignore = "RED-PHASE: G27-B — un-ignore at G27-B wave; manifest `shares` scope shape"]
fn grant_backed_policy_non_crud_scope_round_trip_manifest_shares() {
    let plugin_did = "did:key:zPluginDidPlaceholder";
    let scope = format!("shares:{plugin_did}:tasks/list");
    let grants = MockGrants::new(&[scope.as_str()]);
    let policy = GrantBackedPolicy::new(grants);

    let ctx = WriteContext {
        label: String::new(),
        scope: scope.clone(),
        ..Default::default()
    };

    policy
        .check_write(&ctx)
        .expect("RED-PHASE: G27-B — manifest `shares` scope must round-trip");
}
