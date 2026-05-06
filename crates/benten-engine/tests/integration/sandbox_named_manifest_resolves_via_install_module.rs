//! Phase 2b Wave-8h audit-gap fix #1 — Named-manifest SANDBOX dispatch
//! resolves through `Engine::install_module` persisted state.
//!
//! Pin source:
//! `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
//! "module manifest production lookup" DRIFT verdict.
//!
//! ## Pre-fix behaviour (the bug)
//!
//! `crates/benten-engine/src/primitive_host.rs::execute_sandbox` at
//! lines 759, 770, 810 (pre-wave-8h) constructed
//! `benten_eval::sandbox::ManifestRegistry::new()` per call — a fresh
//! registry pre-loaded with the codegen defaults ONLY. The engine's
//! `installed_modules` active-set (populated by `install_module`) was
//! never consulted at SANDBOX dispatch time. A handler that declared
//! `manifest: "<installed-module-name>"` could never resolve, even
//! after a successful `engine.install_module(manifest, expected_cid)`
//! call: the resolution failed with
//! `SandboxError::ManifestUnknown` → `E_SANDBOX_MANIFEST_UNKNOWN`.
//!
//! ## Post-fix behaviour (this test)
//!
//! Wave-8h adds `Engine::manifest_registry()` projecting
//! `installed_modules` into the registry overlay; the three
//! `ManifestRegistry::new()` callsites switch to
//! `self.manifest_registry()`. A SANDBOX node carrying
//! `manifest: "<entry-name>"` for a `ModuleManifestEntry` from an
//! installed manifest now resolves to that entry's `requires` caps
//! and the dispatch reaches the wasmtime executor.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::manifest_signing::ManifestVerifyArgs;
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

/// Build a SANDBOX subgraph that references the manifest by NAME (not
/// inline `caps: [...]`). Pre-wave-8h this path always errored with
/// `E_SANDBOX_MANIFEST_UNKNOWN`; post-fix it resolves through the
/// engine's installed-modules registry projection.
fn sandbox_spec_with_named_manifest(
    handler_id: &str,
    module_cid_str: &str,
    manifest_name: &str,
) -> SubgraphSpec {
    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str.to_string()));
    sandbox_props.insert("manifest".into(), Value::Text(manifest_name.to_string()));

    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .respond()
        .build()
}

fn trivial_run_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))")
        .expect("trivial run module compiles")
}

fn cid_for_bytes(bytes: &[u8]) -> Cid {
    let digest = *blake3::hash(bytes).as_bytes();
    Cid::from_blake3_digest(digest)
}

/// Wave-8h audit-gap fix #1 — install a module manifest, then dispatch
/// a SANDBOX node referencing one of its `ModuleManifestEntry` names.
/// The dispatch MUST succeed (not error with `E_SANDBOX_MANIFEST_UNKNOWN`).
#[test]
fn sandbox_named_manifest_resolves_via_install_module() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Register module bytes so the wasmtime executor has something to
    // run when the manifest resolves.
    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    // Build a manifest whose single ModuleManifestEntry is named
    // "wave8h-named-handler" with a `requires: ["host:compute:time"]`
    // capability surface. After install, the engine's manifest_registry()
    // projection MUST expose "wave8h-named-handler" as a registry key
    // mapping to a CapBundle whose caps == ["host:compute:time"].
    let manifest = benten_engine::testing::testing_make_manifest_with_caps(
        "wave8h.named-manifest-fix",
        &["host:compute:time"],
    );
    // The helper names the entry `<manifest-name>.handler` per
    // `crates/benten-engine/src/testing.rs::testing_make_manifest_with_caps`.
    let entry_name = "wave8h.named-manifest-fix.handler";
    let expected_cid = benten_engine::testing::testing_compute_manifest_cid(&manifest);
    engine
        .install_module(
            manifest,
            expected_cid,
            ManifestVerifyArgs::unsigned_development(),
        )
        .expect("install_module must succeed when expected_cid matches the canonical CID");

    // Register the SANDBOX subgraph that references the manifest BY NAME.
    let spec = sandbox_spec_with_named_manifest(
        "sandbox.named_manifest_resolves_via_install_module",
        &module_cid_str,
        entry_name,
    );
    let handler_id = engine
        .register_subgraph(spec)
        .expect("SANDBOX-bearing SubgraphSpec with named manifest must register cleanly");

    // Dispatch through the production path. Pre-wave-8h this would
    // error with `E_SANDBOX_MANIFEST_UNKNOWN` because execute_sandbox
    // built a fresh `ManifestRegistry::new()` (codegen defaults only)
    // and "wave8h.named-manifest-fix.handler" is not a codegen default.
    // Post-wave-8h, `Engine::manifest_registry()` projects
    // `installed_modules` into the registry overlay so the entry
    // resolves to its `requires` caps.
    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect(
            "Named-manifest SANDBOX dispatch MUST succeed after the manifest \
             is installed via Engine::install_module — the wave-8h audit-gap \
             fix wires manifest_registry() into the dispatch path",
        );
    assert!(
        outcome.is_ok_edge(),
        "outcome must route through the OK edge after the executor \
         returns SandboxResult cleanly; got edge {:?} error_code {:?} \
         error_message {:?}",
        outcome.edge_taken(),
        outcome.error_code(),
        outcome.error_message(),
    );
}

/// Companion regression test — a Named-manifest SANDBOX referencing a
/// name that is NEITHER a codegen-default NOR an installed-module
/// entry MUST surface `E_SANDBOX_MANIFEST_UNKNOWN`. Without this
/// assertion the wave-8h fix could regress to a permissive
/// fall-through and we wouldn't catch it.
#[test]
fn sandbox_named_manifest_truly_unknown_still_errors() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    // No install_module call — the registry overlay is empty.
    let spec = sandbox_spec_with_named_manifest(
        "sandbox.named_manifest_truly_unknown_still_errors",
        &module_cid_str,
        "wave8h.never-installed",
    );
    let handler_id = engine.register_subgraph(spec).unwrap();

    let outcome = engine.call(
        &handler_id,
        "run",
        benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
    );

    // The dispatch MUST error (manifest unknown). Either the call returns
    // Err, OR the outcome routes through a non-OK edge with the
    // E_SANDBOX_MANIFEST_UNKNOWN code. Both paths are acceptable so long
    // as the registry-overlay fix did NOT introduce a permissive
    // fall-through.
    match outcome {
        Err(e) => {
            let s = format!("{e:?}");
            assert!(
                s.contains("ManifestUnknown") || s.contains("E_SANDBOX_MANIFEST_UNKNOWN"),
                "an unknown-named manifest SANDBOX dispatch MUST error \
                 with the typed manifest-unknown variant; got: {s}"
            );
        }
        Ok(o) => {
            assert!(
                !o.is_ok_edge(),
                "an unknown-named manifest SANDBOX dispatch MUST NOT route \
                 through the OK edge — that would mean the wave-8h fix \
                 introduced a permissive fall-through. Got outcome: {o:?}"
            );
        }
    }
}
