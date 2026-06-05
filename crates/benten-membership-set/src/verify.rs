//! Stanza verify — `role_assignments_generation` staleness rejection (F-MS-8).
//!
//! A stanza sealed under a stale `role_assignments_generation` (the 11th field
//! of the `0x6610` group AAD) is **rejected at verify** with the
//! [`benten_errors::ErrorCode::RoleStaleAtVerify`] code
//! (`E_ROLE_STALE_AT_VERIFY`). Sealing at generation `G` and advancing the set
//! to `G+1` invalidates the older stanza at verify time (the generation is
//! AAD-bound, so a post-rotation stanza fails AEAD-open / verify).

use benten_errors::ErrorCode;

/// A stanza sealed under a particular role-assignments generation (the AAD
/// field bound at seal time).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stanza {
    /// The `role_assignments_generation` the stanza was sealed under.
    pub sealed_role_assignments_generation: u32,
}

impl Stanza {
    /// Construct a stanza sealed under `generation`.
    #[must_use]
    pub const fn sealed_under(generation: u32) -> Self {
        Self {
            sealed_role_assignments_generation: generation,
        }
    }
}

/// The role-staleness verify rejection — carries the canonical
/// [`ErrorCode::RoleStaleAtVerify`] (`E_ROLE_STALE_AT_VERIFY`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleStaleError {
    /// The catalog code (`E_ROLE_STALE_AT_VERIFY`).
    pub code: ErrorCode,
}

impl RoleStaleError {
    /// The canonical wire-string of this error (`E_ROLE_STALE_AT_VERIFY`).
    #[must_use]
    pub fn code_str(&self) -> &'static str {
        self.code.as_static_str()
    }
}

/// Verify a stanza against the set's CURRENT `role_assignments_generation`.
///
/// A stanza whose sealed generation is **older** than the current generation is
/// rejected with [`ErrorCode::RoleStaleAtVerify`] (the generation is AAD-bound,
/// so a post-rotation stanza fails verify). Verifying at the SAME (or a future)
/// generation succeeds.
///
/// # Errors
///
/// Returns [`RoleStaleError`] (code `E_ROLE_STALE_AT_VERIFY`) when
/// `stanza.sealed_role_assignments_generation < current_generation`.
pub fn verify_stanza(stanza: &Stanza, current_generation: u32) -> Result<(), RoleStaleError> {
    if stanza.sealed_role_assignments_generation < current_generation {
        return Err(RoleStaleError {
            code: ErrorCode::RoleStaleAtVerify,
        });
    }
    Ok(())
}
