//! R3-B RED-PHASE pins: WAIT-resume cap_snapshot_hash cross-process
//! round-trip (G14-D wave-5a; phase-2-backlog §7.3 + Compromise #10 +
//! CLR-2).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-D +
//! §3.A CLR-2 + §11):
//!
//! - `tests/wait_resume_cap_snapshot_hash_cross_process_round_trip` — phase-2-backlog §7.3 + Compromise #10 (cross-process)
//! - `tests/wait_resume_cap_snapshot_hash_binds_ucan_proof_chain_at_envelope` — CLR-2 (security)
//! - `tests/cap_snapshot_hash_binds_ucan_proof_chain_at_wait_resume` — CLR-2 (security; redundant-distinct shape per CLR-2 §11)
//!
//! ## Architectural intent
//!
//! Phase-2 named Compromise #10 (engine-side asymmetry between WAIT-
//! suspend and WAIT-resume policy metadata). Phase-3 G14-D closes
//! this: the cap_snapshot_hash carried in the suspension envelope
//! BINDS to the UCAN proof chain at suspend; resume re-validates the
//! chain end-to-end against the bound hash. CLR-2 cross-lens cluster
//! pin: replay-resistance at the WAIT-resume seam.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the cross-process test
//! must drive an actual second-process resume (spawned subprocess or
//! re-opened engine instance), not a same-process simulation.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — phase-2-backlog §7.3 + Compromise #10 — WAIT cross-process round-trip"]
fn wait_resume_cap_snapshot_hash_cross_process_round_trip() {
    // phase-2-backlog §7.3 + Compromise #10 pin. G14-D implementer
    // wires this:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //
    //   let suspension_id = {
    //       let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //       let actor_kp = benten_id::keypair::Keypair::generate();
    //       let grant = ... .audience(actor_kp.public_key().to_did()) ... ;
    //       engine.caps().install_proof(&grant).unwrap();
    //
    //       // CALL → WAIT subgraph; suspend mid-execution:
    //       engine.run_with_actor(actor_kp.public_key().to_did(), &subgraph_with_wait).unwrap();
    //       // The suspension envelope is durable + carries cap_snapshot_hash:
    //       let suspended = engine.list_suspensions();
    //       suspended[0].id().clone()
    //   };
    //   // First engine drops; durable-store flush.
    //
    //   // Re-open in a second "process" (same-test isolated open):
    //   let engine2 = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   // Resume; cap_snapshot_hash re-validates against current grant store:
    //   engine2.resume(&suspension_id).unwrap();
    //
    //   // The resume completed end-to-end without re-running pre-suspend work.
    //   let probe = engine2.read_zone("/zone/probe").unwrap();
    //   assert!(probe.iter().any(|n| n.label() == "post-resume-side-effect"));
    //
    // OBSERVABLE consequence: cross-process resume is the load-
    // bearing Compromise #10 closure pin — the suspension envelope
    // is portable across engine restarts (proves cap_snapshot_hash
    // is durable + re-validatable, not in-memory only).
    unimplemented!(
        "G14-D wires WAIT-resume cap_snapshot_hash cross-process round-trip per Compromise #10"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — CLR-2 — cap_snapshot_hash binds UCAN proof chain at envelope"]
fn wait_resume_cap_snapshot_hash_binds_ucan_proof_chain_at_envelope() {
    // CLR-2 cluster pin. The cap_snapshot_hash MUST be derived from
    // the UCAN proof chain (specifically: the hashes of every UCAN
    // CID in the chain at suspend time + the chain-walk policy
    // identity). Without this binding, a resume could be replayed
    // against a different chain that hashes to the same snapshot.
    //
    // Implementer wires:
    //
    //   // Suspend with a specific UCAN chain bound:
    //   let chain_a = vec![ucan_a1, ucan_a2];
    //   let envelope = engine.suspend_for_test(&actor_did, &chain_a, ...).unwrap();
    //
    //   // The cap_snapshot_hash names the chain:
    //   let recomputed = benten_engine::cap_snapshot_hash::compute(&actor_did, &chain_a);
    //   assert_eq!(envelope.cap_snapshot_hash(), recomputed);
    //
    //   // Substitute a different chain that COULD have produced the
    //   // same effective caps but has different proof-CIDs:
    //   let chain_b = vec![ucan_b1]; // different CIDs, equivalent caps
    //   let recomputed_b = benten_engine::cap_snapshot_hash::compute(&actor_did, &chain_b);
    //   assert_ne!(recomputed_b, envelope.cap_snapshot_hash(),
    //       "cap_snapshot_hash MUST bind specific UCAN-proof-CIDs per CLR-2");
    //
    // OBSERVABLE consequence: cap_snapshot_hash is a STRONG binding
    // to the specific UCAN proof chain — not a weak binding to the
    // effective capability set. Defends against substitution attacks
    // at resume.
    unimplemented!(
        "G14-D wires cap_snapshot_hash binding to UCAN-proof-CID set at suspend envelope"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — §11 CLR-2 — redundant-distinct: hash binds chain at WAIT-resume"]
fn cap_snapshot_hash_binds_ucan_proof_chain_at_wait_resume() {
    // §11 CLR-2 redundant-distinct pin. Composes with
    // `wait_resume_cap_snapshot_hash_binds_ucan_proof_chain_at_envelope`
    // (which pins the suspend side) — this pin is the RESUME side:
    // resume MUST recheck the bound hash before continuing.
    //
    // Implementer wires:
    //
    //   let envelope = engine.suspend_for_test(&actor_did, &chain_at_suspend, ...).unwrap();
    //
    //   // Between suspend + resume, the chain materially changes
    //   // (e.g., issuer revoked one of the chain UCANs):
    //   engine.caps().revoke(&chain_at_suspend[0].cid()).unwrap();
    //
    //   // Resume MUST detect the chain change + reject:
    //   let err = engine.resume(envelope.id()).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::CapSnapshotHashMismatch { .. }),
    //       "resume must reject when bound chain changed per CLR-2 §11");
    //
    // OBSERVABLE consequence: a resume against a chain that has been
    // revoked observably rejects with CapSnapshotHashMismatch.
    // Defends against the "old envelope replayed after revoke" attack.
    unimplemented!("G14-D wires resume-time cap_snapshot_hash mismatch detection per CLR-2 §11");
}
