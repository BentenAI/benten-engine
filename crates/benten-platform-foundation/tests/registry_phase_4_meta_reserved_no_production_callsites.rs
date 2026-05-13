//! Phase-4-Meta-reserved — registry surfaces.
//!
//! Per post-R1-triage ratification #3: decentralized registry →
//! Phase 4-Meta. Phase 4-Foundation v0 uses direct content-addressed-
//! share over Atriums.
//!
//! `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode is RESERVED at Phase 4-
//! Foundation (0 production call sites; carries to Phase 4-Meta).
//! This test pins the surface stub but exercises NO production path.

use benten_errors::ErrorCode;
use benten_platform_foundation::registry::timeout_error_code;

#[test]
fn registry_discover_surface_stub_exists_for_phase_4_meta_with_typed_error_code() {
    // SUBSTANTIVE per pim-2 §3.6b: registry.rs ships
    // `timeout_error_code()` returning `ErrorCode::RegistryDiscoveryTimeout`
    // as the reserved type-anchor for Phase 4-Meta's discover surface.
    // Phase 4-Foundation has 0 production callsites that fire this
    // code (per ERROR-CATALOG.md reachability:ignore annotation).
    // The presence of the type-anchor is what this pin verifies —
    // would-FAIL if a future cleanup pass removed the reserved
    // ErrorCode prematurely before Phase 4-Meta wires the actual
    // discover/timeout path.
    assert_eq!(
        timeout_error_code(),
        ErrorCode::RegistryDiscoveryTimeout,
        "registry::timeout_error_code anchor MUST exist at Phase \
         4-Foundation reserving E_REGISTRY_DISCOVERY_TIMEOUT for \
         Phase 4-Meta wiring"
    );
    // Round-trip via the string form to defend the enum<->string
    // contract against rename drift.
    assert_eq!(
        ErrorCode::RegistryDiscoveryTimeout.as_static_str(),
        "E_REGISTRY_DISCOVERY_TIMEOUT"
    );
    assert!(matches!(
        ErrorCode::from_str("E_REGISTRY_DISCOVERY_TIMEOUT"),
        ErrorCode::RegistryDiscoveryTimeout
    ));
}

#[ignore = "RED-PHASE (Phase 4-Meta wave un-ignores) — \
    Phase 4-Meta wires the actual `registry::discover(plugin_did) -> Vec<Cid>` \
    path with timeout firing E_REGISTRY_DISCOVERY_TIMEOUT. Named destination: \
    phase-4-backlog §3.1 (Phase 4-Foundation → Phase 4-Meta decentralized \
    self-discovered registry carry). HARD RULE 12 clause-(b) BELONGS-NAMED-NOW: \
    that backlog entry pre-exists."]
#[test]
fn registry_discover_fires_typed_timeout_under_network_failure_at_phase_4_meta() {
    // Phase 4-Meta surface (NOT shipped at Phase 4-Foundation):
    //   registry::discover(plugin_did) -> Result<Vec<Cid>, ErrorCode>
    //   under network failure, surfaces ErrorCode::RegistryDiscoveryTimeout
    //   after configured deadline.
    panic!("Phase 4-Meta wires registry::discover with timeout firing");
}
