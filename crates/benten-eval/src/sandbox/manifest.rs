//! Named-manifest registry (Phase 2b G7-A).
//!
//! D2-RESOLVED hybrid:
//!   - Codegen-default static `HashMap<String, CapBundle>` populated at
//!     [`ManifestRegistry::new`] time. Source-of-truth is `host-functions.toml`
//!     at workspace root; the dev-time `[manifest.<name>]` tables compile
//!     into the registry.
//!   - [`ManifestRegistry::register_runtime`] is RESERVED in 2b: it returns
//!     `Err(ManifestError::RuntimeRegistrationDeferred)` (which routes to
//!     `E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED`). Phase 8 marketplace
//!     work lifts the deferral by replacing the body — the public surface
//!     stays stable.
//!
//! D9-RESOLVED canonical-bytes encoding:
//!   - Bundle bytes are DAG-CBOR over a `BTreeMap<String, Vec<String>>`
//!     (field "caps" → sorted cap-strings). Dev-time TOML compiles to
//!     these canonical bytes; the bytes' BLAKE3 is bit-stable across
//!     re-encodes.
//!   - Reserved `signature: Option<ManifestSignature>` field is omitted
//!     from canonical bytes when `None` (the canonical encoder strips
//!     `None` Option fields). Phase-3 signed re-issuance gets a distinct
//!     CID by virtue of the signature bytes joining the canonical map.
//!
//! ESC-15 escape-vector closure: `lookup` of an unknown manifest name
//! returns `Err(ManifestError::Unknown { .. })`. There is NO permissive
//! fall-through to a default manifest.

use benten_core::{Cid, CoreError};
use benten_errors::ErrorCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A named bundle of capabilities a SANDBOX module may request.
///
/// Construction:
/// - Default-bundled entries are loaded by [`ManifestRegistry::new`] from
///   the codegen-emitted [`default_manifests`] table.
/// - Runtime-registered bundles are reserved for Phase 8 (see D2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapBundle {
    /// Sorted list of cap-strings the bundle requires. Sorted-canonical for
    /// DAG-CBOR bit-stability per D9.
    pub caps: Vec<String>,
    /// Optional one-line description; not part of canonical bytes (kept
    /// in dev-time TOML only, populated at codegen time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CapBundle {
    /// Construct a bundle from a sorted, deduplicated list of cap-strings.
    /// Caller is responsible for the sort + dedup invariant; the codegen
    /// path enforces it at build time.
    #[must_use]
    pub fn new(caps: Vec<String>, description: Option<String>) -> Self {
        Self { caps, description }
    }

    /// Return the canonical DAG-CBOR bytes for this bundle (per D9).
    ///
    /// Encoding shape: `BTreeMap<String, Vec<String>>` with single key
    /// `"caps"` bound to the sorted cap-strings list. The
    /// `description` and reserved `signature` fields are NOT part of
    /// canonical bytes.
    ///
    /// # Errors
    /// Returns `Err(ManifestError::Encode { .. })` on DAG-CBOR encode
    /// failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ManifestError> {
        let mut map: BTreeMap<&str, &Vec<String>> = BTreeMap::new();
        map.insert("caps", &self.caps);
        serde_ipld_dagcbor::to_vec(&map).map_err(|e| ManifestError::Encode {
            reason: e.to_string(),
        })
    }

    /// Compute the CID of this bundle (BLAKE3 over [`Self::canonical_bytes`]).
    ///
    /// # Errors
    /// Returns `Err(ManifestError::Encode { .. })` on DAG-CBOR encode failure.
    pub fn cid(&self) -> Result<Cid, ManifestError> {
        let bytes = self.canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

/// Reserved Phase-3 signature wrapper. Always `None` in 2b — keeping the
/// type declared so the future signature-bearing field is additive
/// (the canonical encoder skips `None` Options so 2b CIDs stay stable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSignature {
    /// Reserved. Phase 3 fills with Ed25519 signature bytes.
    pub bytes: Vec<u8>,
}

/// Failure modes for manifest lookup / registration.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestError {
    /// ESC-15 — manifest name was not in the registry. NO fall-through.
    #[error("named manifest not found: {name}")]
    Unknown {
        /// The unrecognized name the caller passed.
        name: String,
    },
    /// D2-RESOLVED hybrid — `register_runtime` is reserved as a typed-error
    /// no-op in Phase 2b. Phase 8 marketplace work flips the body.
    #[error("runtime manifest registration deferred to Phase 8")]
    RuntimeRegistrationDeferred,
    /// DAG-CBOR encode failure when computing canonical bytes.
    #[error("manifest canonical-bytes encode failure: {reason}")]
    Encode {
        /// Human-readable reason.
        reason: String,
    },
}

impl ManifestError {
    /// Stable catalog code for routing.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            ManifestError::Unknown { .. } => ErrorCode::SandboxManifestUnknown,
            ManifestError::RuntimeRegistrationDeferred => {
                ErrorCode::SandboxManifestRegistrationDeferred
            }
            ManifestError::Encode { .. } => ErrorCode::Serialize,
        }
    }
}

impl From<CoreError> for ManifestError {
    fn from(e: CoreError) -> Self {
        ManifestError::Encode {
            reason: e.to_string(),
        }
    }
}

/// Codegen-emitted default manifest table.
///
/// Phase-2b G7-A ships the static set inline (no separate `build.rs`):
/// the codegen-emitted entries live in this constant. The dev-time
/// `host-functions.toml` `[manifest.<name>]` tables are the
/// source-of-truth; the
/// `tests/sandbox_named_manifest_codegen_drift` test parses the TOML
/// at runtime and asserts byte-for-byte equality with this table.
///
/// Adding a new manifest: edit `host-functions.toml` AND append an entry
/// to `default_manifests()` below. The drift test fires before review if
/// the two diverge.
const DEFAULT_MANIFEST_NAMES: &[&str] = &["compute-basic", "compute-with-kv"];

/// Build the codegen-default manifest table.
///
/// This is the in-Rust mirror of `host-functions.toml`'s `[manifest.*]`
/// tables. The drift detector test (`sandbox_named_manifest_codegen_drift`)
/// re-parses the TOML at runtime and asserts byte-for-byte match against
/// the bundle this function emits.
///
/// Caps are sorted-canonical per D9 so [`CapBundle::canonical_bytes`] is
/// bit-stable.
#[must_use]
pub fn default_manifests() -> BTreeMap<String, CapBundle> {
    let mut table: BTreeMap<String, CapBundle> = BTreeMap::new();

    // compute-basic — time + log only.
    table.insert(
        "compute-basic".to_string(),
        CapBundle::new(
            vec![
                "host:compute:log".to_string(),
                "host:compute:time".to_string(),
            ],
            Some("Time + log (no KV, no network).".to_string()),
        ),
    );

    // compute-with-kv — adds kv:read.
    table.insert(
        "compute-with-kv".to_string(),
        CapBundle::new(
            vec![
                "host:compute:kv:read".to_string(),
                "host:compute:log".to_string(),
                "host:compute:time".to_string(),
            ],
            Some("compute-basic + kv:read (per_call cap-recheck).".to_string()),
        ),
    );

    table
}

/// Default manifest names exposed for drift / coverage tests.
#[must_use]
pub fn default_manifest_names() -> &'static [&'static str] {
    DEFAULT_MANIFEST_NAMES
}

/// Named-manifest registry. D2-RESOLVED hybrid construction.
#[derive(Debug, Clone)]
pub struct ManifestRegistry {
    table: BTreeMap<String, CapBundle>,
}

impl ManifestRegistry {
    /// Construct a registry pre-loaded with the codegen-default bundles.
    ///
    /// The default set is loaded eagerly at construction time (NOT lazily
    /// on first lookup) so the working set is observable via `lookup` and
    /// `entries` immediately.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: default_manifests(),
        }
    }

    /// Look up a named manifest. Returns `Err(ManifestError::Unknown)`
    /// for any name not in the registry. ESC-15: NO permissive
    /// fall-through to a default manifest.
    ///
    /// # Errors
    /// Returns `Err(ManifestError::Unknown { .. })` when `name` is not
    /// in the registry.
    pub fn lookup(&self, name: &str) -> Result<&CapBundle, ManifestError> {
        self.table.get(name).ok_or_else(|| ManifestError::Unknown {
            name: name.to_string(),
        })
    }

    /// Iterate over all registered manifest names. Useful for the
    /// codegen-drift detector to walk every entry.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.table.keys().map(String::as_str)
    }

    /// Iterate over `(name, bundle)` pairs.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &CapBundle)> {
        self.table.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// D2-RESOLVED — runtime registration is reserved as a typed-error
    /// no-op in Phase 2b. Phase 8 marketplace work lifts the deferral
    /// by replacing the body; the public surface is preserved across
    /// the lift.
    ///
    /// # Errors
    /// Always returns `Err(ManifestError::RuntimeRegistrationDeferred)`
    /// in Phase 2b.
    pub fn register_runtime(
        &mut self,
        _name: impl Into<String>,
        _bundle: CapBundle,
    ) -> Result<(), ManifestError> {
        Err(ManifestError::RuntimeRegistrationDeferred)
    }
}

impl Default for ManifestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference to a named or inline manifest, used at SANDBOX call sites.
///
/// `Named(name)` resolves through the [`ManifestRegistry`]; `Inline(bundle)`
/// passes the bundle directly (developer-flexibility path; same security
/// posture — caps still intersect against the dispatching grant).
#[derive(Debug, Clone)]
pub enum ManifestRef {
    /// Resolve through [`ManifestRegistry::lookup`].
    Named(String),
    /// Inline cap-bundle.
    Inline(CapBundle),
}

impl ManifestRef {
    /// Construct a Named reference from a static or owned string.
    pub fn named(name: impl Into<String>) -> Self {
        ManifestRef::Named(name.into())
    }

    /// Resolve a `ManifestRef` against a registry. Returns the borrowed
    /// bundle for `Named`, or the inline bundle for `Inline`.
    ///
    /// # Errors
    /// Returns `Err(ManifestError::Unknown)` when a `Named` reference does
    /// not match any registry entry (ESC-15).
    pub fn resolve<'a>(
        &'a self,
        registry: &'a ManifestRegistry,
    ) -> Result<&'a CapBundle, ManifestError> {
        match self {
            ManifestRef::Named(name) => registry.lookup(name),
            ManifestRef::Inline(bundle) => Ok(bundle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_defaults_at_construction() {
        let reg = ManifestRegistry::new();
        assert!(reg.lookup("compute-basic").is_ok());
        assert!(reg.lookup("compute-with-kv").is_ok());
    }

    #[test]
    fn registry_unknown_manifest_returns_typed_error() {
        let reg = ManifestRegistry::new();
        let err = reg.lookup("compute-power").unwrap_err();
        assert_eq!(err.code(), ErrorCode::SandboxManifestUnknown);
    }

    #[test]
    fn register_runtime_returns_deferred_error() {
        let mut reg = ManifestRegistry::new();
        let err = reg
            .register_runtime("custom", CapBundle::new(vec![], None))
            .unwrap_err();
        assert_eq!(err.code(), ErrorCode::SandboxManifestRegistrationDeferred);
    }

    #[test]
    fn canonical_bytes_round_trip_stable() {
        let reg = ManifestRegistry::new();
        let bundle = reg.lookup("compute-basic").unwrap();
        let bytes_1 = bundle.canonical_bytes().unwrap();
        let bytes_2 = bundle.canonical_bytes().unwrap();
        assert_eq!(bytes_1, bytes_2);
    }

    #[test]
    fn cap_bundle_caps_are_sorted_canonical() {
        // D9 canonical-bytes invariant: caps must be sorted in every
        // codegen-default bundle. Drift detector mirror.
        for (name, bundle) in default_manifests() {
            let mut sorted = bundle.caps.clone();
            sorted.sort();
            assert_eq!(
                bundle.caps, sorted,
                "manifest {name} caps must be sorted-canonical for DAG-CBOR stability"
            );
        }
    }
}
