//! R3-B RED-PHASE pin: WAIT-resume persisted-policy metadata
//! (G14-D wave-5a; plan §3 G14-D).
//!
//! Pin source: r2-test-landscape §2.2 G14-D row
//! `wait_resume_persisted_policy_metadata_against_historical_state`.
//!
//! ## Architectural intent
//!
//! When a suspended execution resumes, the policy metadata at resume
//! time MUST match the policy that was in effect at suspend time
//! (the historical state). Without this discipline, a policy
//! upgrade between suspend + resume could silently relax or tighten
//! the active checks against a previously-suspended actor.
//!
//! This is a "bind to historical state" pin — orthogonal to the
//! cap_snapshot_hash chain-binding (which catches UCAN-level changes);
//! this pin catches POLICY-level changes (rate-limit budget,
//! pluggable policy upgrade, etc.).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — plan §3 G14-D — persisted policy metadata against historical state"]
fn wait_resume_persisted_policy_metadata_against_historical_state() {
    // plan §3 G14-D pin. G14-D implementer wires this:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //
    //   // Suspend with policy version 1:
    //   let suspension_id = {
    //       let policy_v1 = benten_caps::rate_limit::RateLimitPolicy::builder()
    //           .actor_writes_per_second(actor_did.clone(), "/zone/posts", 10).build();
    //       let engine = benten_engine::Engine::open_with_policy(store_dir.path(), policy_v1).unwrap();
    //       engine.run_with_actor(actor_did.clone(), &subgraph_with_wait).unwrap();
    //       engine.list_suspensions()[0].id().clone()
    //   };
    //
    //   // Re-open with policy version 2 (more permissive):
    //   let policy_v2 = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .actor_writes_per_second(actor_did.clone(), "/zone/posts", 1000).build();
    //   let engine = benten_engine::Engine::open_with_policy(store_dir.path(), policy_v2).unwrap();
    //
    //   // Resume; the suspension's policy metadata is the v1 metadata
    //   // (e.g., budget remaining at suspend time):
    //   let suspension = engine.fetch_suspension(&suspension_id).unwrap();
    //   assert_eq!(suspension.policy_metadata().historical_writes_per_second, 10);
    //
    //   // Per the binding, the resume EITHER:
    //   //   (a) re-runs against historical metadata → behavior matches suspend-time, OR
    //   //   (b) detects the policy version mismatch + rejects via typed error.
    //   // G14-D plan §3 picks variant (a):
    //   engine.resume(&suspension_id).unwrap();
    //   // Post-resume, the actor's persistent rate-limit accounting reflects
    //   // historical (v1) budgets, not v2:
    //   let acct = engine.policy().actor_accounting(&actor_did, "/zone/posts");
    //   assert_eq!(acct.bound_to_metadata_at_suspend(), true);
    //
    // OBSERVABLE consequence: policy metadata bound at suspend
    // observably governs the resume's accounting state. Defends
    // against the "reload changes the rules mid-execution" failure
    // shape.
    unimplemented!("G14-D wires persisted-policy-metadata historical-state binding at WAIT-resume");
}
