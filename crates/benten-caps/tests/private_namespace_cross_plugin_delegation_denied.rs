//! T7a defense pin — cross-plugin namespace escape.
//!
//! Plugin Y attempts to issue itself a UCAN cap for plugin X's
//! `admin-ui-private` namespace. Cap-policy backend MUST refuse —
//! private-namespace scope is structurally `shares: none`.

#[test]
#[ignore = "DESTINATION-REMAPPED at R6-FP-BF per HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW. The substantive cross-plugin private-namespace delegation refusal landed at `crates/benten-caps/src/manifest_envelope_chain_validation.rs::validate_chain_with_manifest_envelope` (R4b-FP-2 closure under §5.5; private-namespace check at idx > 0 step). The standalone `private_namespace_policy::reject_cross_plugin` symbol was never minted — cluster lives at manifest_envelope_chain_validation in benten-caps. Named destination: docs/future/phase-4-backlog.md §4.28 (Phase-4-Meta private-namespace cross-plugin delegation policy substantive arm)."]
fn private_namespace_cap_rejects_cross_plugin_delegation_at_policy_layer() {
    // Substantive test exists at
    // `crates/benten-caps/tests/manifest_envelope_chain_validation_outside_envelope_denied.rs`
    // (sibling test surface). This pin defers; un-ignore + retarget at §4.28 close.
}
