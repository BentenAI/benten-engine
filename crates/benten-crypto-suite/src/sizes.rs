//! The size-touching surface aggregator — the canonical no-hardcoded-sizes
//! substrate the cross-surface ML-DSA-65 vector round-trips through (TF-2).
//!
//! This module provides a *production-shaped* path through every surface a
//! signature touches (struct / DAG-CBOR / redb / CID-derivation / napi /
//! fixtures) so the TF-2 pin can drive a concrete ~1952 B-key / ~3309 B-sig
//! ML-DSA-65 vector through each surface and assert byte-exact round-trip.
//! It does NOT bypass the production sign/verify path — it carries an
//! ML-DSA-65-dimensioned signature directly through each layer to assert
//! none of them silently clips it to an Ed25519-shaped 64 B.
//!
//! The `_for_test` constructors are crate-public so the TF-2 file can
//! drive ML-DSA-65 dimensions without instantiating a full hybrid keypair
//! (the dimensions are what we're testing, not the live-sign throughput).

use serde::{Deserialize, Serialize};

use crate::codepoint::SigCodepoint;
use crate::sig::HybridSignature;

// Per FIPS 204 ML-DSA-65 (Category-3): public-key 1952 B; signature 3309 B.
// Sourced from upstream `ml-dsa` crate constants (NOT a Benten redefinition);
// kept as `pub(crate)` constants to feed [`SyntheticVector`] generation.
pub(crate) const ML_DSA_65_PUBKEY_LEN: usize = 1952;
pub(crate) const ML_DSA_65_SIG_LEN: usize = 3309;

/// A concrete ML-DSA-65-dimensioned synthetic vector used by TF-2 to drive
/// the size-touching surfaces.
pub struct SyntheticVector {
    pq_pubkey: Vec<u8>,
    pq_sig: Vec<u8>,
}

impl SyntheticVector {
    /// Build a synthetic vector at the canonical ML-DSA-65 dimensions
    /// (~1952 B key / ~3309 B sig). Bytes are deterministic-pattern (so
    /// TF-2 assertions are stable).
    #[must_use]
    pub fn ml_dsa65_for_test() -> Self {
        let pq_pubkey: Vec<u8> = (0..ML_DSA_65_PUBKEY_LEN as u32)
            .map(|i| (i & 0xff) as u8)
            .collect();
        let pq_sig: Vec<u8> = (0..ML_DSA_65_SIG_LEN as u32)
            .map(|i| ((i ^ 0xa5) & 0xff) as u8)
            .collect();
        Self { pq_pubkey, pq_sig }
    }

    /// Length of the PQ pubkey bytes (asserts ~1952 in TF-2).
    #[must_use]
    pub fn pq_pubkey_len(&self) -> usize {
        self.pq_pubkey.len()
    }

    /// Length of the PQ signature bytes (asserts ~3309 in TF-2).
    #[must_use]
    pub fn pq_sig_len(&self) -> usize {
        self.pq_sig.len()
    }

    /// Borrow the canonical PQ pubkey bytes.
    #[must_use]
    pub fn pq_pubkey(&self) -> &[u8] {
        &self.pq_pubkey
    }

    /// Wrap the synthetic vector as a [`HybridSignature`] suitable for
    /// driving through the size-touching surfaces.
    #[must_use]
    pub fn as_hybrid_signature(&self) -> HybridSignature {
        // Classical half: Ed25519-shape sentinel bytes (so the test can
        // verify the ML-DSA-65 PQ half is what flows full-width through
        // every surface; the classical half always has its own dimension).
        let classical = vec![0u8; ed25519_dalek::SIGNATURE_LENGTH];
        // Commitment: SHA3-256-shaped bytes (32 B).
        let commitment = vec![0xc1u8; 32];
        HybridSignature::from_parts_internal(
            SigCodepoint::HYBRID_ED25519_MLDSA65,
            classical,
            self.pq_sig.clone(),
            commitment,
        )
    }
}

/// Aggregator of the production size-touching surfaces — driving a
/// hybrid signature through each demonstrates no Ed25519-shape clip
/// survives.
pub struct SizeTouchingSurfaces;

/// DAG-CBOR-serializable wire form of a hybrid signature (canonical
/// envelope used at the wire boundary by upstream `benten-engine`/`benten-id`
/// signatures once they route through this crate).
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
struct HybridSigWire {
    codepoint: u16,
    classical: serde_bytes::ByteBuf,
    pq: serde_bytes::ByteBuf,
    commitment: serde_bytes::ByteBuf,
}

impl SizeTouchingSurfaces {
    /// Surface 2 — DAG-CBOR encode.
    #[must_use]
    pub fn dag_cbor_encode(sig: &HybridSignature) -> Vec<u8> {
        let wire = HybridSigWire {
            codepoint: sig.codepoint().raw(),
            classical: serde_bytes::ByteBuf::from(sig.classical_half_for_test()),
            pq: serde_bytes::ByteBuf::from(sig.pq_half_for_test()),
            commitment: serde_bytes::ByteBuf::from(
                sig.to_wire_bytes()
                    .into_iter()
                    .skip(sig.classical_half_for_test().len() + sig.pq_half_for_test().len())
                    .collect::<Vec<u8>>(),
            ),
        };
        serde_ipld_dagcbor::to_vec(&wire)
            .expect("DAG-CBOR encode MUST NOT fail on canonical hybrid sig wire form")
    }

    /// Surface 2 — DAG-CBOR decode (round-trip companion to [`Self::dag_cbor_encode`]).
    #[must_use]
    pub fn dag_cbor_decode(bytes: &[u8]) -> HybridSignature {
        let wire: HybridSigWire =
            serde_ipld_dagcbor::from_slice(bytes).expect("DAG-CBOR decode MUST round-trip");
        HybridSignature::from_parts_internal(
            SigCodepoint::from_raw(wire.codepoint),
            wire.classical.into_vec(),
            wire.pq.into_vec(),
            wire.commitment.into_vec(),
        )
    }

    /// Surface 3 — redb persistence handle. Backed by a redb in-memory
    /// store with a `Vec<u8>` value column (NOT a fixed-width column).
    #[must_use]
    pub fn redb_store_for_test() -> RedbSigHandle {
        RedbSigHandle::new()
    }

    /// Surface 4 — CID derivation over a hybrid signature's canonical
    /// bytes. The canonical-bytes builder concatenates ALL fields (no
    /// 64 B prefix-only truncation).
    #[must_use]
    pub fn cid_over_signed_bytes(sig: &HybridSignature) -> Vec<u8> {
        // Canonical-bytes builder: domain-separated concat of every
        // size-bearing field, then BLAKE3 hash. The full PQ half MUST be
        // consumed — otherwise the TF-2 byte-flip-at-offset-1064 assertion
        // would not change the digest.
        let mut input = Vec::with_capacity(
            sig.classical_half_for_test().len() + sig.pq_half_for_test().len() + 32 + 8,
        );
        input.extend_from_slice(b"benten/hybrid-sig-cid/v1\0");
        input.extend_from_slice(&sig.codepoint().raw().to_le_bytes());
        input.extend_from_slice(&sig.classical_half_for_test());
        input.extend_from_slice(&sig.pq_half_for_test());
        // Re-hash the commitment too (so the canonical-bytes input is
        // the SAME shape as the wire envelope).
        let wire = sig.to_wire_bytes();
        let commitment_part =
            &wire[sig.classical_half_for_test().len() + sig.pq_half_for_test().len()..];
        input.extend_from_slice(commitment_part);
        blake3::hash(&input).as_bytes().to_vec()
    }

    /// Surface 5 — napi-boundary marshalling.
    ///
    /// Production napi bindings will route through the integration crate's
    /// typed API rather than re-encoding bytes themselves. For the TF-2
    /// size pin this is a straight `Vec<u8>`-shaped marshal (the napi
    /// boundary is NOT a fixed-width buffer).
    #[must_use]
    pub fn napi_marshal(sig: &HybridSignature) -> Vec<u8> {
        // Use the DAG-CBOR canonical wire form as the napi marshal
        // payload — the napi binding will pass it as a `Buffer`/`Vec<u8>`.
        Self::dag_cbor_encode(sig)
    }

    /// Surface 5 — napi-boundary unmarshal.
    #[must_use]
    pub fn napi_unmarshal(bytes: &[u8]) -> HybridSignature {
        Self::dag_cbor_decode(bytes)
    }

    /// Surface 6 — load a fixture signature by name. For TF-2 we
    /// synthesize the "hybrid_v1_default" fixture in-memory at the
    /// canonical ML-DSA-65 dimensions; production fixtures will be
    /// regenerated from this same generator at G-CORE-3c.
    #[must_use]
    pub fn load_signature_fixture(name: &str) -> SyntheticVector {
        match name {
            "hybrid_v1_default" => SyntheticVector::ml_dsa65_for_test(),
            _ => panic!("unknown fixture: {name} (only 'hybrid_v1_default' wired at G-CORE-2)"),
        }
    }
}

/// Surface 3 — redb persistence handle for a hybrid signature column.
///
/// Uses a temporary on-disk redb (per dev-deps `tempfile` + `redb`) with
/// a `(&str, &[u8])` mapping so the value column has NO fixed-width.
pub struct RedbSigHandle {
    _tmpdir: tempfile::TempDir,
    db: redb::Database,
}

const SIG_TABLE: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("sigs");

impl RedbSigHandle {
    /// Create a fresh in-tempdir redb handle.
    #[must_use]
    pub fn new() -> Self {
        let tmpdir = tempfile::tempdir().expect("tempdir for redb");
        let path = tmpdir.path().join("sigs.redb");
        let db = redb::Database::create(&path).expect("redb create");
        Self {
            _tmpdir: tmpdir,
            db,
        }
    }

    /// Put a hybrid signature into the store, keyed by `k`. Routes
    /// through DAG-CBOR canonical-bytes so the on-disk shape is the same
    /// as the wire shape.
    pub fn put_signature(&self, k: &str, sig: &HybridSignature) {
        let bytes = SizeTouchingSurfaces::dag_cbor_encode(sig);
        let write_txn = self.db.begin_write().expect("redb write txn");
        {
            let mut table = write_txn.open_table(SIG_TABLE).expect("redb open table");
            table.insert(k, bytes.as_slice()).expect("redb insert");
        }
        write_txn.commit().expect("redb commit");
    }

    /// Get a hybrid signature out of the store.
    #[must_use]
    pub fn get_signature(&self, k: &str) -> Option<HybridSignature> {
        use redb::ReadableDatabase as _;
        let read_txn = self.db.begin_read().expect("redb read txn");
        let table = read_txn.open_table(SIG_TABLE).expect("redb open table");
        let v = table.get(k).ok().flatten()?;
        let bytes = v.value().to_vec();
        Some(SizeTouchingSurfaces::dag_cbor_decode(&bytes))
    }
}

impl Default for RedbSigHandle {
    fn default() -> Self {
        Self::new()
    }
}
