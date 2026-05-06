//! Phase 2a G1-A / §9.10: `SYSTEM_ZONE_PREFIXES` const — FROZEN interface.
//!
//! PascalCase prefixes matching HEAD. Every `system:Label`-style literal in
//! `crates/**/*.rs` MUST appear here; CI workflow
//! `.github/workflows/inv-11-system-zone-drift.yml` enforces that contract.
//!
//! Imported by the `benten-eval` registration-time check, the `benten-engine`
//! runtime check, and the `benten-graph` storage stopgap.

/// Phase-2a frozen list of system-zone label prefixes. The const is
/// intentionally a slice — test grep + phf-table-codegen both consume it as
/// an ordered-by-insertion sequence.
///
/// TODO(phase-3 — system-zone phf-table codegen): populate the phf
/// table from this const at build time. Carried from Phase-2a G5-B-i
/// (didn't land); pairs with the Phase-3 first-wave CI-hygiene pass.
pub const SYSTEM_ZONE_PREFIXES: &[&str] = &[
    "system:CapabilityGrant",
    "system:IVMView",
    "system:CapabilityRevocation",
    "system:Principal",
    "system:Grant",
    "system:WaitPending",
    "system:WaitResume",
    "system:ModuleManifest",
    // Phase 2b G10-B: complementary uninstall-side label. Mirrors the
    // `CapabilityRevocation` ↔ `CapabilityGrant` pairing — uninstall
    // writes a revocation Node so a Phase-3 sync replica that has only
    // seen the revocation can still recognize the manifest as
    // uninstalled.
    "system:ModuleManifestRevocation",
    // Phase-3 G14-C wave-4b — Compromise #17 closure. Durable
    // module-bytes side-table written by `Engine::register_module_bytes`
    // through `RedbBlobBackend`; rehydrated at engine open via
    // `Engine::rehydrate_module_bytes_from_zone`.
    "system:ModuleBytes",
    // Phase-3 G14-C wave-4b — Compromise #18 closure. Durable
    // handler-version-chain side-table written by
    // `Engine::register_subgraph` / `Engine::register_subgraph_replace`
    // via `Engine::persist_handler_version_entry`; rehydrated at engine
    // open via `Engine::rehydrate_handler_version_chains_from_zone`.
    "system:HandlerVersion",
    // Phase-3 G14-C wave-4b — Compromise #21 closure. Durable
    // publisher-key registry consulted by `verify_manifest_dual` as the
    // fallback authentication path. Mutations require UCAN delegation
    // rooted at the registry-admin DID per crypto-minor-5.
    "system:PublisherRegistry",
    // Phase 3 R5 wave-3 G13-C BrowserBackend: critical-event browser cache
    // surface. Used by browser_backend.rs put_node_with_context-bypass
    // path for `system:Critical` events propagated to thin-client cache.
    "system:Critical",
    // IVM view-id namespace prefix. Entries like `system:ivm:content_listing`
    // resolve into built-in views; the prefix is engine-privileged. Added at
    // G1-A per the workspace drift-scan surfacing it; see §9.10 addendum
    // "Additional prefixes are surfaced by a workspace grep".
    "system:ivm:",
    // R6-R3-FP Group A: canonical-view label literals registered by
    // `benten-ivm::algorithm_b::CANONICAL_HARDCODED_LABELS` (see
    // `hardcoded_label_for_id`). Added here so the Inv-11 system-zone
    // drift detector (`.github/workflows/inv-11-system-zone-drift.yml` +
    // `crates/benten-engine/tests/inv_11_system_zone_drift_test.rs`)
    // recognizes them as engine-privileged. Mirrors the
    // `system:CapabilityGrant` registration that pairs with
    // `capability_grants` view id.
    "system:EventDispatch",
    "system:GovernanceInheritance",
];
