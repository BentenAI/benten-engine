//! Phase 2b R3-C (consolidated) — D18 host-fn `cap_recheck` codegen
//! drift detector (G7-A).
//!
//! Cross-territory dedup: the named-manifest codegen drift test lives in
//! R3-B's `sandbox_named_manifest.rs` (R2 §1.3-aligned, single consolidated
//! file). This file was previously a duplicate of that test; consolidation
//! kept only the unique companion `sandbox_host_fn_cap_recheck_codegen_drift_total`
//! drift detector per `r3-consolidation.md` §2 item 1.
//!
//! Pin sources: D18-RESOLVED hybrid `cap_recheck` policy; wsa D18 drift
//! detector recommendation; r2-test-landscape.md §1.3 sibling row.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_eval::sandbox::manifest::MANIFEST_DEFAULT_BUNDLE
//       (codegen-emitted const HashMap<&'static str, CapBundle>)
//   benten_eval::sandbox::manifest::CapBundle
//   benten_caps::CapScope
//
// The TOML source path:
//   <workspace_root>/host-functions.toml
//   [manifest."compute-basic"] caps = ["host:compute:time", "host:compute:log"]
//   [manifest."compute-with-kv"] caps = [..., "host:compute:kv:read"]

#[test]
#[ignore = "Phase 2b G7-A pending — D2 hybrid + wsa D18 cap_recheck drift"]
fn sandbox_host_fn_cap_recheck_codegen_drift_total() {
    // Companion drift detector — every [host_fn.<name>] entry's
    // `cap_recheck` field MUST round-trip through codegen.
    //
    // R5 wires:
    //   1. Walk every [host_fn.<name>] in host-functions.toml.
    //   2. For each entry, assert codegen-emitted CapRecheckPolicy
    //      matches the TOML declaration (default PerCall when absent —
    //      sibling test
    //      `sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call`
    //      pins the default semantics).
    //   3. Assert: no codegen CapRecheckPolicy variants exist beyond
    //      what TOML declared.
    //
    // Companion shape to the named-manifest drift; covers the
    // per-host-fn cap_recheck cadence policy (D18) which is the other
    // load-bearing drift surface.
    todo!(
        "R5 G7-A — walk host-functions.toml [host_fn.*] cap_recheck + assert \
         codegen agreement"
    );
}
