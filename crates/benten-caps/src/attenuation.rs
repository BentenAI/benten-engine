//! Attenuation: does a parent [`GrantScope`] permit every child scope?
//!
//! The check is structural, segment-wise, and intentionally strict:
//!
//! - Scopes split on `':'`.
//! - For each aligned `(parent, child)` segment pair:
//!   - A parent segment of `"*"` that is NOT the last parent segment is a
//!     mid-scope wildcard; it matches the child segment at that index and
//!     the walk continues.
//!   - A parent segment of `"*"` that IS the last parent segment is a
//!     trailing wildcard; it permits the child's remaining tail (including
//!     zero tail segments) and the check succeeds immediately.
//!   - Otherwise the segments must compare equal. A mismatch is a denial.
//! - After walking the aligned prefix:
//!   - `len(child) == len(parent)`: exact match, permitted.
//!   - `len(child)  >  len(parent)` (and the trailing-wildcard case did NOT
//!     fire): the child has fabricated segments the parent never authorized.
//!     Denied.
//!   - `len(child)  <  len(parent)`: the child is claiming authority
//!     *broader* than the parent — denied (attenuation must NARROW, never
//!     widen).
//!
//! # Why this is security-critical
//!
//! An earlier G4 draft used a zip-on-shorter loop that stopped after the
//! parent's last segment without examining the child's tail. That draft had
//! two concrete bypasses:
//!
//! 1. Parent `"*"` (a single `*` segment) accepted any child — including
//!    `"store:post:write:admin:override"` — because only one segment pair
//!    was ever compared.
//! 2. Parent `"store:*"` accepted `"store:anything:write:delete"` because
//!    segments 3 and 4 of the child were never examined.
//!
//! Both are R1 triage threat C3 (delegation-chain widening via wildcard
//! abuse) and are the exact failure mode the `proptest_attenuation.rs`
//! suite exercises.
//!
//! Phase 3 replaces this with a UCAN-style lattice check that treats the
//! scope as a typed resource / action / qualifier triple; Phase 1 is a
//! deliberately simple colon-segmented subset.

use std::cmp::Ordering;

use crate::error::CapError;
use crate::grant::GrantScope;

/// Check that `child_required` is an attenuation (strict subset) of
/// `parent_scope`.
///
/// # Errors
///
/// Returns [`CapError::Attenuation`] whenever the child is not a subset of
/// the parent — segment mismatch, child longer than parent without a
/// trailing wildcard, or child shorter than parent.
///
/// # Examples
///
/// ```
/// use benten_caps::{GrantScope, check_attenuation};
///
/// let parent = GrantScope::parse("store:post:*").unwrap();
/// let child = GrantScope::parse("store:post:read").unwrap();
/// assert!(check_attenuation(&parent, &child).is_ok());
///
/// let parent_strict = GrantScope::parse("store:post:read").unwrap();
/// let child_wider = GrantScope::parse("store:post:write").unwrap();
/// assert!(check_attenuation(&parent_strict, &child_wider).is_err());
/// ```
pub fn check_attenuation(
    parent_scope: &GrantScope,
    child_required: &GrantScope,
) -> Result<(), CapError> {
    let parent_segments: Vec<&str> = parent_scope.as_str().split(':').collect();
    let child_segments: Vec<&str> = child_required.as_str().split(':').collect();

    // Walk aligned pairs. A trailing `*` on the parent short-circuits to
    // permit any remaining child tail; a non-trailing `*` matches one
    // segment and the walk continues.
    for (i, (p_seg, c_seg)) in parent_segments
        .iter()
        .zip(child_segments.iter())
        .enumerate()
    {
        if *p_seg == "*" {
            if i == parent_segments.len() - 1 {
                // Trailing wildcard: the child's full tail is permitted.
                return Ok(());
            }
            // Mid-scope wildcard: consume one child segment, keep walking.
            continue;
        }
        if p_seg != c_seg {
            return Err(CapError::Attenuation);
        }
    }

    // Walked every aligned pair without firing a trailing wildcard.
    match child_segments.len().cmp(&parent_segments.len()) {
        Ordering::Equal => Ok(()),
        // Child fabricated tail segments the parent never named. This is
        // the exact g4-uc-2 bypass the earlier draft missed.
        Ordering::Greater => Err(CapError::Attenuation),
        // Child is SHORTER than parent — child is broader, not an
        // attenuation. Attenuation must only narrow.
        Ordering::Less => Err(CapError::Attenuation),
    }
}
