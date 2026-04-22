//! R3 unit tests for ucca-7: `GrantScope::parse` rejects lone `"*"` in Phase 2a.
//!
//! A grant with just `"*"` would permit everything; refusing at parse is the
//! explicit "honest no" against root-scope footguns. A compound `"*:<ns>"` form
//! is still accepted because the second segment anchors the namespace.
//!
//! TDD red-phase: the lone-star rejection does not yet fire — `parse("*")`
//! currently succeeds. Tests will fail until G4-A lands the check.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.4 ucca-7).

#![allow(clippy::unwrap_used)]

use benten_caps::{CapError, GrantScope};
use benten_errors::ErrorCode;

#[test]
fn grant_scope_rejects_lone_star() {
    let err = GrantScope::parse("*").expect_err("lone-star must reject");
    let code = err.code();
    assert_eq!(
        code,
        ErrorCode::CapScopeLoneStarRejected,
        "lone '*' must fire E_CAP_SCOPE_LONE_STAR_REJECTED, got {code:?}"
    );

    // Compound form is still accepted.
    let compound = GrantScope::parse("*:capability_grant")
        .expect("'*:capability_grant' is a valid compound scope");
    assert_eq!(compound.as_str(), "*:capability_grant");
}

#[test]
fn grant_scope_empty_and_lone_star_both_rejected() {
    // Phase-1 behaviour: empty string is rejected. Phase-2a ADDS lone-star
    // rejection. Both must remain rejected.
    let err_empty = GrantScope::parse("").expect_err("empty rejected");
    assert!(
        matches!(
            err_empty,
            CapError::Denied { .. } | CapError::ScopeLoneStarRejected
        ),
        "empty scope must be rejected"
    );
    let err_star = GrantScope::parse("*").expect_err("lone-star rejected");
    assert!(
        matches!(err_star, CapError::ScopeLoneStarRejected),
        "lone-star must map specifically to ScopeLoneStarRejected variant"
    );
}
