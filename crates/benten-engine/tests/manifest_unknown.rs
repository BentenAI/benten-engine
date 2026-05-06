//! Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1):
//! `register_subgraph` SANDBOX named-manifest validation walk.
//!
//! Pin sources (per r2-test-landscape §2.5 G17-C):
//!
//! - `tests/register_subgraph_rejects_unresolved_sandbox_manifest_name_with_e_sandbox_manifest_unknown`
//!   — phase-3-backlog §6.6 deliverable 1 (negative-side end-to-end pin)
//! - `tests/register_subgraph_resolves_colon_joined_manifest_name`
//!   — phase-3-backlog §6.6 (positive-side end-to-end pin per pim-2 §3.6b)
//!
//! ## Named-manifest resolution shape
//!
//! Phase-2b SANDBOX nodes referenced manifests by name but
//! `register_subgraph` did NOT validate that a referenced manifest
//! name resolved to a registered manifest entry. A SANDBOX node
//! referencing `"compute:typo-here"` would silently register and fail
//! at execution time with a confusing
//! `wallclock-after-zero-progress` shape.
//!
//! Phase-3 G17-C wires the validation walk in
//! `crates/benten-engine/src/engine.rs::validate_sandbox_manifest_names`
//! (called from `register_subgraph` post-`Subgraph::validate`):
//!
//! 1. Walk the eval Subgraph for SANDBOX nodes.
//! 2. For each SANDBOX node, resolve the `manifest` property (or the
//!    colon-joined `module` property fallback) against
//!    `manifest_registry_known_names()` (codegen defaults +
//!    `<manifest>:<entry>` colon-joined keys + bare `<entry>` keys
//!    from installed modules).
//! 3. If resolution fails, return typed
//!    [`EngineError::SandboxManifestUnknown`] (catalog code
//!    `E_SANDBOX_MANIFEST_UNKNOWN`).
//!
//! Pairs with G17-C's `register_module_bytes` napi method (see
//! `bindings/napi/tests/register_module_bytes.rs`) — module-bytes
//! registration is the WRITE-side; this is the READ-AND-VALIDATE side.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::manifest_signing::ManifestVerifyArgs;
use benten_engine::{Engine, EngineError, PrimitiveSpec, SubgraphSpec};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;

/// Build a SANDBOX subgraph spec that references a manifest by NAME.
/// The `module` property carries a real (BLAKE3-derived) CID so the
/// validation walk only inspects the `manifest` reference; the SANDBOX
/// dispatch path (which we do NOT exercise in these tests) is the
/// only consumer of `module` as a CID.
fn sandbox_spec_named(handler_id: &str, manifest_name: &str) -> SubgraphSpec {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    // A real (well-formed) CID so primitive_host's CID parse never
    // sees a mangled value if the registration walk somehow lets the
    // bad path through. validation walk reads `manifest` first.
    let module_cid = Cid::from_blake3_digest([0xAA; 32]);
    props.insert("module".into(), Value::Text(module_cid.to_base32()));
    props.insert("manifest".into(), Value::Text(manifest_name.to_string()));

    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: props,
        })
        .respond()
        .build()
}

#[test]
fn register_subgraph_rejects_unresolved_sandbox_manifest_name_with_e_sandbox_manifest_unknown() {
    // phase-3-backlog §6.6 deliverable 1 — negative-side end-to-end pin
    // per pim-2 §3.6b. Drives the production `Engine::register_subgraph`
    // entry point with a SANDBOX node carrying an UNREGISTERED manifest
    // name; observes the typed `EngineError::SandboxManifestUnknown`
    // rejection at registration time.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let spec = sandbox_spec_named("h.typo", "compute:typo-here");
    let err = engine.register_subgraph(spec).expect_err(
        "registering a SANDBOX-bearing spec with an UNRESOLVED manifest name MUST fail at \
         registration time per phase-3-backlog §6.6 deliverable 1",
    );

    // Typed variant carries the offending manifest name.
    match &err {
        EngineError::SandboxManifestUnknown { manifest_name } => {
            assert_eq!(
                manifest_name, "compute:typo-here",
                "the rejection MUST surface the misspelled manifest name back to the caller \
                 so the operator knows what to fix; got: {manifest_name}"
            );
        }
        other => panic!(
            "expected EngineError::SandboxManifestUnknown {{ manifest_name: 'compute:typo-here' }}; \
             got: {other:?}"
        ),
    }

    // Catalog code routes through ErrorCode::SandboxManifestUnknown
    // (`E_SANDBOX_MANIFEST_UNKNOWN`) so napi error mapping + drift
    // detector see the typed code.
    assert_eq!(
        err.error_code(),
        ErrorCode::SandboxManifestUnknown,
        "rejection MUST route through ErrorCode::SandboxManifestUnknown for napi/drift parity"
    );
    assert_eq!(
        err.error_code().as_str(),
        "E_SANDBOX_MANIFEST_UNKNOWN",
        "catalog string code MUST match docs/ERROR-CATALOG.md per §3.5b doc-coupling"
    );
}

#[test]
fn register_subgraph_resolves_colon_joined_manifest_name() {
    // phase-3-backlog §6.6 — positive-side end-to-end pin per pim-2
    // §3.6b. Drives the production `Engine::register_subgraph` with a
    // SANDBOX node that references an INSTALLED manifest by colon-
    // joined `<manifest>:<entry>` name; observes the registration
    // SUCCESS (validation walk found the name in the registry overlay).
    //
    // OBSERVABLE consequence: the colon-joined name resolves through
    // the wave-8h `manifest_registry()` overlay extended by G17-C to
    // ALSO key entries by `<manifest_name>:<entry_name>`. A regression
    // that drops the colon-joined keying (or that fails to consult
    // `installed_modules` at all) would fire
    // `E_SANDBOX_MANIFEST_UNKNOWN` here even though the manifest IS
    // installed — exactly the failure shape this pin defends against.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Install a manifest whose single entry is named `identity` under
    // manifest `echo`. The colon-joined surface is `"echo:identity"`.
    let manifest =
        benten_engine::testing::testing_make_manifest_with_caps("echo", &["host:compute:time"]);
    // The helper names the entry `<manifest-name>.handler`; we'll use
    // that as the entry name since the helper doesn't expose direct
    // entry-name control. The colon-joined key is `<manifest>:<entry>`
    // = `"echo:echo.handler"`.
    let expected_cid = benten_engine::testing::testing_compute_manifest_cid(&manifest);
    engine
        .install_module(
            manifest,
            expected_cid,
            ManifestVerifyArgs::unsigned_development(),
        )
        .expect("install_module must succeed when expected_cid matches");

    // Register a SANDBOX subgraph referencing the colon-joined name.
    // Registration MUST succeed (validation walk found the name).
    let spec = sandbox_spec_named("h.echo", "echo:echo.handler");
    let handler_id = engine.register_subgraph(spec).expect(
        "SANDBOX-bearing spec with installed colon-joined manifest name MUST register cleanly \
         per phase-3-backlog §6.6 + the wave-8h manifest_registry() overlay extension",
    );
    assert_eq!(handler_id, "h.echo");

    // Bare-entry-name keying (legacy wave-8h shape) ALSO still works —
    // both keys point at the same CapBundle in the overlay so the
    // SANDBOX dispatch resolution path agrees with the validation walk
    // regardless of which shape the caller composed.
    let spec_bare = sandbox_spec_named("h.echo.bare", "echo.handler");
    let handler_id_bare = engine.register_subgraph(spec_bare).expect(
        "bare-entry-name keying preserved (wave-8h legacy callers + G17-C colon-joined callers \
         must both work end-to-end)",
    );
    assert_eq!(handler_id_bare, "h.echo.bare");
}

#[test]
fn register_subgraph_accepts_sandbox_with_inline_caps_escape_hatch() {
    // Companion regression — a SANDBOX node using the inline `caps`
    // escape hatch (no `manifest` reference + a raw module CID under
    // `module`) MUST register cleanly. The validation walk only
    // inspects named manifest references; it does NOT block the
    // inline-caps path.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    let module_cid = Cid::from_blake3_digest([0xBB; 32]);
    props.insert("module".into(), Value::Text(module_cid.to_base32()));
    // Inline caps — bypasses the named-manifest registry.
    props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );

    let spec = SubgraphSpec::builder()
        .handler_id("h.inline-caps")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: props,
        })
        .respond()
        .build();

    engine.register_subgraph(spec).expect(
        "SANDBOX with inline caps (no named manifest reference) MUST register without \
         tripping the manifest-name validation walk",
    );
}

#[test]
fn register_subgraph_accepts_codegen_default_manifest_names() {
    // The codegen-default registry ships `compute-basic` + `compute-with-kv`;
    // both MUST resolve through the validation walk without any
    // `installed_modules` overlay being populated.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    for name in ["compute-basic", "compute-with-kv"] {
        let spec = sandbox_spec_named(&format!("h.codegen.{name}"), name);
        engine.register_subgraph(spec).unwrap_or_else(|e| {
            panic!(
                "codegen-default manifest '{name}' MUST resolve through the validation walk \
                 without any install_module call; got: {e:?}"
            )
        });
    }
}
