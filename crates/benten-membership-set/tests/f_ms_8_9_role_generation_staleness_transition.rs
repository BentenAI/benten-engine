//! **F-MS-8** + **F-MS-9** — `role_assignments_generation` staleness +
//! role-transition survives prior UCAN attenuations.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, families **F-MS-8** (merges B3 + T-E1) and **F-MS-9** (merges
//! C5 + T-E5, #60).
//!
//! # What F-MS-8 pins (R0 §2.5 BC-5 / §3.10 / §3.6.B)
//!
//! A stanza sealed under a stale `role_assignments_generation` is **rejected
//! at verify** with the NEW ErrorCode `E_ROLE_STALE_AT_VERIFY`. (The
//! generation counter is one of the AAD 9-tuple fields — F-AAD-2.) Sealing at
//! generation `G` and advancing the set to `G+1` must invalidate the older
//! stanza at verify time.
//!
//! # What F-MS-9 pins (R0 #60 / §3.4; #52)
//!
//! An ephemeral UCAN issued at role `R` **survives a later role downgrade** —
//! it is bounded only by its own `exp` (it is NOT retroactively revoked by the
//! membership-set role change; #60 honest disclosure). The §3.4 default is a
//! tight `exp` (`exp - nbf ≤ default-bound`) so the survival window is small.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::verify::{verify_stanza, RoleStaleError}`
//! and `…::ucan::ephemeral_grant` and un-ignores. Would-FAIL-if-no-op'd: a
//! verifier that ignores the generation field, a missing `E_ROLE_STALE_AT_
//! VERIFY` code, or a downgrade that retroactively invalidates an unexpired
//! UCAN all break a pin.

#![allow(dead_code)]

// ── self-contained in-file stub-shim ────────────────────────────────────────

/// The NEW ErrorCode (mirrors the ErrorCode-catalog discipline; at R5 this is
/// a `benten_errors::ErrorCode::E_ROLE_STALE_AT_VERIFY` variant + TS mirror).
const E_ROLE_STALE_AT_VERIFY: &str = "E_ROLE_STALE_AT_VERIFY";

/// A stanza sealed under a particular role-assignments generation (the AAD
/// field). At R5 this is the real sealed-envelope verify path.
struct Stanza {
    sealed_role_assignments_generation: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct RoleStaleError {
    code: &'static str,
}

/// Production-shaped verify: a stanza whose sealed generation is older than the
/// set's CURRENT generation is rejected (the generation is AAD-bound, so a
/// post-rotation stanza fails AEAD-open / verify). At R5 this is
/// `verify_stanza(stanza, current_generation)`.
fn verify_stanza(stanza: &Stanza, current_generation: u32) -> Result<(), RoleStaleError> {
    if stanza.sealed_role_assignments_generation < current_generation {
        return Err(RoleStaleError {
            code: E_ROLE_STALE_AT_VERIFY,
        });
    }
    Ok(())
}

/// An ephemeral UCAN grant issued at a role, bounded by `nbf`/`exp` only.
struct EphemeralGrant {
    nbf: u64,
    exp: u64,
}

/// The §3.4 default bound on ephemeral-grant lifetime (seconds). Tight by
/// default so the #60 survival window is small.
const DEFAULT_EXP_BOUND_SECS: u64 = 3600;

impl EphemeralGrant {
    /// Production-shaped issuance that ENFORCES the §3.4 tight-exp default: a
    /// requested expiry is CLAMPED to `nbf + DEFAULT_EXP_BOUND_SECS` so the
    /// #60 survival window stays small regardless of what the caller asked
    /// for. At R5 this is the benten-caps ephemeral-grant minting path. A
    /// no-op issuer that honored an unbounded `requested_exp` verbatim would
    /// produce an over-long grant and FAIL `ms9_tight_exp_default_bounds_…`.
    fn issue_bounded(nbf: u64, requested_exp: u64) -> Self {
        let max_exp = nbf.saturating_add(DEFAULT_EXP_BOUND_SECS);
        EphemeralGrant {
            nbf,
            exp: requested_exp.min(max_exp),
        }
    }

    /// Production-shaped validity check: the grant is valid iff `now ∈
    /// [nbf, exp)`. A membership-set role *downgrade* does NOT appear here —
    /// the grant survives until `exp` (#60). At R5 this is the benten-caps
    /// UCAN validation path.
    fn is_valid_at(&self, now: u64) -> bool {
        now >= self.nbf && now < self.exp
    }

    fn lifetime_secs(&self) -> u64 {
        self.exp - self.nbf
    }
}

// ── F-MS-8 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-8 — stanza under stale role_assignments_generation rejects with E_ROLE_STALE_AT_VERIFY; un-ignore at R5"]
fn ms8_stale_generation_rejected_at_verify() {
    // Seal at generation G=7, then advance the set to G+1=8.
    let stanza = Stanza {
        sealed_role_assignments_generation: 7,
    };
    // Verify against the advanced generation → rejected with the new code.
    assert_eq!(
        verify_stanza(&stanza, 8),
        Err(RoleStaleError {
            code: E_ROLE_STALE_AT_VERIFY
        }),
        "a stanza sealed under a stale role_assignments_generation rejects at verify (E_ROLE_STALE_AT_VERIFY)"
    );
    // Paired positive control: verifying at the SAME generation succeeds — so
    // the staleness rejection is a real boundary, not a blanket reject.
    assert!(
        verify_stanza(&stanza, 7).is_ok(),
        "a stanza at the CURRENT generation verifies"
    );
}

#[test]
#[ignore = "RED-PHASE: F-MS-8 — E_ROLE_STALE_AT_VERIFY is the canonical code string (catalog mirror); un-ignore at R5"]
fn ms8_error_code_is_canonical() {
    // The new ErrorCode is the load-bearing observable. Pin the exact string so
    // the Rust↔TS catalog mirror (§3.5g) stays in sync.
    let err = verify_stanza(
        &Stanza {
            sealed_role_assignments_generation: 0,
        },
        5,
    )
    .unwrap_err();
    assert_eq!(
        err.code, "E_ROLE_STALE_AT_VERIFY",
        "the role-staleness verify rejection surfaces the canonical E_ROLE_STALE_AT_VERIFY ErrorCode"
    );
}

// ── F-MS-9 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-9 — ephemeral UCAN at role R survives a later downgrade, bounded only by exp (#60); un-ignore at R5"]
fn ms9_role_transition_survives_prior_ucan() {
    // Issue an Admin-role ephemeral grant valid [1000, 1000+3600).
    let grant = EphemeralGrant {
        nbf: 1000,
        exp: 1000 + DEFAULT_EXP_BOUND_SECS,
    };
    // The member is downgraded from Admin → Viewer at the membership-set layer.
    // That downgrade does NOT retroactively invalidate the already-issued
    // ephemeral grant — it remains valid until its own `exp` (#60 honest
    // disclosure). Would-FAIL if a downgrade flipped this to invalid.
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
#[ignore = "RED-PHASE: F-MS-9 — §3.4 tight-exp default keeps the #60 survival window small (issuer CLAMPS over-long requests); un-ignore at R5"]
fn ms9_tight_exp_default_bounds_survival_window() {
    // F4-033: the prior arm constructed `exp = nbf + DEFAULT_EXP_BOUND_SECS`
    // and then asserted `lifetime ≤ DEFAULT_EXP_BOUND_SECS` — true BY
    // CONSTRUCTION (tautological; a no-op issuer passes). Instead, drive the
    // real ISSUER (`issue_bounded`) with an OVER-LONG requested expiry and
    // assert it CLAMPS the survival window down to the default bound. A
    // no-op issuer that honored the over-long request verbatim would FAIL.
    let nbf = 1000;
    let over_long_request = nbf + 10 * DEFAULT_EXP_BOUND_SECS; // 10× too long
    let grant = EphemeralGrant::issue_bounded(nbf, over_long_request);
    assert!(
        grant.lifetime_secs() <= DEFAULT_EXP_BOUND_SECS,
        "the issuer CLAMPS an over-long requested expiry to the §3.4 default bound (tight survival window)"
    );
    assert_eq!(
        grant.exp,
        nbf + DEFAULT_EXP_BOUND_SECS,
        "an over-long request is clamped exactly to nbf + DEFAULT_EXP_BOUND_SECS"
    );

    // A WITHIN-bound request is honored as-is (the issuer is not a blanket
    // clamp-to-max — would-FAIL-if-no-op'd in the other direction).
    let short_request = nbf + 60; // 1 minute, well under the bound
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
