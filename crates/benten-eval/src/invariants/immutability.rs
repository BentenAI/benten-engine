//! Phase 2a G5-A / E11: Invariant-13 — immutability (registration-time
//! half; storage-layer runtime half lives in `benten-graph`).
//!
//! TODO(phase-2a-G5-A): reject WRITE primitives that target a registered-
//! subgraph CID.

use crate::{EvalError, Subgraph};

/// Registration-time structural reject of WRITE-to-registered-subgraph.
///
/// # Errors
/// Returns [`EvalError`] carrying `ErrorCode::InvImmutability`.
pub fn validate_registration(_subgraph: &Subgraph) -> Result<(), EvalError> {
    todo!(
        "Phase 2a G5-A: implement registration-time Inv-13 WRITE-to-registered \
         subgraph reject per plan §9.11"
    )
}
