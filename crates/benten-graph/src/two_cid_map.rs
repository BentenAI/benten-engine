//! Two-CID mapping types (G-CORE-3d).
//!
//! # Surface
//!
//! - [`TwoCidMap`] — marker type for the mapping concept. The mapping
//!   itself lives inside the `RedbBackend` as a dedicated redb table
//!   keyed `m:<plaintext_cid> → <ciphertext_cid>`; this module owns the
//!   error envelope + per-DID-scoped key helpers consumed by
//!   `redb_backend.rs` (kept here to keep the storage surface focused
//!   on tables + transactions and the cross-cutting two-CID concept
//!   focused on its own module).
//! - [`TwoCidMapError`] — typed error envelope for the mapping path
//!   (mapping-table miss, tamper-detected integrity mismatch, AEAD
//!   authentication failure surfaced through the read path).
//!
//! # Why two CIDs (per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` R2)
//!
//! - `plaintext_cid = BLAKE3(canonical_dag_cbor(Node))` — the
//!   user-facing logical identity of a Node. UCAN scopes are written
//!   against this CID.
//! - `ciphertext_cid = BLAKE3(AeadEnvelope::to_wire_bytes())` — the
//!   storage / transport identity. iroh-blobs serves these bytes; redb
//!   keys storage on this CID inside the per-DID partition.
//!
//! Two distinct CIDs because content-addressing the ciphertext lets
//! sync (iroh-blobs) integrity-check the bytes it serves WITHOUT
//! decrypting them, while content-addressing the plaintext lets UCAN
//! grant scope against the underlying logical Node (whose identity
//! must be stable across re-encryption / re-keying / per-recipient
//! envelope variation).
//!
//! The mapping `plaintext_cid → ciphertext_cid` is redb-backed
//! (durable across restart) so a UCAN scope check at sync time can
//! resolve to the actual served bytes after a process restart. Without
//! durability the resolution silently breaks post-restart (a UCAN
//! grant references a CID nobody knows how to serve), which is a
//! confidentiality + availability hole.

use benten_core::Cid;
use thiserror::Error;

/// Marker for the two-CID mapping concept. The mapping table itself
/// lives inside `RedbBackend` (a dedicated redb table); this type is
/// the conceptual handle so downstream callers can write
/// `TwoCidMap::table_key(plaintext)` symmetrically with other key
/// helpers in `crate::store`.
///
/// Construction is via the backend extension; this type carries no
/// in-memory state.
#[derive(Debug, Clone, Copy, Default)]
pub struct TwoCidMap;

impl TwoCidMap {
    /// Build the redb table key for `plaintext_cid` in the two-CID
    /// mapping table. The mapping table is partition-LOCAL (lives
    /// inside the per-DID namespace prefix) so cross-DID lookups are
    /// structurally invisible — the partition-isolation invariant
    /// (multitenant-r1-5) fires BEFORE the AEAD layer.
    #[must_use]
    pub fn table_key(plaintext_cid: &Cid) -> Vec<u8> {
        let mut k = Vec::with_capacity(b"m:".len() + plaintext_cid.as_bytes().len());
        k.extend_from_slice(b"m:");
        k.extend_from_slice(plaintext_cid.as_bytes());
        k
    }

    /// Partition-local table key — `d:<did>:m:<plaintext_cid>`. Used
    /// by `RedbBackend::two_cid_lookup_scoped` for per-DID mapping
    /// lookups so cross-DID reads return `NotFound` at the prefix
    /// match step (BEFORE any AEAD work).
    #[must_use]
    pub fn partition_table_key(namespace_did: &Cid, plaintext_cid: &Cid) -> Vec<u8> {
        let mut k = Vec::with_capacity(
            b"d:".len()
                + namespace_did.as_bytes().len()
                + b":m:".len()
                + plaintext_cid.as_bytes().len(),
        );
        k.extend_from_slice(b"d:");
        k.extend_from_slice(namespace_did.as_bytes());
        k.extend_from_slice(b":m:");
        k.extend_from_slice(plaintext_cid.as_bytes());
        k
    }
}

/// Typed error envelope for the two-CID mapping read path.
///
/// Distinguishes three classes of failure so callers can dispatch
/// correctly:
/// - `NotFound` — no mapping row exists. NOT a tamper or auth failure;
///   semantically "this plaintext CID was never written here / under
///   this scope". The partition-isolation arm at the head of
///   `read_via_two_cid_scoped` MUST surface this when a cross-DID
///   caller reads, NEVER `AeadAuthenticationFailed` (which would
///   imply the caller saw the ciphertext bytes — a confidentiality
///   leak).
/// - `IntegrityMismatch` — mapping row exists but the
///   `ciphertext_cid` it points at fails its content-hash check (the
///   stored bytes don't BLAKE3 to the ciphertext_cid the mapping
///   claims). The redb-side tamper-detection arm.
/// - `AeadAuthenticationFailed` — bytes recovered + content-hashed
///   correctly but the AEAD authenticator rejected (AAD mismatch /
///   tag mismatch). The cryptographic tamper-detection arm.
#[derive(Debug, Error)]
pub enum TwoCidMapError {
    /// No mapping entry for the queried plaintext CID under the
    /// active scope (un-namespaced or per-DID partition). Semantically
    /// "not in mapping," NOT "mapping subsystem broken."
    #[error("two-CID mapping has no entry for plaintext CID {plaintext_cid}")]
    NotFound {
        /// The plaintext CID the caller queried (in canonical
        /// base32-multibase form for diagnostics).
        plaintext_cid: String,
    },

    /// Mapping row exists but the bytes at the claimed `ciphertext_cid`
    /// don't BLAKE3 to that CID — storage-side tamper / corruption.
    #[error("two-CID mapping integrity mismatch: expected ciphertext CID {expected}, got {actual}")]
    IntegrityMismatch {
        /// The CID the mapping row claimed for the stored ciphertext.
        expected: String,
        /// The CID computed from the stored bytes.
        actual: String,
    },

    /// AEAD authentication rejected at decrypt time. Includes the
    /// AAD-binds-plaintext-CID rebinding-attack defense — if a mapping
    /// row was tampered to point at a foreign ciphertext, the foreign
    /// ciphertext's seal-time AAD won't match the decrypt-time AAD
    /// reconstructed from the queried plaintext CID.
    #[error("AEAD authentication failed reading via two-CID mapping: {reason}")]
    AeadAuthenticationFailed {
        /// Diagnostic note carrying the underlying AEAD error class.
        reason: String,
    },

    /// Underlying graph-storage failure surfaced through the two-CID
    /// read path (redb I/O, decode failure on the AEAD envelope wire
    /// bytes, etc).
    #[error("storage-layer failure reading via two-CID mapping: {reason}")]
    Storage {
        /// Diagnostic note.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_key_prefixes_with_m_colon() {
        let cid = Cid::from_blake3_digest([0x42u8; 32]);
        let key = TwoCidMap::table_key(&cid);
        assert!(key.starts_with(b"m:"));
        assert_eq!(&key[2..], cid.as_bytes());
    }

    #[test]
    fn partition_table_key_starts_with_did_prefix() {
        let did = Cid::from_blake3_digest([0xaau8; 32]);
        let cid = Cid::from_blake3_digest([0x42u8; 32]);
        let key = TwoCidMap::partition_table_key(&did, &cid);
        assert!(key.starts_with(b"d:"));
        assert!(
            key.windows(b":m:".len()).any(|w| w == b":m:"),
            "partition table key must contain `:m:` after the DID bytes"
        );
    }
}
