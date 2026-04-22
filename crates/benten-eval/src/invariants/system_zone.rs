//! Phase 2a G5-B-i / E10: Invariant-11 — system-zone breach from user
//! subgraph. Registration-time half lives here (literal-CID reject);
//! runtime half lives in `benten-engine/src/primitive_host.rs` per §9.10.
//!
//! TODO(phase-2a-G5-B-i): walk the subgraph and reject READ/WRITE ops that
//! target a literal `system:*` CID; emit `E_INV_SYSTEM_ZONE`.

use crate::{EvalError, Subgraph};

/// Registration-time literal-CID reject. Fires `E_INV_SYSTEM_ZONE` on
/// violation.
///
/// # Errors
/// Returns [`EvalError`] carrying `ErrorCode::InvSystemZone`.
pub fn validate_registration(_subgraph: &Subgraph) -> Result<(), EvalError> {
    todo!(
        "Phase 2a G5-B-i: implement registration-time literal-CID \
         system-zone reject per plan §9.10"
    )
}

/// Test harness: build a subgraph that reads the given literal label.
/// A label starting with `system:` must be rejected by
/// [`validate_registration`]; non-system labels must pass.
#[must_use]
pub fn build_subgraph_reading_literal_system_cid_for_test(_label: &str) -> Subgraph {
    todo!(
        "Phase 2a G5-B-i: test harness for `invariant_11_static_system_zone_rejected_at_registration`"
    )
}
