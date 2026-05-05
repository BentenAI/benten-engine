//! R4-FP-R3-B RED-PHASE pins: UCAN parent-child time-window narrowing
//! at durable layer (G14-B wave-4b; cap-r4-2 MAJOR closure of
//! cap-major-1 fix-now-action).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-2):
//!
//! - `tests/ucan_chain_rejects_child_expires_after_parent` — cap-r4-2 (a)
//! - `tests/ucan_chain_rejects_child_not_before_earlier_than_parent` — cap-r4-2 (b)
//! - `tests/ucan_chain_validation_at_replay_time_uses_current_clock_not_issuance_clock` — cap-r4-2 (c)
//!
//! ## Architectural intent (cap-r4-2 MAJOR closure)
//!
//! cap-major-1 (R1 capability-system) specified four named pins to
//! close UCAN time-window correctness. The R3 corpus shipped:
//! - `ucan_chain_walk_propagates_nbf_exp_through_attenuation`
//!   (parent-expired propagates to child — ONE direction)
//! - `prop_ucan_chain_attenuation_never_widens_authority` (10k cases —
//!   AUTHORITY dimension only, NOT window dimension)
//! - `ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup`
//!   (parameterized current-clock — adjacent shape)
//!
//! The MISSING pins addressed here:
//! - **child_expires_after_parent**: explicit child-claims-later-exp
//!   rejection (parent.exp = T+60; child.exp = T+86400 → MUST reject).
//! - **child_not_before_earlier_than_parent**: explicit child-claims-
//!   earlier-nbf rejection (parent.nbf = T+1000; child.nbf = T+500 →
//!   MUST reject).
//! - **replay_time_uses_current_clock**: durable-store re-validates
//!   against fresh wall-clock; issuance-time clock not cached.
//!
//! Note: the 10k window-only proptest `prop_ucan_chain_time_window_never_widens`
//! is added to `crates/benten-id/tests/prop_ucan_attenuation.rs` by a
//! separate fix (per tcc-r1-5 R3-A territory). This file pins the
//! deterministic cases at the durable layer (G14-B).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-B
//! implementer un-ignores AND replaces stub bodies. Per §3.6b pim-2
//! these tests must drive the production
//! `UCANBackend::validate_chain_at` path + assert typed-error variants
//! naming the specific window violation.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-B — cap-r4-2 (a) — child claims exp later than parent rejects"]
fn ucan_chain_rejects_child_expires_after_parent() {
    // cap-r4-2 (a) pin. The child's exp window MUST NOT exceed the
    // parent's exp; widening rejects at chain-walk with a typed error.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let root_kp = benten_id::keypair::Keypair::generate();
    //   let middle_kp = benten_id::keypair::Keypair::generate();
    //
    //   let issuance_secs = 1_000_000_000;
    //
    //   // Parent: short exp window (60s).
    //   let parent = benten_id::ucan::Ucan::builder()
    //       .issuer(root_kp.public_key().to_did())
    //       .audience(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf(issuance_secs)
    //       .exp(issuance_secs + 60)
    //       .sign(&root_kp).unwrap();
    //
    //   // Child: claims much later exp (86400s) — widening!
    //   let widening_child = benten_id::ucan::Ucan::builder()
    //       .issuer(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf(issuance_secs)
    //       .exp(issuance_secs + 86400) // wider than parent
    //       .proof_cids(&[parent.cid()])
    //       .sign(&middle_kp).unwrap();
    //
    //   backend.install_proof(&parent).unwrap();
    //   backend.install_proof(&widening_child).unwrap();
    //
    //   // Validate within parent's window (which is also within child's
    //   // claimed window): MUST still reject because the chain is
    //   // structurally invalid (child widens parent's authority).
    //   let err = backend.validate_chain_at(
    //       &[widening_child], issuance_secs + 30).unwrap_err();
    //   assert!(matches!(err,
    //       benten_caps::UCANBackendError::WindowWidening { .. })
    //         || matches!(err,
    //       benten_caps::UCANBackendError::AttenuationViolated { .. }),
    //       "validate_chain_at must reject child.exp > parent.exp per cap-r4-2 (a)");
    //
    // OBSERVABLE consequence: a chain whose child claims authority
    // beyond the parent's exp window observably rejects at chain-walk
    // with a typed error naming the window-widening. Defends against
    // attenuation bypass at the time-window dimension.
    unimplemented!(
        "G14-B wires durable-backend rejection of child.exp > parent.exp per cap-r4-2 (a)"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — cap-r4-2 (b) — child claims nbf earlier than parent rejects"]
fn ucan_chain_rejects_child_not_before_earlier_than_parent() {
    // cap-r4-2 (b) pin. The child's nbf MUST NOT precede the parent's
    // nbf; predating rejects at chain-walk.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let root_kp = benten_id::keypair::Keypair::generate();
    //   let middle_kp = benten_id::keypair::Keypair::generate();
    //
    //   let issuance_secs = 1_000_000_000;
    //
    //   // Parent: nbf = T+1000.
    //   let parent = benten_id::ucan::Ucan::builder()
    //       .issuer(root_kp.public_key().to_did())
    //       .audience(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf(issuance_secs + 1000)
    //       .exp(issuance_secs + 5000)
    //       .sign(&root_kp).unwrap();
    //
    //   // Child: claims nbf = T+500 — earlier than parent → widening backwards.
    //   let backdating_child = benten_id::ucan::Ucan::builder()
    //       .issuer(middle_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf(issuance_secs + 500) // earlier than parent
    //       .exp(issuance_secs + 5000)
    //       .proof_cids(&[parent.cid()])
    //       .sign(&middle_kp).unwrap();
    //
    //   backend.install_proof(&parent).unwrap();
    //   backend.install_proof(&backdating_child).unwrap();
    //
    //   // Validate at T+750 (within child's claimed nbf, before parent's nbf):
    //   let err = backend.validate_chain_at(
    //       &[backdating_child], issuance_secs + 750).unwrap_err();
    //   assert!(matches!(err,
    //       benten_caps::UCANBackendError::WindowWidening { .. })
    //         || matches!(err,
    //       benten_caps::UCANBackendError::AttenuationViolated { .. }),
    //       "validate_chain_at must reject child.nbf < parent.nbf per cap-r4-2 (b)");
    //
    // OBSERVABLE consequence: a chain whose child claims authority
    // BEFORE the parent's nbf observably rejects. Defends against
    // backdating attacks where a subordinate tries to claim earlier
    // activation than the parent allows.
    unimplemented!(
        "G14-B wires durable-backend rejection of child.nbf < parent.nbf per cap-r4-2 (b)"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — cap-r4-2 (c) — replay-time validation uses current clock not issuance clock"]
fn ucan_chain_validation_at_replay_time_uses_current_clock_not_issuance_clock() {
    // cap-r4-2 (c) pin. Durable-store re-validates against fresh
    // wall-clock; issuance-time clock is NOT cached. This is the
    // replay-time-correctness pin: a UCAN that validated at issuance
    // MUST be re-checked against current wallclock at replay time.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let issuance_secs = 1_000_000_000;
    //   let exp_secs = issuance_secs + 60;
    //
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf(issuance_secs)
    //       .exp(exp_secs)
    //       .sign(&issuer).unwrap();
    //   backend.install_proof(&ucan).unwrap();
    //
    //   // Validate at issuance + 30: passes (within window).
    //   backend.validate_chain_at(&[ucan.clone()], issuance_secs + 30).unwrap();
    //
    //   // Same backend; same durable store; validate at exp + 30 (post-exp
    //   // wallclock at REPLAY time) — MUST reject. The backend MUST NOT
    //   // cache the earlier-passing result; it MUST re-check.
    //   let err = backend.validate_chain_at(&[ucan.clone()], exp_secs + 30).unwrap_err();
    //   assert!(matches!(err, benten_caps::UCANBackendError::ProofExpired { .. }),
    //       "validate_chain_at must use replay-time wallclock per cap-r4-2 (c)");
    //
    //   // Earlier-passing call's result MUST NOT have polluted the cache:
    //   let err2 = backend.validate_chain_at(&[ucan], exp_secs + 60).unwrap_err();
    //   assert!(matches!(err2, benten_caps::UCANBackendError::ProofExpired { .. }),
    //       "subsequent validate_chain_at call must also use replay-time per cap-r4-2 (c)");
    //
    // OBSERVABLE consequence: even if a UCAN passed validation at one
    // wallclock, a later call with a wallclock past exp observably
    // rejects. Defends against the "issuance-clock cached forever"
    // footgun that would silently pass post-exp tokens.
    unimplemented!(
        "G14-B wires replay-time wallclock re-check at validate_chain_at per cap-r4-2 (c)"
    );
}
