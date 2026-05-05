//! R4-FP-R3-B RED-PHASE pins: cap_snapshot_hash spec-input enumeration
//! (G14-D wave-5a; cap-r4-1 MAJOR closure of cap-major-3 fix-now-action).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-1):
//!
//! - `tests/cap_snapshot_hash_changes_when_grant_added` — cap-r4-1
//! - `tests/cap_snapshot_hash_changes_when_revocation_arrives` — cap-r4-1
//! - `tests/cap_snapshot_hash_stable_under_subscriber_churn` — cap-r4-1 (negative control)
//! - `tests/cap_snapshot_hash_changes_when_policy_backend_swapped` — cap-r4-1
//!
//! ## Architectural intent (cap-r4-1 MAJOR closure)
//!
//! Plan §1 + §3 G14-D row + plan §5 D-PHASE-3-5 extension prose all
//! state cap_snapshot_hash inputs are:
//!
//! 1. Durable grant-store CID-set
//! 2. Revocation-set CID-set
//! 3. Policy-backend identity tag
//! NOT subscriber list (negative control)
//!
//! The existing R3 pins at wait_resume_cross_process.rs cover the
//! UCAN-proof-CID dimension only. These four pins close the spec:
//!
//! - **grant_added**: installing a new proof changes the hash
//! - **revocation_arrives**: revoking an existing proof changes the
//!   hash (ORTHOGONAL to grant-added, since revocation-set is a
//!   separate input dimension)
//! - **subscriber_churn**: attaching/detaching subscribers does NOT
//!   change the hash (defends against false-positive resume rejections
//!   from subscriber churn; subscriber list is an EXCLUDED input)
//! - **policy_backend_swapped**: NoAuthBackend → UcanBackend → custom
//!   fingerprint produce distinct hashes (defends against the "resume
//!   succeeds against structurally-different policy that happens to
//!   produce the same superficial hash" attack class)
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores AND replaces stub bodies. Per §3.6b pim-2
//! these tests must drive the production `cap_snapshot_hash::compute`
//! path + assert observable hash difference / stability.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — cap-r4-1 — hash changes when grant added"]
fn cap_snapshot_hash_changes_when_grant_added() {
    // cap-r4-1 pin (input dimension 1: durable grant-store CID-set).
    // Installing a new proof into the durable grant store changes the
    // cap_snapshot_hash for any actor whose authority chain
    // structurally references that proof.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let actor_kp = benten_id::keypair::Keypair::generate();
    //   let actor_did = actor_kp.public_key().to_did();
    //
    //   let grant_a = ... .audience(actor_did.clone())
    //                     .capability("/zone/a", "read") ... ;
    //   engine.caps().install_proof(&grant_a).unwrap();
    //   let hash_before = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   // Install an additional grant for the same actor:
    //   let grant_b = ... .audience(actor_did.clone())
    //                     .capability("/zone/b", "read") ... ;
    //   engine.caps().install_proof(&grant_b).unwrap();
    //   let hash_after = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   assert_ne!(hash_before, hash_after,
    //       "cap_snapshot_hash must change when grant added per cap-r4-1");
    //
    // OBSERVABLE consequence: a resume against an envelope bound at
    // hash_before observably rejects after the new grant is installed
    // (because hash_after != hash_before). Defends against the resume
    // pattern picking up unintended grants.
    unimplemented!(
        "G14-D wires cap_snapshot_hash::compute to include grant-store CID-set per cap-r4-1"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — cap-r4-1 — hash changes when revocation arrives"]
fn cap_snapshot_hash_changes_when_revocation_arrives() {
    // cap-r4-1 pin (input dimension 2: revocation-set CID-set).
    // Revoking an existing proof changes the cap_snapshot_hash. This
    // is ORTHOGONAL to grant-added because revocation-set is a
    // separate input dimension — a hash that omitted revocation would
    // accept resume against revoked proofs.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let actor_kp = benten_id::keypair::Keypair::generate();
    //   let actor_did = actor_kp.public_key().to_did();
    //
    //   let grant = ... .audience(actor_did.clone()) ... ;
    //   engine.caps().install_proof(&grant).unwrap();
    //   let hash_before = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   // Revoke the proof:
    //   engine.caps().revoke(&grant.cid()).unwrap();
    //   let hash_after = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   assert_ne!(hash_before, hash_after,
    //       "cap_snapshot_hash must change when revocation arrives per cap-r4-1");
    //
    // OBSERVABLE consequence: a resume against an envelope bound at
    // hash_before rejects after revocation, even though the grant CID
    // itself is unchanged in the durable store. Defends against
    // "revocation invisible to resume" — the hash MUST observe
    // revocation as a separate input dimension.
    unimplemented!(
        "G14-D wires cap_snapshot_hash::compute to include revocation-set CID-set per cap-r4-1"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — cap-r4-1 — hash STABLE under subscriber churn (negative control)"]
fn cap_snapshot_hash_stable_under_subscriber_churn() {
    // cap-r4-1 pin (negative control: subscriber list is NOT an input).
    // Attaching / detaching subscribers MUST NOT change the
    // cap_snapshot_hash. Without this control, every subscribe()
    // invalidates every suspended envelope — false-positive resume
    // rejection storm.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let actor_kp = benten_id::keypair::Keypair::generate();
    //   let actor_did = actor_kp.public_key().to_did();
    //   let grant = ...;
    //   engine.caps().install_proof(&grant).unwrap();
    //
    //   let hash_before = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   // Attach a subscriber on a totally unrelated zone:
    //   let _sub = engine.subscribe("/zone/unrelated", actor_did.clone(), |_| {}).unwrap();
    //   let hash_after_attach = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   // Detach the subscriber:
    //   engine.unsubscribe(_sub).unwrap();
    //   let hash_after_detach = engine.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   assert_eq!(hash_before, hash_after_attach,
    //       "cap_snapshot_hash must NOT change on subscriber attach per cap-r4-1 negative control");
    //   assert_eq!(hash_before, hash_after_detach,
    //       "cap_snapshot_hash must NOT change on subscriber detach per cap-r4-1 negative control");
    //
    // OBSERVABLE consequence: subscribers can churn freely without
    // invalidating suspended envelopes. Defends against false-positive
    // resume-mismatch storms when subscribe/unsubscribe is high-rate.
    unimplemented!(
        "G14-D wires cap_snapshot_hash::compute to EXCLUDE subscriber list per cap-r4-1 negative control"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — cap-r4-1 — hash changes when policy backend swapped"]
fn cap_snapshot_hash_changes_when_policy_backend_swapped() {
    // cap-r4-1 pin (input dimension 3: policy-backend identity tag).
    // Different CapabilityPolicy backends produce DISTINCT
    // cap_snapshot_hashes even when the grant store + revocation set
    // are identical. Defends against the "resume succeeds against
    // structurally-different policy that happens to produce the same
    // superficial effective-cap-set" attack class.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //
    //   let actor_kp = benten_id::keypair::Keypair::generate();
    //   let actor_did = actor_kp.public_key().to_did();
    //
    //   // Engine 1: NoAuthBackend (permissive default):
    //   let engine_a = benten_engine::Engine::builder()
    //       .with_policy(benten_caps::NoAuthBackend::default())
    //       .open(store_dir.path()).unwrap();
    //   let hash_a = engine_a.cap_snapshot_hash_for(&actor_did).unwrap();
    //   drop(engine_a);
    //
    //   // Engine 2: UcanBackend on the same durable store:
    //   let engine_b = benten_engine::Engine::builder()
    //       .with_policy(benten_caps::UCANBackend::open(store_dir.path()).unwrap())
    //       .open(store_dir.path()).unwrap();
    //   let hash_b = engine_b.cap_snapshot_hash_for(&actor_did).unwrap();
    //   drop(engine_b);
    //
    //   // Engine 3: Custom rate-limit-policy fingerprint:
    //   let engine_c = benten_engine::Engine::builder()
    //       .with_policy(benten_caps::rate_limit::RateLimitPolicy::default())
    //       .open(store_dir.path()).unwrap();
    //   let hash_c = engine_c.cap_snapshot_hash_for(&actor_did).unwrap();
    //
    //   assert_ne!(hash_a, hash_b,
    //       "cap_snapshot_hash must differ across NoAuthBackend vs UcanBackend per cap-r4-1");
    //   assert_ne!(hash_b, hash_c,
    //       "cap_snapshot_hash must differ across UcanBackend vs RateLimitPolicy per cap-r4-1");
    //   assert_ne!(hash_a, hash_c,
    //       "cap_snapshot_hash must differ across NoAuthBackend vs RateLimitPolicy per cap-r4-1");
    //
    // OBSERVABLE consequence: an envelope suspended under one policy
    // backend rejects on resume under a different policy backend, even
    // when the durable grant store is bit-identical. Defends against
    // the policy-substitution attack class at the WAIT-resume seam.
    unimplemented!(
        "G14-D wires cap_snapshot_hash::compute to include policy-backend identity tag per cap-r4-1"
    );
}
