//! Phase-4-Foundation G24-D + G24-D-FP-1 — plugin uninstall lifecycle.
//!
//! Hosts the `uninstall_plugin` cascade per CLAUDE.md baked-in #18 +
//! `docs/PLUGIN-MANIFEST.md` §4.2.
//!
//! ## Substantive cascade (G24-D-FP-1)
//!
//! `uninstall_plugin` cascades through five concerns:
//!
//! 1. **Held caps revoke** — every UCAN grant with `audience=plugin_did`
//!    is revoked. Couples to PR #199 `revoke_capability_by_grant_cid`.
//!    (T10-uninstall (a) per threat-model.)
//!
//! 2. **Downstream delegations cascade-revoke** — every grant the
//!    plugin issued (`issuer=plugin_did`) is revoked. Per CLAUDE.md
//!    baked-in #18 Layer 3: the manifest envelope's transitivity
//!    guarantee requires cascade. (T10-uninstall (b).)
//!
//! 3. **Live subscription termination** — every active SUBSCRIBE under
//!    the plugin-DID is terminated. LOAD-BEARING per threat-model §T10
//!    + T12 cross-process amplification defense. (T10-uninstall (c).)
//!
//! 4. **Private-namespace teardown** — every row under
//!    `private:<plugin_did>:*` scope-prefix is deleted; re-install does
//!    not inherit stale state (T7 isolation guarantee).
//!
//! 5. **Library-entry removal + plugin-DID revoke** — the entry is
//!    removed from the `PluginLibrary` (also clears active reference);
//!    the plugin-DID is revoked from the `PluginDidStore` so future
//!    re-install would mint a fresh DID and re-prompt user consent.
//!
//! ## Dep-direction discipline (arch-r1-1 + arch-r1-15)
//!
//! The `benten-platform-foundation` crate MUST NOT depend on
//! `benten-eval` / `benten-graph` / `benten-engine` in production. The
//! cascade therefore models its cross-crate consumers via the
//! [`CapRevoker`], [`PrivateNamespaceTeardown`], and
//! [`SubscriptionRegistry`] trait ports — engine-side adapters
//! implement these against real cap-store / graph-backend /
//! subscription-registry surfaces; the foundation crate's tests
//! consume the [`InMemoryUninstallCascade`] default which faithfully
//! reproduces the observable semantics without a backend.
//!
//! ## Couples
//!
//! - Phase-3 G16-B-F per-row cap-recheck — once a grant is revoked
//!   here, in-flight reads surface as `E_CAP_REVOKED` via the per-row
//!   gate (test fixture mirrors that semantic).
//! - PR #199 `Engine::revoke_capability_by_grant_cid` — engine-side
//!   adapter routes through that typed surface.

use crate::plugin_library::{LibraryEntry, PluginLibrary};
use crate::plugin_manifest::{
    InstallRecord, MANIFEST_CLOCK_NOT_INJECTED_SENTINEL, PluginManifest, ValidationOutcome,
    detect_composition_cycle,
};
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::plugin_did::{PluginDidStore, mint as mint_plugin_did};
use std::collections::{HashMap, HashSet};

/// Result of `uninstall_plugin` — observable counters that callers can
/// pin in tests + observability.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UninstallOutcome {
    /// Count of grants with `audience=plugin_did` that were revoked
    /// (T10-uninstall (a)).
    pub held_caps_revoked: usize,
    /// Count of grants with `issuer=plugin_did` that were cascade-
    /// revoked (T10-uninstall (b)).
    pub delegations_cascade_revoked: usize,
    /// Count of live SUBSCRIBE subscriptions terminated for the
    /// uninstalled plugin-DID (T10-uninstall (c)).
    pub subscriptions_terminated: usize,
    /// Count of private-namespace rows deleted under
    /// `private:<plugin_did>:*`.
    pub private_namespace_rows_deleted: usize,
    /// Whether the library entry was removed.
    pub library_entry_removed: bool,
    /// Whether the plugin-DID was revoked from the store.
    pub plugin_did_revoked: bool,
}

// =====================================================================
// Cascade trait ports
// =====================================================================

/// Port the engine adapter implements to drive cap-revocation as part
/// of `uninstall_plugin`.
///
/// Engine-side adapter wires the two methods to
/// `Engine::revoke_capability_by_grant_cid` (PR #199) iterated over
/// the cap-store's `audience` / `issuer` indexes. The foundation
/// crate provides [`InMemoryUninstallCascade`] as the substantive
/// in-memory default consumed by every G24-D-FP-1 RED-PHASE pin.
pub trait CapRevoker {
    /// Revoke every grant whose audience equals `plugin_did`. Returns
    /// the count revoked.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` on adapter failure.
    fn revoke_grants_with_audience(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode>;

    /// Cascade-revoke every grant whose issuer equals `plugin_did`.
    /// Returns the count revoked. Cascade source is `plugin_did`.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` on adapter failure.
    fn cascade_revoke_grants_with_issuer(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode>;
}

/// Port the storage-backend adapter implements to drive private-
/// namespace teardown.
///
/// Engine-side adapter walks the storage backend for rows scope-
/// prefixed `private:<plugin_did>:` and deletes each. The foundation
/// crate's tests use [`InMemoryUninstallCascade`] as the substantive
/// default.
pub trait PrivateNamespaceTeardown {
    /// Delete every row under `private:<plugin_did>:*`. Returns the
    /// row count.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` on adapter failure.
    fn delete_private_namespace_for(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode>;
}

/// Port the subscription-registry adapter implements to terminate
/// live SUBSCRIBE handles owned by an uninstalled plugin-DID.
///
/// LOAD-BEARING per threat-model §T10 + T12 cross-process
/// amplification defense — without this, post-uninstall writes still
/// deliver events to the orphaned subscriber.
pub trait SubscriptionRegistry {
    /// Terminate every active subscription whose subscriber DID
    /// equals `plugin_did`. Returns the count terminated.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` on adapter failure.
    fn terminate_subscriptions_for(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode>;

    /// Count of currently-active subscriptions for `plugin_did`
    /// (defense-in-depth observable; tests pin "registry empty for
    /// uninstalled DID").
    fn active_subscription_count(&self, plugin_did: &Did) -> usize;
}

/// Bundle of the three engine-side ports — passed to
/// [`uninstall_plugin`] so the cascade can drive each concern in turn.
pub struct UninstallContext<'a, R, P, S>
where
    R: CapRevoker,
    P: PrivateNamespaceTeardown,
    S: SubscriptionRegistry,
{
    /// Cap revocation port (held + delegated cascades).
    pub cap_revoker: &'a mut R,
    /// Private-namespace teardown port.
    pub private_ns: &'a mut P,
    /// Subscription-registry termination port.
    pub subscriptions: &'a mut S,
}

// =====================================================================
// uninstall_plugin — the substantive cascade
// =====================================================================

/// Uninstall a plugin by manifest-CID with the FULL cascade.
///
/// Cascade order:
/// 1. Revoke held caps (audience=plugin_did)
/// 2. Cascade-revoke downstream delegations (issuer=plugin_did)
/// 3. Terminate live subscriptions
/// 4. Tear down private namespace
/// 5. Remove library entry + revoke plugin-DID from store
///
/// The cascade is ordered so that in-flight reads / subscribes that
/// observe the cap-revoke first cannot race against the library/DID
/// removal (per-row recheck per Phase-3 G16-B-F surfaces
/// `E_CAP_REVOKED` immediately after step 1).
///
/// # Errors
///
/// - `E_PLUGIN_MANIFEST_INVALID` if `manifest_cid` is not in the library.
/// - `E_INTERNAL` propagated from any adapter port.
pub fn uninstall_plugin<R, P, S>(
    library: &mut PluginLibrary,
    plugin_did_store: &mut PluginDidStore,
    ctx: &mut UninstallContext<'_, R, P, S>,
    manifest_cid: &Cid,
) -> Result<UninstallOutcome, ErrorCode>
where
    R: CapRevoker,
    P: PrivateNamespaceTeardown,
    S: SubscriptionRegistry,
{
    // Resolve plugin-DID FIRST — if the manifest isn't in the library
    // we surface E_PLUGIN_MANIFEST_INVALID without touching any
    // cascade-port (so a bogus manifest-CID can't observably revoke
    // unrelated caps).
    let plugin_did = library
        .get(manifest_cid)
        .map(|e| e.plugin_did.clone())
        .ok_or(ErrorCode::PluginManifestInvalid)?;

    // 1. Revoke held caps (T10-uninstall (a)).
    let held_caps_revoked = ctx.cap_revoker.revoke_grants_with_audience(&plugin_did)?;

    // 2. Cascade-revoke downstream delegations (T10-uninstall (b)).
    let delegations_cascade_revoked = ctx
        .cap_revoker
        .cascade_revoke_grants_with_issuer(&plugin_did)?;

    // 3. Terminate live subscriptions (T10-uninstall (c) LOAD-BEARING).
    let subscriptions_terminated = ctx.subscriptions.terminate_subscriptions_for(&plugin_did)?;

    // 4. Tear down private namespace.
    let private_namespace_rows_deleted =
        ctx.private_ns.delete_private_namespace_for(&plugin_did)?;

    // 5. Remove library entry + revoke plugin-DID.
    let library_entry_removed = library.remove(manifest_cid).is_some();
    let plugin_did_revoked = plugin_did_store.revoke(&plugin_did);

    Ok(UninstallOutcome {
        held_caps_revoked,
        delegations_cascade_revoked,
        subscriptions_terminated,
        private_namespace_rows_deleted,
        library_entry_removed,
        plugin_did_revoked,
    })
}

/// Surface the plugin-DID associated with a library entry, for engine-
/// side adapters that need to look it up directly.
#[must_use]
pub fn plugin_did_for_entry(library: &PluginLibrary, manifest_cid: &Cid) -> Option<Did> {
    library.get(manifest_cid).map(|e| e.plugin_did.clone())
}

// =====================================================================
// PULL-not-PUSH new-version discovery (plugin-arch-r1-13)
// =====================================================================

/// Outcome of a `discover_new_version` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewVersionDiscoveryOutcome {
    /// The announced CID is a DAG-descendant of the installed plugin's
    /// CURRENT pointer — admin UI is notified via the
    /// `E_PLUGIN_NEW_VERSION_AVAILABLE` typed code.
    NewVersionAvailable {
        /// The CID of the announced newer version.
        announced_cid: Cid,
        /// The plugin name (admin UI uses this to surface
        /// "<plugin_name> has a new version" prompt).
        plugin_name: String,
    },
    /// The announced CID is unrelated to any installed plugin OR is
    /// not a descendant of the CURRENT pointer — no notification
    /// emitted.
    NoChange,
}

/// PULL-not-PUSH new-version discovery.
///
/// Per `docs/PLUGIN-MANIFEST.md` §4 + plugin-arch-r1-13: when a peer
/// announces a candidate CID through an Atrium channel, the receiver
/// consults its installed plugins' DAG-version-chain. If the announced
/// CID is a descendant of the installed plugin's CURRENT pointer, the
/// admin UI surfaces the typed `E_PLUGIN_NEW_VERSION_AVAILABLE`
/// notification (NOT a hard-reject — pull-not-push: user decides
/// whether to upgrade).
///
/// This function is the engine-boundary anchor; the atrium-side
/// peer-discovery wiring (Phase-3 `benten-sync` topic subscription;
/// inactive until the foundation crate is wired into the sync
/// runtime) calls this with `(library, announced_cid, version_chain)`
/// on each peer announce.
///
/// Dep-direction discipline: this function takes a `&DagVersionChain`
/// + a `&PluginLibrary` rather than driving atrium discovery itself —
/// the foundation crate does NOT depend on benten-sync. The atrium-
/// side adapter is responsible for calling this when peer announces
/// arrive.
#[must_use]
pub fn discover_new_version(
    library: &PluginLibrary,
    announced_cid: Cid,
    version_chain: &benten_core::version_chain::DagVersionChain,
) -> NewVersionDiscoveryOutcome {
    // The announced CID matches an already-installed plugin → no new
    // version (this is a re-announce of what we already have).
    if library.get(&announced_cid).is_some() {
        return NewVersionDiscoveryOutcome::NoChange;
    }
    // Find any installed entry whose CID is an ancestor of the
    // announced CID via the supplied version chain. If found, surface
    // the new-version-available notification.
    for entry in library.entries() {
        if version_chain.is_ancestor_of(&entry.manifest_cid, &announced_cid) {
            return NewVersionDiscoveryOutcome::NewVersionAvailable {
                announced_cid,
                plugin_name: entry.manifest.plugin_name.clone(),
            };
        }
    }
    NewVersionDiscoveryOutcome::NoChange
}

// =====================================================================
// InMemoryUninstallCascade — the substantive in-memory default
// =====================================================================

/// In-memory record of a grant for substantive testing.
///
/// Mirrors the shape of a Phase-3 UCAN grant (issuer / audience /
/// scope / cid) without depending on `benten-caps`. The
/// [`InMemoryUninstallCascade`] indexes grants by CID + maintains
/// audience / issuer / revocation views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryGrant {
    /// Stable CID handle (test-supplied; production adapter routes
    /// through real grant CIDs).
    pub grant_cid: Cid,
    /// The audience DID — the entity the cap is issued to.
    pub audience: Did,
    /// The issuer DID — the entity that issued the cap.
    pub issuer: Did,
    /// Capability scope string.
    pub scope: String,
}

/// Revocation log entry captured by [`InMemoryUninstallCascade`] for
/// defense-in-depth observability (tests assert revocation source,
/// audience, and cascade tag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevocationLogEntry {
    /// The CID of the revoked grant.
    pub grant_cid: Cid,
    /// The grant's audience at revocation time.
    pub audience: Did,
    /// The grant's issuer at revocation time.
    pub issuer: Did,
    /// `Some(plugin_did)` if this was a cascade-revoke (T10-uninstall
    /// (b)); `None` for direct revocation (T10-uninstall (a)).
    pub cascade_source: Option<Did>,
}

/// In-memory subscription handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemorySubscription {
    /// The plugin-DID that owns this subscription.
    pub subscriber: Did,
    /// Cap scope subscribed to (for observability; not consulted at
    /// terminate-time).
    pub scope: String,
}

/// Substantive in-memory implementation of every cascade port.
///
/// Powers G24-D-FP-1 RED-PHASE test pins WITHOUT requiring the
/// engine. Production engine-side adapters route the same trait
/// methods to real cap-store / graph-backend / subscription-registry
/// surfaces, preserving the observable semantics.
#[derive(Debug, Default)]
pub struct InMemoryUninstallCascade {
    /// All grants keyed by CID.
    grants: HashMap<Cid, InMemoryGrant>,
    /// Revocation log — append-only.
    revocation_log: Vec<RevocationLogEntry>,
    /// Active subscriptions keyed by an opaque id (Vec ordering
    /// preserves insertion).
    subscriptions: Vec<InMemorySubscription>,
    /// Private-namespace rows; key is the full scope (e.g.
    /// `private:did:key:z6Mk...:notes/2024`); value is the row body.
    private_rows: HashMap<String, Vec<u8>>,
    /// Set of revoked grant CIDs (defense-in-depth: an attacker that
    /// re-inserts via the public test surface still appears in the
    /// log).
    revoked: HashSet<Cid>,
}

impl InMemoryUninstallCascade {
    /// New empty cascade.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a grant (test fixture surface).
    pub fn insert_grant(&mut self, grant: InMemoryGrant) {
        self.grants.insert(grant.grant_cid, grant);
    }

    /// Insert a subscription (test fixture surface).
    pub fn insert_subscription(&mut self, sub: InMemorySubscription) {
        self.subscriptions.push(sub);
    }

    /// Insert a private-namespace row (test fixture surface). Scope
    /// MUST start with `private:<plugin_did>:`.
    pub fn insert_private_row(&mut self, scope: String, body: Vec<u8>) {
        self.private_rows.insert(scope, body);
    }

    /// Snapshot active grants for an audience (test observable for
    /// T10-uninstall (a) baseline + post-uninstall assertion).
    #[must_use]
    pub fn active_grants_for_audience(&self, audience: &Did) -> Vec<&InMemoryGrant> {
        self.grants
            .values()
            .filter(|g| g.audience == *audience && !self.revoked.contains(&g.grant_cid))
            .collect()
    }

    /// Snapshot active grants issued by `issuer` (test observable for
    /// T10-uninstall (b) baseline).
    #[must_use]
    pub fn active_grants_with_issuer(&self, issuer: &Did) -> Vec<&InMemoryGrant> {
        self.grants
            .values()
            .filter(|g| g.issuer == *issuer && !self.revoked.contains(&g.grant_cid))
            .collect()
    }

    /// Revocation log (defense-in-depth observable for cascade-source
    /// tagging assertion).
    #[must_use]
    pub fn revocation_log(&self) -> &[RevocationLogEntry] {
        &self.revocation_log
    }

    /// Snapshot private-namespace rows for a plugin-DID (test
    /// observable for T7 isolation guarantee).
    #[must_use]
    pub fn private_rows_for(&self, plugin_did: &Did) -> Vec<&String> {
        let prefix = format!("private:{}:", plugin_did.as_str());
        self.private_rows
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .collect()
    }

    /// Snapshot subscriptions for a plugin-DID (test observable for
    /// T10-uninstall (c) defense-in-depth).
    #[must_use]
    pub fn active_subscriptions_for(&self, plugin_did: &Did) -> Vec<&InMemorySubscription> {
        self.subscriptions
            .iter()
            .filter(|s| s.subscriber == *plugin_did)
            .collect()
    }
}

impl CapRevoker for InMemoryUninstallCascade {
    fn revoke_grants_with_audience(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode> {
        let to_revoke: Vec<Cid> = self
            .grants
            .values()
            .filter(|g| g.audience == *plugin_did && !self.revoked.contains(&g.grant_cid))
            .map(|g| g.grant_cid)
            .collect();
        let count = to_revoke.len();
        for cid in to_revoke {
            // SAFETY: every cid we collected just exists in grants.
            let grant = self
                .grants
                .get(&cid)
                .expect("collected cid was iterated from grants")
                .clone();
            self.revoked.insert(cid);
            self.revocation_log.push(RevocationLogEntry {
                grant_cid: cid,
                audience: grant.audience,
                issuer: grant.issuer,
                cascade_source: None,
            });
        }
        Ok(count)
    }

    fn cascade_revoke_grants_with_issuer(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode> {
        let to_revoke: Vec<Cid> = self
            .grants
            .values()
            .filter(|g| g.issuer == *plugin_did && !self.revoked.contains(&g.grant_cid))
            .map(|g| g.grant_cid)
            .collect();
        let count = to_revoke.len();
        for cid in to_revoke {
            let grant = self
                .grants
                .get(&cid)
                .expect("collected cid was iterated from grants")
                .clone();
            self.revoked.insert(cid);
            self.revocation_log.push(RevocationLogEntry {
                grant_cid: cid,
                audience: grant.audience,
                issuer: grant.issuer,
                cascade_source: Some(plugin_did.clone()),
            });
        }
        Ok(count)
    }
}

impl PrivateNamespaceTeardown for InMemoryUninstallCascade {
    fn delete_private_namespace_for(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode> {
        let prefix = format!("private:{}:", plugin_did.as_str());
        let keys: Vec<String> = self
            .private_rows
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        let count = keys.len();
        for k in keys {
            self.private_rows.remove(&k);
        }
        Ok(count)
    }
}

impl SubscriptionRegistry for InMemoryUninstallCascade {
    fn terminate_subscriptions_for(&mut self, plugin_did: &Did) -> Result<usize, ErrorCode> {
        let before = self.subscriptions.len();
        self.subscriptions.retain(|s| s.subscriber != *plugin_did);
        Ok(before - self.subscriptions.len())
    }

    fn active_subscription_count(&self, plugin_did: &Did) -> usize {
        self.subscriptions
            .iter()
            .filter(|s| s.subscriber == *plugin_did)
            .count()
    }
}

// =====================================================================
// install_plugin lifecycle — R4b-FP-1 Seam 1 + Seam 2 + Seam 4
// =====================================================================
//
// Symmetric companion to [`uninstall_plugin`] at the lifecycle layer.
// Wires the SIX install-time concerns named in
// `docs/PLUGIN-MANIFEST.md` §4.1 + the FOUR new R4b-FP-1 seams:
//
// - Seam 1: lifecycle integration (install_record consent + cap
//   cascade + library entry + plugin-DID minted-and-persisted)
// - Seam 2: engine-injected clock at install boundary
//   (`PluginManifest::validate_with_clock`)
// - Seam 4: cycle detection wired at the install entry-point (already
//   present at `module_ecosystem::install_plugin`; the lifecycle seam
//   layers consent + clock-injection + trust-list on top)
//
// Per CLAUDE.md baked-in #18 four-identity-concepts model: this is
// where the InstallRecord's user-DID signature is verified, the
// plugin-DID is minted, and the library entry is added. The cap
// cascade (Layer 1 trace from user-DID grants → plugin-DID audience)
// is consulted via the supplied `CapMinter` port — the foundation
// crate doesn't depend on `benten-caps` for production paths, so the
// engine adapter wires the real grant store; the [`InMemoryInstallCascade`]
// substantive default powers test pins.

/// Shape of the installing peer (mirrors
/// [`crate::module_ecosystem::InstallerShape`] — re-declared here to
/// keep the lifecycle seam dep-direction clean against
/// `module_ecosystem`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerShape {
    /// Full peer (native Rust; runs_sandbox=true; shape (a)).
    FullPeer,
    /// Thin compute surface (browser wasm32 / edge / Tauri webview;
    /// shapes (b)/(c)).
    ThinClient,
}

/// Port the engine adapter implements to mint root grants for caps
/// the manifest declares as `requires` (Layer 1 trace anchor per
/// CLAUDE.md #18).
///
/// Default test impl: [`InMemoryInstallCascade::mint_root_grant`] —
/// records each scope under the plugin-DID audience.
pub trait CapMinter {
    /// Mint a root grant from `user_did` to `plugin_did` for `scope`.
    ///
    /// Returns the grant CID on success.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` propagated from the adapter.
    fn mint_root_grant(
        &mut self,
        user_did: &Did,
        plugin_did: &Did,
        scope: &str,
    ) -> Result<Cid, ErrorCode>;
}

/// Port the storage-backend adapter implements to provision the
/// plugin's private namespace.
pub trait PrivateNamespaceProvisioner {
    /// Create the `private:<plugin_did>:*` scope-prefix root for the
    /// plugin. Idempotent.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` propagated from the adapter.
    fn provision_private_namespace(&mut self, plugin_did: &Did) -> Result<(), ErrorCode>;
}

/// Bundle of the engine-side ports — passed to [`install_plugin`].
pub struct InstallContext<'a, M, P>
where
    M: CapMinter,
    P: PrivateNamespaceProvisioner,
{
    /// Cap-minter port.
    pub cap_minter: &'a mut M,
    /// Private-namespace provisioner port.
    pub private_ns: &'a mut P,
    /// Engine-injected wall-clock (seconds since UNIX epoch). Pass
    /// [`MANIFEST_CLOCK_NOT_INJECTED_SENTINEL`] when the engine builder
    /// did NOT inject a clock — Seam 2 fail-closes for manifests with
    /// time-bounded requirements.
    pub now_secs: u64,
    /// Installer shape (heterogeneity check at step 5).
    pub installer_shape: InstallerShape,
    /// Per-user trust-list of plugin-author peer-DIDs. Empty list per
    /// D-4F-3 default = trust-list-empty. When non-empty AND the
    /// manifest's `peer_did` is absent, install fails with
    /// `E_PLUGIN_AUTHOR_NOT_TRUSTED`. (Empty list = legacy "trust by
    /// signature alone" — first-install consent prompt handled at
    /// caller layer.)
    pub user_trust_list: &'a [Did],
    /// User-DID that consents to the install (anchors Layer 1).
    pub user_did: &'a Did,
    /// Optional version-chain view for upgrade DAG-descendant check.
    /// When `Some` AND `prior_installed_cid` is `Some`, the seam
    /// enforces T10-upgrade (b) per D-4F-14: the new `expected_cid`
    /// MUST be a DAG-descendant of `prior_installed_cid`.
    pub version_chain: Option<&'a benten_core::version_chain::DagVersionChain>,
    /// Optional prior CID — when both this and `version_chain` are
    /// supplied, the seam runs the upgrade DAG-descendant check.
    pub prior_installed_cid: Option<Cid>,
}

/// Outcome of a successful [`install_plugin`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    /// The newly-inserted library entry.
    pub entry: LibraryEntry,
    /// Manifest validation outcome (carries any rotated-key warning).
    pub validation: ValidationOutcome,
    /// Count of root grants minted under the plugin-DID audience
    /// (one per [`PluginManifest::requires`] entry).
    pub grants_minted: usize,
    /// Whether the private namespace was provisioned.
    pub private_namespace_provisioned: bool,
}

/// **Phase-4-Foundation R4b-FP-1 Seam 1** — full install lifecycle.
///
/// Wires the SIX install concerns per `docs/PLUGIN-MANIFEST.md` §4.1
/// plus the four R4b-FP-1 hardening seams (Seam 2 clock injection,
/// Seam 4 cycle wiring, consent gate integration, trust-list check).
///
/// Order:
///
/// 1. Decode + verify content-CID matches declared.
/// 2. **Trust-list check** — if `user_trust_list` non-empty, reject with
///    `E_PLUGIN_AUTHOR_NOT_TRUSTED` when manifest's `peer_did` is not
///    in the list.
/// 3. **Seam 2** — `validate_with_clock(now_secs)`. Fail-closes with
///    `E_UCAN_CLOCK_NOT_INJECTED` when clock missing + manifest declares
///    time-bounded requirements.
/// 4. **Consent gate** — verify InstallRecord's user-DID signature
///    (`E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID`) AND match
///    `record.manifest_cid == expected_cid` (defense vs. consent-record-
///    substitution).
/// 5. **Heterogeneity** check (`E_PLUGIN_HETEROGENEITY_INCOMPATIBLE`).
/// 6. **Seam 4** — cycle detection.
/// 7. **Upgrade DAG-descendant check** (T10-upgrade (b)) when
///    `prior_installed_cid` + `version_chain` are supplied.
/// 8. **Mint plugin-DID** + **persist into store**.
/// 9. **Cap cascade** — mint a root grant under
///    `audience=plugin_did` for each `requires` scope.
/// 10. **Private namespace** provision.
/// 11. **Insert** into library + set active reference.
///
/// # Errors
///
/// See `docs/PLUGIN-MANIFEST.md` §4.1 for the full failure-mode list.
#[allow(clippy::too_many_arguments)]
pub fn install_plugin<F, M, P>(
    library: &mut PluginLibrary,
    plugin_did_store: &mut PluginDidStore,
    ctx: &mut InstallContext<'_, M, P>,
    received_bytes: &[u8],
    expected_cid: &Cid,
    install_record: &InstallRecord,
    installed_at_nanos: u64,
    resolver: &F,
) -> Result<InstallOutcome, ErrorCode>
where
    F: Fn(&Cid) -> Option<PluginManifest>,
    M: CapMinter,
    P: PrivateNamespaceProvisioner,
{
    // 1. Decode manifest + verify content-CID.
    let manifest: PluginManifest = serde_ipld_dagcbor::from_slice(received_bytes)
        .map_err(|_| ErrorCode::PluginManifestInvalid)?;
    if manifest.compute_content_cid() != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }
    if manifest.content_cid != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }

    // 2. Trust-list check (R4b-FP-1: T5b user-trust-list arm).
    if !ctx.user_trust_list.is_empty() && !ctx.user_trust_list.contains(&manifest.peer_did) {
        return Err(ErrorCode::PluginAuthorNotTrusted);
    }

    // 3. Seam 2 — clock-injected validation (delegates to validate +
    // verify_peer_signature internally).
    let validation = manifest.validate_with_clock(ctx.now_secs)?;

    // 4. Consent gate — install record signature + manifest-CID
    // binding. Defense vs. consent-record-substitution where attacker
    // re-uses Alice's consent record for Bob's manifest.
    install_record.verify_user_signature()?;
    if install_record.manifest_cid != *expected_cid {
        return Err(ErrorCode::PluginInstallConsentRequired);
    }
    if install_record.consenting_user_did != *ctx.user_did {
        return Err(ErrorCode::PluginInstallConsentRequired);
    }

    // 5. Heterogeneity check.
    if matches!(ctx.installer_shape, InstallerShape::ThinClient) && manifest.requires_sandbox_exec()
    {
        return Err(ErrorCode::PluginHeterogeneityIncompatible);
    }

    // 6. Seam 4 — composition cycle detection.
    detect_composition_cycle(*expected_cid, &manifest, resolver)?;

    // 7. Upgrade DAG-descendant check (T10-upgrade (b)).
    if let (Some(chain), Some(prior_cid)) = (ctx.version_chain, ctx.prior_installed_cid) {
        // Same CID = re-install (no-op upgrade); otherwise must be
        // a strict descendant.
        if prior_cid != *expected_cid && !chain.is_ancestor_of(&prior_cid, expected_cid) {
            return Err(ErrorCode::PluginManifestInvalid);
        }
    }

    // 8. Mint plugin-DID + persist into store.
    let plugin_did_handle = mint_plugin_did();
    let plugin_did = plugin_did_handle.did().clone();
    plugin_did_store.insert(plugin_did_handle);

    // 9. Cap cascade — mint root grants from user_did → plugin_did.
    let mut grants_minted = 0usize;
    for req in &manifest.requires {
        ctx.cap_minter
            .mint_root_grant(ctx.user_did, &plugin_did, &req.scope)?;
        grants_minted += 1;
    }

    // 10. Provision private namespace.
    ctx.private_ns.provision_private_namespace(&plugin_did)?;

    // 11. Insert into library + set active reference.
    let entry = LibraryEntry {
        manifest_cid: *expected_cid,
        manifest: manifest.clone(),
        plugin_did,
        installed_at_nanos,
    };
    library.insert(entry.clone());
    library.set_active(&manifest.plugin_name, *expected_cid)?;

    Ok(InstallOutcome {
        entry,
        validation,
        grants_minted,
        private_namespace_provisioned: true,
    })
}

/// Substantive in-memory default for the install-side cascade ports.
///
/// Mirrors the [`InMemoryUninstallCascade`] discipline — gives
/// substantive test pins a working backend without dragging
/// `benten-caps` into the foundation crate's production-path deps.
#[derive(Debug, Default)]
pub struct InMemoryInstallCascade {
    minted_grants: Vec<(Did, Did, String, Cid)>,
    provisioned_namespaces: HashSet<Did>,
    next_grant_byte: u8,
}

impl InMemoryInstallCascade {
    /// New empty cascade.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot all minted grants `(user_did, plugin_did, scope, grant_cid)`.
    #[must_use]
    pub fn minted_grants(&self) -> &[(Did, Did, String, Cid)] {
        &self.minted_grants
    }

    /// Whether the cascade has provisioned the private namespace for
    /// `plugin_did`.
    #[must_use]
    pub fn has_provisioned(&self, plugin_did: &Did) -> bool {
        self.provisioned_namespaces.contains(plugin_did)
    }

    /// Count of plugin-DIDs whose private namespace has been provisioned.
    /// Used by no-partial-state-commit pins (e.g. cycle-rejected install)
    /// where no plugin-DID is known at assertion time.
    #[must_use]
    pub fn provisioned_count(&self) -> usize {
        self.provisioned_namespaces.len()
    }
}

impl CapMinter for InMemoryInstallCascade {
    fn mint_root_grant(
        &mut self,
        user_did: &Did,
        plugin_did: &Did,
        scope: &str,
    ) -> Result<Cid, ErrorCode> {
        let cid = {
            let mut digest = [0u8; 32];
            digest[0] = self.next_grant_byte;
            self.next_grant_byte = self.next_grant_byte.wrapping_add(1);
            Cid::from_blake3_digest(digest)
        };
        self.minted_grants
            .push((user_did.clone(), plugin_did.clone(), scope.to_string(), cid));
        Ok(cid)
    }
}

impl PrivateNamespaceProvisioner for InMemoryInstallCascade {
    fn provision_private_namespace(&mut self, plugin_did: &Did) -> Result<(), ErrorCode> {
        self.provisioned_namespaces.insert(plugin_did.clone());
        Ok(())
    }
}

// =====================================================================
// Tests — internal sanity for the cascade defaults
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_library::LibraryEntry;
    use crate::plugin_manifest::{
        CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
    };
    use benten_id::did::Did;

    fn fake_did(suffix: &str) -> Did {
        Did::from_string_unchecked(format!("did:key:z{suffix}"))
    }

    fn fake_cid(b: u8) -> Cid {
        Cid::from_blake3_digest([b; 32])
    }

    fn fake_library_entry(plugin_did: Did, cid: Cid) -> LibraryEntry {
        let manifest = PluginManifest {
            plugin_name: "test".to_string(),
            content_cid: cid,
            peer_did: fake_did("AuthorX"),
            peer_signature: vec![0u8; 64],
            requires: vec![CapRequirement::new("store:notes:read")],
            shares: SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
            renderer_config: None,
            composes_plugins: None,
            accepts_content: None,
            requires_schema_authors: None,
            requires_plugin_authors: None,
        };
        LibraryEntry {
            manifest_cid: cid,
            manifest,
            plugin_did,
            installed_at_nanos: 1,
        }
    }

    #[test]
    fn uninstall_bogus_cid_returns_plugin_manifest_invalid_without_cascade() {
        let mut library = PluginLibrary::new();
        let mut store = PluginDidStore::new();
        let mut cascade = InMemoryUninstallCascade::new();
        let mut private = InMemoryUninstallCascade::new();
        let mut subs = InMemoryUninstallCascade::new();
        let mut ctx = UninstallContext {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let err = uninstall_plugin(&mut library, &mut store, &mut ctx, &fake_cid(99)).unwrap_err();
        assert_eq!(err, ErrorCode::PluginManifestInvalid);
        // No revocation log entries — cascade not driven on bogus CID.
        assert!(cascade.revocation_log().is_empty());
    }

    #[test]
    fn uninstall_held_caps_revokes_via_audience_index() {
        let plugin_did = fake_did("Plugin");
        let user_did = fake_did("User");
        let mut library = PluginLibrary::new();
        let mut store = PluginDidStore::new();
        let cid = fake_cid(7);
        library.insert(fake_library_entry(plugin_did.clone(), cid));

        let mut cascade = InMemoryUninstallCascade::new();
        cascade.insert_grant(InMemoryGrant {
            grant_cid: fake_cid(11),
            audience: plugin_did.clone(),
            issuer: user_did.clone(),
            scope: "store:notes:read".to_string(),
        });
        cascade.insert_grant(InMemoryGrant {
            grant_cid: fake_cid(12),
            audience: plugin_did.clone(),
            issuer: user_did.clone(),
            scope: "store:notes:write".to_string(),
        });
        // Distractor — different audience must NOT be revoked.
        cascade.insert_grant(InMemoryGrant {
            grant_cid: fake_cid(13),
            audience: fake_did("OtherPlugin"),
            issuer: user_did.clone(),
            scope: "store:notes:read".to_string(),
        });

        // 3 separate cascade instances would be inconvenient — use one
        // for all 3 ports.
        let mut private = InMemoryUninstallCascade::new();
        let mut subs = InMemoryUninstallCascade::new();
        let mut ctx = UninstallContext {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).unwrap();
        assert_eq!(outcome.held_caps_revoked, 2);
        // Distractor untouched.
        assert!(
            !cascade
                .revocation_log()
                .iter()
                .any(|r| r.audience == fake_did("OtherPlugin"))
        );
    }

    #[test]
    fn uninstall_cascade_revokes_issuer_with_cascade_source_tag() {
        let plugin_a = fake_did("PluginA");
        let plugin_b = fake_did("PluginB");
        let mut library = PluginLibrary::new();
        let mut store = PluginDidStore::new();
        let cid_a = fake_cid(1);
        library.insert(fake_library_entry(plugin_a.clone(), cid_a));

        let mut cascade = InMemoryUninstallCascade::new();
        // A delegates to B.
        cascade.insert_grant(InMemoryGrant {
            grant_cid: fake_cid(21),
            audience: plugin_b.clone(),
            issuer: plugin_a.clone(),
            scope: "store:notes:read".to_string(),
        });
        let mut private = InMemoryUninstallCascade::new();
        let mut subs = InMemoryUninstallCascade::new();
        let mut ctx = UninstallContext {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid_a).unwrap();
        assert_eq!(outcome.delegations_cascade_revoked, 1);
        let log = cascade.revocation_log();
        assert!(
            log.iter()
                .any(|r| r.cascade_source == Some(plugin_a.clone()) && r.audience == plugin_b)
        );
    }
}
