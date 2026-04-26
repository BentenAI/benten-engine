//! Phase 2b R4-FP B-3 — G10-B module manifest canonical-bytes tests.
//!
//! TDD red-phase. Pin sources:
//!   - `r1-security-auditor.json` D9 RESOLVED — `ModuleManifest` MUST
//!     serialize via DAG-CBOR canonical mode (deterministic, no
//!     map-ordering ambiguity, no float NaN payload variance) so that
//!     two logically-equivalent authoring inputs (different field-order
//!     in the source JSON / TOML / TS literal) produce **the same CID**.
//!   - `r1-security-auditor.json` D9 RESOLVED — `signature: Option<ManifestSignature>`
//!     field is reserved on the struct for forward-compatibility with
//!     Phase-3 Ed25519 signing, but is OMITTED from the canonical bytes
//!     when `None` (so Phase-2b CIDs remain stable when Phase-3 signing
//!     lands and back-fills the field).
//!   - `r2-test-landscape.md` §1.8 — module-manifest canonical-bytes pin.
//!   - `.addl/phase-2b/00-implementation-plan.md` §3.2 G10-B.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::module_manifest::ManifestSignature (Phase-3 reserved)
//   benten_engine::testing::testing_compute_manifest_cid
//   benten_engine::testing::testing_make_minimal_manifest

#[test]
#[ignore = "Phase 2b G10-B pending — D9 canonical-bytes stability"]
fn module_manifest_canonical_bytes_dagcbor() {
    // D9 RESOLVED — the manifest MUST round-trip through DAG-CBOR
    // canonical mode (RFC-8949 §4.2 Core Deterministic Encoding plus the
    // DAG-CBOR strict subset: no indefinite-length items, sorted map
    // keys by bytewise lex of the encoded key, smallest-int-encoding,
    // no NaN payload variance).
    //
    // R5 G10-B wires:
    //   1. let m = testing_make_minimal_manifest("acme.posts");
    //   2. let bytes_a = serde_ipld_dagcbor::to_vec(&m).unwrap();
    //   3. let bytes_b = serde_ipld_dagcbor::to_vec(&m).unwrap();
    //   4. ASSERT bytes_a == bytes_b (idempotence — trivial baseline).
    //   5. let m_round_tripped: ModuleManifest =
    //          serde_ipld_dagcbor::from_slice(&bytes_a).unwrap();
    //   6. let bytes_c = serde_ipld_dagcbor::to_vec(&m_round_tripped).unwrap();
    //   7. ASSERT bytes_a == bytes_c (decode-reencode stability).
    //
    // The *cross-input* stability case (different authoring inputs that
    // SHOULD canonicalize to the same bytes) lives in
    // `module_manifest_two_logically_identical_authoring_inputs_produce_same_cid`
    // below.
    todo!("R5 G10-B — assert ModuleManifest DAG-CBOR canonical round-trip");
}

#[test]
#[ignore = "Phase 2b G10-B pending — D9 logically-equivalent inputs collapse to same CID"]
fn module_manifest_two_logically_identical_authoring_inputs_produce_same_cid() {
    // D9 RESOLVED — the canonical-bytes property MUST collapse two
    // logically-identical authoring inputs (different source field-order,
    // different whitespace, etc.) into the SAME CID. This is what makes
    // the install-time CID pin operator-actionable: an operator pinning a
    // CID computed from a JSON source MUST get the same CID computed by a
    // reviewer using a TOML source.
    //
    // R5 G10-B wires:
    //   1. Build manifest A with provides-subgraphs in order [x, y, z]
    //      and requires-caps in order [a, b].
    //   2. Build manifest B with the same logical content but
    //      provides-subgraphs declared in order [z, y, x] and
    //      requires-caps in order [b, a].
    //   3. Both manifests MUST canonicalize to identical bytes:
    //         ASSERT testing_compute_manifest_cid(&A)
    //             == testing_compute_manifest_cid(&B).
    //
    // CRITICAL: the canonical-bytes invariant is what closes
    // sec-pre-r1-01 (manifest-forge) — without it, two reviewers seeing
    // the same logical manifest but encoded differently would compute
    // DIFFERENT CIDs, defeating the install-time pin.
    todo!(
        "R5 G10-B — assert canonical CID collapses logically-equivalent \
         authoring inputs"
    );
}

#[test]
#[ignore = "Phase 2b G10-B pending — D9 strict-mode rejection of non-canonical bytes"]
fn module_manifest_dagcbor_strict_mode_rejects_non_canonical() {
    // D9 RESOLVED — when an attacker hand-crafts a NON-canonical CBOR
    // byte sequence (e.g. unsorted map keys, indefinite-length items,
    // larger-than-needed integer encoding) and presents it as a manifest,
    // the decoder MUST REJECT IT rather than silently accept-and-canonicalize.
    //
    // Why: silent acceptance would let an attacker create a manifest with
    // bytes B1 that decodes to manifest M, while the canonical encoding
    // of M is bytes B2 != B1. The CID computed from B1 would NOT match
    // the CID an honest verifier computes from M, but the engine would
    // still install the manifest -- a CID-pin bypass.
    //
    // R5 G10-B wires:
    //   1. Build a valid canonical-bytes encoding of a manifest M.
    //   2. Hand-mutate the bytes to produce a non-canonical-but-still-decodable
    //      sequence (e.g. swap two map-key orders).
    //   3. ASSERT serde_ipld_dagcbor::from_slice::<ModuleManifest>(&mutated)
    //      returns Err (strict mode rejects).
    //   4. ASSERT the error type maps to a typed manifest-decode error
    //      (NOT a generic serde error) so the caller can match on it.
    todo!("R5 G10-B — assert DAG-CBOR strict mode rejects non-canonical bytes");
}
