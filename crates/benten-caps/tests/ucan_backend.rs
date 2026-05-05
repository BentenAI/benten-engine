//! R3-B RED-PHASE pins for `benten-caps` durable UCAN backend
//! (G14-B wave-4b; plan §3 G14-B + crypto-blocker-2 + CLR-2).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-B +
//! §3.A CLR-2 cluster):
//!
//! - `tests/ucan_backend_chain_walk_against_durable_store` — plan §3 G14-B
//! - `tests/ucan_backend_delegation_attenuation` — plan §3 G14-B
//! - `tests/ucan_backend_revocation_durable_across_restart` — plan §3 G14-B
//! - `tests/ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup` — crypto-blocker-2 + CLR-2
//! - `tests/ucan_backend_no_longer_returns_not_implemented` — plan §3 G14-B
//!
//! ## Architectural intent
//!
//! Phase-2b shipped `benten-caps::UCANBackend` as a stub returning
//! `CapError::NotImplemented` (Compromise tracked in
//! SECURITY-POSTURE.md). G14-B wave-4b lights it up against a durable
//! grant store backed by `GraphBackend` (the umbrella trait landing
//! at G13-A). Chain validation + delegation + revocation all work
//! end-to-end + persist across engine restarts.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-B
//! implementer un-ignores AND replaces the stub bodies. Per
//! §3.6b pim-2, the un-ignored tests must drive the production
//! `UCANBackend` entry point through the durable store seam — not
//! against an in-memory short-circuit. The
//! `ucan_backend_no_longer_returns_not_implemented` pin is the
//! load-bearing "stub IS gone" sentinel.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-B wave-4b fills UCANBackend chain-walk against durable store"]
fn ucan_backend_chain_walk_against_durable_store() {
    // plan §3 G14-B pin. G14-B implementer wires this:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let audience = benten_id::keypair::Keypair::generate();
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .audience(audience.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf_now()
    //       .exp_in_secs(3600)
    //       .sign(&issuer).unwrap();
    //
    //   backend.install_proof(&ucan).unwrap();
    //
    //   // Chain-walk: invocation cites the proof CID; backend resolves
    //   // it from the durable store + validates:
    //   let invocation = ... .proof_cids(&[ucan.cid()]) ... ;
    //   backend.validate_invocation(&invocation).unwrap();
    //
    // OBSERVABLE consequence: chain-walk fetches proofs from disk, not
    // from a hardcoded in-memory map; full UCAN semantics enforced at
    // the durable seam. Defends against the stub-implementation
    // anti-pattern where chain-walk silently passes against an empty
    // or in-memory-only store.
    unimplemented!(
        "G14-B wires UCANBackend::open() + install_proof() + validate_invocation() against durable store"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — plan §3 G14-B — delegation attenuation"]
fn ucan_backend_delegation_attenuation() {
    // plan §3 G14-B pin. The durable backend MUST enforce attenuation
    // at chain-walk: a delegated UCAN cannot widen its parent's
    // authority. Composes with G14-A1 keypair-side
    // `ucan_chain_attenuation_rejects_overgrant` but at the durable
    // store layer.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let root_kp = benten_id::keypair::Keypair::generate();
    //   let middle_kp = benten_id::keypair::Keypair::generate();
    //
    //   let parent = benten_id::ucan::Ucan::builder()
    //       .issuer(root_kp.public_key().to_did())
    //       .audience(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .sign(&root_kp).unwrap();
    //
    //   let overgrant_child = benten_id::ucan::Ucan::builder()
    //       .issuer(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "write") // widening!
    //       .proof_cids(&[parent.cid()])
    //       .sign(&middle_kp).unwrap();
    //
    //   backend.install_proof(&parent).unwrap();
    //   backend.install_proof(&overgrant_child).unwrap();
    //
    //   let err = backend.validate_chain(&[overgrant_child]).unwrap_err();
    //   assert!(matches!(err, benten_caps::UCANBackendError::AttenuationViolated { .. }));
    //
    // OBSERVABLE consequence: the durable backend rejects the
    // attenuation-violating chain at validate_chain with a typed
    // error that names the specific widening (capability + action).
    unimplemented!("G14-B wires durable-backend attenuation rejection at chain-walk");
}

#[test]
#[ignore = "RED-PHASE: G14-B — plan §3 G14-B — revocation durable across restart"]
fn ucan_backend_revocation_durable_across_restart() {
    // plan §3 G14-B pin. Revocation MUST persist across engine
    // restarts via the durable grant store. Composes with
    // `ucan_chain_revocation_propagates` (G14-A1) but pins persistence
    // at the durable seam.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let ucan = ... ;
    //
    //   {
    //       let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //       backend.install_proof(&ucan).unwrap();
    //       backend.validate_chain(&[ucan.clone()]).unwrap();
    //       backend.revoke(&ucan.cid()).unwrap();
    //       // backend dropped at end of scope; durable-store flush
    //   }
    //
    //   // Re-open backend at same store path; revocation MUST persist:
    //   {
    //       let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //       let err = backend.validate_chain(&[ucan]).unwrap_err();
    //       assert!(matches!(err, benten_caps::UCANBackendError::Revoked { .. }));
    //   }
    //
    // OBSERVABLE consequence: the same UCAN that validated before the
    // revocation rejects after re-opening the durable store. Defends
    // against the in-memory-only-revocation footgun where a process
    // restart silently re-validates revoked tokens.
    unimplemented!("G14-B wires durable revocation persistence across UCANBackend::open() re-open");
}

#[test]
#[ignore = "RED-PHASE: G14-B — crypto-blocker-2 BLOCKER + CLR-2 — expired proof rejection at lookup"]
fn ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup() {
    // crypto-blocker-2 BLOCKER pin (CLR-2 cluster). Even if a UCAN is
    // stored in the durable proof index, presenting it AFTER its `exp`
    // window MUST reject at chain-walk. This closes the "old proof
    // sitting in disk forever, replayed by attacker who sniffed it
    // pre-exp" attack class.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let issuance_secs = 1_000_000_000;
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .nbf(issuance_secs)
    //       .exp(issuance_secs + 60)
    //       .sign(&issuer).unwrap();
    //
    //   backend.install_proof(&ucan).unwrap();
    //   // Validate at issuance + 30: passes (within window).
    //   backend.validate_chain_at(&[ucan.clone()], issuance_secs + 30).unwrap();
    //   // Validate at issuance + 120: rejects (post-exp), even though
    //   // the proof IS in the durable store:
    //   let err = backend.validate_chain_at(&[ucan], issuance_secs + 120).unwrap_err();
    //   assert!(matches!(err, benten_caps::UCANBackendError::ProofExpired { .. }));
    //
    // OBSERVABLE consequence: the durable-store lookup is followed by
    // a time-window check; expired proofs reject at chain-walk even
    // when present in the store. CLR-2 cross-lens cluster pin.
    unimplemented!(
        "G14-B wires post-exp rejection at durable-store chain-walk lookup per crypto-blocker-2"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — plan §3 G14-B — backend no longer returns NotImplemented"]
fn ucan_backend_no_longer_returns_not_implemented() {
    // plan §3 G14-B pin. The Phase-2b stub returned
    // `CapError::NotImplemented` from every UCANBackend entry point
    // (Compromise #X tracked in SECURITY-POSTURE.md). G14-B closes
    // this; no entry point should produce NotImplemented anymore.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let ucan = ... ;
    //
    //   // Every entry point: the result IS NOT NotImplemented:
    //   match backend.install_proof(&ucan) {
    //       Err(benten_caps::CapError::NotImplemented(_)) =>
    //           panic!("G14-B closed; install_proof must not return NotImplemented"),
    //       _ => {}
    //   }
    //   match backend.validate_chain(&[ucan.clone()]) {
    //       Err(benten_caps::CapError::NotImplemented(_)) =>
    //           panic!("G14-B closed; validate_chain must not return NotImplemented"),
    //       _ => {}
    //   }
    //   match backend.revoke(&ucan.cid()) {
    //       Err(benten_caps::CapError::NotImplemented(_)) =>
    //           panic!("G14-B closed; revoke must not return NotImplemented"),
    //       _ => {}
    //   }
    //
    // OBSERVABLE consequence: post-G14-B, every public UCANBackend
    // method returns either Ok or a real (non-NotImplemented) typed
    // error. This is the load-bearing "stub gone" sentinel pin.
    unimplemented!(
        "G14-B wires assertion that every UCANBackend entry point returns non-NotImplemented results"
    );
}

// =====================================================================
// R4-FP-R3-B RED-PHASE pins: D-PHASE-3-21 D2 closure — UCAN-gated
// host:atrium:publish_view_result capability accepted in attenuation
// chain (per Ben's 2026-05-05 ratification of D2: option (iii) +
// UCAN-gated capability, no new trust-policy primitive).
//
// Pin sources (per R4 R1 capability-system-reviewer brief D2 + plan
// D-PHASE-3-21 resolution):
//
// - `ucan_backend_accepts_host_atrium_publish_view_result_capability`
// - `ucan_backend_attenuates_host_atrium_publish_view_result_in_chain`
// =====================================================================

#[test]
#[ignore = "RED-PHASE: G14-B — D2 D-PHASE-3-21 — UCAN backend accepts host:atrium:publish_view_result capability"]
fn ucan_backend_accepts_host_atrium_publish_view_result_capability() {
    // D2 D-PHASE-3-21 closure pin (per Ben's 2026-05-05 ratification:
    // option (iii) + UCAN-gated `host:atrium:publish_view_result`
    // capability, no new trust-policy primitive). The durable backend
    // recognizes the new capability string in the chain-walk.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let viewer_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Issue UCAN granting host:atrium:publish_view_result:
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(publisher_kp.public_key().to_did())
    //       .audience(viewer_kp.public_key().to_did())
    //       .capability("host:atrium:publish_view_result", "*")
    //       .nbf_now()
    //       .exp_in_secs(3600)
    //       .sign(&publisher_kp).unwrap();
    //
    //   backend.install_proof(&ucan).unwrap();
    //
    //   // Validate at chain-walk: the capability is recognized:
    //   let invocation = ... .proof_cids(&[ucan.cid()])
    //                        .invoke_capability("host:atrium:publish_view_result", "*") ... ;
    //   backend.validate_invocation(&invocation).unwrap();
    //
    // OBSERVABLE consequence: the `host:atrium:publish_view_result`
    // capability is recognized by the durable backend; downstream
    // user-view-replication paths can gate publish via UCAN delegation
    // without a new trust-policy primitive. Closes D-PHASE-3-21 D2.
    unimplemented!(
        "G14-B recognizes host:atrium:publish_view_result capability per D-PHASE-3-21 D2"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — D2 D-PHASE-3-21 — host:atrium:publish_view_result attenuates correctly in chain"]
fn ucan_backend_attenuates_host_atrium_publish_view_result_in_chain() {
    // D2 D-PHASE-3-21 closure pin (proptest companion at the durable
    // layer). The new capability participates in attenuation chain
    // walks: a child cannot widen a parent's host:atrium:publish_view_result
    // grant.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let root_kp = benten_id::keypair::Keypair::generate();
    //   let middle_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Parent: only specific atrium namespace.
    //   let parent = benten_id::ucan::Ucan::builder()
    //       .issuer(root_kp.public_key().to_did())
    //       .audience(middle_kp.public_key().to_did())
    //       .capability("host:atrium:publish_view_result", "/atrium/specific")
    //       .sign(&root_kp).unwrap();
    //
    //   // Child: tries to widen to ALL atriums (*):
    //   let widening_child = benten_id::ucan::Ucan::builder()
    //       .issuer(middle_kp.public_key().to_did())
    //       .capability("host:atrium:publish_view_result", "*") // widening!
    //       .proof_cids(&[parent.cid()])
    //       .sign(&middle_kp).unwrap();
    //
    //   backend.install_proof(&parent).unwrap();
    //   backend.install_proof(&widening_child).unwrap();
    //
    //   let err = backend.validate_chain(&[widening_child]).unwrap_err();
    //   assert!(matches!(err, benten_caps::UCANBackendError::AttenuationViolated { .. }),
    //       "host:atrium:publish_view_result MUST attenuate per D2 + standard UCAN semantics");
    //
    // OBSERVABLE consequence: the new capability participates in
    // standard UCAN attenuation; no special-case bypass. Defends
    // against the "new capability bypasses chain-walk" failure shape
    // (acceptable per option (iii) since standard UCAN semantics apply).
    unimplemented!(
        "G14-B applies standard attenuation to host:atrium:publish_view_result per D-PHASE-3-21 D2"
    );
}
