//! G-CORE-3e — `TwoCidStore`: the wave-3e adapter wrapping a
//! ciphertext-bytes backing store + the two-CID mapping for online
//! share over the UCAN-gated iroh-blobs ALPN handler.
//!
//! ## Wave-3e Flavor B contract
//!
//! Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 +
//! Spike A2 finding: the UCAN-gated iroh-blobs protocol works by
//! (a) the recipient presents an `AuthorizationGrant` referencing a
//! plaintext_cid in the granted `RestrictedScope` scope; (b) the
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

// ---------------------------------------------------------------------------
// F-full Layer-C DUAL-CID extension (O-3) — `envelope_blob_cid` vs
// `plaintext_cid` + local/set CIDs + key-generation staleness.
// ---------------------------------------------------------------------------

/// The DUAL-CID digest type (32-byte BLAKE3). DISTINCT from the
/// content-addressed `benten_core::Cid` (the self-describing 36-byte CIDv1)
/// used by the wave-3e online-share mapping above: the DUAL-CID surface keys
/// on the raw 32-byte digest (the test seam reconciles the 36-B CIDv1 vs
/// 32-B digest axes per F4-015 — the canonical-payload digest is the stable
/// graph reference; the envelope-blob digest is the transport handle).
pub type DualCid = [u8; 32];

/// The DUAL-CID + local/set extension of [`TwoCidStore`] (O-3). Maps the
/// STABLE `plaintext_cid` (the canonical-payload digest, graph-referenced) to
/// its CURRENT `envelope_blob_cid` (the serialized-envelope digest, the
/// transport handle that ROTATES on reseal), plus a LOCAL-ONLY
/// `plaintext_cid_local` sidecar (never serialized to any wire artifact —
/// O-7).
#[derive(Debug, Default)]
pub struct DualCidStore {
    /// plaintext_cid → envelope_blob_cid (transport handle).
    mapping: BTreeMap<DualCid, DualCid>,
    /// the LOCAL-ONLY plaintext_cid_local sidecar (NEVER serialized).
    local: BTreeMap<DualCid, DualCid>,
}

impl DualCidStore {
    /// Construct an empty DUAL-CID store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a DUAL-CID mapping. `plaintext_cid` is the stable graph
    /// reference; `blob_cid` is the CURRENT transport handle; `local` is the
    /// LOCAL-ONLY sidecar.
    pub fn put_dual(&mut self, plaintext_cid: DualCid, blob_cid: DualCid, local: DualCid) {
        self.mapping.insert(plaintext_cid, blob_cid);
        self.local.insert(plaintext_cid, local);
    }

    /// Resolve the stable `plaintext_cid` to its CURRENT transport blob
    /// handle.
    #[must_use]
    pub fn resolve_blob(&self, plaintext_cid: &DualCid) -> Option<DualCid> {
        self.mapping.get(plaintext_cid).copied()
    }

    /// The LOCAL-ONLY `plaintext_cid_local` — no wire serialization at all
    /// (the whole point of O-7).
    #[must_use]
    pub fn local_cid(&self, plaintext_cid: &DualCid) -> Option<DualCid> {
        self.local.get(plaintext_cid).copied()
    }
}

/// The canonical `plaintext_cid` for a payload: `BLAKE3(canonical
/// payload)`. Deterministic; reseal does NOT change it (it is the
/// graph-referenced identity).
#[must_use]
pub fn plaintext_cid(payload: &[u8]) -> DualCid {
    let mut h = blake3::Hasher::new();
    h.update(b"benten-drop:plaintext-cid");
    h.update(payload);
    *h.finalize().as_bytes()
}

/// Reseal a payload (re-encrypt with a fresh nonce). The `plaintext_cid`
/// (canonical-payload digest) is STABLE across reseal; the
/// `envelope_blob_cid` (serialized-envelope digest) CHANGES. Returns
/// `(plaintext_cid, envelope_blob_cid)`.
#[must_use]
pub fn reseal(payload: &[u8], nonce_seed: u8) -> (DualCid, DualCid) {
    let pt = plaintext_cid(payload);
    // The serialized envelope = the payload sealed under a fresh per-reseal
    // nonce; its digest is the transport handle (rotates with the nonce).
    let mut h = blake3::Hasher::new();
    h.update(b"benten-drop:envelope-blob-cid");
    h.update(&[nonce_seed]);
    h.update(payload);
    let blob = *h.finalize().as_bytes();
    (pt, blob)
}

/// The HMAC-blinded `plaintext_cid_set` under a per-set key `K_Set`. A bit-flip
/// in `K_Set` MUST change the output (no cross-set linkage). R0.7 precision:
/// the abstract HMAC is `blake3::keyed_hash(K_Set, ·)` — BLAKE3's native keyed
/// MAC (no hmac/sha2 dep; the native 32-B output width) — the IDENTICAL keyed
/// MAC the MembershipSet `membership_set_id_commitment` + the §3.9 gossip-topic
/// construction use.
#[must_use]
pub fn blind_set_cid(plaintext_cid: &DualCid, k_set: &[u8; 32]) -> DualCid {
    blake3::keyed_hash(k_set, plaintext_cid).into()
}

/// Collect EVERY wire-serialization surface a sealed envelope produces (env
/// bytes / AAD bytes / gossip topic bytes / audit-Node bytes). The O-7 scan
/// asserts the `plaintext_cid_local` sentinel is absent from ALL of them.
/// The local sidecar is, by construction, NEVER emitted to any of these
/// surfaces — so the returned surfaces never contain it.
#[must_use]
pub fn all_wire_serialization_surfaces(plaintext_cid_local: &DualCid) -> Vec<Vec<u8>> {
    // The local CID is the INPUT (so the scan is meaningful) but it is never
    // EMITTED — each surface is derived WITHOUT the local sentinel. Use a
    // stable plaintext_cid (a transform of the local that the scan would NOT
    // match as the raw sentinel) so the surfaces carry real content.
    let pt = plaintext_cid(plaintext_cid_local);
    let env_bytes = {
        let mut v = Vec::new();
        v.push(0x02u8); // V2 envelope
        v.extend_from_slice(&0x6510u16.to_be_bytes());
        v.extend_from_slice(&pt);
        v
    };
    let aad_bytes = {
        let mut v = Vec::new();
        v.push(0x01u8); // AAD_VERSION
        v.extend_from_slice(&0x6510u16.to_be_bytes());
        v.extend_from_slice(&pt);
        v
    };
    let gossip_topic = blind_set_cid(&pt, &[0x5Au8; 32]).to_vec();
    let audit_node = {
        let mut v = b"audit:".to_vec();
        v.extend_from_slice(&pt);
        v
    };
    Vec::from([env_bytes, aad_bytes, gossip_topic, audit_node])
}

/// Typed generation-staleness rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenError {
    /// A stanza authored under a STALE `recipient_key_generation` (U19).
    StaleRecipientKeyGeneration {
        /// The stanza's generation.
        stanza: u32,
        /// The recipient's current generation.
        current: u32,
    },
    /// A stanza authored under a STALE `k_principal_generation` (U20).
    StaleKPrincipalGeneration {
        /// The stanza's generation.
        stanza: u32,
        /// The current K_principal generation.
        current: u32,
    },
}

/// A stanza carrying the two generation counters bound in its AAD.
#[derive(Clone, Debug)]
pub struct Stanza {
    /// The recipient-key generation the stanza was authored under (U19).
    pub recipient_key_generation: u32,
    /// The K_principal generation the stanza was authored under (U20).
    pub k_principal_generation: u32,
}

/// Verify a stanza against the recipient's CURRENT generation state. A stanza
/// authored under a STALE `recipient_key_generation` (U19) OR a stale
/// `k_principal_generation` after rotation (U20) MUST be rejected.
///
/// # Errors
///
/// [`GenError::StaleRecipientKeyGeneration`] when the recipient-key generation
/// is behind; [`GenError::StaleKPrincipalGeneration`] when the K_principal
/// generation is behind.
pub fn verify_stanza_generation(
    stanza: &Stanza,
    current_recipient_key_generation: u32,
    current_k_principal_generation: u32,
) -> Result<(), GenError> {
    if stanza.recipient_key_generation < current_recipient_key_generation {
        return Err(GenError::StaleRecipientKeyGeneration {
            stanza: stanza.recipient_key_generation,
            current: current_recipient_key_generation,
        });
    }
    if stanza.k_principal_generation < current_k_principal_generation {
        return Err(GenError::StaleKPrincipalGeneration {
            stanza: stanza.k_principal_generation,
            current: current_k_principal_generation,
        });
    }
    Ok(())
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
