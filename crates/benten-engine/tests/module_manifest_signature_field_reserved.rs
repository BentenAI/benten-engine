//! Phase 2b G10-B — `ModuleManifest.signature` field forward-compat
//! reservation pin (D9 RESOLVED).
//!
//! Pin sources:
//!   - `r1-security-auditor.json` D9 RESOLVED — the `signature:
//!     Option<ManifestSignature>` field is reserved on `ModuleManifest`
//!     for forward-compatibility with Phase-3 Ed25519 manifest signing.
//!     When `None`, the field MUST be OMITTED from the canonical DAG-
//!     CBOR encoding (NOT serialized as `null`) so that Phase-2b CIDs
//!     remain stable when Phase-3 lands and back-fills the field for
//!     signed manifests.
//!   - `r2-test-landscape.md` §1.8.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::testing::testing_make_minimal_manifest;
use benten_engine::{ManifestSignature, ModuleManifest};

#[test]
fn module_manifest_signature_field_exists_as_optional() {
    // D9 — the field exists on the struct typed as
    // Option<ManifestSignature>. Default = None on a fresh manifest.
    let m = testing_make_minimal_manifest("acme.posts");
    assert!(m.signature.is_none());
    // Compile-time assertion the field is the documented Optional shape.
    let _: Option<ManifestSignature> = m.signature;
}

#[test]
fn module_manifest_signature_none_omitted_from_canonical_bytes() {
    // D9 RESOLVED — when `signature` is `None`, it MUST NOT appear in
    // the canonical DAG-CBOR map at all (not as `null`, not as the key
    // with an absent value). This is THE forward-compat invariant.
    let m = testing_make_minimal_manifest("acme.posts");
    let bytes = m.to_canonical_bytes().unwrap();
    // DAG-CBOR map keys are encoded as text strings; the literal
    // "signature" appears verbatim if and only if the key is emitted.
    let needle = b"signature";
    let found = bytes.windows(needle.len()).any(|w| w == needle);
    assert!(
        !found,
        "signature=None MUST be omitted (skip_serializing_if) from canonical bytes; \
         literal 'signature' found in: {bytes:?}"
    );
}

#[test]
fn module_manifest_signature_some_appears_in_canonical_bytes() {
    // Complementary side: when signature is Some(_), it DOES appear
    // in canonical bytes (and the CID changes). Phase-3 signed
    // re-issuance gets a distinct CID per the D9 forward-compat note.
    let mut m = testing_make_minimal_manifest("acme.posts");
    let cid_unsigned = m.compute_cid().unwrap();
    m.signature = Some(ManifestSignature {
        ed25519: Some("dummy_phase3_signature_base64".into()),
    });
    let bytes_signed = m.to_canonical_bytes().unwrap();
    let needle = b"signature";
    let found = bytes_signed.windows(needle.len()).any(|w| w == needle);
    assert!(
        found,
        "signature=Some MUST appear in canonical bytes; got: {bytes_signed:?}"
    );
    let cid_signed = m.compute_cid().unwrap();
    assert_ne!(
        cid_unsigned, cid_signed,
        "Phase-3 signed re-issuance gets a DISTINCT CID per D9 forward-compat"
    );
}

#[test]
fn module_install_signature_field_omitted_from_canonical_bytes_when_none() {
    // Brief-named must-pass test — covered by the canonical-bytes pin
    // above; this is the named alias from the brief's must-pass list.
    let m = ModuleManifest {
        name: "acme.posts".into(),
        version: "0.0.1".into(),
        modules: vec![],
        migrations: vec![],
        signature: None,
    };
    let bytes = m.to_canonical_bytes().unwrap();
    assert!(
        !bytes.windows(b"signature".len()).any(|w| w == b"signature"),
        "field omitted when None"
    );
}
