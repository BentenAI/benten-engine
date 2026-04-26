//! Phase 2b R3-B — Named-manifest registry unit tests (G7-A).
//!
//! D2-RESOLVED hybrid:
//!   - HashMap<String, CapBundle> registry populated at construction
//!     from build.rs codegen of host-functions.toml.
//!   - `register_runtime()` API reserved as no-op-with-typed-error in 2b
//!     (returns E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED).
//!   - Phase 8 lifts the deferral.
//!
//! Pin sources: D2-RESOLVED, ESC-15, D9 (canonical DAG-CBOR), wsa-D2.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D2 named manifest resolution"]
fn sandbox_named_manifest_resolves_caps() {
    // Plan §3 G7-A + D2 — `compute-basic` named manifest bundles
    // ["host:compute:time", "host:compute:log"]; lookup returns the
    // bundle; the bundle is what's intersected against the dispatcher's
    // grant.
    //
    // Test:
    //   `ManifestRegistry::new().lookup("compute-basic").unwrap().caps`
    //   == ["host:compute:time", "host:compute:log"] (sorted-canonical).
    todo!("R5 G7-A — assert default registry contains compute-basic + caps");
}

#[test]
#[ignore = "Phase 2b G7-A pending — ESC-15 + D2 unknown manifest"]
fn sandbox_unknown_named_manifest_rejected_e_sandbox_manifest_unknown() {
    // ESC-15 + D2 — `compute-power` is NOT in the registry. Lookup
    // returns Err(ManifestError::Unknown("compute-power")) which routes
    // to E_SANDBOX_MANIFEST_UNKNOWN at the public surface.
    //
    // Anti-pattern: NO permissive fall-through to a default manifest
    // (sec-pre-r1-04 — ESC-15 escape vector). This test enforces that.
    todo!("R5 G7-A — assert lookup of unknown name returns typed error");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D2 + wsa-D2 codegen drift"]
fn sandbox_named_manifest_codegen_drift() {
    // D2 + wsa-D2 — `host-functions.toml` is the source of truth;
    // build.rs codegen emits `generated::DEFAULT_MANIFESTS`. The drift
    // detector mirrors error_code_drift_test pattern: parse the TOML at
    // runtime + compare to the generated table; assert byte-for-byte
    // equality.
    //
    // If TOML is edited without re-running codegen, this test fires
    // before review.
    todo!("R5 G7-A — runtime parse of TOML + compare to generated table");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D9 canonical DAG-CBOR bytes"]
fn sandbox_named_manifest_canonical_bytes_dagcbor() {
    // D9 + D2 — named manifests serialize to canonical DAG-CBOR for
    // CID computation. `compute-basic` manifest's canonical bytes are
    // bit-stable across re-encodes (BTreeMap key ordering).
    //
    // Test:
    //   let bundle = ManifestRegistry::new().lookup("compute-basic")?;
    //   let bytes_1 = bundle.canonical_bytes()?;
    //   let bytes_2 = bundle.canonical_bytes()?;
    //   assert_eq!(bytes_1, bytes_2);
    //   // Stable across crate restarts: pin the SHA256 / BLAKE3 of
    //   // bytes_1 in the test for cross-run drift detection.
    todo!("R5 G7-A — canonical bytes round-trip + pinned hash");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D2 reserved runtime API"]
fn sandbox_register_runtime_returns_e_sandbox_manifest_registration_deferred() {
    // D2-RESOLVED — `register_runtime(name, bundle)` exists in 2b but
    // returns Err(ManifestError::RuntimeRegistrationDeferred), which
    // routes to E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED.
    //
    // API-surface preservation: Phase 8 marketplace work flips
    // `runtime_registration_enabled = true` and lifts the deferral
    // without breaking callers.
    //
    // Test:
    //   let mut reg = ManifestRegistry::new();
    //   let result = reg.register_runtime("custom", CapBundle::default());
    //   assert!(matches!(result, Err(ManifestError::RuntimeRegistrationDeferred)));
    //   // Or via public surface: typed error code matches.
    todo!("R5 G7-A — assert register_runtime returns deferred error");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D2 default bundle present at construction"]
fn sandbox_manifest_registry_default_bundle_loaded_at_construction() {
    // D2 positive — `ManifestRegistry::new()` populates the default
    // entries from `generated::DEFAULT_MANIFESTS` at construction time
    // (NOT lazily on first lookup).
    //
    // Test:
    //   let reg = ManifestRegistry::new();
    //   assert!(reg.lookup("compute-basic").is_ok());
    //   assert!(reg.lookup("compute-with-kv").is_ok());
    //   // Exact set of default names pinned via R5 codegen.
    todo!("R5 G7-A — assert default entries present immediately after new()");
}
