//! Phase-3 G14-D wave-5a + G16-B (r4b-cap-2 BLOCKER closure):
//! `cap_snapshot_hash` derivation + verification.
//!
//! The cap_snapshot_hash binds a suspended WAIT envelope to a specific
//! capability state at suspend time per CLR-2 + r4b-cap-2. A WAIT-resume
//! recomputes the hash against the live cap state and rejects with
//! [`benten_errors::ErrorCode::CapSnapshotHashMismatch`] when ANY of the
//! four input dimensions has materially changed (e.g. a UCAN was revoked,
//! the policy backend was swapped, the revocation set grew).
//!
//! This module is the single source-of-truth for the derivation
//! algorithm. The [`compute`] function is deliberately a pure function
//! with no I/O so test pins can pin the exact bytes the algorithm
//! produces.
//!
//! ## Closes
//!
//! - **Compromise #10 engine-side asymmetry** (WAIT-suspend / resume):
//!   the hash carried in the suspension envelope's side table is
//!   re-validatable across a process boundary.
//! - **CLR-2 cross-atrium replay** (proof-chain binding): substituting
//!   a different chain that COULD produce the same effective caps but
//!   has different proof-CIDs MUST NOT validate.
//! - **r4b-cap-2 BLOCKER (Phase-3)**: the spec-required revocation-set
//!   + policy-backend identity dimensions land alongside the original
//!   actor + grant-chain dimensions; a substituted policy backend with
//!   a structurally-identical effective cap set still produces a
//!   distinct hash (resume-mismatch defense).
//! - **phase-2-backlog §7.3** persisted-policy-metadata wire-through
//!   (composes with the historical-policy metadata blob captured at
//!   suspend per `historical_policy_metadata`).
//!
//! ## Closure semantics (CLR-2 + r4b-cap-2)
//!
//! The hash is BLAKE3 of the canonical concatenation:
//!
//! ```text
//! domain_separator
//!   || actor_cid_bytes
//!   || u32_be(grant_chain_count) || sorted([grant_chain_cid_bytes...])
//!   || u32_be(revocation_count)  || sorted([revocation_cid_bytes...])
//!   || u32_be(tag_len) || policy_backend_identity_tag_bytes
//! ```
//!
//! Length-prefixing each list defends against ambiguous concatenation
//! (e.g. moving a CID from `grant_chain` to `revocation_set` must change
//! the hash even if the underlying byte sequence is unchanged).
//! Sorting both lists makes the hash order-stable across re-imports.
//! The actor CID + domain separator prevent cross-actor reuse.

use std::collections::BTreeSet;

use benten_core::Cid;

/// Domain separator written at the start of the BLAKE3 input — defends
/// against cross-protocol pre-image attacks where bytes prepared for a
/// different content-addressed scheme could pre-image into a valid
/// `cap_snapshot_hash` for some attacker-chosen actor.
const DOMAIN_SEPARATOR: &[u8] = b"benten:cap_snapshot_hash:v2";

/// Phase-3 r4b-cap-2 closure: opaque tag identifying which
/// [`benten_caps::CapabilityPolicy`] backend produced the snapshot.
///
/// Distinguishes structurally-different policies that happen to produce
/// the same superficial effective-cap set — `NoAuthBackend` (always-allow)
/// vs `UCANBackend` (chain-walked) vs custom rate-limit policies all
/// produce DISTINCT hashes for the same `(actor, grant_chain,
/// revocation_set)` triple.
///
/// Rendering convention: callers stringify their backend's stable
/// identity (e.g. `"NoAuthBackend"`, `"UCANBackend"`,
/// `"RateLimitPolicy"`); the policy crate exposes a stable accessor on
/// each backend that the engine consults at suspend time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolicyBackendTag(String);

impl PolicyBackendTag {
    /// Construct a tag from a backend identity string.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow the tag bytes for hashing. Always UTF-8.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// The default Phase-2a tag used when the engine carries no
    /// configured policy (i.e. NoAuth-equivalent paths). Callers passing
    /// this tag explicitly produce the same hash as
    /// [`compute_legacy`] for backward-compat.
    #[must_use]
    pub fn no_auth() -> Self {
        Self("NoAuthBackend".into())
    }
}

/// Compute the `cap_snapshot_hash` binding `actor_cid` to a UCAN
/// proof-chain CID list, the orthogonal revocation set, and the
/// policy-backend identity tag at suspend time.
///
/// Algorithm (CLR-2 + r4b-cap-2 binding):
/// 1. Domain-separator prefix prevents cross-protocol pre-image attacks.
/// 2. Length-prefix + sort each CID set so reorders / set-membership
///    moves observably change the hash.
/// 3. Length-prefix the policy-backend tag bytes so a tag of length 0
///    is distinct from absence.
///
/// Returns the 32-byte digest. Pure function — no I/O — so test pins
/// can assert exact bytes.
#[must_use]
pub fn compute(
    actor_cid: &Cid,
    grant_chain_cids: &[Cid],
    revocation_set: &BTreeSet<Cid>,
    policy_backend_tag: &PolicyBackendTag,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DOMAIN_SEPARATOR);
    hasher.update(actor_cid.as_bytes());

    // Grant-chain (sorted, length-prefixed).
    let mut chain_sorted: Vec<&Cid> = grant_chain_cids.iter().collect();
    chain_sorted.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    hasher.update(
        &u32::try_from(chain_sorted.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for cid in chain_sorted {
        hasher.update(cid.as_bytes());
    }

    // Revocation set (BTreeSet already sorted; length-prefixed).
    hasher.update(
        &u32::try_from(revocation_set.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for cid in revocation_set {
        hasher.update(cid.as_bytes());
    }

    // Policy-backend identity tag (length-prefixed).
    let tag_bytes = policy_backend_tag.as_bytes();
    hasher.update(
        &u32::try_from(tag_bytes.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    hasher.update(tag_bytes);

    *hasher.finalize().as_bytes()
}

/// Backward-compat helper: compute the hash with the empty revocation
/// set + the [`PolicyBackendTag::no_auth`] tag. Callers that haven't yet
/// surfaced their backend tag / revocation set use this transitional
/// helper. New callers SHOULD pass all four dimensions explicitly.
///
/// This helper preserves the legacy 2-input call shape used by
/// `engine_wait.rs::resume_from_bytes_inner` Step 3.5 + the
/// `Engine::put_cap_snapshot_for_envelope` helper until the engine-side
/// snapshot capture sites surface the full 4-dimension input.
#[must_use]
pub fn compute_legacy(actor_cid: &Cid, proof_chain_cids: &[Cid]) -> [u8; 32] {
    compute(
        actor_cid,
        proof_chain_cids,
        &BTreeSet::new(),
        &PolicyBackendTag::no_auth(),
    )
}

/// Verify that the bound `expected` hash matches a freshly-computed
/// hash over the 4-dimension input. Returns `true` on match.
///
/// The mismatch case is the load-bearing CLR-2 §11 closure: a resume
/// against a chain/revocation/policy state that materially changed
/// between suspend + resume MUST reject. Callers wrap a `false` result
/// into the typed
/// [`benten_errors::ErrorCode::CapSnapshotHashMismatch`] error.
#[must_use]
pub fn verify(
    actor_cid: &Cid,
    grant_chain_cids: &[Cid],
    revocation_set: &BTreeSet<Cid>,
    policy_backend_tag: &PolicyBackendTag,
    expected: &[u8; 32],
) -> bool {
    &compute(
        actor_cid,
        grant_chain_cids,
        revocation_set,
        policy_backend_tag,
    ) == expected
}

/// Backward-compat verify against the 2-input legacy hash shape.
///
/// Mirrors [`compute_legacy`] for the call sites that have not yet
/// surfaced revocation-set + policy-backend-tag inputs.
#[must_use]
pub fn verify_legacy(actor_cid: &Cid, proof_chain_cids: &[Cid], expected: &[u8; 32]) -> bool {
    &compute_legacy(actor_cid, proof_chain_cids) == expected
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn cid_for(seed: &[u8]) -> Cid {
        Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
    }

    fn empty_revocations() -> BTreeSet<Cid> {
        BTreeSet::new()
    }

    #[test]
    fn cap_snapshot_hash_round_trip() {
        let actor = cid_for(b"actor:alice");
        let chain = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        let revs = empty_revocations();
        let tag = PolicyBackendTag::no_auth();
        let h = compute(&actor, &chain, &revs, &tag);
        assert!(verify(&actor, &chain, &revs, &tag, &h));
    }

    #[test]
    fn cap_snapshot_hash_order_stable_across_reorder() {
        // CLR-2 design: chain order doesn't affect the hash (sorted
        // canonicalisation). Two re-imports of the same chain in
        // different orders produce the same hash.
        let actor = cid_for(b"actor:alice");
        let chain_a = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2"), cid_for(b"ucan:3")];
        let chain_b = vec![cid_for(b"ucan:3"), cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        let revs = empty_revocations();
        let tag = PolicyBackendTag::no_auth();
        assert_eq!(
            compute(&actor, &chain_a, &revs, &tag),
            compute(&actor, &chain_b, &revs, &tag),
        );
    }

    #[test]
    fn cap_snapshot_hash_distinguishes_actors_with_same_chain() {
        // CLR-2 design: actor binding prefix prevents cross-actor reuse.
        let chain = vec![cid_for(b"ucan:1")];
        let revs = empty_revocations();
        let tag = PolicyBackendTag::no_auth();
        let h_a = compute(&cid_for(b"actor:alice"), &chain, &revs, &tag);
        let h_b = compute(&cid_for(b"actor:bob"), &chain, &revs, &tag);
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
        let revs = empty_revocations();
        let tag = PolicyBackendTag::no_auth();
        let h_a = compute(&actor, &chain_a, &revs, &tag);
        assert!(!verify(&actor, &chain_b, &revs, &tag, &h_a));
    }

    #[test]
    fn cap_snapshot_hash_rejects_partial_chain_revoke() {
        // CLR-2 §11: revoking one UCAN from the chain (peer's durable
        // store no longer surfaces it) materially changes the chain;
        // the recompute fails.
        let actor = cid_for(b"actor:alice");
        let chain_full = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2"), cid_for(b"ucan:3")];
        let chain_minus_2 = vec![cid_for(b"ucan:1"), cid_for(b"ucan:3")];
        let revs = empty_revocations();
        let tag = PolicyBackendTag::no_auth();
        let h = compute(&actor, &chain_full, &revs, &tag);
        assert!(!verify(&actor, &chain_minus_2, &revs, &tag, &h));
    }

    // r4b-cap-2 closure: revocation_set + policy_backend_tag dimensions.

    #[test]
    fn cap_snapshot_hash_changes_when_revocation_arrives() {
        // r4b-cap-2 input dimension 2: revoking an existing proof
        // changes the hash even when the grant chain CID list is
        // unchanged. ORTHOGONAL to grant-set: the revocation list is a
        // separate hash input.
        let actor = cid_for(b"actor:alice");
        let chain = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
        let revs_empty = empty_revocations();
        let revs_one: BTreeSet<Cid> = [cid_for(b"ucan:revoked")].into_iter().collect();
        let tag = PolicyBackendTag::no_auth();
        let h_empty = compute(&actor, &chain, &revs_empty, &tag);
        let h_one = compute(&actor, &chain, &revs_one, &tag);
        assert_ne!(h_empty, h_one);
    }

    #[test]
    fn cap_snapshot_hash_changes_when_policy_backend_swapped() {
        // r4b-cap-2 input dimension 3: NoAuthBackend → UcanBackend →
        // custom-fingerprint produce DISTINCT hashes for the same
        // (actor, chain, revocation_set) triple.
        let actor = cid_for(b"actor:alice");
        let chain = vec![cid_for(b"ucan:1")];
        let revs = empty_revocations();
        let tag_a = PolicyBackendTag::no_auth();
        let tag_b = PolicyBackendTag::new("UCANBackend");
        let tag_c = PolicyBackendTag::new("RateLimitPolicy");
        let h_a = compute(&actor, &chain, &revs, &tag_a);
        let h_b = compute(&actor, &chain, &revs, &tag_b);
        let h_c = compute(&actor, &chain, &revs, &tag_c);
        assert_ne!(h_a, h_b);
        assert_ne!(h_b, h_c);
        assert_ne!(h_a, h_c);
    }

    #[test]
    fn cap_snapshot_hash_distinguishes_grant_in_chain_vs_revocation_set() {
        // Length-prefix defense: moving a CID from grant_chain to
        // revocation_set must change the hash even though the byte
        // sequence of CIDs is unchanged.
        let actor = cid_for(b"actor:alice");
        let cid = cid_for(b"ucan:moveable");
        let tag = PolicyBackendTag::no_auth();

        let revs_empty = empty_revocations();
        let h_in_chain = compute(&actor, &[cid], &revs_empty, &tag);

        let revs_one: BTreeSet<Cid> = [cid].into_iter().collect();
        let h_in_revoke = compute(&actor, &[], &revs_one, &tag);

        assert_ne!(
            h_in_chain, h_in_revoke,
            "moving a CID from grant_chain to revocation_set must change \
             the hash (length-prefix defense)"
        );
    }

    #[test]
    fn cap_snapshot_hash_legacy_helper_matches_full_compute_with_no_auth_defaults() {
        // compute_legacy = compute(.., empty_revocations, no_auth_tag).
        // Backward-compat invariant for sites that haven't surfaced the
        // full 4-input shape yet.
        let actor = cid_for(b"actor:alice");
        let chain = vec![cid_for(b"ucan:1")];
        let h_legacy = compute_legacy(&actor, &chain);
        let h_full = compute(
            &actor,
            &chain,
            &empty_revocations(),
            &PolicyBackendTag::no_auth(),
        );
        assert_eq!(h_legacy, h_full);
        assert!(verify_legacy(&actor, &chain, &h_legacy));
    }
}
