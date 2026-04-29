//! Manifest schema parity pin — symmetric to the TS-side pin in
//! `packages/engine/test/manifest_schema_parity.test.ts`.
//!
//! Both sides recompute the canonical-DAG-CBOR CID of the same logical
//! manifest input and assert against the same pinned base32 string. If
//! the encoding (DAG-CBOR map-key sort, optional-field omission, the
//! CID multibase rendering, etc.) drifts, exactly ONE side fails first,
//! pinpointing the regression to whichever side changed.
//!
//! The TS-side pin lives at
//! `packages/engine/test/manifest_schema_parity.test.ts::"computeManifestCid agrees with Rust testing_compute_manifest_cid"`.
//! Re-pin both together when a canonical-bytes change is intentional.

use benten_engine::{ModuleManifest, ModuleManifestEntry};

#[test]
fn manifest_parity_fixture_cid_matches_ts_pin() {
    let manifest = ModuleManifest {
        name: "manifest-parity-fixture".to_string(),
        version: "1.0.0".to_string(),
        modules: vec![
            ModuleManifestEntry {
                name: "module-alpha".to_string(),
                cid: "bafy-fixture-cid-alpha".to_string(),
                requires: vec!["host:compute:time".to_string()],
            },
            ModuleManifestEntry {
                name: "module-beta".to_string(),
                cid: "bafy-fixture-cid-beta".to_string(),
                requires: vec![],
            },
        ],
        migrations: vec![],
        signature: None,
    };

    // Pinned by the same one-shot Rust compute-and-print run that pinned
    // the TS-side EXPECTED_CID. Both sides MUST agree.
    const EXPECTED_CID: &str = "bafyr4igihvodf4lnqp5wsjfiotjxiux6kz5bumaf3irk4rjtwlwlkzmer4";

    let cid = manifest
        .compute_cid()
        .expect("manifest canonical-bytes encoding is infallible");
    assert_eq!(
        cid.to_base32(),
        EXPECTED_CID,
        "manifest parity fixture CID drifted on the Rust side; if intentional, re-pin both \
         sides by recomputing via this test, then update the TS-side EXPECTED_CID at \
         packages/engine/test/manifest_schema_parity.test.ts"
    );
}
