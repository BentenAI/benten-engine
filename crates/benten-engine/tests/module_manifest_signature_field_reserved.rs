//! Phase 2b R4-FP B-3 — `ModuleManifest.signature` field forward-compat
//! reservation pin.
//!
//! TDD red-phase. Pin sources:
//!   - `r1-security-auditor.json` D9 RESOLVED — the `signature:
//!     Option<ManifestSignature>` field is reserved on `ModuleManifest`
//!     for forward-compatibility with Phase-3 Ed25519 manifest signing.
//!     When `None`, the field MUST be OMITTED from the canonical DAG-
//!     CBOR encoding (NOT serialized as `null`) so that Phase-2b CIDs
//!     remain stable when Phase-3 lands and back-fills the field for
//!     signed manifests.
//!   - `r2-test-landscape.md` §1.8.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::module_manifest::ManifestSignature (Phase-3 reserved type)
//   benten_engine::testing::testing_compute_manifest_cid
//   benten_engine::testing::testing_make_minimal_manifest

#[test]
#[ignore = "Phase 2b G10-B pending — signature field exists on ModuleManifest"]
fn module_manifest_signature_field_exists_as_optional() {
    // D9 RESOLVED — the field MUST exist on the struct and be typed
    // `Option<ManifestSignature>`. Reserving the field NOW (with the
    // canonical-bytes omission rule below) means Phase-3 signing can
    // back-fill it on signed manifests WITHOUT changing the CID of any
    // already-installed Phase-2b manifest.
    //
    // R5 G10-B wires:
    //   1. let m = testing_make_minimal_manifest("acme.posts");
    //   2. ASSERT m.signature.is_none();  // default = None
    //   3. (Compile-only) the type signature `m.signature: Option<ManifestSignature>`
    //      is enforced by the field access compiling.
    todo!("R5 G10-B — assert ModuleManifest.signature: Option<ManifestSignature> exists");
}

#[test]
#[ignore = "Phase 2b G10-B pending — signature: None is OMITTED from canonical bytes"]
fn module_manifest_signature_none_omitted_from_canonical_bytes() {
    // D9 RESOLVED — when `signature` is `None`, it MUST NOT appear in
    // the canonical DAG-CBOR map at all (not as `null`, not as the key
    // with an absent value). This is THE forward-compat invariant: a
    // Phase-2b manifest whose CID is X today must canonicalize to the
    // exact same bytes after Phase-3 lands the signing surface,
    // producing the exact same CID X.
    //
    // R5 G10-B wires:
    //   1. let m_unsigned = testing_make_minimal_manifest("acme.posts");
    //      (with m_unsigned.signature == None)
    //   2. let bytes_unsigned = serde_ipld_dagcbor::to_vec(&m_unsigned).unwrap();
    //   3. ASSERT !bytes_unsigned.windows(b"signature".len())
    //         .any(|w| w == b"signature");
    //      // The "signature" byte string must NOT appear as a map key
    //      // when the field value is None.
    //   4. (Equivalent CID-stability check) compute the manifest CID
    //      and pin it as the canonical fixture for the empty-signature
    //      shape. When Phase-3 lands and adds the signature variant,
    //      this CID must remain stable.
    todo!(
        "R5 G10-B — assert signature=None is OMITTED (skip_serializing_if) \
         from DAG-CBOR canonical bytes"
    );
}
