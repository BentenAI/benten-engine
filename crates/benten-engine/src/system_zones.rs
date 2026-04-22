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
/// TODO(phase-2a-G5-B-i): populate the phf table from this const at build
/// time.
pub const SYSTEM_ZONE_PREFIXES: &[&str] = &[
    "system:CapabilityGrant",
    "system:IVMView",
    "system:CapabilityRevocation",
    "system:Principal",
    "system:Grant",
    "system:WaitPending",
    "system:ModuleManifest",
    // IVM view-id namespace prefix. Entries like `system:ivm:content_listing`
    // resolve into built-in views; the prefix is engine-privileged. Added at
    // G1-A per the workspace drift-scan surfacing it; see §9.10 addendum
    // "Additional prefixes are surfaced by a workspace grep".
    "system:ivm:",
];
