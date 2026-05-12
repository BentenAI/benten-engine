//! Phase-4-Meta-reserved — registry surfaces.
//!
//! Per post-R1-triage ratification #3: decentralized registry →
//! Phase 4-Meta. Phase 4-Foundation v0 uses direct content-addressed-
//! share over Atriums.
//!
//! `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode is RESERVED at Phase 4-
//! Foundation (0 production call sites; carries to Phase 4-Meta).
//! This test pins the surface stub but exercises NO production path.

#[test]
#[ignore = "RED-PHASE: Phase-4-Meta scope; reserves the surface stub only at Phase 4-Foundation"]
fn registry_discover_surface_stub_exists_for_phase_4_meta() {
    // Future surface (Phase 4-Meta):
    //   registry::discover(plugin_did) -> Result<Vec<Cid>, ErrorCode>
    //   Times out with ErrorCode::RegistryDiscoveryTimeout.
    //
    // At Phase 4-Foundation, this test serves as a placeholder
    // anchoring the future surface name; un-ignore happens at Phase 4-
    // Meta.
    panic!("RED-PHASE: Phase-4-Meta wave wires registry::discover");
}
