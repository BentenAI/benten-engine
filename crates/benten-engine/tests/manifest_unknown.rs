//! R3-D RED-PHASE pins for `register_subgraph` SANDBOX named-manifest
//! resolution (G17-C wave 5b; phase-3-backlog §6.6).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-C):
//!
//! - `tests/register_subgraph_rejects_unresolved_sandbox_manifest_name_with_e_sandbox_manifest_unknown`
//!   — phase-3-backlog §6.6 deliverable 1
//! - `tests/register_subgraph_resolves_colon_joined_manifest_name`
//!   — phase-3-backlog §6.6
//!
//! ## Named-manifest resolution shape
//!
//! Phase-2b SANDBOX nodes referenced manifests by colon-joined name
//! (`"compute:safe-default"`), but `register_subgraph` did NOT validate
//! that a referenced manifest name resolved to a registered
//! manifest CID. A SANDBOX node referencing
//! `"compute:typo-here"` would silently register and fail at execution.
//!
//! Phase-3 G17-C wires the validation walk in
//! `crates/benten-engine/src/engine.rs::register_subgraph`:
//!
//! 1. Walk the subgraph for SANDBOX nodes.
//! 2. For each SANDBOX node, resolve the `manifest_name` against the
//!    `manifest_registry` (in `engine_modules.rs`).
//! 3. If resolution fails, return typed `E_SANDBOX_MANIFEST_UNKNOWN`.
//!
//! Pairs with G17-C's `register_module_bytes` napi method (see
//! `bindings/napi/tests/register_module_bytes.rs`) — registration is
//! the WRITE side; this is the READ-AND-VALIDATE side.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-C wave 5b wires register_subgraph manifest-name validation walk per phase-3-backlog §6.6"]
fn register_subgraph_rejects_unresolved_sandbox_manifest_name_with_e_sandbox_manifest_unknown() {
    // phase-3-backlog §6.6 deliverable 1 pin. G17-C implementer wires:
    //
    //   use benten_engine::Engine;
    //
    //   let engine = Engine::open_in_memory(/* test config */);
    //
    //   // Build a subgraph that references an UNREGISTERED manifest name:
    //   let subgraph = build_subgraph_with_sandbox_node("compute:typo-here");
    //
    //   // register_subgraph rejects with typed error:
    //   let err = engine.register_subgraph(subgraph).unwrap_err();
    //   assert!(matches!(
    //       err,
    //       benten_engine::EngineError::SandboxManifestUnknown { manifest_name }
    //         if manifest_name == "compute:typo-here"
    //   ));
    //
    //   // Error catalog has the variant:
    //   let catalog = std::fs::read_to_string("docs/ERROR-CATALOG.md").unwrap();
    //   assert!(catalog.contains("E_SANDBOX_MANIFEST_UNKNOWN"),
    //       "ERROR-CATALOG.md must list E_SANDBOX_MANIFEST_UNKNOWN per §6.6 + §3.5b doc-coupling");
    //
    // OBSERVABLE consequence: a misspelled manifest name fails at
    // REGISTRATION TIME (not at execution time, where the failure
    // shape would be a wallclock-after-zero-progress that's harder to
    // diagnose). Defends §6.6 deliverable 1.
    unimplemented!(
        "G17-C wires register_subgraph manifest-validation walk + E_SANDBOX_MANIFEST_UNKNOWN typed error"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-C wave 5b extends manifest_registry to key by colon-joined name per §6.6"]
fn register_subgraph_resolves_colon_joined_manifest_name() {
    // phase-3-backlog §6.6 pin. G17-C implementer wires:
    //
    //   let engine = Engine::open_in_memory(/* test config */);
    //
    //   // Register a manifest by colon-joined name + some bytes:
    //   let manifest_bytes = b"...some valid module bytes..."; // implementer pins fixture
    //   let cid = engine.register_module_bytes("compute:safe-default", manifest_bytes).unwrap();
    //
    //   // Build subgraph referencing that name:
    //   let subgraph = build_subgraph_with_sandbox_node("compute:safe-default");
    //
    //   // Registration succeeds (resolution found the named manifest):
    //   let registered = engine.register_subgraph(subgraph).unwrap();
    //
    //   // The resolved manifest CID is preserved in the registered
    //   // subgraph spec (per pim-2 § 3.6b end-to-end pin):
    //   assert_eq!(
    //       registered.sandbox_node_manifest_cid("compute:safe-default"),
    //       Some(cid)
    //   );
    //
    // OBSERVABLE consequence: a registered manifest can be referenced
    // by name AND the CID is resolved + preserved. Defends §6.6
    // resolution path.
    unimplemented!("G17-C wires colon-joined manifest_registry keying + resolution assertion");
}
