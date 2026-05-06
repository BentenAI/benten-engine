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
use crate::manifest_signing::{
    ManifestVerifyArgs, ManifestVerifyError, ManifestVerifyMode, verify_manifest_with_mode,
};
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
    /// ## Bytes-persistence asymmetry (Compromise #17)
    ///
    /// `install_module` persists the **manifest** (canonical-DAG-CBOR
    /// bytes + summary + name + version) into the
    /// `system:ModuleManifest` zone via the privileged write path. This
    /// is durable: the manifest survives engine restart and is
    /// sync-eligible for Phase-3 federation. R6FP-Group-1 (r6-arch-1):
    /// the in-memory active set is rebuilt at engine open via
    /// `Self::rehydrate_installed_modules_from_zone` — pre-fix the
    /// docstring claim above was honoured on disk only and a freshly-
    /// opened engine returned `false` from
    /// [`Self::is_module_installed`] for previously-installed CIDs.
    ///
    /// However, the underlying **wasm bytes** that each manifest entry
    /// references (`modules[i].cid`) are NOT auto-persisted. Operators
    /// must call [`Engine::register_module_bytes`] separately, and must
    /// re-call it after every engine open. The asymmetry IS Compromise
    /// #17 — see `docs/SECURITY-POSTURE.md` "Compromise #17 — In-memory
    /// module-bytes registry" for the full narrative + Phase-3
    /// promotion path (durable `BlobBackend` lifts both arms together).
    ///
    /// ## Signature verification (Compromise #21 closure — g14-c-mr-1)
    ///
    /// Pre-fix-pass, this method did NOT invoke
    /// [`crate::manifest_signing::verify_manifest_with_mode`] — the
    /// helper existed but was never wired into the production install
    /// path. SECURITY-POSTURE.md Compromise #21 narrative (audience-
    /// binding to UCAN-proof-chain at install_module verification)
    /// was therefore false.
    ///
    /// Post-fix-pass, the caller MUST supply a
    /// [`crate::manifest_signing::ManifestVerifyArgs`] explicitly. The
    /// argument's [`crate::manifest_signing::ManifestVerifyMode`]
    /// names the policy:
    ///
    /// - [`ManifestVerifyMode::Unsigned`] — development-only;
    ///   verification skipped. The relaxation is NAMED at the
    ///   call-site so production code can't fall through silently.
    /// - [`ManifestVerifyMode::Any`] — UCAN OR registry path verifies.
    /// - [`ManifestVerifyMode::All`] — BOTH UCAN AND registry paths
    ///   verify.
    ///
    /// Verification runs BEFORE the privileged-write of the manifest
    /// to the `system:ModuleManifest` zone — a verification failure
    /// returns [`EngineError::ModuleManifestVerify`] without persisting
    /// the manifest.
    ///
    /// # Errors
    ///
    /// * [`EngineError::ModuleManifestCidMismatch`] on D16 mismatch.
    /// * [`EngineError::ModuleMigrationsRequirePersistence`] when
    ///   migrations are declared on a wasm32 target.
    /// * [`EngineError::ModuleManifestVerify`] when signature
    ///   verification fails per the supplied [`ManifestVerifyArgs`].
    /// * [`EngineError::Other`] wrapping a manifest encode failure
    ///   (infallible in practice for the [`ModuleManifest`] schema).
    /// * [`EngineError::Graph`] on backend write failure.
    pub fn install_module(
        &self,
        manifest: ModuleManifest,
        expected_cid: Cid,
        verify_args: ManifestVerifyArgs<'_>,
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

        // g14-c-mr-1 BLOCKER fix-pass (Compromise #21 closure):
        // signature verification BEFORE persistence + active-set
        // mutation. Unsigned mode skips verification (development-
        // only relaxation, NAMED at call-site). Any/All modes run
        // through `verify_manifest_with_mode` which enforces audience-
        // binding (CLR-2 / cap-major-2 cross-atrium replay defense)
        // when a UCAN chain is present.
        if !matches!(verify_args.mode, ManifestVerifyMode::Unsigned) {
            // Audience DID is required for non-Unsigned modes — the
            // typed argument shape forbids a UCAN path without one.
            // We surface UcanRequiredByModeAll for the All variant
            // and NoPathPresent for Any when the caller failed to
            // supply a path.
            let audience = verify_args.engine_audience_did.ok_or_else(|| {
                EngineError::ModuleManifestVerify(ManifestVerifyError::UcanInvalid(
                    "engine_audience_did required for non-Unsigned verify modes".to_string(),
                ))
            })?;
            verify_manifest_with_mode(
                &manifest,
                verify_args.ucan_chain,
                verify_args.registry_pubkey,
                audience,
                verify_args.mode,
                verify_args.now,
            )
            .map_err(EngineError::ModuleManifestVerify)?;
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

    /// R6FP-Group-1 (r6-arch-1) — Rehydrate the in-memory installed-modules
    /// active set from the durable `system:ModuleManifest` zone.
    ///
    /// Pre-R6FP-G1, [`Self::install_module`] persisted manifests to the
    /// `system:ModuleManifest` zone (durable across engine restart) but
    /// the in-memory `installed_modules` BTreeMap was NOT rebuilt at
    /// engine open — so a freshly-restarted Engine returned `false` from
    /// [`Self::is_module_installed`] for previously-installed CIDs even
    /// though the system-zone Node was on disk. The install_module
    /// docstring claimed "the manifest survives engine restart and is
    /// sync-eligible for Phase-3 federation"; the code only honoured the
    /// "survives engine restart" half on disk, not in the in-memory
    /// indexes the dispatcher consults.
    ///
    /// This helper closes the docs/code drift surfaced by R6's
    /// architect-reviewer lens (r6-arch-1). It is invoked by
    /// `EngineBuilder::assemble` once after the backend is open + the
    /// engine is constructed; failures during rehydration are
    /// non-fatal (logged via tracing warn) so a corrupt or partial
    /// system-zone Node does not block engine startup.
    ///
    /// **wasm32 cut:** the SANDBOX subsystem is `cfg(not(target_arch =
    /// "wasm32"))`, but the manifest zone scan is target-agnostic
    /// (uses only `benten_graph` accessors). The wasm32 engine has no
    /// durable backend so the scan returns an empty set.
    ///
    /// # Errors
    ///
    /// [`EngineError::Graph`] if the backend's `get_by_label` /
    /// `get_node` accessors error. Decode failures of individual
    /// manifest Nodes are logged + skipped rather than aborting the
    /// scan (best-effort hydration).
    pub(crate) fn rehydrate_installed_modules_from_zone(&self) -> Result<usize, EngineError> {
        let cids = self.backend.get_by_label("system:ModuleManifest")?;
        let mut rebuilt: BTreeMap<Cid, InstalledModule> = BTreeMap::new();
        for cid in cids {
            let Some(node) = self.backend.get_node(&cid)? else {
                continue;
            };
            // Pull the canonical-bytes blob out of the Node + decode.
            let bytes = match node.properties.get("manifest_cbor") {
                Some(Value::Bytes(b)) => b.clone(),
                _ => continue, // malformed Node — skip
            };
            let manifest = match ModuleManifest::from_canonical_bytes(&bytes) {
                Ok(m) => m,
                Err(_) => continue, // decode failure — skip
            };
            // Recompute the manifest CID under the canonical hash so a
            // Node whose `manifest_cid` property has drifted from the
            // bytes-hash is rejected (defense against on-disk
            // corruption surfacing as a phantom installed module).
            let computed = match manifest.compute_cid() {
                Ok(c) => c,
                Err(_) => continue,
            };
            rebuilt.insert(computed, InstalledModule { manifest });
        }
        let count = rebuilt.len();
        let mut active = self
            .installed_modules()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Merge into the active set; in practice the active set is
        // empty at engine open (this is called from assemble()
        // immediately after `Engine::from_parts_with_clocks`).
        for (cid, installed) in rebuilt {
            active.insert(cid, installed);
        }
        Ok(count)
    }

    /// Wave-8h audit-gap fix — build a [`ManifestRegistry`] hydrated from
    /// the codegen-default manifests PLUS the engine's currently-installed
    /// modules.
    ///
    /// **Why this exists:** the audit at
    /// `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
    /// surfaced that `execute_sandbox` was constructing
    /// `ManifestRegistry::new()` (codegen-defaults only) at every call,
    /// so a SANDBOX node carrying `manifest: '<installed-name>'` could
    /// never resolve through the production path even after the caller
    /// invoked [`Self::install_module`]. The Named-manifest API was
    /// effectively a placeholder.
    ///
    /// **Mapping rule:** each installed [`ModuleManifest`]'s
    /// `modules: Vec<ModuleManifestEntry>` projects to one registry
    /// entry per `ModuleManifestEntry`, keyed by `entry.name` with
    /// `entry.requires` lifted into the [`CapBundle`]'s caps. Entry
    /// names that collide with codegen-default registry keys (or with
    /// each other across multiple installed manifests) are overridden
    /// last-write-wins per `BTreeMap::insert` semantics.
    ///
    /// The registry is reconstructed per-call (fresh `BTreeMap` clone
    /// from `installed_modules` + codegen defaults) so manifest install
    /// / uninstall is observable on the next SANDBOX dispatch without
    /// any cache-invalidation work.
    ///
    /// **wasm32 cut:** the SANDBOX subsystem (`benten_eval::sandbox`)
    /// is `#[cfg(not(target_arch = "wasm32"))]` because wasmtime does
    /// not compile to wasm32. The production `execute_sandbox` override
    /// in `primitive_host.rs` is gated the same way; this accessor's
    /// only consumer is that override, so the cfg-gate matches.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn manifest_registry(&self) -> benten_eval::sandbox::ManifestRegistry {
        let active = self
            .installed_modules()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let mut overlay: BTreeMap<String, benten_eval::sandbox::CapBundle> = BTreeMap::new();
        for installed in active.values() {
            for entry in &installed.manifest.modules {
                // Sort + dedupe the entry's requires list so the
                // resulting CapBundle satisfies the D9 sorted-canonical
                // invariant `default_manifests()` enforces. The
                // codegen-default bundles are already sorted; the
                // install-time path needs to apply the same discipline.
                let mut caps: Vec<String> = entry.requires.clone();
                caps.sort();
                caps.dedup();
                overlay.insert(
                    entry.name.clone(),
                    benten_eval::sandbox::CapBundle::new(caps, None),
                );
            }
        }
        benten_eval::sandbox::ManifestRegistry::from_overlay(overlay)
    }

    /// Phase-3 G17-A2 — look up the per-manifest `random` host-fn
    /// per-call entropy-budget override (if any) for a Named manifest.
    ///
    /// Walks the engine's `installed_modules` set; for each installed
    /// manifest whose **entry name** OR **manifest name** matches
    /// `manifest_name`, returns
    /// `manifest.host_fns.random.budget_bytes_per_call` if all
    /// intermediate fields are present. Returns `None` when:
    ///   - the manifest name is not installed,
    ///   - the manifest carries no `host_fns` override,
    ///   - the override carries no `random` entry, or
    ///   - the `random` entry carries no `budget_bytes_per_call`.
    ///
    /// `None` flows through `SandboxConfig::random_budget_bytes_per_call`
    /// as "no override" so the codegen default (4096 per r1-wsa-8)
    /// applies.
    ///
    /// **Lookup rationale:** the `manifest` property on a SANDBOX node
    /// is the manifest registry key at SANDBOX dispatch (e.g. an entry
    /// `name` from `manifest.modules[i].name`). The manifest-level
    /// `host_fns` override hangs off the parent `ModuleManifest`, so
    /// we resolve via entry-match → parent. Codegen-default registry
    /// names (`compute-basic`, etc.) carry no override and yield
    /// `None` here, which is correct.
    ///
    /// **wasm32 cut:** matches the [`Self::manifest_registry`] gate —
    /// the only consumer is the production `execute_sandbox` override
    /// in `primitive_host.rs`, which is itself wasm32-cut.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn random_budget_for_named_manifest(&self, manifest_name: &str) -> Option<u64> {
        let active = self
            .installed_modules()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        for installed in active.values() {
            let manifest_match = installed.manifest.name == manifest_name;
            let entry_match = installed
                .manifest
                .modules
                .iter()
                .any(|e| e.name == manifest_name);
            if manifest_match || entry_match {
                return installed
                    .manifest
                    .host_fns
                    .as_ref()
                    .and_then(|hf| hf.random.as_ref())
                    .and_then(|r| r.budget_bytes_per_call);
            }
        }
        None
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
