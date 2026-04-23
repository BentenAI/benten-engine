//! Phase 2a G5-A / E11: Invariant-13 — immutability (registration-time
//! declaration-layer half; storage-layer runtime half lives in
//! `benten-graph` per plan §9.11).
//!
//! G5-A owns the registration-time static reject of WRITE-to-registered-
//! CID; authoritative enforcement lives in
//! `benten_graph::RedbBackend::put_node_with_context` per the 5-row
//! matrix.

use std::collections::BTreeSet;

use benten_core::{Cid, Value};

use crate::{EvalError, InvariantViolation, PrimitiveKind, Subgraph};

/// Property key on a WRITE [`crate::OperationNode`] whose value, when set
/// to a literal CID already in `registered_cids`, triggers the
/// registration-time Inv-13 reject.
pub const WRITE_TARGET_CID_PROPERTY: &str = "target_cid";

/// Registration-time declaration-layer reject of WRITE-to-registered-CID.
///
/// Walks the subgraph's WRITE [`crate::OperationNode`]s and inspects each
/// one's `target_cid` property. If the property's [`Value`] is a literal
/// text-form CID that parses and is present in `registered_cids`, fires
/// Inv-13. This is the declaration-layer affordance; the authoritative
/// storage-layer enforcement lives in
/// `benten_graph::RedbBackend::put_node_with_context`.
///
/// # Errors
/// Returns [`EvalError::Invariant`] carrying
/// [`InvariantViolation::Immutability`] (maps to
/// [`benten_errors::ErrorCode::InvImmutability`]).
pub fn validate_registration_against(
    subgraph: &Subgraph,
    registered_cids: &BTreeSet<Cid>,
) -> Result<(), EvalError> {
    for op in subgraph.nodes() {
        if op.kind != PrimitiveKind::Write {
            continue;
        }
        let Some(value) = op.property(WRITE_TARGET_CID_PROPERTY) else {
            continue;
        };
        let Some(text) = value_as_text(value) else {
            continue;
        };
        let Ok(cid) = Cid::from_str(text) else {
            continue;
        };
        if registered_cids.contains(&cid) {
            return Err(EvalError::Invariant(InvariantViolation::Immutability));
        }
    }
    Ok(())
}

/// Convenience form used when the caller has no registered-CID set
/// available. Fires Inv-13 for the self-referential case — a WRITE whose
/// `target_cid` literal equals this subgraph's own computed CID is
/// necessarily an immutability violation. For the broader known-
/// registered-CIDs check, see [`validate_registration_against`].
///
/// # Errors
/// Returns [`EvalError::Invariant`] carrying
/// [`InvariantViolation::Immutability`] on a self-referential WRITE.
pub fn validate_registration(subgraph: &Subgraph) -> Result<(), EvalError> {
    let mut singleton = BTreeSet::new();
    if let Ok(own) = subgraph.cid() {
        singleton.insert(own);
    }
    validate_registration_against(subgraph, &singleton)
}

fn value_as_text(value: &Value) -> Option<&str> {
    match value {
        Value::Text(s) => Some(s.as_str()),
        _ => None,
    }
}

/// Test harness — build a minimal [`Subgraph`] whose sole WRITE declares
/// `target_cid` literally set to `target_cid`. Used by R5 unit tests to
/// exercise the declaration-layer reject in isolation from the full DSL.
#[must_use]
pub fn build_subgraph_writing_to_literal_cid_for_test(
    handler_id: &str,
    target_cid: &Cid,
) -> Subgraph {
    use crate::OperationNode;
    let mut sg = Subgraph::new(handler_id);
    let mut write = OperationNode::new("w", PrimitiveKind::Write);
    write.properties.insert(
        WRITE_TARGET_CID_PROPERTY.to_string(),
        Value::Text(target_cid.to_base32()),
    );
    sg.nodes.push(write);
    sg
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;

    fn cid(byte: u8) -> Cid {
        Cid::from_blake3_digest([byte; 32])
    }

    #[test]
    fn validate_registration_against_rejects_write_to_registered_cid() {
        let target = cid(7);
        let sg = build_subgraph_writing_to_literal_cid_for_test("handler_x", &target);
        let mut registered = BTreeSet::new();
        registered.insert(target);
        let err = validate_registration_against(&sg, &registered)
            .expect_err("WRITE to registered CID must reject at declaration layer");
        assert!(matches!(
            err,
            EvalError::Invariant(InvariantViolation::Immutability)
        ));
    }

    #[test]
    fn validate_registration_against_accepts_write_to_unregistered_cid() {
        let target = cid(7);
        let sg = build_subgraph_writing_to_literal_cid_for_test("handler_x", &target);
        let registered = BTreeSet::new();
        validate_registration_against(&sg, &registered)
            .expect("WRITE to unregistered CID must pass the declaration check");
    }

    #[test]
    fn validate_registration_accepts_empty_subgraph() {
        let sg = Subgraph::new("empty");
        validate_registration(&sg).expect("empty subgraph must pass");
    }
}
