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
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored. Walks host-functions.toml
//! `[host_fn.<name>]` entries and asserts each entry's `cap_recheck`
//! declaration matches the codegen-emitted `default_host_fns()` table.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::{CapRecheckPolicy, default_host_fns};

#[test]
fn sandbox_host_fn_cap_recheck_codegen_drift_total() {
    // Walk every [host_fn.<name>] in host-functions.toml. Parse the
    // `cap_recheck` field (defaulting to PerCall per D18 fail-secure).
    // Compare to codegen-emitted `default_host_fns()` agreement.
    let toml_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("host-functions.toml"),
    )
    .expect("workspace host-functions.toml must be readable");

    // Parse [host_fn.<name>] sections with their cap_recheck fields.
    // The TOML keys may appear quoted (`[host_fn."kv:read"]`) or
    // unquoted (`[host_fn.time]`); handle both shapes.
    let mut current_name: Option<String> = None;
    let mut toml_recheck: std::collections::BTreeMap<String, CapRecheckPolicy> =
        std::collections::BTreeMap::new();
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[host_fn.") {
            // Strip prefix + trailing `]`. Inner may have surrounding quotes.
            let inner = trimmed
                .trim_start_matches("[host_fn.")
                .trim_end_matches(']');
            let name = inner.trim_matches('"').to_string();
            // Default is PerCall (D18 fail-secure) if entry omits the
            // declaration; pre-populate so we capture undeclared cases.
            toml_recheck.insert(name.clone(), CapRecheckPolicy::PerCall);
            current_name = Some(name);
            continue;
        }
        if trimmed.starts_with('[') {
            // Different section starting (e.g. [manifest.X]).
            current_name = None;
            continue;
        }
        if let Some(ref name) = current_name
            && let Some(rest) = trimmed.strip_prefix("cap_recheck")
        {
            // `trimmed` after the prefix is e.g. ` = "per_boundary"`.
            // Walk past whitespace + `=` + whitespace; strip the
            // surrounding double-quotes from the literal.
            let after_eq: &str = rest.trim_start().strip_prefix('=').unwrap_or(rest);
            let value = after_eq.trim().trim_matches('"').to_string();
            let policy = match value.as_str() {
                "per_call" => CapRecheckPolicy::PerCall,
                "per_boundary" => CapRecheckPolicy::PerBoundary,
                other => panic!(
                    "host-functions.toml [host_fn.{name}] cap_recheck = \
                     {other:?}: unknown variant"
                ),
            };
            toml_recheck.insert(name.clone(), policy);
        }
    }

    // Compare against codegen-emitted table.
    let codegen = default_host_fns();
    assert!(
        !toml_recheck.is_empty(),
        "host-functions.toml MUST declare at least one [host_fn.*] entry"
    );
    for (name, toml_policy) in &toml_recheck {
        let spec = codegen.get(name).unwrap_or_else(|| {
            panic!(
                "host-functions.toml declares [host_fn.{name}] but \
                 codegen-emitted default_host_fns() does NOT carry an \
                 entry — drift detector FIRED"
            )
        });
        assert_eq!(
            &spec.cap_recheck, toml_policy,
            "host_fn {name:?} cap_recheck drift: TOML={:?} codegen={:?}",
            toml_policy, spec.cap_recheck
        );
    }
    for codegen_name in codegen.keys() {
        assert!(
            toml_recheck.contains_key(codegen_name),
            "codegen-emitted host_fn {codegen_name:?} is NOT declared \
             in host-functions.toml — drift detector FIRED"
        );
    }
}
