//! Ephemeral UCAN grants for role-bounded access (F-MS-9 / #60).
//!
//! An ephemeral UCAN issued at role `R` **survives a later role downgrade** —
//! it is bounded only by its own `exp` (it is NOT retroactively revoked by the
//! membership-set role change; #60 honest disclosure). The §3.4 default is a
//! tight `exp` ([`DEFAULT_EXP_BOUND_SECS`]) so the survival window is small;
//! the issuer CLAMPS an over-long requested expiry down to the default bound.

/// The §3.4 default bound on ephemeral-grant lifetime (seconds). Tight by
/// default (1 hour) so the #60 survival window stays small.
pub const DEFAULT_EXP_BOUND_SECS: u64 = 3600;

/// An ephemeral UCAN grant issued at a role, bounded by `nbf`/`exp` only. At
/// v1-beta this composes against the `benten-caps` UCAN validation path; the
/// canary surface is the lifetime + validity contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EphemeralGrant {
    /// Not-before (seconds).
    pub nbf: u64,
    /// Expiry (seconds; exclusive — the grant is valid on `[nbf, exp)`).
    pub exp: u64,
}

impl EphemeralGrant {
    /// Issue an ephemeral grant, ENFORCING the §3.4 tight-exp default: a
    /// requested expiry is CLAMPED to `nbf + DEFAULT_EXP_BOUND_SECS` so the #60
    /// survival window stays small regardless of what the caller asked for. A
    /// within-bound request is honored verbatim (NOT force-extended).
    #[must_use]
    pub fn issue_bounded(nbf: u64, requested_exp: u64) -> Self {
        let max_exp = nbf.saturating_add(DEFAULT_EXP_BOUND_SECS);
        EphemeralGrant {
            nbf,
            exp: requested_exp.min(max_exp),
        }
    }

    /// Whether the grant is valid at `now` (`now ∈ [nbf, exp)`). A
    /// membership-set role *downgrade* does NOT appear here — the grant
    /// survives until `exp` (#60).
    #[must_use]
    pub fn is_valid_at(&self, now: u64) -> bool {
        now >= self.nbf && now < self.exp
    }

    /// The grant's lifetime in seconds (`exp - nbf`).
    #[must_use]
    pub fn lifetime_secs(&self) -> u64 {
        self.exp.saturating_sub(self.nbf)
    }
}

/// Issue an ephemeral grant for a role, clamped to the §3.4 default bound.
/// Convenience wrapper over [`EphemeralGrant::issue_bounded`].
#[must_use]
pub fn ephemeral_grant(nbf: u64, requested_exp: u64) -> EphemeralGrant {
    EphemeralGrant::issue_bounded(nbf, requested_exp)
}
