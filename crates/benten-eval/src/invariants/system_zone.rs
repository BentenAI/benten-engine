//! Phase 2a G5-B-i / E10: Invariant-11 — system-zone breach from user
//! subgraph. Registration-time half lives here (literal-CID reject);
//! runtime half lives in `benten-engine/src/primitive_host.rs` per §9.10.
//!
//! The registration-time walker scans a Subgraph's READ and WRITE operation
//! nodes for literal system-zone targets. Two surfaces encode the target:
//!
//! 1. The node's `"label"` property — the idiomatic path used by the
//!    TypeScript DSL wrapper and the engine's CRUD synthesis (see
//!    `primitives/{read,write}.rs::execute`).
//! 2. The node's `id` field — the legacy builder path exercised by
//!    `SubgraphBuilder::{read,write}(literal)` and by the engine-side
//!    stopgap co-existence test (`system_zone_stopgap_and_full_coexist.rs`).
//!
//! Either surface carrying a `system:*` literal triggers
//! `InvariantViolation::SystemZone` → `E_INV_SYSTEM_ZONE`. The prefix list
//! is the Phase-2a frozen const `SYSTEM_ZONE_PREFIXES` living in
//! `benten-engine::system_zones` (imported here via a hardcoded mirror to
//! keep `benten-eval` free of an upstream engine dependency — a dedicated
//! drift test in the engine crate asserts the two lists stay in sync).

use std::collections::BTreeMap;

use benten_core::Value;

use crate::{
    EvalError, InvariantViolation, OperationNode, PrimitiveKind, RegistrationError, Subgraph,
};

/// Phase-2a frozen mirror of `benten_engine::system_zones::SYSTEM_ZONE_PREFIXES`.
///
/// Held here (rather than imported) so `benten-eval` stays free of a
/// `benten-engine` dep — the arch-1 thinning goal. A drift test in the
/// engine crate
/// (`tests/inv_11_system_zone_drift_test::all_system_zone_writers_registered_in_prefix_table`)
/// + the CI workflow keep the two lists consistent.
pub const SYSTEM_ZONE_PREFIXES_EVAL_MIRROR: &[&str] = &[
    "system:CapabilityGrant",
    "system:IVMView",
    "system:CapabilityRevocation",
    "system:Principal",
    "system:Grant",
    "system:WaitPending",
    "system:WaitResume",
    "system:ModuleManifest",
    "system:ivm:",
];

/// Return `true` when `label` falls inside a Phase-2a system-zone prefix.
///
/// The Phase-2a classification matches the Phase-1 storage-layer stopgap
/// (`benten_graph::guard_system_zone_node`) — **every `system:*`-prefixed
/// label is reserved** for engine internals. This keeps the registration-
/// time walker, the runtime probe, and the graph storage guard aligned on
/// a single deniable set (`both_paths_agree_on_deniable_set`); a user-
/// declared `system:internal:forbidden` must route through Inv-11 even
/// though the specific prefix isn't listed in
/// [`SYSTEM_ZONE_PREFIXES_EVAL_MIRROR`].
///
/// [`SYSTEM_ZONE_PREFIXES_EVAL_MIRROR`] remains documented as the list
/// of concrete system zones the engine itself writes; the
/// `inv_11_system_zone_drift_test` CI guard uses it to ensure every
/// engine-side `system:*` literal is enumerated there. Classification —
/// the shape of what counts as a system-zone label — is intentionally
/// broader so unknown-but-still-`system:`-prefixed labels are rejected,
/// not accidentally allowed.
#[must_use]
pub fn is_system_zone_label(label: &str) -> bool {
    label.starts_with("system:")
}

/// Extract the literal target label the walker should probe for a given
/// operation node. READ / WRITE nodes carry the label via a `"label"`
/// property (idiomatic DSL path) or via the node id (legacy builder). We
/// probe both so either shape routes through Inv-11.
fn literal_target_label(op: &OperationNode) -> Option<&str> {
    if let Some(Value::Text(s)) = op.properties.get("label")
        && !s.is_empty()
    {
        return Some(s.as_str());
    }
    // Fallback: the node id. `SubgraphBuilder::{read,write}(literal)`
    // stores the literal directly in `id`, and the coexist-tests
    // exercise that shape against the registration-time gate.
    Some(op.id.as_str())
}

/// Registration-time literal-CID reject. Fires `E_INV_SYSTEM_ZONE` on
/// violation.
///
/// Walks every READ / WRITE operation node in `sg` and rejects when the
/// literal target label (from either the `"label"` property or the node
/// id) falls within a Phase-2a system-zone prefix. Other primitives
/// (TRANSFORM, CALL, etc.) are not reachable from a literal target-label
/// encoding; the runtime probe in `benten-engine/src/primitive_host.rs`
/// closes the TRANSFORM-computed-CID channel per Code-as-graph Major #1.
///
/// # Errors
/// Returns [`EvalError::Invariant`] carrying
/// [`InvariantViolation::SystemZone`] (→ `E_INV_SYSTEM_ZONE`).
pub fn validate_registration(sg: &Subgraph) -> Result<(), EvalError> {
    for node in &sg.nodes {
        if !matches!(node.kind, PrimitiveKind::Read | PrimitiveKind::Write) {
            continue;
        }
        let Some(label) = literal_target_label(node) else {
            continue;
        };
        if is_system_zone_label(label) {
            return Err(EvalError::Invariant(InvariantViolation::SystemZone));
        }
    }
    Ok(())
}

/// Registration-time rejection routed through the `RegistrationError`
/// diagnostic envelope used by `SubgraphBuilder::build_validated` +
/// `validate_subgraph`. The [`validate_registration`] entry point returns
/// the thinner `EvalError` so direct callers (and the R3 unit test) can
/// match on the typed variant; the builder path prefers this shape so the
/// error funnels through the same `RegistrationError` that other
/// registration-time invariants produce.
///
/// # Errors
/// Returns a [`RegistrationError`] carrying [`InvariantViolation::SystemZone`]
/// when any READ/WRITE operation node declares a `system:*`-prefixed
/// literal label or id.
pub(crate) fn validate_registration_with_diagnostics(
    sg_nodes: &[OperationNode],
) -> Result<(), RegistrationError> {
    for node in sg_nodes {
        if !matches!(node.kind, PrimitiveKind::Read | PrimitiveKind::Write) {
            continue;
        }
        let Some(label) = literal_target_label(node) else {
            continue;
        };
        if is_system_zone_label(label) {
            let mut err = RegistrationError::new(InvariantViolation::SystemZone);
            err.fanout_node_id = Some(node.id.clone());
            return Err(err);
        }
    }
    Ok(())
}

/// Test harness: build a subgraph that reads the given literal label.
/// A label starting with `system:` must be rejected by
/// [`validate_registration`]; non-system labels must pass.
///
/// The helper threads the literal via the operation-node's `"label"`
/// property — the idiomatic DSL path — so the walker probes both channels
/// end-to-end. The handler id is a fixed string; tests that care about
/// naming can build their own subgraph.
#[must_use]
pub fn build_subgraph_reading_literal_system_cid_for_test(label: &str) -> Subgraph {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("label".to_string(), Value::text(label.to_string()));
    let read_node = OperationNode {
        id: format!("read_{label}"),
        kind: PrimitiveKind::Read,
        properties: props,
    };
    Subgraph {
        nodes: vec![read_node],
        edges: Vec::new(),
        handler_id: "inv11_literal_read_fixture".to_string(),
        deterministic: false,
    }
}
