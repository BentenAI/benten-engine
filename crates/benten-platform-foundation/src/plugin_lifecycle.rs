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

use crate::module_ecosystem::{
    UpgradeConsentDecision, decide_upgrade_consent, verify_upgrade_author_continuity,
};
use crate::plugin_library::{LibraryEntry, PluginLibrary};
use crate::plugin_manifest::{
    InstallRecord, MANIFEST_CLOCK_NOT_INJECTED_SENTINEL, PluginManifest, ValidationOutcome,
    detect_composition_cycle,
};
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;
use std::collections::{HashMap, HashSet};

/// Fail-closed ceiling on the raw bytes accepted by the manifest-decode
/// inside [`install_plugin`] (Compromise #28 / META #629 DoS-sweep).
///
/// `received_bytes` is an attacker-authored content-addressed shared plugin
/// manifest — same threat class as `benten_engine::module_manifest::MAX_MODULE_MANIFEST_BYTES`
/// (256 KiB), which the sweep capped. A manifest whose body (name + requires
/// list + shares policy + optional renderer/composition refs) exceeds 256 KiB
/// is adversarial; the cap bounds a hostile blob BEFORE `serde` allocates.
pub const MAX_PLUGIN_MANIFEST_BYTES: usize = 256 * 1024;

/// Fail-closed ceiling on the decoded [`PluginManifest::requires`] count
/// (Compromise #28 / META #629). A count-prefixed array amplification guard
/// sitting alongside the byte cap — mirrors
/// `benten_engine::module_manifest::MAX_MODULE_MANIFEST_MODULES` (4096).
/// A legitimate plugin declares a handful of capability requirements; 4096
/// clears any realistic manifest while bounding an amplification vector.
pub const MAX_PLUGIN_MANIFEST_REQUIRES: usize = 4096;

/// Fail-closed ceiling on the decoded
/// [`InstallRecord::granted_caps_bytes`](crate::plugin_manifest::InstallRecord::granted_caps_bytes)
/// element count (Compromise #28 / META #629). `granted_caps_bytes` is an
/// attacker-appendable `Vec<Vec<u8>>` decoded per-element in
/// `install_record_covers_required_caps`; a hostile record can append
/// unbounded tiny CBOR blobs to blow up `HashSet::with_capacity` + the
/// per-element decode loop. 4096 clears any realistic consent record (one
/// entry per granted cap) while bounding the amplification vector.
pub const MAX_GRANTED_CAPS: usize = 4096;

/// Fail-closed per-element byte ceiling on each
/// [`InstallRecord::granted_caps_bytes`](crate::plugin_manifest::InstallRecord::granted_caps_bytes)
/// entry (Compromise #28 / META #629). Each element is the DAG-CBOR encoding
/// of one [`crate::plugin_manifest::CapRequirement`] (a scope string + small
/// fields); 64 KiB is generous headroom while bounding a single hostile
/// element before `serde` decodes it.
pub const MAX_GRANTED_CAP_BYTES: usize = 64 * 1024;

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
///
/// **Symmetric with [`InstallPorts`]** (Surf-1 #888): both lifecycle
/// entry-points take a pure-port bundle (`*Ports`) for swappable
/// engine-side adapters, keeping non-port scalar data in a separate
/// `*Params` bundle. This is the composability seam a future
/// install-then-rollback-on-failure lifecycle (#635) composes against.
pub struct UninstallPorts<'a, R, P, S>
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
    ports: &mut UninstallPorts<'_, R, P, S>,
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
    let held_caps_revoked = ports.cap_revoker.revoke_grants_with_audience(&plugin_did)?;

    // 2. Cascade-revoke downstream delegations (T10-uninstall (b)).
    let delegations_cascade_revoked = ports
        .cap_revoker
        .cascade_revoke_grants_with_issuer(&plugin_did)?;

    // 3. Terminate live subscriptions (T10-uninstall (c) LOAD-BEARING).
    let subscriptions_terminated = ports
        .subscriptions
        .terminate_subscriptions_for(&plugin_did)?;

    // 4. Tear down private namespace.
    let private_namespace_rows_deleted =
        ports.private_ns.delete_private_namespace_for(&plugin_did)?;

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
// - Seam 4: cycle detection wired at the install entry-point (the
//   lifecycle seam layers consent + clock-injection + trust-list on
//   top of the structural cycle detection over `composes_plugins`)
//
// Per CLAUDE.md baked-in #18 four-identity-concepts model: this is
// where the InstallRecord's user-DID signature is verified, the
// plugin-DID is minted, and the library entry is added. The cap
// cascade (Layer 1 trace from user-DID grants → plugin-DID audience)
// is consulted via the supplied `CapMinter` port — the foundation
// crate doesn't depend on `benten-caps` for production paths, so the
// engine adapter wires the real grant store; the [`InMemoryInstallCascade`]
// substantive default powers test pins.
//
// Phase-4-Meta-Core G-CORE-0 (plan §1.A.FROZEN item 7) deleted the
// legacy `module_ecosystem::install_plugin` precursor + its sibling
// `install_plugin_persisting_did`; this is now the canonical
// (only-public) install path per CLAUDE.md #18.

/// Shape of the installing peer.
///
/// At Phase-4-Meta-Core G-CORE-0 (plan §1.A.FROZEN item 7) this is the
/// canonical `InstallerShape`; the duplicate `module_ecosystem::
/// InstallerShape` that sat alongside the deleted legacy install path
/// was removed in the same wave (HARD-RULE-12 clause-(a)).
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

    /// **G-CORE-7 §4.21 + §4.35 rollback hook.**
    ///
    /// Revoke a previously-minted root grant by its `grant_cid`. The
    /// install lifecycle calls this for every grant minted in Step 9
    /// when ANY subsequent step (or a mid-Step-9 cascade entry) fails,
    /// to leave **zero residual minted grants**.
    ///
    /// Production engine adapters route this to the cap-store's
    /// per-grant-CID revocation surface (couples to Phase-3 PR #199
    /// `Engine::revoke_capability_by_grant_cid`). The default impl is a
    /// no-op so existing adapters compile against an additive
    /// signature — the install lifecycle's rollback discipline is the
    /// load-bearing surface, NOT every `CapMinter` impl.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` propagated from the adapter. The install
    /// lifecycle's rollback loop logs but does NOT propagate errors
    /// from individual revocations — partial-rollback is observably
    /// captured in the cap-store's revocation log; the install itself
    /// has already failed and surfaces its primary error.
    fn revoke_root_grant(&mut self, _grant_cid: &Cid) -> Result<(), ErrorCode> {
        Ok(())
    }
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

    /// **G-CORE-7 §4.21 rollback hook.**
    ///
    /// Tear down the `private:<plugin_did>:*` namespace previously
    /// provisioned for this plugin. The install lifecycle calls this
    /// when Step 11 (library insert) fails AFTER Step 10 (namespace
    /// provision) succeeded — leaving the partially-provisioned
    /// namespace in place would be observable residue.
    ///
    /// Default impl is a no-op so existing adapters compile against an
    /// additive signature.
    ///
    /// # Errors
    ///
    /// `E_INTERNAL` propagated from the adapter. The install
    /// lifecycle's rollback loop does NOT propagate errors from this
    /// hook (same discipline as `CapMinter::revoke_root_grant`).
    fn deprovision_private_namespace(&mut self, _plugin_did: &Did) -> Result<(), ErrorCode> {
        Ok(())
    }
}

/// Bundle of the engine-side ports — passed to [`install_plugin`].
///
/// **Symmetric with [`UninstallPorts`]** (Surf-1 #888): a pure-port
/// bundle for swappable engine-side adapters. Non-port scalar data
/// (clock, trust-list, version-chain, user/plugin DIDs, …) lives in the
/// separate [`InstallParams`] bundle so the two lifecycle entry-points
/// share the same composable shape.
///
/// **No `SubscriptionRegistry` port here (scoped DISAGREE w/ #888
/// sub-point):** the v1 install path has no subscription-registration
/// step — a plugin's reactive SUBSCRIBE handles are attached lazily by
/// the engine when the plugin subgraph is first walked, not at install
/// time (uninstall *terminates* them, which is why uninstall carries
/// the port). Adding a never-driven port at install would be dead
/// surface (a Hyg-2 finding). The real asymmetry #888 names — the
/// missing common abstraction — is resolved by the `*Ports` / `*Params`
/// split, not by a dead port.
pub struct InstallPorts<'a, M, P>
where
    M: CapMinter,
    P: PrivateNamespaceProvisioner,
{
    /// Cap-minter port.
    pub cap_minter: &'a mut M,
    /// Private-namespace provisioner port.
    pub private_ns: &'a mut P,
    /// **Phase-4-Meta-Core G-CORE-8 §4.37** — install-record replay
    /// check port. **R6 R1 FP-F4 §S2 (Row D-2 closure):** the prior
    /// `Option<>` wrapper is dropped (the `None` arm silently
    /// disabled the §4.37 TOCTOU defense in shipped binaries). Every
    /// caller MUST supply a substantive closure.
    ///
    /// **R6 R2 FP-B (L7-R2-F-4 phantom-cite delete):** the production
    /// wiring pattern is a closure over the engine's replay store —
    /// e.g.
    /// ```ignore
    /// let store = engine.install_record_replay_store().clone();
    /// let mut replay_check = move |hash: &[u8; 32]| -> Result<(), ErrorCode> {
    ///     store.record_and_check(*hash)
    /// };
    /// ```
    /// (the previously-cited helper
    /// `make_engine_replay_check_closure(...)` was a phantom — no
    /// such function exists; the closure inline IS the canonical
    /// pattern).
    ///
    /// Test fixtures wire `crate::testing::noop_replay_check()`
    /// (admit-all, intentional non-defense; documents the test that
    /// is NOT exercising the replay-defense surface).
    ///
    /// Called BEFORE the cap-cascade with the canonical signing-
    /// payload hash of the install record. If the closure returns
    /// `Err(_)`, install rejects pre-mint (zero duplicate-mint
    /// window per §4.37 TOCTOU contract). Production engines wire
    /// this to a closure over the engine's
    /// `Engine::install_record_replay_store().record_and_check`.
    pub install_record_replay_check: &'a mut InstallRecordReplayCheckFn,
    /// **R6 R1 FP-F4 §S3a (Row D-3-a closure)** — CLAUDE.md baked-in #18
    /// §8-E hook #1 install-time consent policy port.
    ///
    /// Called at install-pipeline step 3c (BEFORE the cap-cascade
    /// runs). The configured `CapabilityPolicy::check_install_consent`
    /// hook is invoked with the install record's canonical signing-
    /// payload hash + the plugin-DID string. If the hook returns
    /// `Err(_)`, the install rejects with typed
    /// `ErrorCode::PluginInstallConsentDenied` (forensic-discrimination
    /// vs `PluginInstallConsentRequired` which is the caps-grew
    /// fresh-consent gap at upgrade time).
    ///
    /// Threaded via this port (NOT via an `Engine::capability_policy()`
    /// accessor) per CRITIC-2 F-1.2 + Class B β + §8-E sealed-discipline:
    /// the policy is engine-internal; install pipelines that need it
    /// receive it as an explicit input port.
    ///
    /// Production callers wire the engine glue that delegates to
    /// `CapabilityPolicy::check_install_consent` (the engine-side
    /// blanket adapter; lives at engine-side because `benten-caps`
    /// already depends on this crate, so the trait + adapter can't
    /// both live here); tests can use
    /// [`crate::install_consent::AdmitAllInstallConsent`] which
    /// admits every install.
    pub policy: &'a dyn crate::install_consent::InstallConsentPolicy,
}

/// Closure type for the [`InstallPorts::install_record_replay_check`]
/// port. Takes the canonical 32-byte BLAKE3 hash of the install
/// record's `signing_payload`; returns `Ok(())` to admit OR
/// `Err(ErrorCode::PluginInstallRecordAlreadyApplied)` on replay.
/// The 32-byte hash shape matches
/// `benten_engine::install_record_replay::signing_payload_hash`.
pub type InstallRecordReplayCheckFn = dyn FnMut(&[u8; 32]) -> Result<(), ErrorCode>;

/// Non-port install parameters — the scalar / borrowed inputs the
/// install cascade consumes (symmetric counterpart kept distinct from
/// [`InstallPorts`] per Surf-1 #888 so a future install-then-rollback
/// lifecycle (#635) can compose ports + params independently).
pub struct InstallParams<'a> {
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
    /// **R6-FP-A-fp (mr-1 BLOCKER closure)** — the caller's assertion
    /// of which plugin-DID the user signed the InstallRecord for.
    /// Step 8 enforces `install_record.plugin_did == *expected_plugin_did`
    /// (surfacing `E_PLUGIN_INSTALL_RECORD_PLUGIN_DID_MISMATCH` on
    /// mismatch) AND `plugin_did_store.get(expected_plugin_did).is_some()`
    /// (surfacing `E_PLUGIN_DID_HANDLE_NOT_PRE_INSERTED` if the caller
    /// did not first mint + insert the keypair handle).
    ///
    /// **Caller-mint-first flow contract (CLAUDE.md #18):**
    /// 1. Caller mints `PluginDidHandle` via
    ///    [`benten_id::plugin_did::mint`].
    /// 2. Caller calls `plugin_did_store.insert(handle)`.
    /// 3. Caller builds `InstallRecord { plugin_did: handle.did().clone(), .. }`
    ///    + signs with user-DID.
    /// 4. Caller passes both the store + this `expected_plugin_did =
    ///    handle.did()` into install_plugin.
    ///
    /// The engine cannot synthesize keypairs matching an arbitrary
    /// `did:key:...` string (Ed25519 derives the DID from the public
    /// key, not vice versa), so the caller-mint-first pattern is a
    /// structural requirement rather than ergonomic preference.
    pub expected_plugin_did: &'a Did,
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
/// Order (R6 R2 FP-B updated — docstring drift closure for L7-R2-F-1):
///
/// 1. Decode + verify content-CID matches declared.
/// 2. **Trust-list check** — if `user_trust_list` non-empty, reject with
///    `E_PLUGIN_AUTHOR_NOT_TRUSTED` when manifest's `peer_did` is not
///    in the list.
/// 3. **InstallRecord consent verify** — verify the user-DID signature
///    over the install-record + match `record.manifest_cid ==
///    expected_cid` (defense vs. consent-record substitution). Errors
///    `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` /
///    `E_PLUGIN_INSTALL_RECORD_MANIFEST_CID_MISMATCH`.
/// 3b. **§4.37 InstallRecord replay-defense** —
///    `install_record_replay_check(payload_hash)` consulted BEFORE any
///    mint occurs. Replay → `PluginInstallRecordAlreadyApplied`.
/// 3c. **§S3a (R6 R1 FP-F4) install-time consent** —
///    `policy.check_install_consent(payload_hash, plugin_did_str)`
///    consulted BEFORE the cap-cascade. Default impl admits all;
///    custom `CapabilityPolicy` impls (e.g. wrapping a curated DID
///    trust-list) can reject with typed
///    `PluginInstallConsentDenied`. The engine-side adapter
///    `benten_engine::capability_policy_install_consent::CapabilityPolicyInstallConsent`
///    (minted at R6 R2 FP-B; plain-backtick cite to avoid rustdoc
///    intra-doc-link resolution against benten-engine which isn't a
///    dep of benten-platform-foundation) bridges the engine's configured
///    `CapabilityPolicy` to the install-pipeline's
///    [`crate::install_consent::InstallConsentPolicy`] port.
/// 4. **Seam 2 — clock-injected validation** —
///    `validate_with_clock(now_secs)`. Fail-closes with
///    `E_UCAN_CLOCK_NOT_INJECTED` when clock missing + manifest declares
///    time-bounded requirements.
/// 5. **Heterogeneity** check (`E_PLUGIN_HETEROGENEITY_INCOMPATIBLE`).
/// 6. **Seam 4** — cycle detection (`detect_composition_cycle`).
/// 7. **Upgrade DAG-descendant check** (T10-upgrade (b)) when
///    `prior_installed_cid` + `version_chain` are supplied.
/// 7a. **T10-upgrade (a) — same-author DID continuity (R6 R1 Bundle
///    L2-R6-MAJOR-1 closure).** When `prior_installed_cid` resolves to
///    a manifest, `verify_upgrade_author_continuity` rejects an
///    upgrade whose new `peer_did` differs from the prior
///    (`PluginAuthorNotTrusted`). T10-(a) + T10-(b) are co-defensive
///    per `docs/admin-ui-v0-threat-model.md` §T10; both gates fire on
///    every upgrade attempt.
/// 7b. **Fresh-consent gap at upgrade time** — if the new manifest
///    `requires` a capability the prior did NOT, reject with
///    `PluginInstallConsentRequired` (separate from §S3a
///    `PluginInstallConsentDenied` — forensic-discriminate the caps-
///    grew gap vs the policy-said-no gap).
/// 8. **Plugin-DID adoption (caller-mint-first)** — assert
///    `install_record.plugin_did == *params.expected_plugin_did`
///    (`E_PLUGIN_INSTALL_RECORD_PLUGIN_DID_MISMATCH` on mismatch); then
///    assert `plugin_did_store.get(expected_plugin_did).is_some()`
///    (`E_PLUGIN_DID_HANDLE_NOT_PRE_INSERTED` if the caller did not
///    pre-mint + insert). Eliminates the legacy fresh-mint-and-discard
///    code path which silently ignored the user's signed plugin_did.
/// 9. **Cap cascade** — mint a root grant under
///    `audience=plugin_did` for each `requires` scope.
/// 10. **Private namespace** provision.
/// 11. **Insert** into library + set active reference.
///
/// # Errors
///
/// See `docs/PLUGIN-MANIFEST.md` §4.1 for the full failure-mode list.
#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "install_plugin is the single linear v1-beta install pipeline; \
              R6-R1-FP-F4 §S3a (check_install_consent) + §S3c (T10-upgrade) \
              wires brought it to 103/100. Helper-extraction refactor named at \
              V1-FROZEN-INTERFACE-DEFERRED.md Row D-22 follow-up cluster (post \
              phase-4-meta-core-close); inlining keeps the named-pipeline \
              ordering audit-able as a single page until then."
)]
pub fn install_plugin<F, M, P>(
    library: &mut PluginLibrary,
    plugin_did_store: &mut PluginDidStore,
    ports: &mut InstallPorts<'_, M, P>,
    params: &InstallParams<'_>,
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
    //
    //    Fail-closed byte cap (Compromise #28 / META #629 DoS-sweep):
    //    reject an over-large attacker-authored manifest blob BEFORE
    //    `serde` allocates. Mirrors the ModuleManifest cap
    //    (`benten_engine::module_manifest::from_canonical_bytes`).
    if received_bytes.len() > MAX_PLUGIN_MANIFEST_BYTES {
        return Err(ErrorCode::PluginManifestInvalid);
    }
    let manifest: PluginManifest = serde_ipld_dagcbor::from_slice(received_bytes)
        .map_err(|_| ErrorCode::PluginManifestInvalid)?;
    if manifest.compute_content_cid() != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }
    if manifest.content_cid != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }

    // 2. Trust-list check (R4b-FP-1: T5b user-trust-list arm).
    if !params.user_trust_list.is_empty() && !params.user_trust_list.contains(&manifest.peer_did) {
        return Err(ErrorCode::PluginAuthorNotTrusted);
    }

    // 3. Consent gate FIRST (sec-r6r1-6 R6-FP-A reorder):
    //
    //    - cheap-check before expensive ed25519 peer-signature verify;
    //      eliminates DoS-amplification surface where attacker hits the
    //      expensive verify ahead of consent.
    //    - structural trust anchor (CLAUDE.md #18 Layer 2) should be
    //      verified BEFORE we touch the manifest's cryptographic
    //      provenance check; consent is the gate that says "the user
    //      asked for this to happen".
    //
    //    arch-r6-r1-5 R6-FP-A split: three substitution-attack arms
    //    now surface distinct typed codes (forensic discrimination for
    //    defenders). `PluginInstallConsentRequired` narrows to mean
    //    "no record / null consent"; `*RecordUserSignatureInvalid` =
    //    cryptographic forge; the three `*RecordMismatch` codes
    //    discriminate the three substitution-attack arms.
    install_record.verify_user_signature()?;
    if install_record.manifest_cid != *expected_cid {
        return Err(ErrorCode::PluginInstallRecordManifestCidMismatch);
    }
    if install_record.consenting_user_did != *params.user_did {
        return Err(ErrorCode::PluginInstallRecordConsentingUserMismatch);
    }

    // **3b. G-CORE-8 §4.37 InstallRecord replay-defense + atomic
    //      record-and-check.** Fires BEFORE Step 4 (clock validation)
    //      AND BEFORE Step 9 (cap-cascade). A second presentation of
    //      the same install-record canonical bytes is rejected with
    //      typed `PluginInstallRecordAlreadyApplied` here — zero
    //      duplicate-mint window per the §4.37 TOCTOU contract.
    //      Backward-compat: when the caller did NOT wire the port
    //      (the `None` arm), this step no-ops (Phase-3-baseline
    //      behavior preserved).
    //
    //      The canonical identity is the BLAKE3 hash of the
    //      `signing_payload()` bytes (matches the canonical
    //      content identity per the manifest_envelope_recheck.rs
    //      `signing_payload_hash` helper in benten-engine; the
    //      two-step composition stays free of platform-foundation
    //      deps on benten-engine — the caller computes the hash).
    let payload_hash: [u8; 32];
    {
        let payload = install_record.signing_payload();
        payload_hash = *blake3::hash(&payload).as_bytes();
        (ports.install_record_replay_check)(&payload_hash)?;
    }

    // **3c. R6 R1 FP-F4 §S3a (Row D-3-a closure)** — CLAUDE.md baked-in
    //      #18 §8-E hook #1 install-time consent. Consult the configured
    //      `CapabilityPolicy::check_install_consent` BEFORE the cap-
    //      cascade runs (mirrors the §4.37 replay-check ordering — both
    //      pre-mint gates fire before any Step-9 grant lands, preserving
    //      §4.35 zero partial-mint atomicity).
    //
    //      Mapped to typed `PluginInstallConsentDenied` for forensic
    //      discrimination from `PluginInstallConsentRequired` (which is
    //      caps-grew upgrade-time at Step 7b above).
    //
    //      Default policy impl returns `Ok(())` (admit-all-installs); a
    //      custom CapabilityPolicy that wants to apply install-time
    //      policy (trust-list, curated-DIDs, etc.) overrides.
    let plugin_did_str = install_record.plugin_did.as_str();
    if ports
        .policy
        .check_install_consent(&payload_hash, plugin_did_str)
        .is_err()
    {
        return Err(ErrorCode::PluginInstallConsentDenied);
    }

    // 4. Seam 2 — clock-injected validation (delegates to validate +
    //    verify_peer_signature internally). Runs AFTER the consent
    //    gate so the expensive ed25519 peer-signature work doesn't
    //    fire on unconsented installs.
    // v1: validate_with_clock only; rotation-log-aware variant
    // (validate_with_rotation_log_and_clock) deferred to Phase-4-Meta per
    // docs/future/phase-4-backlog.md §4.10 — at v1 a rotated peer-DID
    // still passes install (D-4F-12: rotation → WARNING not hard-reject)
    // but the WARNING-emitting path isn't called yet.
    let validation = manifest.validate_with_clock(params.now_secs)?;

    // 5. Heterogeneity check.
    if matches!(params.installer_shape, InstallerShape::ThinClient)
        && manifest.requires_sandbox_exec()
    {
        return Err(ErrorCode::PluginHeterogeneityIncompatible);
    }

    // 6. Seam 4 — composition cycle detection.
    detect_composition_cycle(*expected_cid, &manifest, resolver)?;

    // 7. Upgrade DAG-descendant check (T10-upgrade (b)).
    if let (Some(chain), Some(prior_cid)) = (params.version_chain, params.prior_installed_cid) {
        // Same CID = re-install (no-op upgrade); otherwise must be
        // a strict descendant.
        if prior_cid != *expected_cid && !chain.is_ancestor_of(&prior_cid, expected_cid) {
            return Err(ErrorCode::PluginManifestInvalid);
        }
    }

    // 7a. **T10-upgrade (a) — same-author DID continuity (R6 R1 Bundle
    //     L2-R6-MAJOR-1 closure).**
    //
    //     Per `docs/PLUGIN-MANIFEST.md` §4.3 + admin-ui-v0-threat-model.md
    //     §T10: an upgrade from peer-DID `alice` to peer-DID `attacker`
    //     within the same version-DAG (T10-upgrade (a) — transitive
    //     substitution) MUST be REJECTED. The standalone helper
    //     `verify_upgrade_author_continuity` (LANDED earlier; test pin
    //     at `plugin_upgrade_requires_same_author_did.rs`) was unwired
    //     at HEAD; step 7 above only enforced the DAG-descendant half
    //     (T10-upgrade (b)). META #707 asymmetric-at-parallel-entry-
    //     points: T10-(a) + T10-(b) are co-defensive per the threat
    //     model; this fix wires (a).
    //
    //     The check fires when prior_manifest is resolvable. Initial
    //     installs (no prior_cid) skip — first-install peer-DID review
    //     happens at the user-trust-list / install-record-consent
    //     surfaces above. The check returns
    //     `ErrorCode::PluginAuthorNotTrusted` (the typed code the
    //     helper already emits — semantically identical to
    //     "upgrade-author-broken": the new peer_did is NOT in the
    //     trust chain established at prior_cid; no new ErrorCode
    //     minted per orchestrator DISAGREE-WITH-EXPLANATION on the
    //     L2-R6-MAJOR-1 brief's mint suggestion — avoids cross-language
    //     mirror churn for zero security gain).
    if let Some(prior_cid) = params.prior_installed_cid
        && prior_cid != *expected_cid
        && let Some(prior_manifest) = resolver(&prior_cid)
    {
        verify_upgrade_author_continuity(&prior_manifest, &manifest)?;
    }

    // 7b. **G-CORE-7 §4.41 — caps-grew fresh-consent (e2e wiring).**
    //
    //    When the install is a within-lineage upgrade (`prior_installed_cid`
    //    Some) AND the resolver returns the prior manifest, consult the
    //    LANDED-GREEN pure decision fn `decide_upgrade_consent`:
    //    if `ConsentRequired` (caps grew) the install record MUST carry an
    //    explicit fresh-consent token covering the widened cap envelope.
    //
    //    The fresh-consent token shape: every entry in `install_record.
    //    granted_caps_bytes` is the DAG-CBOR encoding of one
    //    `CapRequirement`; the union of those scopes MUST be a superset
    //    of the new manifest's `requires`. An empty `granted_caps_bytes`
    //    (the v0 install path produces this for INITIAL installs where
    //    the manifest_cid binding suffices) is treated as "no explicit
    //    upgrade consent" and rejects the caps-grew upgrade with the
    //    typed `E_PLUGIN_INSTALL_CONSENT_REQUIRED` code.
    //
    //    Initial installs (`prior_installed_cid = None`) skip this check
    //    entirely — the manifest_cid binding in the user-signed
    //    InstallRecord is itself the consent for the initial cap set.
    //
    //    Couples LANDED `module_ecosystem::decide_upgrade_consent` (pure
    //    fn); the gap closed here is the production-path e2e wiring.
    if let Some(prior_cid) = params.prior_installed_cid
        && prior_cid != *expected_cid
        && let Some(prior_manifest) = resolver(&prior_cid)
        && decide_upgrade_consent(&prior_manifest, &manifest)
            == UpgradeConsentDecision::ConsentRequired
        && !install_record_covers_required_caps(install_record, &manifest)
    {
        return Err(ErrorCode::PluginInstallConsentRequired);
    }

    // 8. Plugin-DID adoption (caller-mint-first contract per R6-FP-A
    //    + R6-FP-A-fp mr-1 + mr-2 BLOCKER closures).
    //
    //    The legacy pre-R6-FP-A path minted a fresh OsRng plugin-DID
    //    here + discarded `install_record.plugin_did` even though that
    //    field IS in the user's signed `InstallRecord::signing_payload`
    //    via `plugin_did_bytes`. That defeated the consent-payload
    //    integrity guarantee.
    //
    //    The post-fp path enforces:
    //
    //    (a) `install_record.plugin_did == *params.expected_plugin_did`
    //        — the caller's claim about which plugin-DID the user
    //        signed for. Mismatch surfaces typed
    //        `PluginInstallRecordPluginDidMismatch` (sec-r6r1-1
    //        forensic-discrimination ErrorCode).
    //
    //    (b) `plugin_did_store.get(expected_plugin_did).is_some()` —
    //        the caller-mint-first contract requires the keypair
    //        handle to be inserted BEFORE install_plugin runs.
    //        Eliminates the keypair-orphan failure mode where install
    //        succeeded but no handle ever entered the store
    //        (downstream sign-as-plugin / revoke-on-uninstall would
    //        observably no-op).
    //
    //    Defense narrative: the engine CANNOT synthesize a keypair
    //    matching an arbitrary `did:key:...` string (Ed25519 derives
    //    the DID from the public key, not vice versa). The
    //    caller-mint-first pattern is therefore a structural
    //    requirement, not ergonomic preference. The substitution
    //    attack against the plugin-DID slot is bounded at the
    //    Step 3 user-signature check (the signing payload binds
    //    `plugin_did_bytes`; any tamper invalidates the signature);
    //    the new Step 8 checks add defense-in-depth via the caller's
    //    explicit `expected_plugin_did` claim + store-membership
    //    assertion.
    if install_record.plugin_did != *params.expected_plugin_did {
        return Err(ErrorCode::PluginInstallRecordPluginDidMismatch);
    }
    if plugin_did_store.get(params.expected_plugin_did).is_none() {
        return Err(ErrorCode::PluginDidHandleNotPreInserted);
    }
    let plugin_did = install_record.plugin_did.clone();

    // 9. **G-CORE-7 §4.35 — Step-9 cap-cascade atomicity** (all-or-
    //    nothing mint loop): track every minted grant CID; on any
    //    mid-loop mint failure, unwind already-minted grants before
    //    propagating the error so the cascade is observably atomic
    //    (zero partial-grant residue).
    let mut minted_grant_cids: Vec<Cid> = Vec::with_capacity(manifest.requires.len());
    for req in &manifest.requires {
        match ports
            .cap_minter
            .mint_root_grant(params.user_did, &plugin_did, &req.scope)
        {
            Ok(cid) => minted_grant_cids.push(cid),
            Err(e) => {
                // §4.35 unwind: revoke every grant minted earlier in
                // this same mint-loop. Revocation errors are observed
                // in the cap-store's revocation log; the primary
                // failure is the mint error which we propagate.
                rollback_step9(ports.cap_minter, &minted_grant_cids);
                return Err(e);
            }
        }
    }
    let grants_minted = minted_grant_cids.len();

    // 10. Provision private namespace. **G-CORE-7 §4.21** — if this
    //     fails, the Step-9 grants are stranded under the current HEAD
    //     impl; the fix is to unwind them before propagating the error.
    if let Err(e) = ports.private_ns.provision_private_namespace(&plugin_did) {
        rollback_step9(ports.cap_minter, &minted_grant_cids);
        return Err(e);
    }

    // 11. Insert into library + set active reference. **G-CORE-7 §4.21**
    //     — `set_active` can fail (e.g. unknown-plugin-name race); on
    //     failure unwind BOTH Step-9 grants AND Step-10 namespace
    //     provision, AND remove the just-inserted library entry, so the
    //     install_plugin call leaves no observable residue.
    let entry = LibraryEntry {
        manifest_cid: *expected_cid,
        manifest: manifest.clone(),
        plugin_did: plugin_did.clone(),
        installed_at_nanos,
    };
    library.insert(entry.clone());
    if let Err(e) = library.set_active(&manifest.plugin_name, *expected_cid) {
        // Step-11b rollback: remove the just-inserted entry, deprovision
        // the namespace, unwind the Step-9 grant cascade.
        library.remove(expected_cid);
        let _ = ports.private_ns.deprovision_private_namespace(&plugin_did);
        rollback_step9(ports.cap_minter, &minted_grant_cids);
        return Err(e);
    }

    Ok(InstallOutcome {
        entry,
        validation,
        grants_minted,
        private_namespace_provisioned: true,
    })
}

/// **G-CORE-7 §4.21 + §4.35 helper** — unwind every Step-9 minted
/// grant. Revocation errors are deliberately swallowed (the install
/// has already failed; the cap-store's own revocation log captures any
/// partial-rollback failures); see the `revoke_root_grant` doc comment
/// for the discipline narrative.
fn rollback_step9<M: CapMinter + ?Sized>(minter: &mut M, minted: &[Cid]) {
    for cid in minted {
        let _ = minter.revoke_root_grant(cid);
    }
}

/// **G-CORE-7 §4.41 helper** — whether the install record's
/// `granted_caps_bytes` set covers every scope in the new manifest's
/// `requires`. Each entry of `granted_caps_bytes` is the DAG-CBOR
/// encoding of one [`crate::plugin_manifest::CapRequirement`]; an entry
/// that fails to decode is treated as un-coverage (fail-closed —
/// malformed consent tokens MUST NOT smuggle un-consented caps).
fn install_record_covers_required_caps(record: &InstallRecord, manifest: &PluginManifest) -> bool {
    use crate::plugin_manifest::CapRequirement;
    // Fail-closed decode caps (Compromise #28 / META #629 DoS-sweep):
    // `granted_caps_bytes` is an attacker-appendable `Vec<Vec<u8>>`. Reject an
    // over-large consent record (element count) BEFORE `HashSet::with_capacity`
    // allocates, and reject any single over-large element BEFORE `serde`
    // decodes it. An over-cap record is treated as un-coverage (fail closed —
    // never smuggle un-consented caps through an amplification blob).
    if record.granted_caps_bytes.len() > MAX_GRANTED_CAPS {
        return false;
    }
    let mut consented: HashSet<String> = HashSet::with_capacity(record.granted_caps_bytes.len());
    for bytes in &record.granted_caps_bytes {
        if bytes.len() > MAX_GRANTED_CAP_BYTES {
            // Malformed / oversized consent token → un-coverage. Fail closed.
            return false;
        }
        match serde_ipld_dagcbor::from_slice::<CapRequirement>(bytes) {
            Ok(req) => {
                consented.insert(req.scope);
            }
            Err(_) => {
                // Malformed consent token → un-coverage. Fail closed.
                return false;
            }
        }
    }
    manifest
        .requires
        .iter()
        .all(|req| consented.contains(&req.scope))
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

    /// G-CORE-7 §4.21 + §4.35 rollback observable: drop the entry for
    /// `grant_cid` from `minted_grants` so `minted_grants().len()` is
    /// the correct post-rollback observable.
    fn revoke_root_grant(&mut self, grant_cid: &Cid) -> Result<(), ErrorCode> {
        self.minted_grants.retain(|(_, _, _, cid)| cid != grant_cid);
        Ok(())
    }
}

impl PrivateNamespaceProvisioner for InMemoryInstallCascade {
    fn provision_private_namespace(&mut self, plugin_did: &Did) -> Result<(), ErrorCode> {
        self.provisioned_namespaces.insert(plugin_did.clone());
        Ok(())
    }

    /// G-CORE-7 §4.21 rollback observable for the Step-11 fail-path.
    fn deprovision_private_namespace(&mut self, plugin_did: &Did) -> Result<(), ErrorCode> {
        self.provisioned_namespaces.remove(plugin_did);
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
        Did::from_string_for_test_fixture(format!("did:key:z{suffix}"))
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

    /// Build a minimal valid manifest with a given `requires` list — for the
    /// M-1 requires-count ceiling pin.
    fn manifest_with_requires(requires: Vec<CapRequirement>) -> PluginManifest {
        PluginManifest {
            plugin_name: "test".to_string(),
            content_cid: fake_cid(1),
            peer_did: fake_did("AuthorX"),
            peer_signature: vec![0u8; 64],
            requires,
            shares: SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
            renderer_config: None,
            composes_plugins: None,
            accepts_content: None,
            requires_schema_authors: None,
            requires_plugin_authors: None,
        }
    }

    /// M-1: `PluginManifest::validate()` rejects a `requires` array that
    /// exceeds the amplification ceiling. Would-FAIL-on-revert: without the
    /// `MAX_PLUGIN_MANIFEST_REQUIRES` check, an over-cap manifest validates.
    #[test]
    fn m1_validate_rejects_over_cap_requires_count() {
        // At the cap: valid.
        let at_cap = manifest_with_requires(
            (0..MAX_PLUGIN_MANIFEST_REQUIRES)
                .map(|i| CapRequirement::new(format!("store:n{i}:read")))
                .collect(),
        );
        assert!(
            at_cap.validate().is_ok(),
            "M-1: a manifest at exactly the requires cap must validate"
        );
        // One over the cap: rejected fail-closed.
        let over_cap = manifest_with_requires(
            (0..MAX_PLUGIN_MANIFEST_REQUIRES + 1)
                .map(|i| CapRequirement::new(format!("store:n{i}:read")))
                .collect(),
        );
        assert_eq!(
            over_cap.validate().unwrap_err(),
            ErrorCode::PluginManifestInvalid,
            "M-1: a manifest whose requires count exceeds the cap must be rejected"
        );
    }

    /// M-2: `install_record_covers_required_caps` fails closed (returns false)
    /// when `granted_caps_bytes` exceeds the element-count cap. Would-FAIL-on
    /// -revert: without the count cap the over-cap record is decoded and can
    /// smuggle coverage through an amplification blob.
    #[test]
    fn m2_over_cap_granted_caps_count_fails_closed() {
        let manifest = manifest_with_requires(vec![CapRequirement::new("store:notes:read")]);
        let covering =
            serde_ipld_dagcbor::to_vec(&CapRequirement::new("store:notes:read")).unwrap();
        // Over the count cap, every element a valid covering token: still
        // fails closed (rejected before the decode loop), so coverage is
        // denied even though the caps would otherwise cover `requires`.
        let record = InstallRecord {
            manifest_cid: fake_cid(1),
            plugin_did: fake_did("PluginA"),
            consenting_user_did: fake_did("UserA"),
            user_signature: vec![0u8; 64],
            timestamp_stub_nanos: 1,
            nonce: vec![0u8; 16],
            granted_caps_bytes: vec![covering; MAX_GRANTED_CAPS + 1],
        };
        assert!(
            !install_record_covers_required_caps(&record, &manifest),
            "M-2: an over-cap granted_caps_bytes count must fail closed (no coverage)"
        );
    }

    /// M-2: `install_record_covers_required_caps` fails closed when any single
    /// `granted_caps_bytes` element exceeds the per-element byte cap.
    #[test]
    fn m2_over_cap_granted_cap_element_bytes_fails_closed() {
        let manifest = manifest_with_requires(vec![CapRequirement::new("store:notes:read")]);
        let record = InstallRecord {
            manifest_cid: fake_cid(1),
            plugin_did: fake_did("PluginA"),
            consenting_user_did: fake_did("UserA"),
            user_signature: vec![0u8; 64],
            timestamp_stub_nanos: 1,
            nonce: vec![0u8; 16],
            // A single oversized element (all-zero padding well past the cap).
            granted_caps_bytes: vec![vec![0u8; MAX_GRANTED_CAP_BYTES + 1]],
        };
        assert!(
            !install_record_covers_required_caps(&record, &manifest),
            "M-2: an oversized granted_caps_bytes element must fail closed"
        );
    }

    #[test]
    fn uninstall_bogus_cid_returns_plugin_manifest_invalid_without_cascade() {
        let mut library = PluginLibrary::new();
        let mut store = PluginDidStore::new();
        let mut cascade = InMemoryUninstallCascade::new();
        let mut private = InMemoryUninstallCascade::new();
        let mut subs = InMemoryUninstallCascade::new();
        let mut ports = UninstallPorts {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let err =
            uninstall_plugin(&mut library, &mut store, &mut ports, &fake_cid(99)).unwrap_err();
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
        let mut ports = UninstallPorts {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let outcome = uninstall_plugin(&mut library, &mut store, &mut ports, &cid).unwrap();
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
        let mut ports = UninstallPorts {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let outcome = uninstall_plugin(&mut library, &mut store, &mut ports, &cid_a).unwrap();
        assert_eq!(outcome.delegations_cascade_revoked, 1);
        let log = cascade.revocation_log();
        assert!(
            log.iter()
                .any(|r| r.cascade_source == Some(plugin_a.clone()) && r.audience == plugin_b)
        );
    }

    /// Surf-1 #888 regression (Refs #1199): the uninstall cascade keys
    /// off the *install-time* provenance — the library entry's
    /// plugin-DID + the manifest `requires` envelope minted at install.
    /// This pins that the InstallPorts/InstallParams ↔ UninstallPorts
    /// symmetry split preserves the provenance chain end-to-end: every
    /// grant minted under the manifest's `requires` envelope at install
    /// is exactly the set the uninstall cascade revokes (and a
    /// distractor grant under an unrelated DID is untouched).
    ///
    /// WOULD-FAIL if uninstall keyed off anything other than the
    /// install-recorded plugin-DID, or if the manifest-envelope→grant
    /// provenance were dropped across the lifecycle boundary.
    #[test]
    fn uninstall_cascade_preserves_install_time_manifest_envelope_provenance() {
        let plugin_did = fake_did("ProvenancePlugin");
        let user_did = fake_did("ProvenanceUser");
        let other_did = fake_did("UnrelatedPlugin");
        let cid = fake_cid(42);

        // Manifest envelope declared at install: two `requires` scopes.
        let mut library = PluginLibrary::new();
        let mut store = PluginDidStore::new();
        let envelope = ["store:notes:read", "store:notes:write"];
        library.insert(fake_library_entry(plugin_did.clone(), cid));

        // The install-time cap cascade mints one grant per `requires`
        // scope under audience=plugin_did (mirrors install_plugin step
        // 9). Recorded with issuer=user_did = the Layer-1 trust anchor.
        let mut cascade = InMemoryUninstallCascade::new();
        for (i, scope) in envelope.iter().enumerate() {
            cascade.insert_grant(InMemoryGrant {
                grant_cid: fake_cid(100 + i as u8),
                audience: plugin_did.clone(),
                issuer: user_did.clone(),
                scope: (*scope).to_string(),
            });
        }
        // Distractor: a grant under an unrelated plugin-DID that shares
        // an identical scope string — MUST survive (provenance is
        // keyed on the install-recorded plugin-DID, not the scope).
        cascade.insert_grant(InMemoryGrant {
            grant_cid: fake_cid(200),
            audience: other_did.clone(),
            issuer: user_did.clone(),
            scope: "store:notes:read".to_string(),
        });

        let mut private = InMemoryUninstallCascade::new();
        let mut subs = InMemoryUninstallCascade::new();
        let mut ports = UninstallPorts {
            cap_revoker: &mut cascade,
            private_ns: &mut private,
            subscriptions: &mut subs,
        };
        let outcome = uninstall_plugin(&mut library, &mut store, &mut ports, &cid).unwrap();

        // Exactly the install-envelope grants (keyed on the
        // install-recorded plugin-DID) were revoked.
        assert_eq!(outcome.held_caps_revoked, envelope.len());
        assert_eq!(
            cascade
                .revocation_log()
                .iter()
                .filter(|r| r.audience == plugin_did)
                .count(),
            envelope.len(),
            "every manifest-envelope grant minted at install MUST be revoked"
        );
        // The unrelated-DID grant (identical scope, different
        // provenance) survives — provenance is keyed on the
        // install-recorded plugin-DID, not the scope string.
        assert!(
            !cascade
                .revocation_log()
                .iter()
                .any(|r| r.audience == other_did),
            "unrelated-DID grant (same scope, different provenance) MUST survive"
        );
        assert!(
            !cascade.active_grants_for_audience(&other_did).is_empty(),
            "distractor grant MUST remain active post-uninstall"
        );
    }
}
