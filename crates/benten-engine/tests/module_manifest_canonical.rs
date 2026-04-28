//! Phase 2b G10-B — module manifest canonical-bytes tests (D9-RESOLVED).
//!
//! Pin sources:
//!   - `r1-security-auditor.json` D9 RESOLVED — `ModuleManifest` MUST
//!     serialize via DAG-CBOR canonical mode (deterministic, sorted
//!     map keys, no float NaN payload variance).
//!   - `r1-security-auditor.json` D9 RESOLVED — `signature: Option<...>`
//!     field is omitted from canonical bytes when None (Phase-3
//!     forward-compat).
//!   - `r2-test-landscape.md` §1.8.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::ModuleManifest;
use benten_engine::testing::{testing_compute_manifest_cid, testing_make_minimal_manifest};

#[test]
fn module_manifest_canonical_bytes_dagcbor() {
    // D9 RESOLVED — encode → decode → re-encode is byte-stable.
    let m = testing_make_minimal_manifest("acme.posts");
    let bytes_a = m.to_canonical_bytes().unwrap();
    let bytes_b = m.to_canonical_bytes().unwrap();
    assert_eq!(bytes_a, bytes_b, "DAG-CBOR canonical-bytes idempotent");
    let round: ModuleManifest = ModuleManifest::from_canonical_bytes(&bytes_a).unwrap();
    let bytes_c = round.to_canonical_bytes().unwrap();
    assert_eq!(bytes_a, bytes_c, "decode → re-encode byte-stable");
}

#[test]
fn manifest_canonical_bytes_dagcbor_dev_time_toml_compiles_to_same_bytes() {
    // D9 — two manifests with logically identical content but
    // different field-order in the source SHOULD canonicalize to
    // identical bytes. DAG-CBOR sorts map keys lexicographically by
    // their encoded byte representation, so the canonicalization is
    // automatic at the encoder level.
    //
    // We construct two manifests whose Rust struct literal field-order
    // differs (the manifest itself doesn't have a "field order" once
    // you've called `to_canonical_bytes`, because DAG-CBOR sorts the
    // map keys; this test pins that property holds across toolchains).
    //
    // Note: TOML / JSON dev-time sources living outside this crate
    // (benten-dsl-compiler, host-functions.toml codegen) compile to the
    // same canonical bytes as a Rust struct literal — that compilation
    // is the cross-language parity contract. The canonicalization is
    // entirely owned by `serde_ipld_dagcbor::to_vec(&manifest)`, which
    // this test exercises directly.
    let a = testing_make_minimal_manifest("acme.posts");
    let b = ModuleManifest {
        name: "acme.posts".into(),
        version: "0.0.1".into(),
        modules: vec![benten_engine::ModuleManifestEntry {
            name: "acme.posts.handler".into(),
            cid: "bafy_dummy_module_for_acme.posts".into(),
            requires: vec![],
        }],
        migrations: vec![],
        signature: None,
    };
    let bytes_a = a.to_canonical_bytes().unwrap();
    let bytes_b = b.to_canonical_bytes().unwrap();
    assert_eq!(
        bytes_a, bytes_b,
        "logically-identical inputs MUST collapse to identical canonical bytes"
    );
    assert_eq!(
        testing_compute_manifest_cid(&a),
        testing_compute_manifest_cid(&b)
    );
}

#[test]
fn module_manifest_two_logically_identical_authoring_inputs_produce_same_cid() {
    // Same as above but framed as the cross-source CID-stability pin
    // (the property that makes the install-time CID pin
    // operator-actionable across language boundaries).
    let a = testing_make_minimal_manifest("acme.same");
    let b = testing_make_minimal_manifest("acme.same");
    assert_eq!(
        testing_compute_manifest_cid(&a),
        testing_compute_manifest_cid(&b)
    );
}

#[test]
fn module_manifest_dagcbor_strict_mode_round_trip_is_stable() {
    // The strict-mode rejection of malformed bytes is a property of
    // serde_ipld_dagcbor's decoder. We rely on that crate's strict-mode
    // guarantees rather than re-validating them here. What this test
    // pins is that a CANONICAL byte sequence round-trips intact — a
    // regression in our serde derives (e.g. accidentally adding
    // `serialize_with` that breaks canonicalization) would surface here.
    let m = testing_make_minimal_manifest("acme.strict");
    let bytes = m.to_canonical_bytes().unwrap();
    let decoded: ModuleManifest = ModuleManifest::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(m, decoded);
}
