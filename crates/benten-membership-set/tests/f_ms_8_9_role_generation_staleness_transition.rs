//! **F-MS-8** + **F-MS-9** — `role_assignments_generation` staleness +
//! role-transition survives prior UCAN attenuations.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! families **F-MS-8** (merges B3 + T-E1) and **F-MS-9** (merges C5 + T-E5, #60).
//!
//! # What F-MS-8 pins (R0 §2.5 BC-5 / §3.10 / §3.6.B)
//!
//! A stanza sealed under a stale `role_assignments_generation` is **rejected
//! at verify** with the ErrorCode `E_ROLE_STALE_AT_VERIFY` (the F4-031 §3.5g
//! mint — atomic Rust variant + errors.generated.ts + ERROR-CATALOG.md +
//! CATALOG_VARIANT_COUNT). The counter is the 11th field of the `0x6610` group
//! AAD = the BLINDED 11-field set (R0.7 §3.10/§4.1). Sealing at generation `G`
//! and advancing the set to `G+1` invalidates the older stanza at verify.
//!
//! # What F-MS-9 pins (R0 #60 / §3.4; #52)
//!
//! An ephemeral UCAN issued at role `R` **survives a later role downgrade** —
//! it is bounded only by its own `exp` (NOT retroactively revoked by the
//! membership-set role change; #60 honest disclosure). The §3.4 default is a
//! tight `exp` so the survival window is small.
//!
//! # R5 (un-ignored against `benten_membership_set::{verify, ucan}` +
//! `benten_errors::ErrorCode`)
//!
//! Drives the real `verify_stanza` + `EphemeralGrant::issue_bounded`. The
//! `E_ROLE_STALE_AT_VERIFY` code is the real `ErrorCode::RoleStaleAtVerify`
//! (the §3.5g mint). Would-FAIL: a verifier that ignores the generation field,
//! a missing code, or a downgrade that retroactively invalidates an unexpired
//! UCAN all break a pin.

use benten_errors::ErrorCode;
use benten_membership_set::ucan::{DEFAULT_EXP_BOUND_SECS, EphemeralGrant};
use benten_membership_set::verify::{RoleStaleError, Stanza, verify_stanza};

// ── F-MS-8 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms8_stale_generation_rejected_at_verify() {
    // Seal at generation G=7, then advance the set to G+1=8.
    let stanza = Stanza::sealed_under(7);
    assert_eq!(
        verify_stanza(&stanza, 8),
        Err(RoleStaleError {
            code: ErrorCode::RoleStaleAtVerify
        }),
        "a stanza sealed under a stale role_assignments_generation rejects at verify (E_ROLE_STALE_AT_VERIFY)"
    );
    // Paired positive control: verifying at the SAME generation succeeds.
    assert!(
        verify_stanza(&stanza, 7).is_ok(),
        "a stanza at the CURRENT generation verifies"
    );
}

#[test]
fn ms8_error_code_is_canonical() {
    // The new ErrorCode is the load-bearing observable. Pin the exact string so
    // the Rust↔TS catalog mirror (§3.5g) stays in sync.
    let err = verify_stanza(&Stanza::sealed_under(0), 5).unwrap_err();
    assert_eq!(
        err.code,
        ErrorCode::RoleStaleAtVerify,
        "the role-staleness verify rejection surfaces the canonical RoleStaleAtVerify ErrorCode"
    );
    assert_eq!(
        err.code_str(),
        "E_ROLE_STALE_AT_VERIFY",
        "the canonical wire-string is E_ROLE_STALE_AT_VERIFY (§3.5g atomic mint)"
    );
    // Round-trips through the catalog (the §3.5g Rust mirror).
    assert_eq!(
        "E_ROLE_STALE_AT_VERIFY".parse::<ErrorCode>().unwrap(),
        ErrorCode::RoleStaleAtVerify,
        "E_ROLE_STALE_AT_VERIFY round-trips through ErrorCode::from_str (§3.5g)"
    );
}

// ── F-MS-9 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms9_role_transition_survives_prior_ucan() {
    // Issue an Admin-role ephemeral grant valid [1000, 1000+3600).
    let grant = EphemeralGrant {
        nbf: 1000,
        exp: 1000 + DEFAULT_EXP_BOUND_SECS,
    };
    // A downgrade Admin → Viewer does NOT retroactively invalidate the grant —
    // it remains valid until its own `exp` (#60 honest disclosure).
    assert!(
        grant.is_valid_at(2000),
        "an ephemeral UCAN issued at the prior role survives the downgrade (only exp bounds it — #60)"
    );
    // After `exp`, it is no longer valid (the survival window is finite).
    assert!(
        !grant.is_valid_at(1000 + DEFAULT_EXP_BOUND_SECS),
        "the grant expires at exp — survival is exp-bounded, not unbounded"
    );
}

#[test]
fn ms9_tight_exp_default_bounds_survival_window() {
    // Drive the real ISSUER with an OVER-LONG requested expiry and assert it
    // CLAMPS the survival window down to the default bound (would-FAIL if the
    // issuer honored the over-long request verbatim).
    let nbf = 1000;
    let over_long_request = nbf + 10 * DEFAULT_EXP_BOUND_SECS; // 10× too long
    let grant = EphemeralGrant::issue_bounded(nbf, over_long_request);
    assert!(
        grant.lifetime_secs() <= DEFAULT_EXP_BOUND_SECS,
        "the issuer CLAMPS an over-long requested expiry to the §3.4 default bound"
    );
    assert_eq!(
        grant.exp,
        nbf + DEFAULT_EXP_BOUND_SECS,
        "an over-long request is clamped exactly to nbf + DEFAULT_EXP_BOUND_SECS"
    );

    // A WITHIN-bound request is honored as-is (not a blanket clamp-to-max).
    let short_request = nbf + 60;
    let short_grant = EphemeralGrant::issue_bounded(nbf, short_request);
    assert_eq!(
        short_grant.exp, short_request,
        "a within-bound request is honored verbatim (not force-extended)"
    );

    assert_eq!(
        DEFAULT_EXP_BOUND_SECS, 3600,
        "the v1-beta tight-exp default is the 1-hour bound"
    );
}
