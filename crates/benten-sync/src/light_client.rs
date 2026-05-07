//! Light-client verification API per ROADMAP-2 + ds-r4r2-3 mode-(a)
//! commitment.
//!
//! ## What this module provides
//!
//! [`LightClient`] — a stateless verifier that consumes a
//! [`crate::mst::MerkleProof`] + a published root [`MstCid`] +
//! verifies inclusion WITHOUT downloading the full subgraph. The
//! distinguishing characteristic per ROADMAP-2 is that
//! verification works against an externally-published root + a
//! proof; the light-client is a pure verifier and does NOT
//! participate in MST diff exchange.
//!
//! [`BandwidthBudget`] — a per-verification bandwidth bound. The
//! light-client tracks bytes consumed against this budget so the
//! ROADMAP-2 commitment ("light-client verifies subgraph membership
//! using <<full subgraph bytes") is observable in test assertions.
//!
//! ## ds-r4r2-3 mode commitment
//!
//! Phase-3 commits to mode-(a) "single-CID inclusion proof" only.
//! Mode-(b) range-query proof + mode-(c) signed-checkpoint are OOS
//! for Phase-3; architectural-absence pins live at
//! `tests/light_client_distinct.rs`. Phase-4+ light-client extensions
//! per FULL-ROADMAP.md re-open mode-(b) + mode-(c).
//!
//! ## Distinct from MST diff per ROADMAP-2
//!
//! MST diff ([`crate::mst::run_mst_diff_to_convergence`]) is a
//! two-peer protocol that synchronises subgraph state by exchanging
//! divergent entries until both peers' roots match. Light-client
//! verification is a single-peer surface: a thin client holding a
//! published root verifies a single Node's inclusion via a Merkle
//! proof from a full peer. The two surfaces share no code path
//! except for the [`crate::mst::MerkleProof`] reconstruction
//! primitive — by design, per ROADMAP-2.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `mst_light_client_verification_against_content_addressed_root`.
//! - r2-test-landscape §2.4 G16-C row
//!   `light_client_verifies_node_cid_inclusion_in_subgraph_root_via_merkle_proof`
//!   (renamed per ds-r4r2-3 — closes ds-r4-6 mode-(a/b/c) ambiguity).
//! - plan §3 G16-C row.
//! - `ROADMAP-2` (light-client distinct deliverable).
//! - CLAUDE.md baked-in #17 (full-peer / thin-client deployment shape
//!   distinction; light-client serves the thin-client view-into-peer
//!   commitment).

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::mst::{MerkleProof, MstCid, MstError};

/// Bandwidth budget for light-client verification operations.
///
/// The thin-client surface has a hard ceiling on bytes consumed per
/// verification — exceeding the budget means the verification path
/// has degraded to "fetch full subgraph", which violates the
/// ROADMAP-2 commitment. The budget is checked at proof-ingestion
/// time + tracked across the verification call.
///
/// Default budget for the canonical fixture is 64 KiB
/// ([`BandwidthBudget::default`]) — sufficient for proofs over
/// MSTs up to ~1024 entries with reasonable key lengths, well below
/// the 4 MiB iroh recv cap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BandwidthBudget {
    /// Maximum bytes the verifier may consume per verification.
    /// Exceeded → [`LightClientError::BandwidthBudgetExceeded`].
    limit_bytes: usize,
}

impl BandwidthBudget {
    /// Construct a budget with the given byte limit.
    #[must_use]
    pub fn limit_bytes(limit_bytes: usize) -> Self {
        Self { limit_bytes }
    }
}

impl Default for BandwidthBudget {
    fn default() -> Self {
        // 64 KiB — covers proofs for MSTs up to ~1024 entries with
        // reasonable key lengths. Pairs with the ROADMAP-2 assertion
        // at `tests/light_client_distinct.rs` that the light-client
        // verifies WITHOUT full subgraph download.
        Self::limit_bytes(64 * 1024)
    }
}

/// Light-client verification errors.
///
/// Distinct from [`crate::mst::MstError`] (application-layer MST
/// errors) and [`crate::errors::AtriumTransportError`] (transport
/// layer): light-client errors arise specifically at the verification
/// boundary + the bandwidth-budget enforcement layer.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum LightClientError {
    /// Underlying MST error (proof reconstruction / key-absent /
    /// rehash mismatch).
    #[error("MST error during light-client verification: {0}")]
    Mst(#[from] MstError),

    /// The verification consumed more bytes than the budget allowed.
    /// Signals the verification path has degraded to "fetch full
    /// subgraph"; per ROADMAP-2 the light-client commitment is that
    /// verification stays within the budget.
    #[error("light-client bandwidth budget exceeded: consumed {consumed} bytes, budget {budget}")]
    BandwidthBudgetExceeded {
        /// Bytes actually consumed by the proof.
        consumed: usize,
        /// Configured budget cap.
        budget: usize,
    },

    /// The proof claims to prove a key under a root, but the proof's
    /// `target_key` does not match the verifier-provided `path`.
    /// Defends against a proof-substitution attack where a peer
    /// returns a valid proof for a DIFFERENT key.
    #[error("proof key mismatch: proof targets {proof_key} but caller asked for {requested_key}")]
    ProofKeyMismatch {
        /// Key the proof itself claims to prove.
        proof_key: String,
        /// Key the caller requested verification for.
        requested_key: String,
    },
}

/// Light-client verifier per ROADMAP-2 + ds-r4r2-3 mode-(a).
///
/// Stateless: the verifier holds only the configured
/// [`BandwidthBudget`] + a per-instance bytes-consumed counter for
/// observability (`bytes_consumed()` is the load-bearing accessor
/// for the ROADMAP-2 assertion).
///
/// The verification operation is pure — given a published root + a
/// Merkle proof + a key, returns Ok(VerificationResult) or
/// Err(LightClientError). No state mutates outside the
/// bytes-consumed counter.
#[derive(Debug)]
pub struct LightClient {
    /// Configured bandwidth budget per verification operation.
    budget: BandwidthBudget,
    /// Total bytes consumed across all verifications by this
    /// instance. Used by the ROADMAP-2 assertion in
    /// `tests/light_client_distinct.rs`.
    bytes_consumed: AtomicUsize,
}

impl LightClient {
    /// Construct a light-client with the default 64 KiB budget.
    #[must_use]
    pub fn new() -> Self {
        Self::with_budget(BandwidthBudget::default())
    }

    /// Construct a light-client with an explicit budget.
    #[must_use]
    pub fn with_budget(budget: BandwidthBudget) -> Self {
        Self {
            budget,
            bytes_consumed: AtomicUsize::new(0),
        }
    }

    /// Verify a Merkle proof against a published root for a given
    /// key. Per ds-r4r2-3 mode-(a) commitment.
    ///
    /// On success: returns a [`VerificationResult`] confirming the
    /// proof verified within budget against the published root for
    /// the requested key.
    ///
    /// On failure:
    /// - [`LightClientError::Mst`] (`ProofRootMismatch` /
    ///   `ProofKeyAbsent`) if the proof does not reconstruct to the
    ///   published root or the proof's key is absent;
    /// - [`LightClientError::BandwidthBudgetExceeded`] if the proof
    ///   bytes exceed the configured budget;
    /// - [`LightClientError::ProofKeyMismatch`] if the proof targets
    ///   a different key than the caller requested.
    ///
    /// # Errors
    ///
    /// See [`LightClientError`] variants above.
    pub fn verify(
        &self,
        published_root: &MstCid,
        path: &str,
        proof: &MerkleProof,
    ) -> Result<VerificationResult, LightClientError> {
        // (1) bandwidth-budget check at ingest time. The proof
        // bytes are the dominant bandwidth consumer.
        let proof_bytes = proof.approximate_bytes();
        if proof_bytes > self.budget.limit_bytes {
            return Err(LightClientError::BandwidthBudgetExceeded {
                consumed: proof_bytes,
                budget: self.budget.limit_bytes,
            });
        }
        self.bytes_consumed.fetch_add(proof_bytes, Ordering::SeqCst);

        // (2) proof-key-vs-requested-key check (defends against
        // proof-substitution).
        if proof.target_key != path {
            return Err(LightClientError::ProofKeyMismatch {
                proof_key: proof.target_key.clone(),
                requested_key: path.to_string(),
            });
        }

        // (3) the proof's claimed target key MUST appear in the
        // sorted (key, cid) pairs at the declared CID. Otherwise
        // the proof is internally inconsistent.
        let pair_match = proof
            .sorted_pairs
            .iter()
            .find(|(k, _)| k == &proof.target_key);
        match pair_match {
            None => {
                return Err(LightClientError::Mst(MstError::ProofKeyAbsent {
                    key: path.to_string(),
                }));
            }
            Some((_, cid)) if *cid != proof.target_cid => {
                // Internally inconsistent proof: target_cid does not
                // match the pair entry under target_key.
                return Err(LightClientError::Mst(MstError::ProofKeyAbsent {
                    key: path.to_string(),
                }));
            }
            _ => {}
        }

        // (4) reconstruct the root from the proof + compare to the
        // published root. This is the load-bearing tampering check:
        // any modification to the proof (target CID, sibling
        // (key, cid) pair, etc.) shifts the reconstructed root.
        let reconstructed = proof.reconstruct_root();
        if reconstructed != *published_root {
            return Err(LightClientError::Mst(MstError::ProofRootMismatch {
                expected: *published_root,
                got: reconstructed,
            }));
        }

        Ok(VerificationResult {
            verified: true,
            verified_key: path.to_string(),
            verified_cid: proof.target_cid,
            bytes_consumed: proof_bytes,
        })
    }

    /// Cumulative bytes consumed across all verifications by this
    /// instance. Used by the ROADMAP-2 assertion that the
    /// light-client stays within the bandwidth budget.
    #[must_use]
    pub fn bytes_consumed(&self) -> usize {
        self.bytes_consumed.load(Ordering::SeqCst)
    }
}

impl Default for LightClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Successful verification outcome — the proof verified within
/// budget against the published root for the requested key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationResult {
    /// `true` on a successful verification. Always `true` for an
    /// `Ok` value; convenience flag for callers who want a single
    /// boolean accessor for the success case.
    pub verified: bool,
    /// The key whose inclusion was verified.
    pub verified_key: String,
    /// The CID of the verified entry.
    pub verified_cid: MstCid,
    /// Bytes consumed by this single verification.
    pub bytes_consumed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mst::{Mst, MstEntry};

    fn build_mst(n: usize) -> Mst {
        let mut mst = Mst::new();
        for i in 0..n {
            let key = format!("/zone/posts/p{i:04}");
            let payload = format!("post-content-{i}").into_bytes();
            mst.insert(MstEntry::from_payload(key, payload));
        }
        mst
    }

    #[test]
    fn verify_present_key_against_published_root_succeeds() {
        let mst = build_mst(64);
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/posts/p0008").unwrap();
        let lc = LightClient::new();
        let result = lc.verify(&root, "/zone/posts/p0008", &proof).unwrap();
        assert!(result.verified);
        assert_eq!(result.verified_key, "/zone/posts/p0008");
        assert!(result.bytes_consumed > 0);
        assert!(lc.bytes_consumed() > 0);
    }

    #[test]
    fn verify_against_different_root_rejects() {
        let mst = build_mst(16);
        let proof = mst.merkle_proof_for("/zone/posts/p0001").unwrap();
        // Use a fabricated different root.
        let different_root = MstCid::from_bytes(b"different-root");
        let lc = LightClient::new();
        let result = lc.verify(&different_root, "/zone/posts/p0001", &proof);
        assert!(matches!(
            result,
            Err(LightClientError::Mst(MstError::ProofRootMismatch { .. }))
        ));
    }

    #[test]
    fn verify_with_tampered_proof_rejects() {
        let mst = build_mst(16);
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/posts/p0001").unwrap();
        let tampered = proof.with_tampered_node();
        let lc = LightClient::new();
        let result = lc.verify(&root, "/zone/posts/p0001", &tampered);
        assert!(matches!(
            result,
            Err(LightClientError::Mst(MstError::ProofRootMismatch { .. }))
        ));
    }

    #[test]
    fn verify_with_mismatched_path_rejects_proof_substitution() {
        let mst = build_mst(16);
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/posts/p0001").unwrap();
        let lc = LightClient::new();
        // Caller asks to verify p0002 but supplied a proof for p0001.
        let result = lc.verify(&root, "/zone/posts/p0002", &proof);
        assert!(matches!(
            result,
            Err(LightClientError::ProofKeyMismatch { .. })
        ));
    }

    #[test]
    fn budget_exceeded_rejects_typed() {
        let mst = build_mst(64);
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/posts/p0001").unwrap();
        // Tiny budget that the proof certainly exceeds.
        let lc = LightClient::with_budget(BandwidthBudget::limit_bytes(8));
        let result = lc.verify(&root, "/zone/posts/p0001", &proof);
        assert!(matches!(
            result,
            Err(LightClientError::BandwidthBudgetExceeded { .. })
        ));
    }

    #[test]
    fn bytes_consumed_tracks_across_verifications() {
        let mst = build_mst(8);
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/posts/p0001").unwrap();
        let lc = LightClient::new();
        let _ = lc.verify(&root, "/zone/posts/p0001", &proof).unwrap();
        let after_first = lc.bytes_consumed();
        let _ = lc.verify(&root, "/zone/posts/p0001", &proof).unwrap();
        let after_second = lc.bytes_consumed();
        assert!(after_second > after_first);
    }

    #[test]
    fn bandwidth_stays_below_full_subgraph_per_roadmap_2() {
        // ROADMAP-2 commitment unit-test (the end-to-end pin lives at
        // tests/light_client_distinct.rs).
        // Build an MST with non-trivial payloads; verify proof bytes
        // are bounded by a budget << total payload bytes.
        let mut mst = Mst::new();
        let total_payload_bytes = 1024 * 1024; // 1 MiB total
        let n_entries = 16;
        let payload_per_entry = total_payload_bytes / n_entries;
        for i in 0..n_entries {
            let key = format!("/zone/items/i{i:04}");
            let payload = vec![0u8; payload_per_entry];
            mst.insert(MstEntry::from_payload(key, payload));
        }
        let root = mst.root_cid();
        let proof = mst.merkle_proof_for("/zone/items/i0008").unwrap();

        let budget = BandwidthBudget::limit_bytes(64 * 1024); // 64 KiB cap
        let lc = LightClient::with_budget(budget);
        let result = lc.verify(&root, "/zone/items/i0008", &proof).unwrap();

        assert!(result.verified);
        // Distinguishing assertion: bytes consumed << full subgraph.
        assert!(
            lc.bytes_consumed() < total_payload_bytes / 16,
            "light-client consumed {} bytes for 1 MiB subgraph; expected <<<6.4 KiB-ish",
            lc.bytes_consumed()
        );
    }
}
