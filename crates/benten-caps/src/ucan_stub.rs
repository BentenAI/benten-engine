//! [`UcanBackend`] — Phase 1 stub.
//!
//! The full UCAN backend lands with `benten-id` in Phase 3. Today the type
//! exists so:
//!
//! - the trait shape is exercised against a second backend (not just
//!   [`crate::NoAuthBackend`]),
//! - operators who wire `UcanBackend` in a config file receive a clean
//!   [`crate::CapError::NotImplemented`] with a message that names Phase 3
//!   and the interim alternative, instead of a silent misbehavior.
//!
//! The error-routing contract (must surface as `ON_ERROR`, not `ON_DENIED`)
//! is tested in `tests/ucan_stub_messages.rs` — the evaluator (G6) honors it.

use crate::error::CapError;
use crate::policy::{CapabilityPolicy, WriteContext};

/// UCAN capability backend stub. Every `check_write` call returns
/// [`CapError::NotImplemented`] with `backend = "UCANBackend"` and
/// `lands_in_phase = 3` so the displayed message names both the backend and
/// the scheduled landing phase.
#[derive(Debug, Default, Clone, Copy)]
pub struct UcanBackend;

impl UcanBackend {
    /// Construct a UCAN backend stub.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl CapabilityPolicy for UcanBackend {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Err(CapError::NotImplemented {
            backend: "UCANBackend",
            lands_in_phase: 3,
        })
    }
}
