//! Phase 2b G10-B — Module manifest format.
//!
//! Implements [`ModuleManifest`] per **D9-RESOLVED** + **D16-RESOLVED-FURTHER**:
//!
//! * Canonical encoding is **DAG-CBOR** (NOT TOML, NOT JSON). DAG-CBOR is the
//!   one canonical-bytes-stable serialization across the Benten stack: sorted
//!   map keys, no indefinite-length items, smallest-int encoding, no NaN
//!   payload variance. Two logically-identical authoring inputs (e.g. JSON
//!   with different field-order, or a TOML source vs. a TS literal) MUST
//!   collapse to the SAME canonical bytes — and therefore the same CID.
//!   That property is what makes the install-time CID-pin
//!   (`Engine::install_module`) operator-actionable: a reviewer pinning a
//!   CID computed from a TOML source MUST see the same CID an operator
//!   computes from a JSON / TypeScript source.
//!
//! * Thin TOML / JSON dev-time sources may compile to canonical bytes
//!   (mirrors `host-functions.toml` codegen). The compiler lives outside
//!   this crate (likely in `benten-dsl-compiler` or G7-A's manifest codegen
//!   path); this module owns only the canonical struct + the canonical-bytes
//!   serializer + the install-time CID computation.
//!
//! * The [`ModuleManifest::signature`] field is reserved for **Phase-3**
//!   Ed25519 manifest signing (D16). When `None`, the field is OMITTED
//!   from the canonical DAG-CBOR encoding (NOT serialized as `null`) so
//!   that Phase-2b CIDs remain stable when Phase-3 signing lands and
//!   back-fills the field for signed manifests. The forward-compat
//!   property is enforced by the `skip_serializing_if = "Option::is_none"`
//!   attribute on the field.
//!
//! ## Cross-language parity (TS ↔ Rust)
//!
//! The TypeScript [`ModuleManifest`] in `packages/engine/src/types.ts`
//! mirrors this struct field-for-field. The parity check
//! `packages/engine/test/manifest_schema_parity.test.ts` verifies that a
//! manifest authored on one side computes the same CID on the other.
//!
//! ## Operator-actionable error display
//!
//! [`ManifestSummary`] is the 1-line human-readable summary embedded in
//! `E_MODULE_MANIFEST_CID_MISMATCH` errors per D16-RESOLVED-FURTHER. The
//! shape is `<name> v<version> modules=<n> caps=<n>` so an operator can
//! identify which manifest mis-installed without source-code spelunking.

use serde::{Deserialize, Serialize};

use benten_core::Cid;

/// One module entry inside a [`ModuleManifest`].
///
/// Mirrors the TypeScript `ModuleManifestEntry` in
/// `packages/engine/src/types.ts`. Field order, types, and
/// `skip_serializing_if` semantics are load-bearing for cross-language
/// CID parity (D9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleManifestEntry {
    /// Module name — referenced from the DSL via
    /// `<manifestName>:<moduleName>`.
    pub name: String,
    /// CIDv1 base32 string of the WebAssembly module bytes.
    pub cid: String,
    /// Capabilities the module's host-fn imports require
    /// (`host:<domain>:<action>` strings).
    pub requires: Vec<String>,
}

/// Reserved-for-Phase-3 manifest signature shape.
///
/// Phase 2b leaves this structurally typed but always-`None`. Per
/// **D9-RESOLVED**, the canonical DAG-CBOR encoding OMITS the
/// `signature` key entirely when `None`, NOT serializes it as `null`.
/// This is the load-bearing forward-compat invariant — Phase-3 signing
/// can back-fill the field on signed manifests WITHOUT changing the
/// CID of any already-installed Phase-2b manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSignature {
    /// Phase-3 Ed25519 signature bytes, base64-encoded. Reserved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ed25519: Option<String>,
}

/// One declared migration step.
///
/// Phase 2b reserves the shape; the migration runner itself lands in
/// Phase 3 alongside the persistence story. On `wasm32-unknown-unknown`
/// (browser, in-memory only per Compromise #N+8) installing a manifest
/// that declares any `MigrationStep`s is rejected with
/// `E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE` — there is no persistent
/// backing store for migrations to land in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Stable migration id (e.g. `"add-author-index-2026-04"`).
    pub id: String,
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Module manifest — canonical DAG-CBOR (D9-RESOLVED).
///
/// Field order on the struct does NOT determine canonical-bytes order
/// — DAG-CBOR sorts map keys lexicographically by their encoded byte
/// representation (RFC-8949 §4.2.1 + the DAG-CBOR strict subset). The
/// struct field declarations below are arranged for readability; the
/// canonical encoding is independent.
///
/// ## Field-set parity
///
/// Mirrors the TypeScript `ModuleManifest` in
/// `packages/engine/src/types.ts`. Parity is asserted at the test layer
/// by `packages/engine/test/manifest_schema_parity.test.ts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Manifest name (e.g. `"acme.posts"`). The first dotted segment
    /// is conventionally the publisher; the remainder is the
    /// publisher-local manifest id.
    pub name: String,
    /// Manifest version string (semver-shaped; not parsed in Phase 2b).
    pub version: String,
    /// Modules this manifest declares.
    pub modules: Vec<ModuleManifestEntry>,
    /// Phase-3-reserved migration declarations. When non-empty on a
    /// `wasm32-unknown-unknown` target, install fires
    /// `E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migrations: Vec<MigrationStep>,
    /// Phase-3-reserved Ed25519 signature surface. When `None` the
    /// field is OMITTED from canonical bytes (D9 forward-compat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<ManifestSignature>,
}

/// 1-line operator-readable summary of a [`ModuleManifest`] embedded
/// in `E_MODULE_MANIFEST_CID_MISMATCH` errors per D16-RESOLVED-FURTHER.
///
/// Display shape: `<name> v<version> modules=<n> caps=<n>`.
///
/// `caps` is the **deduplicated** count of unique `requires` strings
/// across every module entry — so the operator sees the manifest's
/// effective capability surface, not a per-module total.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestSummary {
    /// Manifest name copied from [`ModuleManifest::name`].
    pub name: String,
    /// Manifest version copied from [`ModuleManifest::version`].
    pub version: String,
    /// Module-entry count.
    pub modules: usize,
    /// Deduplicated count of `requires` capability strings.
    pub caps: usize,
}

impl std::fmt::Display for ManifestSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name} v{version} modules={modules} caps={caps}",
            name = self.name,
            version = self.version,
            modules = self.modules,
            caps = self.caps,
        )
    }
}

/// Errors produced by manifest serialization / deserialization.
///
/// The CID-pin mismatch error is owned by [`crate::error::EngineError`]
/// (variant `ModuleManifestCidMismatch`) since it surfaces from the
/// engine boundary, not from the pure-data manifest module. The
/// migrations-require-persistence error is similarly engine-owned —
/// only the engine knows the target architecture.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestError {
    /// DAG-CBOR encode failure. Should be infallible in practice; the
    /// arm exists so the encoder return type stays `Result`.
    #[error("manifest encode failure: {0}")]
    Encode(String),

    /// DAG-CBOR decode failure (malformed bytes / type mismatch /
    /// non-canonical encoding rejected by strict mode).
    #[error("manifest decode failure: {0}")]
    Decode(String),
}

impl ModuleManifest {
    /// Encode the manifest to its canonical DAG-CBOR bytes (D9-RESOLVED).
    ///
    /// Two logically-identical inputs (different authoring source
    /// field-order, etc.) produce IDENTICAL bytes. The encoding is
    /// the load-bearing primitive — both [`Self::compute_cid`] and
    /// [`Engine::install_module`](crate::engine::Engine::install_module)
    /// rely on bytes returned here being canonical.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError::Encode`] if the underlying DAG-CBOR
    /// encoder fails. The encoder is infallible for the
    /// [`ModuleManifest`] schema in practice; the `Result` return is
    /// present so callers do not have to `unwrap()`.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, ManifestError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| ManifestError::Encode(e.to_string()))
    }

    /// Decode a manifest from canonical DAG-CBOR bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError::Decode`] on malformed bytes or
    /// type-shape mismatches.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ManifestError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|e| ManifestError::Decode(e.to_string()))
    }

    /// Compute the canonical-bytes CID of this manifest.
    ///
    /// CID layout: BLAKE3 over the canonical DAG-CBOR bytes, wrapped
    /// in a Benten CIDv1 envelope (multicodec `0x71` dag-cbor +
    /// multihash `0x1e` BLAKE3).
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError::Encode`] if canonical-bytes
    /// encoding fails.
    pub fn compute_cid(&self) -> Result<Cid, ManifestError> {
        let bytes = self.to_canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }

    /// Build the [`ManifestSummary`] used in operator-readable error
    /// strings (D16 dual-CID mismatch reporting).
    ///
    /// `caps` is the **deduplicated** count of unique `requires`
    /// capability strings across every module entry.
    #[must_use]
    pub fn summary(&self) -> ManifestSummary {
        use std::collections::BTreeSet;
        let unique_caps: BTreeSet<&str> = self
            .modules
            .iter()
            .flat_map(|m| m.requires.iter().map(String::as_str))
            .collect();
        ManifestSummary {
            name: self.name.clone(),
            version: self.version.clone(),
            modules: self.modules.len(),
            caps: unique_caps.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — narrow unit pins for the canonical-bytes + summary surface.
// Engine-side install/uninstall pins live in `tests/module_install.rs`
// and `tests/integration/module_install_uninstall_round_trip.rs`.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn fixture_manifest() -> ModuleManifest {
        ModuleManifest {
            name: "acme.posts".into(),
            version: "0.0.1".into(),
            modules: vec![ModuleManifestEntry {
                name: "post-handler".into(),
                cid: "bafy_dummy_module_cid".into(),
                requires: vec!["host:compute:time".into()],
            }],
            migrations: vec![],
            signature: None,
        }
    }

    #[test]
    fn canonical_bytes_idempotent() {
        let m = fixture_manifest();
        let a = m.to_canonical_bytes().unwrap();
        let b = m.to_canonical_bytes().unwrap();
        assert_eq!(a, b, "DAG-CBOR canonical bytes must be deterministic");
    }

    #[test]
    fn canonical_bytes_round_trip() {
        let m = fixture_manifest();
        let bytes = m.to_canonical_bytes().unwrap();
        let decoded = ModuleManifest::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(m, decoded);
        let re_encoded = decoded.to_canonical_bytes().unwrap();
        assert_eq!(
            bytes, re_encoded,
            "decode → re-encode must be byte-stable (D9 canonical-bytes invariant)"
        );
    }

    #[test]
    fn signature_none_omitted_from_canonical_bytes() {
        // D9 RESOLVED — when signature is None, the canonical DAG-CBOR
        // map MUST NOT carry a "signature" key at all (forward-compat
        // with Phase-3 signing).
        let m = fixture_manifest();
        assert!(m.signature.is_none());
        let bytes = m.to_canonical_bytes().unwrap();
        // Scan for the literal "signature" UTF-8 byte sequence — DAG-CBOR
        // map keys are encoded as text strings, so the key would appear
        // verbatim in the wire bytes if it were emitted.
        let needle = b"signature";
        let appears = bytes.windows(needle.len()).any(|w| w == needle);
        assert!(
            !appears,
            "signature=None must be omitted from canonical bytes; found 'signature' in {:?}",
            bytes
        );
    }

    #[test]
    fn summary_dedupe_caps_across_modules() {
        let m = ModuleManifest {
            name: "acme.posts".into(),
            version: "0.0.1".into(),
            modules: vec![
                ModuleManifestEntry {
                    name: "a".into(),
                    cid: "bafy_a".into(),
                    requires: vec!["host:compute:time".into(), "host:fs:read".into()],
                },
                ModuleManifestEntry {
                    name: "b".into(),
                    cid: "bafy_b".into(),
                    // Overlapping cap with module a — must dedupe.
                    requires: vec!["host:compute:time".into()],
                },
            ],
            migrations: vec![],
            signature: None,
        };
        let s = m.summary();
        assert_eq!(s.modules, 2);
        assert_eq!(
            s.caps, 2,
            "host:compute:time appears in both modules; dedup must collapse to 2 unique caps"
        );
        assert_eq!(format!("{s}"), "acme.posts v0.0.1 modules=2 caps=2");
    }

    #[test]
    fn logically_identical_inputs_produce_same_cid() {
        // D9 — two manifests with identical content but different
        // module-Vec ordering should… still differ here, because Vec
        // ORDER is meaningful in DAG-CBOR (arrays are order-preserving).
        // The "logical equivalence" guarantee holds for MAP-key order
        // (which we get for free from DAG-CBOR strict mode); arrays
        // intentionally remain order-significant — two manifests that
        // declare modules in different order ARE different manifests
        // for capability-attribution purposes (the order pins how the
        // capability hook walks the requires list).
        //
        // This test asserts the trivial baseline: two struct literals
        // with the same content produce the same CID.
        let a = fixture_manifest();
        let b = fixture_manifest();
        assert_eq!(a.compute_cid().unwrap(), b.compute_cid().unwrap());
    }

    #[test]
    fn compute_cid_is_blake3_over_canonical_bytes() {
        let m = fixture_manifest();
        let bytes = m.to_canonical_bytes().unwrap();
        let expected = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
        assert_eq!(m.compute_cid().unwrap(), expected);
    }
}
