//! Structured `Scope` chain validator — enforces chain non-widening
//! (R1 of RATIFIED-S&C 2026-05-21).
//!
//! Distinct from the existing [`crate::chain_authority`] envelope-ceiling
//! seam: this module validates that a delegation chain over the
//! structured-`Scope` language is **monotonically narrowing** — every
//! step's `Scope` must be CONTAINED by its predecessor's. Any widening
//! step is rejected with typed [`ChainValidationError::ChainNotNarrowing`]
//! carrying the offending edge's `step_index`.
//!
//! ## Per-arm narrowing semantics
//!
//! - **`Scope::Hashes(parent)` → `Scope::Hashes(child)`** — `child ⊆ parent`
//!   (every child hash must appear in the parent set). Hash widening
//!   (adding a hash) ⇒ reject.
//! - **`Scope::RestrictedSelector(parent)` → `Scope::RestrictedSelector(child)`** —
//!   `parent.contains(&child)` per the 6-dim AND-composed
//!   [`crate::restricted_spec::RestrictedSpec::contains`].
//! - **Cross-arm** — at v1-beta, cross-arm transitions are rejected as
//!   structurally non-narrowing (a `Hashes`-to-`RestrictedSelector`
//!   transition would require a structural-comparability seam that
//!   Spike H+1.1 deferred to future ratification). Treated as
//!   `ChainNotNarrowing` for now — explicit + symmetric typed-reject.

use crate::scope::Scope;

/// Successful validator outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainValidatorOutcome {
    /// The chain is monotonically narrowing — every step is contained
    /// by its predecessor.
    Admitted,
}

/// Typed chain-validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ChainValidationError {
    /// An empty chain is degenerate — no root spec to anchor narrowing.
    #[error("chain validator received an empty chain")]
    EmptyChain,
    /// A step in the chain WIDENS its predecessor's scope. `step_index`
    /// names the offending edge — the transition from `chain[step_index-1]`
    /// to `chain[step_index]`.
    #[error("chain step {step_index} widens predecessor scope (E_CHAIN_NARROWING_VIOLATION)")]
    ChainNotNarrowing {
        /// 1-based index of the offending edge — `chain[step_index-1]`
        /// is the predecessor, `chain[step_index]` is the widening
        /// successor.
        step_index: usize,
    },
}

/// Validate that a delegation chain over [`Scope`] is monotonically
/// narrowing.
///
/// Returns:
/// - [`ChainValidatorOutcome::Admitted`] on success (single-step
///   chains are trivially admitted — no edges to widen).
/// - [`ChainValidationError::EmptyChain`] for an empty input slice.
/// - [`ChainValidationError::ChainNotNarrowing`] with `step_index` of
///   the first widening edge.
///
/// # Errors
///
/// Returns [`ChainValidationError::EmptyChain`] for an empty input and
/// [`ChainValidationError::ChainNotNarrowing`] on the first widening
/// edge.
pub fn validate_chain_narrowing(
    chain: &[Scope],
) -> Result<ChainValidatorOutcome, ChainValidationError> {
    if chain.is_empty() {
        return Err(ChainValidationError::EmptyChain);
    }
    // Single-step chain: no edges, so no widening possible.
    if chain.len() == 1 {
        return Ok(ChainValidatorOutcome::Admitted);
    }
    for i in 1..chain.len() {
        let parent = &chain[i - 1];
        let child = &chain[i];
        if !scope_contains(parent, child) {
            return Err(ChainValidationError::ChainNotNarrowing { step_index: i });
        }
    }
    Ok(ChainValidatorOutcome::Admitted)
}

/// Returns true iff `child` is no-wider-than `parent` under structured
/// `Scope` semantics. Cross-arm transitions return false (not
/// structurally comparable at v1-beta).
fn scope_contains(parent: &Scope, child: &Scope) -> bool {
    match (parent, child) {
        (Scope::Hashes(parent_set), Scope::Hashes(child_set)) => {
            // Subset check: every child hash must appear in parent.
            child_set.iter().all(|c| parent_set.iter().any(|p| p == c))
        }
        (Scope::RestrictedSelector(parent_spec), Scope::RestrictedSelector(child_spec)) => {
            parent_spec.contains(child_spec)
        }
        // Cross-arm transitions are structurally non-comparable at
        // v1-beta — reject as non-narrowing rather than silently admit.
        (Scope::Hashes(_), Scope::RestrictedSelector(_))
        | (Scope::RestrictedSelector(_), Scope::Hashes(_)) => false,
    }
}
