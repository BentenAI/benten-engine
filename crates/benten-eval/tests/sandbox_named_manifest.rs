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
//!
//! **cr-g7a-mr-1 fix-pass:** all 6 tests in this file FLIPPED from
//! `#[ignore]` `todo!()` to live assertions. Surfaces are landed in
//! `crates/benten-eval/src/sandbox/manifest.rs` (G7-A PR #30).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_errors::ErrorCode;
use benten_eval::sandbox::{CapBundle, ManifestError, ManifestRegistry, default_manifests};

#[test]
fn sandbox_named_manifest_resolves_caps() {
    // Plan §3 G7-A + D2 — `compute-basic` named manifest bundles
    // ["host:compute:log", "host:compute:time"] (sorted-canonical for
    // DAG-CBOR stability per D9). Lookup returns the bundle; the
    // bundle is what's intersected against the dispatcher's grant.
    let bundle = ManifestRegistry::new()
        .lookup("compute-basic")
        .unwrap()
        .clone();
    assert_eq!(
        bundle.caps,
        vec![
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ]
    );
}

#[test]
fn sandbox_unknown_named_manifest_rejected_e_sandbox_manifest_unknown() {
    // ESC-15 + D2 — `compute-power` is NOT in the registry. Lookup
    // returns Err(ManifestError::Unknown("compute-power")) which routes
    // to E_SANDBOX_MANIFEST_UNKNOWN at the public surface.
    //
    // Anti-pattern: NO permissive fall-through to a default manifest
    // (sec-pre-r1-04 — ESC-15 escape vector). This test enforces that.
    let reg = ManifestRegistry::new();
    let err = reg.lookup("compute-power").unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxManifestUnknown);
    assert!(matches!(err, ManifestError::Unknown { name } if name == "compute-power"));
}

#[test]
fn sandbox_named_manifest_codegen_drift() {
    // D2 + wsa-D2 — `host-functions.toml` is the source of truth;
    // the hand-mirrored `default_manifests()` table in
    // `crates/benten-eval/src/sandbox/manifest.rs` MUST agree with
    // every `[manifest.<name>]` entry in the TOML.
    //
    // **G20-A1 wave-8a body** (Phase 3): walks the TOML at runtime
    // and compares to `default_manifests()` output. A regression where
    // someone adds a manifest to TOML without mirroring it (or vice
    // versa) fires this drift detector.
    let toml_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("host-functions.toml");
    let toml_src = std::fs::read_to_string(&toml_path)
        .expect("workspace host-functions.toml must be readable");

    // Parse out [manifest.<name>] entries + their `caps = [...]`.
    let mut toml_manifests: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut current_name: Option<String> = None;
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[manifest.") {
            // [manifest.compute-basic] → "compute-basic"
            let name = trimmed
                .trim_start_matches("[manifest.")
                .trim_end_matches(']')
                .to_string();
            current_name = Some(name);
            continue;
        }
        if let Some(ref name) = current_name
            && trimmed.starts_with("caps = [")
        {
            // Parse: caps = ["host:compute:log", "host:compute:time"]
            let inner = trimmed.trim_start_matches("caps = [").trim_end_matches(']');
            let caps: Vec<String> = inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            toml_manifests.insert(name.clone(), caps);
            current_name = None;
        }
    }

    // Compare to the codegen-emitted table.
    let codegen = default_manifests();
    for (name, toml_caps) in &toml_manifests {
        let bundle = codegen.get(name).unwrap_or_else(|| {
            panic!(
                "TOML declares manifest {name:?} but codegen-emitted \
                 default_manifests() does NOT carry it — drift detector \
                 (sandbox_named_manifest_codegen_drift) FIRED"
            )
        });
        assert_eq!(
            &bundle.caps, toml_caps,
            "manifest {name:?} caps drift: TOML={:?} codegen={:?}",
            toml_caps, bundle.caps
        );
    }
    for codegen_name in codegen.keys() {
        assert!(
            toml_manifests.contains_key(codegen_name),
            "codegen-emitted manifest {codegen_name:?} is NOT declared \
             in host-functions.toml — drift detector FIRED"
        );
    }
}

#[test]
fn sandbox_named_manifest_canonical_bytes_dagcbor() {
    // D9 + D2 — named manifests serialize to canonical DAG-CBOR for
    // CID computation. `compute-basic` manifest's canonical bytes are
    // bit-stable across re-encodes (BTreeMap key ordering).
    let reg = ManifestRegistry::new();
    let bundle = reg.lookup("compute-basic").unwrap();
    let bytes_1 = bundle.to_canonical_bytes().unwrap();
    let bytes_2 = bundle.to_canonical_bytes().unwrap();
    assert_eq!(bytes_1, bytes_2, "DAG-CBOR encoding must be stable");
    assert!(!bytes_1.is_empty(), "canonical bytes must be non-empty");
}

#[test]
fn sandbox_register_runtime_returns_e_sandbox_manifest_registration_deferred() {
    // D2-RESOLVED — `register_runtime(name, bundle)` exists in 2b but
    // returns Err(ManifestError::RuntimeRegistrationDeferred), which
    // routes to E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED.
    //
    // API-surface preservation: Phase-4-Meta plugin-install work flips
    // `runtime_registration_enabled = true` and lifts the deferral
    // without breaking callers. (The historical "Phase 8 marketplace"
    // framing pre-dates the CLAUDE.md baked-in #15 v1-platform-shippable
    // widening ratified 2026-05-10 + baked-in #18 plugins-as-subgraphs.)
    let mut reg = ManifestRegistry::new();
    let result = reg.register_runtime("custom", CapBundle::new(vec![], None));
    assert!(matches!(
        result,
        Err(ManifestError::RuntimeRegistrationDeferred)
    ));
    let err = result.unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxManifestRegistrationDeferred);
}

#[test]
fn sandbox_manifest_registry_default_bundle_loaded_at_construction() {
    // D2 positive — `ManifestRegistry::new()` populates the default
    // entries from `default_manifests()` at construction time
    // (NOT lazily on first lookup).
    let reg = ManifestRegistry::new();
    assert!(reg.lookup("compute-basic").is_ok());
    assert!(reg.lookup("compute-with-kv").is_ok());
}
