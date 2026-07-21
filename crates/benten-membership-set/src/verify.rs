//! Stanza verify: the `role_assignments_generation` staleness gate (data-half
//! MODEL — [`verify_stanza`] has **zero production callers** at HEAD; the LIVE
//! staleness enforcement is the `benten_drop::layer_c` open-side recompute per
//! the crate-root disclosure in `lib.rs`).
//!
//! In this model a stanza sealed under a stale `role_assignments_generation` is
//! rejected with the ErrorCode `E_ROLE_STALE_AT_VERIFY` (R0 §2.5 BC-5 /
//! §3.10 / §3.6.B). The `role_assignments_generation` counter is the 11th
//! field of the `0x6610` group AAD (the BLINDED 11-field set — F-AAD-2), so a
//! stanza sealed at generation `G` fails AEAD-open / verify once the set
//! advances to `G+1`.

use crate::error::{E_ROLE_STALE_AT_VERIFY, MembershipSetError};

/// A stanza sealed under a particular role-assignments generation (the AAD
/// field). At v1-beta [`verify_stanza`] is a **data-half role-staleness model
/// with zero production callers** (see the crate-root disclosure at
/// `lib.rs`); the LIVE role-staleness / generation-freshness enforcement is the
/// `benten_drop::layer_c` open-side recompute (`open_group_stanza` /
/// `open_membership_set_group` re-derive the key-epoch generation from the
/// recipient's INDEPENDENTLY-held set-state and fail-close the hybrid LAMPS
/// verify), NOT this standalone comparator. The canary surface here is the
/// generation comparison only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stanza {
    /// The `role_assignments_generation` the stanza was sealed under.
    pub sealed_role_assignments_generation: u32,
}

impl Stanza {
    /// Construct a stanza sealed at a given role-assignments generation.
    #[must_use]
    pub fn sealed_at_generation(generation: u32) -> Self {
        Stanza {
            sealed_role_assignments_generation: generation,
        }
    }
}

/// The role-staleness verify rejection — carries the canonical
/// `E_ROLE_STALE_AT_VERIFY` ErrorCode (the §3.5g-minted catalog code).
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "a stanza sealed under a stale role_assignments_generation was rejected at verify ({code})"
)]
pub struct RoleStaleError {
    /// The canonical wire string — `E_ROLE_STALE_AT_VERIFY`.
    pub code: &'static str,
}

impl RoleStaleError {
    /// The role-staleness rejection with its canonical code string.
    #[must_use]
    pub fn new() -> Self {
        RoleStaleError {
            code: E_ROLE_STALE_AT_VERIFY,
        }
    }

    /// The crate-local typed error this rejection maps to.
    #[must_use]
    pub fn as_membership_error(&self) -> MembershipSetError {
        MembershipSetError::RoleStaleAtVerify
    }
}

impl Default for RoleStaleError {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify a stanza against the set's CURRENT role-assignments generation. A
/// stanza whose sealed generation is OLDER than the current generation is
/// rejected — the generation is AAD-bound, so a post-rotation stanza fails
/// AEAD-open / verify.
///
/// # Errors
///
/// Returns [`RoleStaleError`] (`E_ROLE_STALE_AT_VERIFY`) if the stanza's sealed
/// generation is stale (`< current_generation`).
pub fn verify_stanza(stanza: &Stanza, current_generation: u32) -> Result<(), RoleStaleError> {
    if stanza.sealed_role_assignments_generation < current_generation {
        return Err(RoleStaleError::new());
    }
    Ok(())
}
