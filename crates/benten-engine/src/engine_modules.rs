//! Phase 2b G10-B — Module manifest install / uninstall lifecycle on
//! [`crate::engine::Engine`].
//!
//! Per Phase-2b plan §3 G10-B exclusive ownership (wsa-r1-5 plan-internal
//! conflict resolution): G7-C does NOT own these methods. The lifecycle
//! lives here on the engine orchestrator alongside the privileged
//! system-zone writes (`grant_capability`, `create_view`).
//!
//! ## D16-RESOLVED-FURTHER — REQUIRED expected_cid arg
//!
//! [`Engine::install_module`] takes `expected_cid: Cid` as a REQUIRED
//! positional arg, NOT `Option<Cid>` and NOT a defaulted builder method.
//! The compile-time requirement closes the lazy-developer footgun where
//! a one-arg `install_module(m)` overload would silently
//! compute-and-trust the CID. On mismatch the error includes BOTH CIDs +
//! a 1-line manifest summary so the failure is operator-actionable
//! without a source-code dive.
//!
//! ## D9 — canonical DAG-CBOR
//!
//! The CID the engine computes is BLAKE3-of-the-canonical-DAG-CBOR-bytes
//! per [`crate::module_manifest::ModuleManifest::compute_cid`]. Two
//! logically-identical authoring inputs (e.g. JSON with different
//! field-order) collapse to the SAME CID — that property is what makes
//! the CID-pin operator-actionable across language boundaries.
//!
//! ## System-zone storage
//!
//! Installed manifests are written to the `system:ModuleManifest` zone
//! via the privileged write path (mirrors `grant_capability`). The
//! prefix is already declared in [`crate::system_zones::SYSTEM_ZONE_PREFIXES`].
//! The Node carries the canonical-bytes blob under property
//! `manifest_cbor` so a subsequent uninstall can re-derive the manifest
//! shape if needed (Phase 3 sync forwards the bytes verbatim).
//!
//! ## In-memory active set
//!
//! The engine keeps an in-memory `BTreeMap<Cid, InstalledModule>` of
//! actively-installed manifests. Uninstall removes the entry from the
//! active set (idempotent — a second uninstall on the same CID is a
//! no-op `Ok(())`) and also writes a `system:ModuleManifestRevocation`
//! Node so a sync replica that has only seen the revocation can still
//! recognize the manifest as uninstalled. Phase 3 generalizes this to
//! the full sync-aware retraction story.
//!
//! ## Capability retraction (D9 + plan §3.2 G10-B)
//!
//! Installing a manifest declares its `requires` capabilities into the
//! engine's manifest-scoped active-cap set. Uninstall retracts the
//! declaration **subject to multi-manifest cap-overlap**: if another
//! installed manifest still requires the same cap, the cap survives the
//! uninstall — only the M-scoped declaration is retracted. Verified by
//! `tests/integration/module_uninstall_releases_capabilities.rs`.

use std::collections::{BTreeMap, BTreeSet};

use benten_core::{Cid, Node, Value};

use crate::engine::Engine;
use crate::error::EngineError;
use crate::module_manifest::{ManifestError, ModuleManifest};

/// One installed-module record held in the engine's in-memory active set.
///
/// Phase-2b Phase-3 forward-compat note: when the persistent system-zone
/// store gains its IndexedDB / OPFS browser variant in Phase 3, the
/// in-memory active set is rebuilt at engine open by scanning the
/// `system:ModuleManifest` zone. The `manifest` field is what we'd
/// re-deserialize from the canonical-bytes property of the Node.
#[derive(Clone, Debug)]
pub(crate) struct InstalledModule {
    /// The decoded manifest (kept in memory for fast `requires`-lookup
    /// during capability-retraction calculations).
    pub(crate) manifest: ModuleManifest,
}

impl Engine {
    /// Install a module manifest — D16-RESOLVED-FURTHER.
    ///
    /// `expected_cid` is REQUIRED (not Optional). The engine computes
    /// the canonical CID of `manifest` via DAG-CBOR + BLAKE3 and asserts
    /// `computed == expected_cid`; on mismatch the call returns
    /// [`EngineError::ModuleManifestCidMismatch`] carrying BOTH CIDs and
    /// a 1-line manifest summary so the operator can identify the
    /// mis-installed manifest from logs alone.
    ///
    /// On success the manifest is persisted to the
    /// `system:ModuleManifest` zone via the privileged write path and
    /// added to the engine's in-memory active set. The returned `Cid`
    /// is the same value the caller passed as `expected_cid` (the
    /// engine confirms it by recomputing).
    ///
    /// ## Idempotence
    ///
    /// Re-installing a manifest whose CID is already in the active set
    /// returns `Ok(cid)` without re-writing the system-zone Node. This
    /// matches the storage layer's `inv_13` dedup behaviour.
    ///
    /// ## wasm32 in-memory-only constraint (Compromise #N+8)
    ///
    /// On `wasm32-unknown-unknown` the engine has no persistent backing
    /// store. Installing a manifest that declares any
    /// [`MigrationStep`](crate::module_manifest::MigrationStep)s fires
    /// [`EngineError::ModuleMigrationsRequirePersistence`] — there is
    /// nowhere durable for migrations to land. The in-memory active set
    /// itself works on every target.
    ///
    /// # Errors
    ///
    /// * [`EngineError::ModuleManifestCidMismatch`] on D16 mismatch.
    /// * [`EngineError::ModuleMigrationsRequirePersistence`] when
    ///   migrations are declared on a wasm32 target.
    /// * [`EngineError::Other`] wrapping a manifest encode failure
    ///   (infallible in practice for the [`ModuleManifest`] schema).
    /// * [`EngineError::Graph`] on backend write failure.
    pub fn install_module(
        &self,
        manifest: ModuleManifest,
        expected_cid: Cid,
    ) -> Result<Cid, EngineError> {
        // wasm32-unknown-unknown — Compromise #N+8 enforcement. The
        // browser engine ships in-memory-only manifests in Phase 2b;
        // migrations need a durable store that arrives in Phase 3.
        #[cfg(target_arch = "wasm32")]
        {
            if !manifest.migrations.is_empty() {
                return Err(EngineError::ModuleMigrationsRequirePersistence {
                    migration_count: manifest.migrations.len(),
                });
            }
        }

        // D16 — recompute the canonical CID and compare to the
        // caller-supplied expected_cid BEFORE any side-effecting work.
        let computed = manifest
            .compute_cid()
            .map_err(manifest_error_to_engine_error)?;
        if computed != expected_cid {
            return Err(EngineError::ModuleManifestCidMismatch {
                expected: expected_cid,
                computed,
                summary: manifest.summary().to_string(),
            });
        }

        // Idempotent re-install — the active set already has this CID,
        // no new system-zone Node, no cap-set churn.
        {
            let active = self
                .installed_modules()
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if active.contains_key(&computed) {
                return Ok(computed);
            }
        }

        // Persist to the system:ModuleManifest zone via the privileged
        // path. The Node carries the canonical-bytes blob so a Phase-3
        // sync replica can rehydrate without re-encoding.
        let bytes = manifest
            .to_canonical_bytes()
            .map_err(manifest_error_to_engine_error)?;
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("manifest_cbor".into(), Value::Bytes(bytes));
        props.insert("manifest_cid".into(), Value::Text(computed.to_base32()));
        props.insert("name".into(), Value::Text(manifest.name.clone()));
        props.insert("version".into(), Value::Text(manifest.version.clone()));
        let node = Node::new(vec!["system:ModuleManifest".into()], props);
        // Privileged write — mirrors grant_capability's path. We do NOT
        // depend on the returned storage CID matching `computed` (the
        // storage CID hashes the FULL Node including the label, which is
        // a different shape than the manifest-bytes hash). The
        // `manifest_cid` property + the engine's in-memory active set
        // are the indexes of record.
        self.backend()
            .put_node_with_context(
                &node,
                &benten_graph::WriteContext::privileged_for_engine_api(),
            )
            .map_err(EngineError::from)?;

        // Add to in-memory active set.
        {
            let mut active = self
                .installed_modules()
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            active.insert(computed, InstalledModule { manifest });
        }

        Ok(computed)
    }

    /// Uninstall a module manifest by CID.
    ///
    /// **Idempotent** — uninstalling an already-uninstalled CID (or a
    /// CID that was NEVER installed) returns `Ok(())`. The idempotence
    /// boundary is the CID, not the install history.
    ///
    /// On a real uninstall the engine removes the manifest from the
    /// in-memory active set and writes a `system:ModuleManifestRevocation`
    /// Node (mirrors `revoke_capability`). The revocation Node lets a
    /// Phase-3 sync replica recognize the uninstall even if it never saw
    /// the original install.
    ///
    /// Capabilities the manifest declared in its `requires` block are
    /// retracted from the manifest-scoped active-cap set, **subject to
    /// multi-manifest overlap**: if another installed manifest still
    /// requires the same cap, the cap survives the uninstall.
    ///
    /// # Errors
    ///
    /// [`EngineError::Graph`] on backend write failure when writing the
    /// revocation Node.
    pub fn uninstall_module(&self, cid: Cid) -> Result<(), EngineError> {
        // Pull the manifest out of the active set; if absent the call
        // is a no-op (idempotence boundary).
        let removed = {
            let mut active = self
                .installed_modules()
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            active.remove(&cid)
        };

        // Idempotence: a never-installed CID also returns Ok(()). We
        // still skip the revocation-Node write for a never-installed
        // CID — there's nothing to revoke. Phase 3 sync may want to
        // forward the revocation regardless; that decision lands with
        // the sync layer.
        if removed.is_none() {
            return Ok(());
        }

        // Write the revocation Node. Mirrors revoke_capability's
        // pattern. The Node carries the manifest CID so a sync replica
        // can match it against any previously-seen install.
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("manifest_cid".into(), Value::Text(cid.to_base32()));
        let node = Node::new(vec!["system:ModuleManifestRevocation".into()], props);
        self.backend()
            .put_node_with_context(
                &node,
                &benten_graph::WriteContext::privileged_for_engine_api(),
            )
            .map_err(EngineError::from)?;

        Ok(())
    }

    /// Returns `true` iff the manifest with the given CID is currently
    /// in the engine's installed-modules active set.
    ///
    /// Used by tests + the Phase-2b devtools introspection accessor.
    /// Cheap: pure in-memory lookup.
    #[must_use]
    pub fn is_module_installed(&self, cid: &Cid) -> bool {
        let active = self
            .installed_modules()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        active.contains_key(cid)
    }

    /// Returns the deduplicated set of capabilities currently declared
    /// across every installed manifest.
    ///
    /// Used by the capability-retraction tests (`module_uninstall.rs` +
    /// `tests/integration/module_uninstall_releases_capabilities.rs`)
    /// to assert the cap-overlap rule: uninstalling M does NOT retract
    /// caps that a sibling manifest still requires.
    #[must_use]
    pub fn active_module_capabilities(&self) -> BTreeSet<String> {
        let active = self
            .installed_modules()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        active
            .values()
            .flat_map(|m| {
                m.manifest
                    .modules
                    .iter()
                    .flat_map(|entry| entry.requires.iter().cloned())
            })
            .collect()
    }

    /// Compute the canonical CID of a manifest WITHOUT installing it.
    ///
    /// Used by callers that want to verify the CID before passing it
    /// as the required arg to [`Self::install_module`]. Pure function
    /// — no engine state inspected.
    ///
    /// # Errors
    ///
    /// [`EngineError::Other`] wrapping a manifest encode failure
    /// (infallible in practice for the [`ModuleManifest`] schema).
    pub fn compute_manifest_cid(&self, manifest: &ModuleManifest) -> Result<Cid, EngineError> {
        manifest
            .compute_cid()
            .map_err(manifest_error_to_engine_error)
    }
}

/// Map a [`ManifestError`] (encode / decode failure from the
/// manifest module) into an [`EngineError`]. The typed CID-mismatch
/// + migrations-require-persistence variants are constructed at the
/// engine call site — they do not flow through this function because
/// they originate from engine-side checks, not from the pure-data
/// serializer.
fn manifest_error_to_engine_error(err: ManifestError) -> EngineError {
    use benten_errors::ErrorCode;
    match err {
        ManifestError::Encode(msg) => EngineError::Other {
            code: ErrorCode::Unknown("E_MODULE_MANIFEST_ENCODE_FAILURE".into()),
            message: format!("manifest encode failure: {msg}"),
        },
        ManifestError::Decode(msg) => EngineError::Other {
            code: ErrorCode::Unknown("E_MODULE_MANIFEST_DECODE_FAILURE".into()),
            message: format!("manifest decode failure: {msg}"),
        },
    }
}
