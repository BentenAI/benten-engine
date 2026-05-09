//! GREEN-PHASE pins: `cap_snapshot_hash` 4-input dimension closure
//! (G14-D wave-5a + G16-B canary post-wave; r4b-cap-2 BLOCKER closure).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-1 + R4b r4b-cap-2):
//!
//! - `cap_snapshot_hash_changes_when_grant_added` — input dim 1 (chain)
//! - `cap_snapshot_hash_changes_when_revocation_arrives` — input dim 2 (revocation set)
//! - `cap_snapshot_hash_stable_under_subscriber_churn` — negative control
//! - `cap_snapshot_hash_changes_when_policy_backend_swapped` — input dim 3 (policy tag)
//!
//! ## Architectural intent (cap-r4-1 + r4b-cap-2 BLOCKER closure)
//!
//! Plan §1 + §3 G14-D row + plan §5 D-PHASE-3-5 extension prose all
//! state cap_snapshot_hash inputs are:
//!
//! 1. Durable grant-store CID-set
//! 2. Revocation-set CID-set
//! 3. Policy-backend identity tag
//! NOT subscriber list (negative control)
//!
//! Pre-G16-B-canary the function signature carried only dimension 1.
//! G16-B canary extends `cap_snapshot_hash::compute` to take all 3
//! dimensions (BLOCKER r4b-cap-2 structural-surface closure); these
//! pins exercise the new 4-input API directly at the pure-function
//! layer.
//!
//! These tests drive [`benten_engine::cap_snapshot_hash::compute`]
//! directly — the pure-function entry point. Production callers (e.g.
//! `Engine::put_cap_snapshot_for_envelope`,
//! `engine_wait::resume_from_bytes_inner`) currently route through
//! [`compute_legacy`] (2-input back-compat shape) pending engine-side
//! capture-of-revocation-set + policy-backend-tag at suspend time
//! (downstream wave per phase-3-backlog). The pure-function layer is
//! the load-bearing seam — the engine wires through it once the
//! suspend-side capture sites exist.
//!
//! Per pim-2 §3.6b the assertions OBSERVABLY would fail if any of the
//! three input dimensions were elided from the hash algorithm.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use benten_core::Cid;
use benten_engine::cap_snapshot_hash::{PolicyBackendTag, compute};

fn cid_for(seed: &[u8]) -> Cid {
    Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
}

fn empty_revocations() -> BTreeSet<Cid> {
    BTreeSet::new()
}

#[test]
fn cap_snapshot_hash_changes_when_grant_added() {
    // r4b-cap-2 input dimension 1: a new proof in the grant chain
    // changes the hash. The pure-function layer asserts the dimension
    // observably affects the digest; engine integration wires the
    // chain accessor in the post-canary wave.
    let actor = cid_for(b"actor:alice");
    let chain_before = vec![cid_for(b"ucan:1")];
    let chain_after = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
    let revs = empty_revocations();
    let tag = PolicyBackendTag::no_auth();

    let h_before = compute(&actor, &chain_before, &revs, &tag);
    let h_after = compute(&actor, &chain_after, &revs, &tag);

    assert_ne!(
        h_before, h_after,
        "cap_snapshot_hash MUST change when a grant is added per cap-r4-1"
    );
}

#[test]
fn cap_snapshot_hash_changes_when_revocation_arrives() {
    // r4b-cap-2 input dimension 2: revoking an existing proof changes
    // the cap_snapshot_hash. ORTHOGONAL to grant-added because
    // revocation-set is a SEPARATE input dimension — a hash that
    // omitted revocation would accept resume against revoked proofs.
    let actor = cid_for(b"actor:alice");
    let chain = vec![cid_for(b"ucan:1"), cid_for(b"ucan:2")];
    let revs_empty = empty_revocations();
    let revs_one: BTreeSet<Cid> = [cid_for(b"ucan:1-revoked")].into_iter().collect();
    let tag = PolicyBackendTag::no_auth();

    let h_before = compute(&actor, &chain, &revs_empty, &tag);
    let h_after = compute(&actor, &chain, &revs_one, &tag);

    assert_ne!(
        h_before, h_after,
        "cap_snapshot_hash MUST change when revocation set grows per cap-r4-1"
    );
}

#[test]
fn cap_snapshot_hash_stable_under_subscriber_churn() {
    // r4b-cap-2 negative control: subscriber list is NOT an input.
    // The pure-function `compute` takes (actor, chain, revs, tag) — no
    // subscriber dimension. Calling it twice with identical inputs
    // (between which a hypothetical attach/detach occurred at the
    // engine layer) yields the same hash. Defends against the
    // false-positive resume-rejection storm shape.
    let actor = cid_for(b"actor:alice");
    let chain = vec![cid_for(b"ucan:1")];
    let revs = empty_revocations();
    let tag = PolicyBackendTag::no_auth();

    let h_first = compute(&actor, &chain, &revs, &tag);
    // ... hypothetical engine.subscribe + engine.unsubscribe between
    //     these two calls would not affect the pure-function layer ...
    let h_second = compute(&actor, &chain, &revs, &tag);

    assert_eq!(
        h_first, h_second,
        "cap_snapshot_hash MUST be stable across pure-function repeats \
         (subscriber churn cannot enter the hash algorithm — there is no \
         subscriber input)",
    );
}

#[test]
fn cap_snapshot_hash_changes_when_policy_backend_swapped() {
    // r4b-cap-2 input dimension 3: NoAuthBackend → UCANBackend → custom
    // produce DISTINCT hashes for the same (actor, chain, revs) triple.
    // Defends against the policy-substitution attack class.
    let actor = cid_for(b"actor:alice");
    let chain = vec![cid_for(b"ucan:1")];
    let revs = empty_revocations();

    let tag_no_auth = PolicyBackendTag::no_auth();
    let tag_ucan = PolicyBackendTag::new("UCANBackend");
    let tag_rate_limit = PolicyBackendTag::new("RateLimitPolicy");

    let h_a = compute(&actor, &chain, &revs, &tag_no_auth);
    let h_b = compute(&actor, &chain, &revs, &tag_ucan);
    let h_c = compute(&actor, &chain, &revs, &tag_rate_limit);

    assert_ne!(
        h_a, h_b,
        "cap_snapshot_hash MUST differ across NoAuthBackend vs UCANBackend per cap-r4-1"
    );
    assert_ne!(
        h_b, h_c,
        "cap_snapshot_hash MUST differ across UCANBackend vs RateLimitPolicy per cap-r4-1"
    );
    assert_ne!(
        h_a, h_c,
        "cap_snapshot_hash MUST differ across NoAuthBackend vs RateLimitPolicy per cap-r4-1"
    );
}
