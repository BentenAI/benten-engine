//! Phase-4-Foundation G24-D — plugin-DID mint + store.
//!
//! Per CLAUDE.md baked-in #18 (four-identity-concepts model):
//!
//! **Plugin-DID is a UCAN audience handle AND constrained issuer**
//! within the manifest envelope. It is **NOT an attested sub-identity**
//! of user-DID. There is NO `PluginAttestationEnvelope`, NO
//! `attestation_chain_for_plugin_did`, NO device-DID-style
//! attestation chain — those patterns belong to device-DIDs (which
//! represent physical hardware) and explicitly do NOT apply to
//! plugin-DIDs (which are code running inside the user's engine).
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

/// Verify that a UCAN audience claim is bound to a known plugin-DID.
///
/// This is a SHAPE-check only — it does NOT traverse an
/// attestation chain. The cap-policy backend at
/// `benten-caps::manifest_envelope_chain_validation` walks the actual
/// UCAN chain; this function just confirms the audience field
/// matches the plugin-DID's resolved string form.
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
    pub fn mint_and_store(&mut self) -> Did {
        let handle = mint();
        let did = handle.did.clone();
        self.handles.push(handle);
        did
    }

    /// Look up a handle by DID.
    #[must_use]
    pub fn get(&self, did: &Did) -> Option<&PluginDidHandle> {
        self.handles.iter().find(|h| h.did == *did)
    }

    /// Iterate over all minted plugin-DIDs.
    pub fn iter(&self) -> impl Iterator<Item = &Did> {
        self.handles.iter().map(|h| &h.did)
    }

    /// Remove a plugin-DID from the store (uninstall path).
    pub fn revoke(&mut self, did: &Did) -> bool {
        let before = self.handles.len();
        self.handles.retain(|h| h.did != *did);
        self.handles.len() < before
    }

    /// Count of minted plugin-DIDs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Whether the store is empty.
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
