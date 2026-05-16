//! Phase-4-Foundation G24-D — plugin-DID mint + store.
//!
//! Per CLAUDE.md baked-in #18 (four-identity-concepts model):
//!
//! **Plugin-DID is a UCAN audience handle AND constrained issuer**
//! within the manifest envelope. It is **NOT an attested sub-identity**
//! of user-DID. The attestation-chain patterns that belong to
//! device-DIDs (which represent physical hardware) explicitly do NOT
//! apply to plugin-DIDs (which are code running inside the user's
//! engine). See `device_attestation` module for the device-DID side;
//! this module intentionally does NOT mirror its surface.
//!
//! Plugin-DID:
//! - is freshly minted at every install via OsRng (per D-4F-16)
//! - identifies the audience of UCAN delegations from user-DID
//!   (`audience = plugin-DID`)
//! - issues UCAN delegations to OTHER plugin-DIDs WITHIN the manifest
//!   envelope (constrained issuer; chain validator at
//!   `benten-caps::manifest_envelope_chain_validation` enforces)
//! - has no inherent authority — issuance is bounded by the source
//!   plugin's manifest `shares` policy AND the chain must still trace
//!   back to a user-DID-issued root grant
//!
//! Per `docs/PLUGIN-MANIFEST.md` §2 "The four identity concepts".

use benten_errors::ErrorCode;

use crate::did::Did;
use crate::keypair::Keypair;

/// A minted plugin-DID handle.
///
/// Holds the keypair (for issuance) + the resolved Did (for audience
/// matching). Cloning the handle SHARES the keypair via the underlying
/// SigningKey storage; production callers should NOT clone — the store
/// owns the canonical instance.
#[derive(Debug)]
pub struct PluginDidHandle {
    /// The DID string form (`did:key:z...`).
    did: Did,
    /// The signing keypair (per-install fresh-OsRng).
    keypair: Keypair,
}

impl PluginDidHandle {
    /// The minted DID identifier.
    #[must_use]
    pub fn did(&self) -> &Did {
        &self.did
    }

    /// Borrow the keypair for signing-issuance.
    ///
    /// **Hyg-1 #313 — DISAGREE-WITH-EXPLANATION (HARD RULE 12 (c)).**
    /// This accessor has no production caller AT HEAD, but it is NOT
    /// speculative dead code: per CLAUDE.md baked-in #18, plugin-DID
    /// is a *constrained issuer* — it "issues UCAN delegations to
    /// OTHER plugin-DIDs WITHIN the manifest envelope." That issuance
    /// call site (the `benten-caps` manifest-envelope-chain issuer
    /// path) is Phase-4-Meta scope. The keypair MUST be reachable for
    /// that wiring; deleting it now would force a SemVer-breaking
    /// re-add at the exact moment the constrained-issuer path lands.
    /// The `keypair` field is read solely through this accessor by
    /// design (the store owns the canonical instance; production
    /// callers must not clone — see the struct docstring).
    #[must_use]
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

/// Mint a fresh plugin-DID at install time.
///
/// Per D-4F-16: `did:key:z...` shape with engine-held Ed25519 keypair
/// generated via OsRng. One keypair per install.
///
/// This function is the ONLY surface that mints plugin-DIDs. It does
/// NOT compute an attestation envelope, does NOT bind to a parent
/// user-DID via signature, does NOT consult `RotationLog`. The minted
/// DID is a fresh identity (audience handle) — the binding to the
/// user happens at the `InstallRecord` layer where user-DID signs an
/// envelope referencing the plugin-DID, NOT at this minting layer.
#[must_use]
pub fn mint() -> PluginDidHandle {
    let keypair = Keypair::generate();
    let did = keypair.public_key().to_did();
    PluginDidHandle { did, keypair }
}

/// Test-only constructor for `PluginDidHandle` that takes a caller-
/// supplied DID + a fresh-mint keypair. The DID and keypair will NOT
/// be cryptographically bound (the DID is supplied independently),
/// which makes this constructor unsuitable for production code paths
/// (production must use [`mint`] so the DID byte-derives from the
/// keypair). Exists to test code paths that need two `PluginDidHandle`
/// values with byte-equal DIDs — primarily the
/// `PluginDidStore::insert` duplicate-rejection arm at
/// [`ErrorCode::PluginDidHandleDuplicate`] (R6-FP-3 cap-r6-r3-1).
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn handle_with_did_for_test(did: Did) -> PluginDidHandle {
    let keypair = Keypair::generate();
    PluginDidHandle { did, keypair }
}

/// Verify that a UCAN audience claim is bound to a known plugin-DID.
///
/// This is a SHAPE-check only — it does NOT traverse an
/// attestation chain. The cap-policy backend at
/// `benten-caps::manifest_envelope_chain_validation` walks the actual
/// UCAN chain; this function just confirms the audience field
/// matches the plugin-DID's resolved string form.
///
/// **Hyg-1 #320 — DISAGREE-WITH-EXPLANATION (HARD RULE 12 (c)),
/// production-zero / test caller exists.** Per CLAUDE.md baked-in #18,
/// plugin-DID is a UCAN *audience handle*; this is the canonical
/// audience-binding shape-check the manifest-envelope chain validator
/// composes with. The chain-validator wiring is Phase-4-Meta scope —
/// deleting the shape-check now would force a re-add when that path
/// lands. T-3v-D wording: "production-zero", not "zero callers"
/// (inline tests exercise it).
#[must_use]
pub fn audience_matches_plugin_did(audience: &Did, plugin_did: &Did) -> bool {
    audience == plugin_did
}

/// An in-memory store of minted plugin-DIDs.
///
/// Production code persists this via redb (Phase 4-Foundation
/// `ManifestStore` — shares storage with `GrantStore` per cap-r1-15).
/// At G24-D wave the in-memory shape is the canonical type.
#[derive(Debug, Default)]
pub struct PluginDidStore {
    handles: Vec<PluginDidHandle>,
}

impl PluginDidStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a new plugin-DID + persist into the store. Returns the
    /// minted DID for the caller to embed in `InstallRecord`.
    ///
    /// **Hyg-1 #318 — production-zero / test caller exists.** The
    /// production install path uses [`mint`] + [`PluginDidStore::insert`]
    /// (caller-mint-first contract per `docs/PLUGIN-MANIFEST.md §3`);
    /// `mint_and_store` is the test-convenience that fuses both for
    /// store round-trip tests. Retained (DISAGREE-WITH-EXPLANATION,
    /// HARD RULE 12 (c)): it is the documented mint+persist convenience
    /// the store-roundtrip pins drive; deleting it would force those
    /// pins to inline the two-step dance. T-3v-D wording sharpen:
    /// "production-zero" (NOT "zero callers" — a test caller exists).
    pub fn mint_and_store(&mut self) -> Did {
        let handle = mint();
        let did = handle.did.clone();
        self.handles.push(handle);
        did
    }

    /// Persist a pre-minted [`PluginDidHandle`] into the store.
    ///
    /// G24-D-FP-1: the install path mints the plugin-DID via
    /// [`mint`] (so the receiver can return the `LibraryEntry`'s
    /// plugin-DID immediately) and then persists the handle into the
    /// store via this method — at uninstall time
    /// [`PluginDidStore::revoke`] can then succeed because the store
    /// actually carries the minted DID. Prior to this method the
    /// install path had nowhere to persist the handle, leaving the
    /// `plugin_did_revoked` observable structurally false (g24d
    /// substantive pipeline test simulation limitation).
    ///
    /// # Errors
    ///
    /// R6-FP-3 (cap-r6-r3-1 defensive-return hardening): returns
    /// [`ErrorCode::PluginDidHandleDuplicate`] if a handle with the
    /// same DID is already present in the store. The caller-mint-first
    /// contract (per `docs/PLUGIN-MANIFEST.md §3 Plugin-DID minting
    /// protocol`) presumes each plugin-DID is minted exactly once +
    /// inserted exactly once; a duplicate-insert attempt indicates
    /// either a caller bug (double-mint or double-insert in the install
    /// path) or an adversarial collision attempt (would require finding
    /// two Ed25519 keypairs whose `did:key:` encodings collide, which
    /// is computationally infeasible).
    pub fn insert(&mut self, handle: PluginDidHandle) -> Result<(), ErrorCode> {
        if self.handles.iter().any(|h| h.did == handle.did) {
            return Err(ErrorCode::PluginDidHandleDuplicate);
        }
        self.handles.push(handle);
        Ok(())
    }

    /// Look up a handle by DID.
    #[must_use]
    pub fn get(&self, did: &Did) -> Option<&PluginDidHandle> {
        self.handles.iter().find(|h| h.did == *did)
    }

    // Hyg-1 #318: `PluginDidStore::iter()` removed — ZERO callers
    // anywhere (production or test). Speculative enumeration surface
    // that never grew a caller (CLAUDE.md #5 / META #355). Lookups go
    // through `get(&Did)`; the uninstall path uses `revoke(&Did)`.

    /// Remove a plugin-DID from the store (uninstall path).
    pub fn revoke(&mut self, did: &Did) -> bool {
        let before = self.handles.len();
        self.handles.retain(|h| h.did != *did);
        self.handles.len() < before
    }

    /// Count of minted plugin-DIDs.
    ///
    /// **Hyg-1 #318 — production-zero / test caller exists.** Driven
    /// by the `plugin_did_store_insert_duplicate_rejected` integration
    /// pin (asserts the duplicate-insert arm does NOT grow the store).
    /// T-3v-D wording sharpen: "production-zero" not "zero callers".
    #[must_use]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Whether the store is empty.
    ///
    /// **Hyg-1 #318 — retained for `clippy::len_without_is_empty`.**
    /// No direct caller, but a public `len()` without a paired
    /// `is_empty()` is a clippy lint; this is the idiomatic pairing,
    /// not speculative surface.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_produces_did_key_shape() {
        let handle = mint();
        assert!(handle.did().as_str().starts_with("did:key:z"));
    }

    #[test]
    fn mint_twice_produces_distinct_dids() {
        let a = mint();
        let b = mint();
        assert_ne!(a.did(), b.did(), "OsRng must produce distinct dids");
    }

    #[test]
    fn store_mint_persist_lookup_revoke() {
        let mut store = PluginDidStore::new();
        let did = store.mint_and_store();
        assert!(store.get(&did).is_some());
        assert_eq!(store.len(), 1);
        assert!(store.revoke(&did));
        assert!(store.get(&did).is_none());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn audience_matches_plugin_did_shape_check_only() {
        let handle = mint();
        assert!(audience_matches_plugin_did(handle.did(), handle.did()));
        let other = mint();
        assert!(!audience_matches_plugin_did(other.did(), handle.did()));
    }
}
