//! Cargo feature-graph closure pin per `docs/future/phase-3-backlog.md`
//! §10.6 (R6 fix-pass Wave B closure of br-r6-r1-3 MAJOR + R6 R1
//! browser-wasm-bundle lens corroboration).
//!
//! ## What this defends
//!
//! Per Phase-2a `sec-r6r2-02` precedent + R4-r1-wsa-3: the harder
//! Cargo-feature-graph regression vector — where `bindings/napi.test-helpers`
//! ends up transitively activated in the production cdylib build via a
//! feature composition that pulls `benten-eval/test-helpers` or
//! `benten-engine/test-helpers` from the default-feature closure.
//!
//! ## Defense-in-depth boundaries
//!
//! Three rungs of defense protect SANDBOX testing-helper widening:
//!
//! 1. **Source-side cfg-gating** — `#![cfg(any(test, feature =
//!    "test-helpers"))]` file-level gate at
//!    `crates/benten-eval/src/sandbox/testing_helpers.rs`. Pinned by
//!    `crates/benten-eval/tests/sandbox_helpers_no_widening.rs`.
//! 2. **Cargo.toml default-feature audit** — `bindings/napi.default`
//!    does NOT include `test-helpers`. Pinned by THIS file.
//! 3. **Cargo feature-graph closure walk** — the closure of features
//!    reachable from `bindings/napi.default` does NOT include
//!    `benten-eval/test-helpers` or `benten-engine/test-helpers`.
//!    Pinned by THIS file.
//!
//! Closes phase-3-backlog §10.6 v1-window destination ahead of v1
//! milestone gate per HARD RULE rule-12 clause-(b).

#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn napi_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn napi_manifest() -> toml::Value {
    let raw = std::fs::read_to_string(napi_manifest_path()).unwrap();
    raw.parse::<toml::Value>().unwrap()
}

fn features_table(manifest: &toml::Value) -> BTreeMap<String, Vec<String>> {
    let features = manifest
        .get("features")
        .and_then(|v| v.as_table())
        .expect("bindings/napi/Cargo.toml MUST declare a [features] table");
    let mut out = BTreeMap::new();
    for (name, raw) in features {
        let entries = raw
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.insert(name.clone(), entries);
    }
    out
}

/// Compute the transitive closure of features reachable from `roots`
/// in the in-crate features table. The closure includes the root
/// features themselves plus every feature transitively named by any
/// reachable feature definition.
///
/// Cross-crate references (form `<crate>/<feat>` or `dep:<crate>`) are
/// recorded separately in `cross_crate_refs` so the caller can audit
/// transitive activation across the workspace.
fn closure_from(
    features: &BTreeMap<String, Vec<String>>,
    roots: &[&str],
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut reachable: BTreeSet<String> = BTreeSet::new();
    let mut cross_crate_refs: BTreeSet<String> = BTreeSet::new();
    let mut frontier: Vec<String> = roots.iter().map(|r| r.to_string()).collect();

    while let Some(feat) = frontier.pop() {
        if !reachable.insert(feat.clone()) {
            continue;
        }
        // Look up the feature definition. Optional features have an
        // implicit definition (single-element vec [`dep:<name>`]) — we
        // skip those without erroring (the in-crate closure only walks
        // explicitly-defined features).
        let Some(entries) = features.get(&feat) else {
            continue;
        };
        for entry in entries {
            if entry.contains('/') {
                // Cross-crate reference: `<crate>/<feat>` or
                // `<crate>?/<feat>` (weak feature). Record + don't
                // recurse (we don't have the dep crate's features
                // table without `cargo metadata`).
                cross_crate_refs.insert(entry.clone());
            } else if let Some(stripped) = entry.strip_prefix("dep:") {
                // `dep:<name>` enables the optional dep without
                // activating any of its features. Record for surface
                // visibility but don't walk.
                cross_crate_refs.insert(format!("dep:{stripped}"));
            } else {
                // In-crate feature reference; recurse.
                frontier.push(entry.clone());
            }
        }
    }
    (reachable, cross_crate_refs)
}

#[test]
fn napi_default_feature_does_not_include_test_helpers() {
    // Defense-in-depth rung 2: `default` MUST NOT include `test-helpers`
    // directly. Phase-2a `sec-r6r2-02` precedent.
    let manifest = napi_manifest();
    let features = features_table(&manifest);

    let default = features
        .get("default")
        .expect("bindings/napi.default feature must be declared");
    assert!(
        !default.iter().any(|f| f == "test-helpers"),
        "bindings/napi.default MUST NOT include `test-helpers` directly \
         per phase-3-backlog §10.6 — production cdylib must not pull \
         testing-helper surface (Phase-2a sec-r6r2-02 precedent)"
    );
}

#[test]
fn napi_default_feature_closure_does_not_activate_test_helpers_transitively() {
    // Defense-in-depth rung 3: the transitive closure from
    // `bindings/napi.default` MUST NOT activate `test-helpers` (the
    // in-crate feature) or `benten-eval/test-helpers` /
    // `benten-engine/test-helpers` (the cross-crate transitive
    // activations).
    //
    // Closes phase-3-backlog §10.6 LOAD-BEARING half of pim-2 §3.6b
    // per r4-r1-wsa-3.
    let manifest = napi_manifest();
    let features = features_table(&manifest);
    let (reachable, cross_crate) = closure_from(&features, &["default"]);

    // The in-crate `test-helpers` feature MUST NOT appear in the
    // default closure. (It would only appear if some feature reachable
    // from `default` named `test-helpers` directly.)
    assert!(
        !reachable.contains("test-helpers"),
        "bindings/napi.default closure MUST NOT activate `test-helpers` \
         transitively per phase-3-backlog §10.6. Closure: {reachable:?}"
    );

    // The cross-crate `benten-eval/test-helpers` activation MUST NOT
    // appear in the default closure's cross-crate references.
    let forbidden_cross_crate = [
        "benten-eval/test-helpers",
        "benten-engine/test-helpers",
        "benten_eval/test-helpers",
        "benten_engine/test-helpers",
    ];
    for forbidden in &forbidden_cross_crate {
        assert!(
            !cross_crate.contains(*forbidden),
            "bindings/napi.default closure MUST NOT activate cross-crate \
             feature `{forbidden}` per phase-3-backlog §10.6 (Phase-2a \
             sec-r6r2-02 precedent — testing-helper widening into \
             production cdylib is the most catastrophic ESC defense \
             bypass mode). Cross-crate refs from default closure: \
             {cross_crate:?}"
        );
    }
}

#[test]
fn napi_test_helpers_feature_only_reachable_when_explicitly_opted_in() {
    // Symmetric pin: verify that `test-helpers` IS reachable when
    // explicitly opted in (so the feature is not vestigial). This is
    // the LIVE-PATH test that complements the previous DEAD-PATH pins —
    // ensures we're testing the right thing.
    let manifest = napi_manifest();
    let features = features_table(&manifest);
    let (reachable, cross_crate) = closure_from(&features, &["test-helpers"]);

    assert!(
        reachable.contains("test-helpers"),
        "bindings/napi.test-helpers feature must be reachable from itself"
    );
    assert!(
        cross_crate
            .iter()
            .any(|s| s == "benten-engine/test-helpers" || s == "benten_engine/test-helpers"),
        "bindings/napi.test-helpers MUST transitively activate \
         `benten-engine/test-helpers` per the existing feature \
         definition. If the feature definition changed, the §10.6 pin \
         set must be retensed. Cross-crate refs from test-helpers \
         closure: {cross_crate:?}"
    );
}

#[test]
fn napi_napi_export_default_feature_closure_uses_only_production_features() {
    // The `default = ["napi-export"]` declaration is the production
    // cdylib build's feature set. Walk its closure + assert the
    // resulting in-crate features are all production-shape:
    //
    //   - `napi-export` (production cdylib symbols)
    //   - dep activations like `dep:napi`, `dep:napi-derive`
    //
    // No test-only features (`in-process-test`, `test-helpers`,
    // `browser-target` is documentation-only) should appear.
    let manifest = napi_manifest();
    let features = features_table(&manifest);
    let (reachable, _cross_crate) = closure_from(&features, &["default"]);

    let test_only_features = ["in-process-test", "test-helpers"];
    for forbidden in &test_only_features {
        assert!(
            !reachable.contains(*forbidden),
            "bindings/napi.default closure MUST NOT contain test-only \
             feature `{forbidden}`. Closure: {reachable:?}"
        );
    }
}
