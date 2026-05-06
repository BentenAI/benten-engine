//! Phase-3 G14-D wave-5a: `cap_snapshot_hash` derivation + verification.
//!
//! The cap_snapshot_hash binds a suspended WAIT envelope to a specific
//! UCAN proof-chain CID list at suspend time per CLR-2. A WAIT-resume
//! recomputes the hash against the chain currently in the durable cap
//! store and rejects with [`benten_errors::ErrorCode::CapSnapshotHashMismatch`]
//! when the chain materially changed (e.g. a UCAN was revoked).
//!
//! This module is the single source-of-truth for the derivation
//! algorithm. The `compute(actor_cid, proof_chain_cids)` function is
//! deliberately a pure function with no I/O so test pins can pin the
//! exact bytes the algorithm produces.
//!
//! ## Closes
//!
//! - **Compromise #10 engine-side asymmetry** (WAIT-suspend / resume):
//!   the hash carried in the suspension envelope's side table is
//!   re-validatable across a process boundary.
//! - **CLR-2 cross-atrium replay** (proof-chain binding): substituting
//!   a different chain that COULD produce the same effective caps but
//!   has different proof-CIDs MUST NOT validate.
//! - **phase-2-backlog §7.3** persisted-policy-metadata wire-through
//!   (composes with the historical-policy metadata blob captured at
//!   suspend per `historical_policy_metadata`).
//!
//! ## Closure semantics (CLR-2)
//!
//! The hash is BLAKE3 of the canonical concatenation:
//!   `actor_cid_bytes || sorted([proof_chain_cid_bytes...])`
//!
//! Sorting the chain CIDs makes the hash order-stable across re-imports
//! that produced the same chain in a different order. The actor CID
//! prefix prevents cross-actor reuse of the same chain.

use benten_core::Cid;

/// Compute the `cap_snapshot_hash` binding `actor_cid` to a UCAN
/// proof-chain CID list at suspend time.
///
/// Algorithm (CLR-2 binding):
/// 1. Sort `proof_chain_cids` by CID byte order (canonicalisation;
///    re-import order doesn't change the hash).
/// 2. Hash `actor_cid_bytes || cid_0_bytes || cid_1_bytes || ...`
///    with BLAKE3.
///
/// Returns the 32-byte digest. Pure function — no I/O — so test pins
/// can assert exact bytes.
#[must_use]
pub fn compute(actor_cid: &Cid, proof_chain_cids: &[Cid]) -> [u8; 32] {
    let mut sorted: Vec<&Cid> = proof_chain_cids.iter().collect();
    sorted.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut hasher = blake3::Hasher::new();
    hasher.update(actor_cid.as_bytes());
    for cid in sorted {
        hasher.update(cid.as_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Verify that the bound `expected` hash matches a freshly-computed
/// hash over `(actor_cid, proof_chain_cids)`. Returns `true` on match.
///
/// The mismatch case is the load-bearing CLR-2 §11 closure: a resume
/// against a chain that was revoked between suspend + resume MUST
/// reject. Callers wrap a `false` result into the typed
/// [`benten_errors::ErrorCode::CapSnapshotHashMismatch`] error.
#[must_use]
pub fn verify(actor_cid: &Cid, proof_chain_cids: &[Cid], expected: &[u8; 32]) -> bool {
    &compute(actor_cid, proof_chain_cids) == expected
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn cid_for(seed: &[u8]) -> Cid {
        Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
    }

    #[test]
    fn cap_snapshot_hash_round_trip() {
        let actor = cid_for(b"actor:alice");
        let chain = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        let h = compute(&actor, &chain);
        assert!(verify(&actor, &chain, &h));
    }

    #[test]
    fn cap_snapshot_hash_order_stable_across_reorder() {
        // CLR-2 design: chain order doesn't affect the hash (sorted
        // canonicalisation). Two re-imports of the same chain in
        // different orders produce the same hash.
        let actor = cid_for(b"actor:alice");
        let chain_a = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2"), cid_for(b"ucan:3")];
        let chain_b = vec![cid_for(b"ucan:3"), cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        assert_eq!(compute(&actor, &chain_a), compute(&actor, &chain_b));
    }

    #[test]
    fn cap_snapshot_hash_distinguishes_actors_with_same_chain() {
        // CLR-2 design: actor binding prefix prevents cross-actor reuse.
        let chain = vec![cid_for(b"ucan:1")];
        let h_a = compute(&cid_for(b"actor:alice"), &chain);
        let h_b = compute(&cid_for(b"actor:bob"), &chain);
        assert_ne!(h_a, h_b);
    }

    #[test]
    fn cap_snapshot_hash_rejects_substituted_chain() {
        // CLR-2 §11 load-bearing pin: a different chain that COULD
        // produce equivalent effective caps but has different
        // proof-CIDs MUST NOT validate (substitution-attack defense).
        let actor = cid_for(b"actor:alice");
        let chain_a = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        let chain_b = vec![cid_for(b"ucan:99")];
        let h_a = compute(&actor, &chain_a);
        assert!(!verify(&actor, &chain_b, &h_a));
    }

    #[test]
    fn cap_snapshot_hash_rejects_partial_chain_revoke() {
        // CLR-2 §11: revoking one UCAN from the chain (peer's durable
        // store no longer surfaces it) materially changes the chain;
        // the recompute fails.
        let actor = cid_for(b"actor:alice");
        let chain_full = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2"), cid_for(b"ucan:3")];
        let chain_minus_2 = vec![cid_for(b"ucan:1"), cid_for(b"ucan:3")];
        let h = compute(&actor, &chain_full);
        assert!(!verify(&actor, &chain_minus_2, &h));
    }
}
