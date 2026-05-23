//! G-CORE-3e — `TwoCidStore`: the wave-3e adapter wrapping a
//! ciphertext-bytes backing store + the two-CID mapping for online
//! share over the UCAN-gated iroh-blobs ALPN handler.
//!
//! ## Wave-3e Flavor B contract
//!
//! Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 +
//! Spike A2 finding: the UCAN-gated iroh-blobs protocol works by
//! (a) the recipient presents an `AuthorizationGrant` referencing a
//! plaintext_cid in the granted `RestrictedSpec` scope; (b) the
//! handler validates the grant; (c) the handler resolves the
//! plaintext_cid → ciphertext_cid through the two-CID mapping; (d) the
//! handler hands the connection to `iroh_blobs::provider::handle_connection`
//! to serve the ciphertext bytes by hash.
//!
//! ## What this module owns
//!
//! - [`TwoCidStore`] — the wave-3e adapter that owns:
//!   1. The plaintext_cid → ciphertext_cid mapping (queried via the
//!      `benten-graph::two_cid_map::TwoCidMap` key-helpers per the
//!      established G-CORE-3d substrate seam).
//!   2. The ciphertext-bytes backing store (an `iroh-blobs::FsStore`
//!      at production wire-up; an in-memory `BTreeMap` at this wave
//!      so the per-request UCAN validation arm + scope-check arm + the
//!      end-to-end share pin can land their substantive proofs WITHOUT
//!      pulling iroh-blobs into the dependency tree mid-wave).
//!
//! ## iroh-blobs FsStore swap-point
//!
//! The production wire-up swaps the in-memory `BTreeMap` for
//! `iroh_blobs::FsStore` at the [`TwoCidStore::ciphertext_bytes`]
//! boundary. The crate-level seam is named here (NOT a separate trait)
//! so the swap is a 1-line change at one call site at the next
//! integration wave (G-CORE-3-real-iroh-blobs); landing iroh-blobs as
//! a workspace dependency mid-wave is OUT-OF-SCOPE for G-CORE-3e
//! (this wave proves the per-request UCAN check + scope check + the
//! identity-cast plumbing — the iroh-blobs *bytes-serving* arm is the
//! `iroh_blobs::provider::handle_connection` call which the brief
//! explicitly says "reuses" from upstream; the named swap-point here
//! is the discipline that closes the wave-completion checklist
//! cleanly without conflating the per-request-UCAN-check arm with the
//! upstream bytes-serving plumbing).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::sync::Mutex;

use benten_core::Cid;

/// The wave-3e two-CID + ciphertext-bytes store. Owns:
///
/// - The plaintext_cid → ciphertext_cid mapping (wave-3e local copy;
///   the production wire-up at integration time reads through the
///   `benten-graph::redb_backend::RedbBackend` two-CID table directly
///   per the G-CORE-3d substrate).
/// - The ciphertext-bytes backing (in-memory at this wave; swaps to
///   `iroh_blobs::FsStore` at the named seam).
///
/// Thread-safe (interior `Mutex`) so the wave-3e ALPN handler can be
/// shared across concurrent inbound connections without an outer
/// `Arc<RwLock<_>>` ceremony.
#[derive(Debug, Default)]
pub struct TwoCidStore {
    /// plaintext_cid → ciphertext_cid mapping.
    mapping: Mutex<BTreeMap<Cid, Cid>>,
    /// ciphertext_cid → ciphertext bytes (the wave-3e in-memory
    /// stand-in for `iroh_blobs::FsStore`). Production swap-point per
    /// module-level docs.
    ciphertext_bytes: Mutex<BTreeMap<Cid, Vec<u8>>>,
}

impl TwoCidStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a plaintext_cid → ciphertext_cid mapping + the
    /// ciphertext bytes. Used at write-side (the sender publishing a
    /// subgraph through the wave-3e share path).
    pub fn put(&self, plaintext_cid: Cid, ciphertext_cid: Cid, ciphertext_bytes: Vec<u8>) {
        self.mapping
            .lock()
            .expect("TwoCidStore mapping mutex poisoned")
            .insert(plaintext_cid, ciphertext_cid);
        self.ciphertext_bytes
            .lock()
            .expect("TwoCidStore ciphertext_bytes mutex poisoned")
            .insert(ciphertext_cid, ciphertext_bytes);
    }

    /// Resolve a plaintext_cid through the two-CID mapping to its
    /// ciphertext_cid. Returns `None` if the plaintext_cid is not
    /// known to this store — at the wave-3e handler this surfaces as
    /// the upstream-layer "no bytes to serve" path (NOT a UCAN
    /// rejection; that's the
    /// [`crate::ucan_blobs_protocol::UcanBlobsHandler`] concern at the
    /// per-request validation step).
    #[must_use]
    pub fn resolve_plaintext_to_ciphertext(&self, plaintext_cid: &Cid) -> Option<Cid> {
        self.mapping
            .lock()
            .expect("TwoCidStore mapping mutex poisoned")
            .get(plaintext_cid)
            .copied()
    }

    /// Fetch the ciphertext bytes for a ciphertext_cid. **This is the
    /// iroh-blobs `FsStore` swap-point** — production wire-up
    /// replaces the in-memory `BTreeMap` lookup with the iroh-blobs
    /// FsStore call. Returns `None` if the ciphertext is not stored
    /// locally.
    #[must_use]
    pub fn ciphertext_bytes(&self, ciphertext_cid: &Cid) -> Option<Vec<u8>> {
        self.ciphertext_bytes
            .lock()
            .expect("TwoCidStore ciphertext_bytes mutex poisoned")
            .get(ciphertext_cid)
            .cloned()
    }

    /// Number of mappings — surface for the wave-3e test seam +
    /// observability.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mapping
            .lock()
            .expect("TwoCidStore mapping mutex poisoned")
            .len()
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid(seed: u8) -> Cid {
        Cid::from_blake3_digest([seed; 32])
    }

    #[test]
    fn put_and_resolve_round_trips() {
        let store = TwoCidStore::new();
        let pt = cid(0x11);
        let ct = cid(0x22);
        store.put(pt, ct, vec![1, 2, 3, 4]);
        assert_eq!(store.resolve_plaintext_to_ciphertext(&pt), Some(ct));
        assert_eq!(store.ciphertext_bytes(&ct), Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn missing_plaintext_cid_resolves_to_none() {
        let store = TwoCidStore::new();
        assert!(store.resolve_plaintext_to_ciphertext(&cid(0x99)).is_none());
    }

    #[test]
    fn empty_store_is_empty() {
        let store = TwoCidStore::new();
        assert!(store.is_empty());
        store.put(cid(0x01), cid(0x02), vec![]);
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }
}
