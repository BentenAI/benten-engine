//! [`NoAuthBackend`] — Phase 1 default capability policy.
//!
//! Zero-cost: [`NoAuthBackend::check_write`] always returns `Ok(())`, no
//! allocations, no branches that depend on the [`crate::WriteContext`] fields.
//! This is the out-of-the-box default for Engine builders so the 10-minute DX
//! path in `docs/QUICKSTART.md` holds.

use crate::error::CapError;
use crate::policy::{CapabilityPolicy, WriteContext};

/// The default zero-auth backend. Permits every write unconditionally.
///
/// `Copy` + `Clone` + `Default`: the type is zero-sized, so all three are
/// trivial and let callers treat it as a value rather than an owned handle.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoAuthBackend;

impl NoAuthBackend {
    /// Construct a new `NoAuthBackend`. Constructor form is kept symmetric
    /// with [`crate::UcanBackend::new`] so swapping backends is a one-token
    /// edit in the builder chain.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Stable pseudo-actor label used when `NoAuthBackend` is the configured
    /// policy and a change-event needs to record an actor. Phase 3 replaces
    /// this with a real principal once `benten-id` lands.
    #[must_use]
    pub fn pseudo_actor_label() -> &'static str {
        "noauth"
    }
}

impl CapabilityPolicy for NoAuthBackend {
    #[inline]
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Ok(())
    }
}
